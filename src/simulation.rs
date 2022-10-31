use crate::modeling::atomic::Atomic;
use crate::modeling::coupled::Coupled;
use crate::modeling::Component;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

/// Interface for simulating DEVS models. All DEVS models must implement this trait.
pub trait Simulator: Debug {
    /// Returns reference to inner [`Component`].
    fn get_component(&self) -> &Component;

    /// Returns mutable reference to inner [`Component`].
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

    /// Sets the tine for the last and next state transitions of the inner DEVS [`Component`].
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

impl<T: Atomic> Simulator for T {
    fn get_component(&self) -> &Component {
        self.get_component()
    }

    fn get_component_mut(&mut self) -> &mut Component {
        self.get_component_mut()
    }

    fn start(&mut self, t_start: f64) {
        Atomic::start(self);
        let ta = self.ta();
        self.set_sim_t(t_start, t_start + ta);
    }

    fn stop(&mut self, t_stop: f64) {
        self.set_sim_t(t_stop, f64::INFINITY);
        Atomic::stop(self);
    }

    fn collection(&mut self, t: f64) {
        if t >= self.get_t_next() {
            self.lambda();
        }
    }

    fn transition(&mut self, t: f64) {
        let t_next = self.get_t_next();
        if !self.get_component().is_input_empty() {
            if t == t_next {
                self.delta_conf();
            } else {
                let e = t - self.get_time();
                self.delta_ext(e);
            }
        } else if t == t_next {
            self.delta_int();
        } else {
            return;
        }
        let ta = self.ta();
        self.set_sim_t(t, t + ta);
    }

    fn clear_ports(&mut self) {
        let component = self.get_component_mut();
        component.clear_input();
        component.clear_output()
    }
}

impl Simulator for Coupled {
    fn get_component(&self) -> &Component {
        &self.component
    }

    fn get_component_mut(&mut self) -> &mut Component {
        &mut self.component
    }

    fn start(&mut self, t_start: f64) {
        let mut t_next = f64::INFINITY;
        for component in self.comps_vec.iter_mut() {
            component.start(t_start);
            let t = component.get_t_next();
            if t < t_next {
                t_next = t;
            }
        }
        self.set_sim_t(t_start, t_next);
    }

    fn stop(&mut self, t_stop: f64) {
        self.comps_vec.iter_mut().for_each(|c| c.stop(t_stop));
        self.set_sim_t(t_stop, f64::INFINITY);
    }

    fn collection(&mut self, t: f64) {
        if t >= self.get_t_next() {
            self.comps_vec.iter_mut().for_each(|c| c.collection(t));
            self.ic_vec
                .iter()
                .for_each(|(port_from, port_to)| port_to.propagate(&**port_from));
            self.eoc_vec
                .iter()
                .for_each(|(port_from, port_to)| port_to.propagate(&**port_from));
        }
    }

    fn transition(&mut self, t: f64) {
        self.eic_vec
            .iter()
            .for_each(|(port_from, port_to)| port_to.propagate(&**port_from));
        let mut next_t = f64::INFINITY;
        for component in self.comps_vec.iter_mut() {
            component.transition(t);
            let t = component.get_t_next();
            if t < next_t {
                next_t = t;
            }
        }
        self.set_sim_t(t, next_t);
    }

    fn clear_ports(&mut self) {
        self.comps_vec.iter_mut().for_each(|c| c.clear_ports());
        self.component.clear_output();
        self.component.clear_input()
    }
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
        self.stop(t_next);
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
        self.stop(t_next);
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
