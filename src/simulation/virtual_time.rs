use super::Simulator;

/// Root coordinator for sequential simulations of DEVS models.
pub struct RootCoordinator<T>(T);

impl<T: Simulator> RootCoordinator<T> {
    /// Creates a new root coordinator from a DEVS-compliant model.
    pub fn new(model: T) -> Self {
        Self(model)
    }

    /// Runs a simulation for a given period of time.
    pub fn simulate(&mut self, t_end: f64) {
        self.0.start(0.);
        let mut t_next = self.0.get_t_next();
        while t_next < t_end {
            self.0.collection(t_next);
            self.0.transition(t_next);
            t_next = self.0.get_t_next();
        }
        self.0.stop(t_next);
    }
}
