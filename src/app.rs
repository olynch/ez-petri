use yew::prelude::*;
use serde::{Serialize, Deserialize};
use plotters::prelude::*;
use ndarray::prelude::*;

type DrawResult<T> = Result<T, Box<dyn std::error::Error>>;

type V = Array<f32, Ix1>;
type YVals = Array<f32, Ix2>;

fn rk_step(f: impl Fn(f32, &V) -> V, t: f32, dt: f32, y: &V) -> V {
    let k1: V = f(t,y);
    let k2 = f(t + 0.5*dt, &(y + &(&k1 * dt * 0.5)));
    let k3 = f(t + 0.5*dt, &(y + &(&k2 * dt * 0.5)));
    let k4 = f(t + dt, &(y + &(&k3 * dt)));
    y + &((dt / 6.) * (&k1 + &(k2*2.) + &(k3*2.) + &k4))
}

fn rk_solve(f: impl Fn(f32, &V) -> V, t0: f32, t1: f32, steps: usize, y0: &V) -> YVals {
    let dt = (t1 - t0) / (steps as f32);
    let mut yvals = Array::zeros((steps,y0.len()));
    let mut y = y0.clone();
    for (i,mut cell) in yvals.axis_iter_mut(Axis(0)).enumerate() {
        let t = t0 + (i as f32) / (steps as f32) * (t1 - t0);
        y = rk_step(&f, t, dt, &y);
        cell.assign(&y);
    }
    yvals
}

/// Axis(0) is transitions
/// Axis(1) is input vs. output
/// Axis(2) is numbers of inputs and outputs
pub type PetriData = Array<i32, Ix3>;

pub fn petri_data_from_nested_vec(v: Vec<Vec<Vec<i32>>>) -> PetriData {
    let (t,s) = (v.len(),v[0][0].len());
    Array::from_shape_vec((t,2,s), (&v.concat()).concat()).unwrap()
}

#[derive(Serialize,Deserialize,Debug)]
pub struct Petri {
    pub data: PetriData,
    pub species: Array1<String>,
    pub transitions: Array1<String>,
    pub name: String
}

fn num_species(p: &PetriData) -> usize {
    p.len_of(Axis(2))
}

fn master_eq(p: &PetriData, rates: &V, _: f32, y: &V) -> V {
    let mut yp = Array::zeros(y.dim());
    let n = num_species(p);
    for (i, trans) in p.axis_iter(Axis(0)).enumerate() {
        let r = rates[i];
        for j in 0..n {
            let coeff = r * ((trans[(1,j)] - trans[(0,j)]) as f32);
            let mut term: f32 = 1.;
            for (k,pop) in y.iter().enumerate() {
                term *= pop.powi(trans[(0,k)]);
            }
            yp[j] += coeff * term;
        }
    }
    yp
}

static COLORS: [RGBColor;4] = [RED, BLUE, YELLOW, GREEN];

/// Draw power function f(x) = x^power.
pub fn draw_petri(canvas_id: &str, petri_net: &Petri, rates: &V, init_vals: &V, xmax: f32, ymax: f32)
                  -> DrawResult<impl Fn((i32, i32)) -> Option<(f32, f32)>> {
    let p = &petri_net.data;
    let backend = CanvasBackend::new(canvas_id).expect("cannot find canvas");
    let root = backend.into_drawing_area();
    let font: FontDesc = ("sans-serif", 20.0).into();

    let steps = 100 * (xmax as usize);

    let yvals = rk_solve(|t,v| master_eq(p,rates,t,v), 0., xmax, 100 * (xmax as usize), init_vals);

    let n = num_species(p);

    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(&petri_net.name, font)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_ranged(0.0..xmax, 0.0..ymax)?;

    chart.configure_mesh().x_labels(5).y_labels(6).disable_mesh().draw()?;


    for i in 0..n {
        let c = &COLORS[i];
        chart.draw_series(LineSeries::new(
            (0..steps)
                .map(|x| (x as f32 / 100.0, *yvals.get((x,i)).unwrap() as f32)),
            c,
        ))?
            .label(&petri_net.species[i])
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], c));
    }

    chart.configure_series_labels().border_style(&BLACK).draw()?;

    root.present()?;
    return Ok(chart.into_coord_trans());
}

pub struct App {
    rates: V,
    init_vals: V,
    petri_net: Petri,
    canvas_id: String,
    name: String,
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
}

impl Component for App {
    type Message = Msg;
    type Properties = Props;

    fn create(p: Self::Properties, _: ComponentLink<Self>) -> Self {

        App {
            rates: Array::from(p.rates0),
            init_vals: Array::from(p.init_vals0),
            petri_net: Petri {
                data: petri_data_from_nested_vec(p.petri_data),
                species: Array::from(p.species),
                transitions: Array::from(p.transitions),
                name: p.name
            },
            canvas_id: p.canvas_id,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::InitValueUpdate(i,x) => { self.init_vals[i] = x; }
            Msg::RateUpdate(i,x) => { self.rates[i] = x; }
        }
        true
    }

    fn view(&self) -> Html<Self> {
        draw_petri(&self.canvas_id, &self.petri_net, &self.rates, &self.init_vals, 5., 6.).unwrap();
        html! {
            <div>
                <div>
                  { for (0..self.init_vals.len()).map(|i| {
                      let idx = i;
                      let c = self.init_vals[i];
                      view_slider(Box::new(move |x| Msg::InitValueUpdate(i,x)), c, &format!("init value for {}: {}", &self.petri_net.species[i], &c))
                    })
                  }
                  { for (0..self.petri_net.data.len_of(Axis(0))).map(|i| {
                      let idx = i;
                      let c = self.rates[i];
                      view_slider(Box::new(move |x| Msg::RateUpdate(i,x)), c, &format!("rate of {}: {}", &self.petri_net.transitions[i], &c))
                    })
                  }
                </div>
            </div>
        }
    }
}

fn view_slider(msg_generator: Box<dyn Fn(f32) -> Msg>, init_val: f32, label: &str) -> Html<App> {
    html!{
        <p>
          <label for=label> { label } </label>
            <input value={init_val} type="range" min="0" max="10" step="0.01" class="slider" id=label oninput=|inputdata| {
              msg_generator(inputdata.value.parse().unwrap())
          }></input>
        </p>
    }
}
