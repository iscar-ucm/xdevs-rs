use crate::Component;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

/// Interface for simulating DEVS models. All DEVS models must implement this trait.
pub trait Simulator: Debug {
    /// All the DEVS simulators must contain a [`Component`] struct.
    /// This method returns a reference to this inner [`Component`].
    fn get_component(&self) -> &Component;

    /// All the DEVS simulators must contain a [`Component`] struct.
    /// This method returns a mutable reference to this inner [`Component`].
    fn get_component_mut(&mut self) -> &mut Component;

    /// Returns the name of the inner DEVS [`Component`].
    fn get_name(&self) -> &str {
        self.get_component().get_name()
    }

    /// Returns the time for the last state transition of the inner DEVS [`Component`].
    fn get_t_last(&self) -> f64 {
        self.get_component().get_t_last()
    }

    /// Returns the time for the next state transition of the inner DEVS [`Component`].
    fn get_t_next(&self) -> f64 {
        self.get_component().get_t_next()
    }

    /// Helper function to allow DEVS models to check the current simulation time.
    fn get_time(&self) -> f64 {
        self.get_t_last()
    }

    fn set_sim_t(&mut self, t_last: f64, t_next: f64) {
        self.get_component_mut().set_sim_t(t_last, t_next);
    }

    /// It starts the simulation, setting the initial time to t_start.
    fn start(&mut self, t_start: f64);

    /// It stops the simulation, setting the last time to t_stop.
    fn stop(&mut self, t_stop: f64);

    /// Executes output functions and propagates messages according to ICs and EOCs.
    fn collection(&mut self, t: f64);

    /// Propagates messages according to EICs and executes model transition functions.
    fn transition(&mut self, t: f64);

    /// Removes all the messages from all the ports.
    fn clear_ports(&mut self);
}

/// Root coordinator for sequential simulations of DEVS models.
pub struct RootCoordinator<T>(T);

impl<T: Simulator> RootCoordinator<T> {
    /// Creates a new root coordinator from a DEVS-compliant model.
    pub fn new(model: T) -> Self {
        Self(model)
    }

    /// Runs a simulation for a given period of time.
    pub fn simulate_time(&mut self, t_end: f64) {
        self.start(0.);
        let mut t_next = self.get_t_next();
        while t_next < t_end {
            self.collection(t_next);
            self.transition(t_next);
            self.clear_ports();
            t_next = self.get_t_next();
        }
    }

    /// Runs a simulation for a given number of simulation cycles.
    pub fn simulate_steps(&mut self, mut n_steps: usize) {
        self.start(0.);
        let mut t_next = self.get_t_next();
        while t_next < f64::INFINITY && n_steps > 0 {
            self.collection(t_next);
            self.transition(t_next);
            self.clear_ports();
            t_next = self.get_t_next();
            n_steps -= 1;
        }
    }
}

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
