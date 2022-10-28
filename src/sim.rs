pub mod root_coordinator;

/// Basic simulator struct. All models have one of these inside to keep track of time.
#[derive(Clone, Debug)]
pub struct Simulator {
    /// Time for the latest model state transition.
    pub t_last: f64,
    /// Time for the next model state transition.
    pub t_next: f64,
}

impl Simulator {
    /// It creates a new simulator with default values.
    pub fn new() -> Self {
        Self {
            t_last: 0.,
            t_next: f64::INFINITY,
        }
    }
}

impl Default for Simulator {
    fn default() -> Self {
        Self::new()
    }
}

/// Models must implement this trait in order to be simulated.
pub trait AbstractSimulator {
    /// It starts the simulation, setting the initial time to t_start.
    fn start(&mut self, t_start: f64);
    /// It stops the simulation, setting the last time to t_stop.
    fn stop(&mut self, t_stop: f64);
    /// It returns the time of the latest model state transition.
    fn t_last(&self) -> f64;
    /// It returns the time of the next model state transition.
    fn t_next(&self) -> f64;
    /// Executes output functions and propagates messages according to ICs and EOCs.
    fn collection(&mut self, t: f64);
    /// Propagates messages according to EICs and executes model transition functions.
    fn transition(&mut self, t: f64);
    /// Removes all the messages from all the ports.
    fn clear(&mut self);
}

/// Blanket implementation of AbstractSimulator for boxed AbstractSimulator trait objects.
impl<T: AbstractSimulator + ?Sized> AbstractSimulator for Box<T> {
    fn start(&mut self, t_start: f64) {
        (**self).start(t_start);
    }

    fn stop(&mut self, t_stop: f64) {
        (**self).stop(t_stop);
    }

    fn t_last(&self) -> f64 {
        (**self).t_last()
    }

    fn t_next(&self) -> f64 {
        (**self).t_next()
    }

    fn collection(&mut self, t: f64) {
        (**self).collection(t);
    }

    fn transition(&mut self, t: f64) {
        (**self).transition(t);
    }

    fn clear(&mut self) {
        (**self).clear();
    }
}
