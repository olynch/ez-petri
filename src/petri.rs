use ndarray::prelude::*;
use serde::{Serialize,Deserialize};
use std::ops::{Index,IndexMut};
use std::fmt;
use plotters::prelude::*;
use crate::math::*;


#[derive(Serialize, Deserialize)]
pub struct Transition {
    pub name: String,
    pub input: Vec<i32>,
    pub output: Vec<i32>
}

#[derive(Copy,Clone)]
pub enum IO {
    Input,
    Output
}

impl IO {
    fn to_idx(self) -> usize {
        match self {
            IO::Input => 0,
            IO::Output => 1
        }
    }
}

impl fmt::Display for IO {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match *self {
            IO::Input => "Input",
            IO::Output => "Output"
        })
    }
}

pub static DIRECTIONS: [IO; 2] = [IO::Input, IO::Output];

impl Index<IO> for Transition {
    type Output = Vec<i32>;

    fn index(&self, index: IO) -> &Self::Output {
        match index {
            IO::Input => &self.input,
            IO::Output => &self.output
        }
    }
}

impl IndexMut<IO> for Transition {
    fn index_mut(&mut self, index: IO) -> &mut Self::Output {
        match index {
            IO::Input => &mut self.input,
            IO::Output => &mut self.output
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PlotControls {
    pub init_vals: Vec<f32>,
    pub rates: Vec<f32>,
    pub xmax: f32,
    pub ymax: f32,
    pub scale: f32
}

impl PlotControls {
    pub fn empty() -> Self {
        Self {
            init_vals: vec![],
            rates: vec![],
            xmax: 10.0,
            ymax: 5.0,
            scale: 5.0
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PetriNet {
    pub name: String,
    pub transitions: Vec<Transition>,
    pub species: Vec<String>
}

static STEPS_PER_UNIT: usize = 100;

static COLORS: [RGBColor; 4] = [RED, BLUE, YELLOW, GREEN];

fn get_color(i: usize) -> &'static RGBColor {
    &COLORS[i]
}


impl PetriNet {
    pub fn empty() -> Self {
        PetriNet {
            name: "".to_string(),
            transitions: vec![],
            species: vec![]
        }
    }
    
    fn get_petri_data(&self) -> PetriData {
        let (t,s) = (self.transitions.len(), self.species.len());
        let mut pd = Array::zeros((t,2,s));
        for (i,t) in self.transitions.iter().enumerate() {
            for d in DIRECTIONS.iter() {
                let j = d.to_idx();
                for k in 0..s {
                    pd[(i,j,k)] = t[*d][k];
                }
            }
        }
        PetriData(pd)
    }
    
    pub fn plot(&self, controls: &PlotControls, canvas_id: &str)
            -> DrawResult<(),CanvasBackend> {
        let steps = (STEPS_PER_UNIT as f32 * controls.xmax) as usize;
        let yvals = self
            .get_petri_data()
            .solve(&Array::from(controls.rates.clone()),
                   &Array::from(controls.init_vals.clone()),
                   controls.xmax,
                   steps);

        let backend = CanvasBackend::new(canvas_id).expect("cannot find canvas");
        let root = backend.into_drawing_area();
        let font: FontDesc = ("sans-serif", 20.0).into();

        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption(&self.name, font)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_ranged(0.0..controls.xmax, 0.0..controls.ymax)?;

        chart
            .configure_mesh()
            .disable_mesh()
            .draw()?;

        for (i,s) in self.species.iter().enumerate() {
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
        Ok(())
    }
}
