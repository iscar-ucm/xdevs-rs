use crate::{Coupled, AbstractSimulator};

pub struct RootCoordinator {
    model: Coupled
}

impl RootCoordinator {
    pub fn new(model: Coupled) -> Self {
        Self {model}
    }

    pub fn simulate_time(&mut self, t_end: f64) {
        self.model.start(0.);
        let mut t_next = self.model.t_next();
        while t_next < t_end {
            self.model.collection(t_next);
            self.model.transition(t_next);
            self.model.clear();
            t_next = self.model.t_next();
        }
    }
}
