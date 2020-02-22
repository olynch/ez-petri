use rand::{Rng};
use rand::distributions::{Distribution};
use statrs::distribution::{Exponential, Categorical};
use ndarray::prelude::*;
use crate::math::*;
use std::fmt;

#[derive(Clone)]
pub struct PetriState {
    state: Array<i32, Ix1>,
    time_elapsed: f64
}

impl fmt::Debug for PetriState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PetriState{{ state: {:?}, time_elapsed: {} }}",self.state.as_slice().unwrap(),self.time_elapsed)
    }
}

fn falling_exp(n: i32, k: i32) -> i32 {
    if (n == 0) && (k == 0) {
        1
    } else if (k > n) || (n <= 0) || (k < 0) {
        0
    } else {
        ((n-k+1)..=n).product()
    }
}

impl PetriState {
    fn transition_rate(&self, petri: &PetriData, rates: &Array<f64, Ix1>, t: usize) -> f64 {
        let n = petri.num_species();
        let combinations: i32 = (0..n)
            .map(|s| { falling_exp(self.state[s],petri.data()[(t,0,s)]) })
            .product();
        (combinations as f64) * rates[t]
    }

    fn apply_transition(&mut self, petri: &PetriData, t: usize) {
        let d = petri.data();
        for s in 0..petri.num_species() {
            self.state[s] += - d[(t,0,s)] + d[(t,1,s)];
        }
    }


    fn step<R: Rng + ?Sized>(&mut self, petri: &PetriData, rates: &Array<f64,Ix1>, rng: &mut R) {
        let m = rates.len();
        let flow_out: Array<f64,Ix1> = (0..m).map(|t| self.transition_rate(petri,rates,t)).collect();
        let total_flow_out = flow_out.sum();
        let probabilities = flow_out / total_flow_out;
        let time_dist = Exponential::new(total_flow_out).unwrap();
        let dt = time_dist.sample(rng);
        let transition_dist = Categorical::new(probabilities.as_slice().unwrap()).unwrap();
        let transition: f64 = transition_dist.sample(rng);
        self.apply_transition(petri,transition as usize);
        self.time_elapsed += dt;
    }
}

pub fn sample_extinction_time(initial_state: &Array<i32,Ix1>, petri: &PetriData, rates: &Array<f64,Ix1>, species: &[usize]) -> f64 {
    let mut rng = rand::thread_rng();
    let mut ps = PetriState {
        state: initial_state.clone(),
        time_elapsed: 0.0
    };
    'outer: loop {
        for s in species.iter() {
            if ps.state[*s] <= 0 {
                break 'outer;
            }
        }
        ps.step(petri,rates,&mut rng);
    }
    ps.time_elapsed
}
