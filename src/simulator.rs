use crate::AsModel;
use std::ops::{Deref, DerefMut};

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

    pub fn simulate_time(&mut self, t_end: f64) {
        self.start_simulation(0.);
        let mut t_next = self.as_model().get_t_next();
        while t_next < t_end {
            self.lambda(t_next);
            self.delta(t_next);
            self.clear_ports();
            t_next = self.as_model().get_t_next();
        }
    }
}
