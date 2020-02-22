use ndarray::prelude::*;

pub type V = Array<f32, Ix1>;
pub type YVals = Array<f32, Ix2>;

fn rk_step(f: impl Fn(f32, &V) -> V, t: f32, dt: f32, y: &V) -> V {
    let k1: V = f(t, y);
    let k2 = f(t + 0.5 * dt, &(y + &(&k1 * dt * 0.5)));
    let k3 = f(t + 0.5 * dt, &(y + &(&k2 * dt * 0.5)));
    let k4 = f(t + dt, &(y + &(&k3 * dt)));
    y + &((dt / 6.) * (&k1 + &(k2 * 2.) + &(k3 * 2.) + &k4))
}

pub fn rk_solve(f: impl Fn(f32, &V) -> V, t0: f32, t1: f32, steps: usize, y0: &V) -> YVals {
    let dt = (t1 - t0) / (steps as f32);
    let mut yvals = Array::zeros((steps, y0.len()));
    let mut y = y0.clone();
    for (i, mut cell) in yvals.axis_iter_mut(Axis(0)).enumerate() {
        let t = t0 + (i as f32) / (steps as f32) * (t1 - t0);
        y = rk_step(&f, t, dt, &y);
        cell.assign(&y);
    }
    yvals
}

pub struct PetriData(pub Array<i32, Ix3>);

impl PetriData {
    pub fn data(&self) -> &Array<i32, Ix3> {
        match self {
            PetriData(d) => d,
        }
    }

    pub fn num_species(&self) -> usize {
        self.data().len_of(Axis(2))
    }

    fn master_eq(&self, rates: &V, y: &V) -> V {
        let d = self.data();
        let mut yp = Array::zeros(y.dim());
        let n = self.num_species();
        for (i, trans) in d.axis_iter(Axis(0)).enumerate() {
            let r = rates[i];
            for j in 0..n {
                let coeff = r * ((trans[(1, j)] - trans[(0, j)]) as f32);
                let mut term: f32 = 1.;
                for (k, pop) in y.iter().enumerate() {
                    term *= pop.powi(trans[(0, k)]);
                }
                yp[j] += coeff * term;
            }
        }
        yp
    }

    pub fn solve(&self, rates: &V, init_vals: &V, t1: f32, steps: usize) -> YVals {
        rk_solve(|_, v| self.master_eq(rates, v), 0.0, t1, steps, init_vals)
    }
}
