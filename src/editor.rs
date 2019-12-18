use yew::prelude::*;
use ndarray::prelude::*;
use crate::math::*;
use crate::petri::*;

trait Edit {
    type Val;

    fn apply_edit(self, v: &mut Self::Val);
}

pub enum PlainEdit<T> {
    PlainEdit(T)
}

impl<T> Edit for PlainEdit<T> {
    type Val = T;
    fn apply_edit(self, v: &mut Self::Val) {
        match self {
            Self::PlainEdit(e) => { *v = e; }
        }
    }
}

pub enum VecEdit<S> {
    Remove(usize),
    Edit(usize,S)
}

impl<T,S> Edit for VecEdit<S> where S: Edit<Val=T> {
    type Val = Vec<T>;
    fn apply_edit(self, v: &mut Self::Val) {
        match self {
            Self::Remove(i) => { v.remove(i); }
            Self::Edit(i,e) => { e.apply_edit(&mut v[i]); }
        };
    }
}

// VecEditDefault
pub enum VED<S> {
    Add,
    Edit(VecEdit<S>)
}

impl<S> VED<S> {
    fn add() -> Self {
        Self::Add
    }

    fn edit(i: usize, e: S) -> Self {
        Self::Edit(VecEdit::Edit(i,e))
    }

    fn remove(i: usize) -> Self {
        Self::Edit(VecEdit::Remove(i))
    }
}

pub enum TransitionEdit {
    NameEdit(String),
    IOEdit(IO,usize,i32),
}

impl Edit for TransitionEdit {
    type Val = Transition;
    fn apply_edit(self, v: &mut Self::Val) {
        match self {
            Self::NameEdit(s) => { v.name = s; }
            Self::IOEdit(d,i,e) => { v[d][i] = e; }
        }
    }
}

pub enum PetriEdit {
    TransitionsEdit(VED<TransitionEdit>),
    SpeciesEdit(VED<PlainEdit<String>>),
    NameEdit(String),
}

impl Edit for PetriEdit {
    type Val = PetriNet;
    fn apply_edit(self, v: &mut Self::Val) {
        match self {
            Self::TransitionsEdit(e) => {
                match e {
                    VED::Add => {
                        let s = v.species.len();
                        let t = Transition {
                            input: Array1::zeros(s).to_vec(),
                            output: Array1::zeros(s).to_vec(),
                            name: "".to_string(),
                        };
                        v.transitions.push(t);
                    }
                    VED::Edit(e) => {
                        e.apply_edit(&mut v.transitions);
                    }
                }
            }
            Self::SpeciesEdit(e) => {
                match e {
                    VED::Add => {
                        for t in v.transitions.iter_mut() {
                            t.input.push(0);
                            t.output.push(0);
                        }
                        v.species.push("".to_string());
                    }
                    VED::Edit(e) => {
                        match &e {
                            VecEdit::Remove(i) => {
                                for t in v.transitions.iter_mut() {
                                    t.input.remove(*i);
                                    t.output.remove(*i);
                                }
                            }
                            _otherwise => { }
                        }
                        e.apply_edit(&mut v.species);
                    }
                }
            }
            Self::NameEdit(s) => { v.name = s; }
        }
    }
}

pub enum ControlsEdit {
    RatesEdit(VED<PlainEdit<f32>>),
    InitValsEdit(VED<PlainEdit<f32>>)
}

impl ControlsEdit {
    fn from_petri_edit(pe: &PetriEdit) -> Option<Self> {
        match pe {
            PetriEdit::TransitionsEdit(te) => {
                match te {
                    VED::Add => {
                        Some(Self::RatesEdit(VED::Add))
                    },
                    VED::Edit(VecEdit::Remove(i)) => {
                        Some(Self::RatesEdit(VED::remove(*i)))
                    }
                    _otherwise => None
                }
            }
            PetriEdit::SpeciesEdit(se) => {
                match se {
                    VED::Add => {
                        Some(Self::InitValsEdit(VED::Add))
                    }
                    VED::Edit(VecEdit::Remove(i)) => {
                        Some(Self::InitValsEdit(VED::remove(*i)))
                    }
                    _otherwise => None
                }
            }
            _otherwise => None
        }
    }
}

impl Edit for ControlsEdit {
    type Val = PlotControls;

    fn apply_edit(self, v: &mut Self::Val) {
        match self {
            Self::RatesEdit(e) => {
                match e {
                    VED::Add => { v.rates.push(0.); }
                    VED::Edit(ve) => { ve.apply_edit(&mut v.rates); }
                }
            },
            Self::InitValsEdit(e) => {
                match e {
                    VED::Add => { v.init_vals.push(0.); }
                    VED::Edit(ve) => { ve.apply_edit(&mut v.init_vals); }
                }
            }
        }
    }
}

pub enum Msg {
    ForPetri(PetriEdit),
    ForControls(ControlsEdit)
}

impl Msg {
    fn name_edit(n: String) -> Self {
        Self::ForPetri(PetriEdit::NameEdit(n))
    }

    fn species_edit(e: VED<PlainEdit<String>>) -> Self {
        Self::ForPetri(PetriEdit::SpeciesEdit(e))
    }

    fn transitions_edit(e: VED<TransitionEdit>) -> Self {
        Self::ForPetri(PetriEdit::TransitionsEdit(e))
    }

    fn rates_edit(i: usize, v: f32) -> Self {
        Self::ForControls(ControlsEdit::RatesEdit(VED::Edit(VecEdit::Edit(i,PlainEdit::PlainEdit(v)))))
    }

    fn init_vals_edit(i: usize, v: f32) -> Self {
        Self::ForControls(ControlsEdit::InitValsEdit(VED::Edit(VecEdit::Edit(i,PlainEdit::PlainEdit(v)))))
    }
}

pub struct Editor {
    petri_net: PetriNet,
    controls: PlotControls,
    canvas_id: String
}

impl Editor {
    fn view_transitions(&self) -> Html<Self> {
        html!{
            <div>
                <button onclick=|_| Msg::transitions_edit(VED::Add)>{"Add Transition"}</button>
                <ul>
                { for self.petri_net.transitions.iter().enumerate().map(|(i,t)| {
                html!{
                    <li>
                        <input type="text" value={&t.name} oninput=|v|
                            Msg::transitions_edit(VED::edit(i,TransitionEdit::NameEdit(v.value)))>
                        </input>
                        <input
                            value={self.controls.rates[i]}
                            type="range"
                            min="0"
                            max={self.controls.scale}
                            step="0.01"
                            class="slider"
                            oninput=|v| { Msg::rates_edit(i,v.value.parse().unwrap()) }>
                        </input>
                        <button onclick=|_| Msg::transitions_edit(VED::remove(i))>{"Remove"}</button>
                        <ul>
                        { for DIRECTIONS.iter().map(|d| {
                            html!{
                                <li>{&format!("{}: ",d)}
                                    <ul>
                                    { for (&t[*d]).iter().enumerate().map(|(j,x)| {
                                        html!{
                                            <li>
                                            {&format!("{}: ", &self.petri_net.species[j])}

                                            <input type="number" value={x.to_string()} oninput=|xp| {
                                                Msg::transitions_edit(
                                                    VED::edit(
                                                        i,TransitionEdit::IOEdit(*d,j,xp.value.parse().unwrap())
                                                    )
                                                )
                                            }>
                                            </input>
                                            </li>
                                        }
                                    })}
                                    </ul>
                                </li>
                            }
                        })}
                        </ul>
                    </li>
                }
                })}
                </ul>
            </div>
        }
    }

    fn view_species(&self) -> Html<Self> {
        html!{
            <div>
                <button onclick=|_| Msg::species_edit(VED::Add)>{"Add Species"}</button>
                <ul>
                { for self.petri_net.species.iter().enumerate().map(|(i,s)| {
                    html!{
                        <li>
                          <input type="text" value={&s} oninput=|v|
                            Msg::species_edit(VED::edit(i,PlainEdit::PlainEdit(v.value)))>
                            </input>
                            <input
                                value={self.controls.init_vals[i]}
                                type="range"
                                min="0"
                                max={self.controls.scale}
                                step="0.01"
                                class="slider"
                                oninput=|v| { Msg::init_vals_edit(i,v.value.parse().unwrap()) }>
                            </input>
                            <button onclick=|_| Msg::species_edit(VED::remove(i))>{"Remove"}</button>
                        </li>
                    }
                })}
                </ul>
            </div>
        }
    }
}

// Communication between editor and plotter

#[derive(Properties)]
pub struct EditorProps {
    pub canvas_id: String
}

impl Component for Editor {
    type Message = Msg;
    type Properties = EditorProps;

    fn create(p: Self::Properties, _: ComponentLink<Self>) -> Self {
        Editor {
            petri_net: PetriNet::empty(),
            controls: PlotControls::empty(),
            canvas_id: p.canvas_id
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ForPetri(e) => {
                ControlsEdit::from_petri_edit(&e).map(|ce| ce.apply_edit(&mut self.controls));
                e.apply_edit(&mut self.petri_net);
            }
            Msg::ForControls(e) => {
                e.apply_edit(&mut self.controls);
            }
        }
        true
    }

    fn view(&self) -> Html<Self> {
        self.petri_net.plot(&self.controls, &self.canvas_id);
        html!{
            <div>
                <label for="name_input">{ "Name: " }</label>
                <input id="name_input" type="text" value={&self.petri_net.name} oninput=|v| Msg::name_edit(v.value)>
                </input>
                <h3>{"Species"}</h3>
                { self.view_species() }
                <h3>{"Transitions"}</h3>
                { self.view_transitions() }
            </div>
        }
    }
}
