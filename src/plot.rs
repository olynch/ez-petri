use serde::{Deserialize, Serialize};
use yew::prelude::*;
use crate::petri::*;

pub struct Plotter {
    controls: PlotControls,
    petri_net: PetriNet,
    canvas_id: String,
}

impl Plotter {
    fn plot(&self) -> Option<()> {
        self.petri_net.plot(&self.controls, &self.canvas_id).ok()
    }
}

pub enum PlotMsg {
    RateUpdate(usize, f32),
    InitValueUpdate(usize, f32),
}

#[derive(Serialize, Deserialize, Properties)]
pub struct PlotProps {
    #[props(required)]
    pub petri_net: PetriNet,

    #[props(required)]
    pub controls: PlotControls,

    #[props(required)]
    pub canvas_id: String,
}

impl Component for Plotter {
    type Message = PlotMsg;
    type Properties = PlotProps;

    fn create(p: Self::Properties, _: ComponentLink<Self>) -> Self {
        Plotter {
            petri_net: p.petri_net,
            controls: p.controls,
            canvas_id: p.canvas_id,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            PlotMsg::InitValueUpdate(i, x) => {
                self.controls.init_vals[i] = x;
            }
            PlotMsg::RateUpdate(i, x) => {
                self.controls.rates[i] = x;
            }
        }
        true
    }

    fn view(&self) -> Html<Self> {
        self.plot().unwrap();
        html! {
            <div>
            { for self.petri_net.species.iter().enumerate().map(|(i,s)| {
                let v = self.controls.init_vals[i];
                view_slider(Box::new(move |x| PlotMsg::InitValueUpdate(i,x)), v, &format!("init value for {}: {}", s, v))
            })
            }
            { for self.petri_net.transitions.iter().enumerate().map(|(i,t)| {
                let r = self.controls.rates[i];
                view_slider(Box::new(move |x| PlotMsg::RateUpdate(i,x)), r, &format!("rate of {}: {}", &t.name, &r))
            })
            }
            </div>
        }
    }
}

fn view_slider(msg_generator: Box<dyn Fn(f32) -> PlotMsg>, init_val: f32, label: &str) -> Html<Plotter> {
    html! {
        <p>
            <label for=label> { label } </label>
            <input value={init_val} type="range" min="0" max="10" step="0.01" class="slider" id=label oninput=|inputdata| {
                msg_generator(inputdata.value.parse().unwrap())
            }></input>
            </p>
    }
}
