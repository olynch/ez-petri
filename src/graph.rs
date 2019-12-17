use crate::math::*;
use plotters::prelude::*;
use serde::{Deserialize, Serialize};
use ndarray::prelude::*;
use yew::prelude::*;

type DrawResult<T> = Result<T, Box<dyn std::error::Error>>;

pub struct PetriNet {
    data: PetriData,
    species: Array1<String>,
    transitions: Array1<String>,
    name: String,
}

static COLORS: [RGBColor; 4] = [RED, BLUE, YELLOW, GREEN];

fn get_color(i: usize) -> &'static RGBColor {
    &COLORS[i]
}

pub struct App {
    rates: V,
    init_vals: V,
    xmax: f32,
    ymax: f32,
    petri_net: PetriNet,
    canvas_id: String,
}

static STEPS_PER_UNIT: usize = 100;

impl App {
    fn draw(&self) -> DrawResult<impl Fn((i32, i32)) -> Option<(f32, f32)>> {
        let steps = STEPS_PER_UNIT * (self.xmax as usize);
        let yvals = self
            .petri_net
            .data
            .solve(&self.rates, &self.init_vals, self.xmax, steps);

        let backend = CanvasBackend::new(&self.canvas_id).expect("cannot find canvas");
        let root = backend.into_drawing_area();
        let font: FontDesc = ("sans-serif", 20.0).into();

        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption(&self.petri_net.name, font)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_ranged(0.0..self.xmax, 0.0..self.ymax)?;

        chart
            .configure_mesh()
            .disable_mesh()
            .draw()?;

        for (i,s) in self.petri_net.species.iter().enumerate() {
            let c = get_color(i);
            chart
                .draw_series(LineSeries::new(
                    (0..steps).map(|x| {
                        (
                            x as f32 / (STEPS_PER_UNIT as f32),
                            *yvals.get((x, i)).unwrap() as f32,
                        )
                    }),
                    c,
                ))?
                .label(s)
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], c));
        }

        chart
            .configure_series_labels()
            .border_style(&BLACK)
            .draw()?;

        root.present()?;
        return Ok(chart.into_coord_trans());
    }
}

pub enum Msg {
    RateUpdate(usize, f32),
    InitValueUpdate(usize, f32),
}

#[derive(Serialize, Deserialize, Properties)]
pub struct Props {
    #[props(required)]
    pub name: String,

    #[props(required)]
    pub petri_data: Vec<Vec<Vec<i32>>>,

    #[props(required)]
    pub species: Vec<String>,

    #[props(required)]
    pub transitions: Vec<String>,

    #[props(required)]
    pub init_vals0: Vec<f32>,

    #[props(required)]
    pub rates0: Vec<f32>,

    #[props(required)]
    pub canvas_id: String,

    #[props(required)]
    pub xmax: f32,

    #[props(required)]
    pub ymax: f32
}

impl Component for App {
    type Message = Msg;
    type Properties = Props;

    fn create(p: Self::Properties, _: ComponentLink<Self>) -> Self {
        App {
            rates: Array::from(p.rates0),
            init_vals: Array::from(p.init_vals0),
            petri_net: PetriNet {
                data: PetriData::from_nested_vec(p.petri_data).unwrap(),
                species: Array::from(p.species),
                transitions: Array::from(p.transitions),
                name: p.name,
            },
            canvas_id: p.canvas_id,
            xmax: p.xmax,
            ymax: p.ymax
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::InitValueUpdate(i, x) => {
                self.init_vals[i] = x;
            }
            Msg::RateUpdate(i, x) => {
                self.rates[i] = x;
            }
        }
        true
    }

    fn view(&self) -> Html<Self> {
        self.draw();
        html! {
            <div>
                <div>
            { for self.petri_net.species.iter().enumerate().map(|(i,s)| {
                let v = self.init_vals[i];
                view_slider(Box::new(move |x| Msg::InitValueUpdate(i,x)), v, &format!("init value for {}: {}", s, v))
            })
            }
            { for self.petri_net.transitions.iter().enumerate().map(|(i,t)| {
                let r = self.rates[i];
                view_slider(Box::new(move |x| Msg::RateUpdate(i,x)), r, &format!("rate of {}: {}", t, &r))
            })
            }
            </div>
                </div>
        }
    }
}

fn view_slider(msg_generator: Box<dyn Fn(f32) -> Msg>, init_val: f32, label: &str) -> Html<App> {
    html! {
        <p>
            <label for=label> { label } </label>
            <input value={init_val} type="range" min="0" max="10" step="0.01" class="slider" id=label oninput=|inputdata| {
                msg_generator(inputdata.value.parse().unwrap())
            }></input>
            </p>
    }
}
