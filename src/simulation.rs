use crate::AsModel;
use std::ops::{Deref, DerefMut};

/// Just a wrapper for a model. In xDEVS Rust, models already implement simulation-related stuff!
pub struct RootCoordinator<T>(T);

impl<T> Deref for RootCoordinator<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for RootCoordinator<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: AsModel> RootCoordinator<T> {
    pub fn new(model: T) -> Self {
        Self(model)
    }

    /// Runs a simulation for a given period of time.
    pub fn simulate_time(&mut self, t_end: f64) {
        self.start_simulation(0.);
        let mut t_next = self.as_model().t_next();
        while t_next < t_end {
            self.lambda(t_next);
            self.delta(t_next);
            self.clear_ports();
            t_next = self.as_model().t_next();
        }
    }

    /// Runs a simulation for a given number of simulation cycles.
    pub fn simulate_steps(&mut self, mut n_steps: usize) {
        self.start_simulation(0.);
        let mut t_next = self.as_model().t_next();
        while t_next < f64::INFINITY && n_steps > 0 {
            self.lambda(t_next);
            self.delta(t_next);
            self.clear_ports();
            t_next = self.as_model().t_next();
            n_steps -= 1;
        }
    }
}
