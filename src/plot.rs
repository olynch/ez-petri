use serde::{Deserialize, Serialize};
use yew::prelude::*;
use crate::petri::*;

#[derive(Serialize, Deserialize, Properties)]
pub struct PlotProps {
    #[props(required)]
    pub petri: PetriNet,

    #[props(required)]
    pub controls: PlotControls,
}

pub struct Plot {
    props: PlotProps,
    live_updating: bool,
    mounted: bool
}

pub enum PlotMsg {
    Draw,
    LiveUpdating
}

const CANVAS_ID: &'static str = "CANVAS_ID";

#[cfg(target_arch = "wasm32")]
impl Component for Plot {
    type Message = PlotMsg;
    type Properties = PlotProps;

    fn create(p: Self::Properties, _: ComponentLink<Self>) -> Self {
        Plot {
            props: p,
            live_updating: false,
            mounted: false
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            PlotMsg::Draw => {
                self.props.petri.plot(&self.props.controls, CANVAS_ID).ok().unwrap();
                false
            }
            PlotMsg::LiveUpdating => {
                self.live_updating ^= true;
                true
            }
        }
    }

    fn view(&self) -> Html<Self> {
        if self.mounted && self.live_updating {
            self.props.petri.plot(&self.props.controls, CANVAS_ID).ok().unwrap();
        }
        html! {
            <>
                <canvas height="400px" width="500px" class="plot" id={CANVAS_ID}> </canvas>
            { if self.live_updating {
                html!{
                    <button style="width:200px;margin-right:30px" onclick=|_| PlotMsg::LiveUpdating>{"Live Updating"}</button>
                }
            } else {
                html!{
                    <>
                    <button style="width:200px;margin-right:30px" onclick=|_| PlotMsg::LiveUpdating>{"Manual Updating"}</button>
                    <button style="width:100px;margin-right:30px" onclick=|_| PlotMsg::Draw>{"Refresh"}</button>
                    </>
                }
            }}
            </>
        }
    }

    fn change(&mut self, p: Self::Properties) -> ShouldRender {
        self.props = p;
        true
    }

    fn mounted(&mut self) -> ShouldRender {
        self.mounted = true;
        self.props.petri.plot(&self.props.controls, CANVAS_ID).ok().unwrap();
        false
    }
}
