use crate::AsModel;

/// Basic simulator struct. All models have one of these inside to keep track of time.
#[derive(Clone, Debug)]
pub struct Clock {
    /// Time for the latest model state transition.
    pub t_last: f64,
    /// Time for the next model state transition.
    pub t_next: f64,
}

impl Clock {
    /// It creates a new simulator with default values.
    pub fn new() -> Self {
        Self {
            t_last: 0.,
            t_next: f64::INFINITY,
        }
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RootCoordinator<T> {
    model: T
}

impl<T: AsModel> RootCoordinator<T> {
    pub fn new(model: T) -> Self {
        Self {model}
    }

    pub fn simulate_time(&mut self, t_end: f64) {
        self.model.start_simulation(0.);
        let mut t_next = self.model.get_t_next();
        while t_next < t_end {
            self.model.lambda(t_next);
            self.model.delta(t_next);
            self.model.clear_ports();
            t_next = self.model.get_t_next();
        }
    }
}
