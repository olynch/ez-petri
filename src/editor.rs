use yew::prelude::*;
use ndarray::prelude::*;
use plotters::style::Color;
use serde::{Serialize, Deserialize};
use crate::plot::*;
use crate::petri::*;

trait Edit {
    type Val;

    fn apply_edit(self, v: &mut Self::Val);
}

#[derive(Debug)]
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

#[derive(Debug)]
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
#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
pub enum Msg {
    ForPetri(PetriEdit),
    ForControls(ControlsEdit),
    ViewSwitch,
    SourceUpdate(String),
    SaveInUrl,
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

#[derive(Serialize, Deserialize)]
pub struct GE {
    petri_net: PetriNet,
    controls: PlotControls,
}

impl Default for GE {
    fn default() -> Self {
        GE {
            petri_net: PetriNet::empty(),
            controls: PlotControls::empty()
        }
    }
}

pub struct SE {
    code: String
}

pub enum EditorState {
    Graphical (GE),
    Source (SE),
}

pub struct Editor {
    state: EditorState
}

fn color_style<T: Color>(c: T) -> String {
    let (r,g,b) = c.rgb();
    format!("background-color:rgba({},{},{},0.2)", r, g, b)
}

impl GE {
    fn to_se(&self) -> SE {
        SE {
            code: serde_json::to_string(&self).unwrap()
        }
    }
    
    fn view_matrix(&self) -> Html<Editor> {
        html!{
            <div class="matrix">
            <label>{"Transition Matrix:"}</label>
            <table style="border:1px solid black">
                <tr>
                    <th style="width:135px" rowspan="2"> </th>
                    { for self.petri_net.species.iter().enumerate().map(|(i,s)| {
                        html!{
                            <th class="species-header" colspan="2">
                                <input type="text" style={color_style(get_color(i))} class="table-form matrix-input" value={&s} oninput=|v|
                                    Msg::species_edit(VED::edit(i,PlainEdit::PlainEdit(v.value)))>
                                </input>
                                <button class="square-button" onclick=|_| Msg::species_edit(VED::remove(i))>{"-"}</button>
                            </th>
                        }
                    })}
                    <th> <button class="square-button" onclick=|_| Msg::species_edit(VED::add())>{"+"}</button> </th>
                </tr>
                <tr>
                    { for self.petri_net.species.iter().enumerate().map(|(i,s)| {
                        html!{
                            <>
                            <td class="transition-direction">{"In"}</td>
                            <td class="transition-direction">{"Out"}</td>
                            </>
                        }
                    })}
                    <td> </td>
                </tr>
                { for self.petri_net.transitions.iter().enumerate().map(|(i,t)| {
                html!{
                    <tr>
                    <td class="transition-header">
                        <input class="matrix-input" type="text" value={&t.name} oninput=|v|
                            Msg::transitions_edit(VED::edit(i,TransitionEdit::NameEdit(v.value)))>
                        </input>
                        <button class="square-button" onclick=|_| Msg::transitions_edit(VED::remove(i))>{"-"}</button>
                    </td>
                    { for (0..self.petri_net.species.len()).map(|j| {
                        html!{
                            { for DIRECTIONS.iter().map(|d| {
                                html!{
                                    <td>
                                    <input class="transitions-counter" type="number" value={t[*d][j].to_string()} oninput=|xp| {
                                        Msg::transitions_edit(
                                            VED::edit(
                                                i,TransitionEdit::IOEdit(*d,j,xp.value.parse().unwrap())
                                            )
                                        )
                                    }> </input>
                                    </td>
                                }
                            })}
                        }
                    })}
                    <td> </td>
                    </tr>
                }
                })}
                <tr>
                    <td> <button class="square-button" onclick=|_| Msg::transitions_edit(VED::add())>{"+"}</button> </td>
                    { for (0..self.petri_net.species.len()+1).map(|_| { html!{ <> <td></td> <td></td> </> } } )}
                </tr>
            </table>
            </div>
        }
    }

    fn view_controls(&self) -> Html<Editor> {
        html!{
            <div class="controls">
            <label class="name-label" for="name-input">{ "Name: " }</label>
            <input id="name-input" type="text" value={&self.petri_net.name} oninput=|v| Msg::name_edit(v.value)>
            </input>
            <hr />
            <label>{"Initial Value Controls:"}</label>

            <table>
                <tr>
                <th class="control-cell"><div class="control-label">{"Species"}</div></th>
                <th class="control-cell"><div class="control-label">{"Initial Value"}</div></th>
                </tr>
            { for self.petri_net.species.iter().enumerate().map(|(i,s)| {
                html!{
                    <tr class="control-row" style={color_style(get_color(i))}>
                        <td class="control-cell"><div class="control-label">{&s}</div></td>
                        <td class="control-cell">
                        <input class="table-form control-slider" value={self.controls.init_vals[i]} type="range" min="0" max={self.controls.scale} step="0.01"
                        oninput=|v| { Msg::init_vals_edit(i,v.value.parse().unwrap()) }></input>
                        </td>
                    </tr>
                }
            })}
            </table>

            <hr />
            <label>{"Rate Controls:"}</label>

            <table>
                <tr>
                <th class="control-cell"><div class="control-label">{"Transition"}</div></th>
                <th class="control-cell"><div class="control-label">{"Rate"}</div></th>
                </tr>
            { for self.petri_net.transitions.iter().enumerate().map(|(i,t)| {
                html!{
                    <tr class="control-row">
                        <td class="control-cell"><div class="control-label">{&t.name}</div></td>
                        <td class="control-cell">
                        <input class="table-form control-slider" value={self.controls.rates[i]} type="range" min="0" max={self.controls.scale} step="0.01"
                        oninput=|v| { Msg::rates_edit(i,v.value.parse().unwrap()) }></input>
                        </td>
                    </tr>
                }
            })}
            </table>
            </div>
        }
    }

    fn view(&self) -> Html<Editor> {
        html!{
            <>
            <div class="row" style="margin-top:50px">
                <div class="column one-third">
                    { self.view_controls() }
                </div>
                <div class="column two-thirds">
                  <Plot petri=self.petri_net.clone() controls=self.controls.clone() />
                </div>
            </div>
            <div class="row">
                <hr />
            </div>
            <div class="row">
                <div class="column">
                    { self.view_matrix() }
                </div>
            </div>
            </>
        }
    }

    pub fn to_url_hash(&self) -> String {
        let mp = rmp_serde::to_vec(self).unwrap();
        base64::encode_config(&mp, base64::URL_SAFE)
    }

    pub fn from_url_hash(h: &str) -> Option<Self> {
        base64::decode_config(h, base64::URL_SAFE).ok()
            .and_then(|d| rmp_serde::from_read::<&[u8],GE>(&d).ok())
    }
}

impl SE {
    fn to_ge(&self) -> Option<GE> {
        serde_json::from_str(&self.code).ok()
    }

    fn view(&self) -> Html<Editor> {
        html!{
            <div class="row" style="margin-top:50px">
                <textarea class="sourceview" oninput=|e| {Msg::SourceUpdate(e.value)}>
                    {&self.code}
                </textarea>
            </div>
        }
    }
}

// Communication between editor and plotter

#[derive(Properties)]
pub struct EditorProps {
    #[props(required)]
    pub initial_state: GE
}

impl Component for Editor {
    type Message = Msg;
    type Properties = EditorProps;

    fn create(p: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Editor {
            state: EditorState::Graphical(
                p.initial_state
            )
        }
    }
    
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match &mut self.state {
            EditorState::Graphical(ge) => match msg {
                Msg::ForPetri(e) => {
                    ControlsEdit::from_petri_edit(&e).map(|ce| ce.apply_edit(&mut ge.controls));
                    e.apply_edit(&mut ge.petri_net);
                }
                Msg::ForControls(e) => {
                    e.apply_edit(&mut ge.controls);
                }
                Msg::ViewSwitch => {
                    self.state = EditorState::Source(ge.to_se())
                }
                Msg::SaveInUrl => {
                    let location = web_sys::window().unwrap().location();
                    location.set_hash(&ge.to_url_hash()).unwrap();
                }
                _otherwise => { }
            },
            EditorState::Source(se) => match msg {
                Msg::SourceUpdate(c) => {
                    se.code = c;
                }
                Msg::ViewSwitch => {
                    match se.to_ge() {
                        Some(ge) => {
                            self.state = EditorState::Graphical(ge);
                        }
                        None => { }
                    }
                }
                _otherwise => { }
            }
        }
        true
    }

    fn view(&self) -> Html<Self> {
        html!{
            <>
                <div class="navbar column">
                    <span>{"EZ Petri"}</span>
                    <span class="menu-action"><a href="https://owenlynch.org/posts/2019-12-28-ez-petri">{"Help"}</a></span>
                    <span class="menu-action" onclick=|_| Msg::ViewSwitch>{"Toggle Source"}</span>
                    <span class="menu-action" onclick=|_| Msg::SaveInUrl>{"Save in URL"}</span>
                </div>
            { match &self.state {
                EditorState::Graphical(ge) => ge.view(),
                EditorState::Source(se) => se.view(),
            }}
            </>
        }
    }
}
