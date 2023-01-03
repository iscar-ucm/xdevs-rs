use crate::modeling::{Atomic, Component, Coupled};
use crate::DynRef;
#[cfg(feature = "parallel")]
use rayon::prelude::*;
use std::ops::{Deref, DerefMut};

/// Interface for simulating DEVS models. All DEVS models must implement this trait.
pub trait Simulator: DynRef {
    /// Returns reference to inner [`Component`].
    fn get_component(&self) -> &Component;

    /// Returns mutable reference to inner [`Component`].
    fn get_component_mut(&mut self) -> &mut Component;

    /// Returns the name of the inner DEVS [`Component`].
    #[inline]
    fn get_name(&self) -> &str {
        self.get_component().get_name()
    }

    /// Returns the time for the last state transition of the inner DEVS [`Component`].
    #[inline]
    fn get_t_last(&self) -> f64 {
        self.get_component().get_t_last()
    }

    /// Returns the time for the next state transition of the inner DEVS [`Component`].
    #[inline]
    fn get_t_next(&self) -> f64 {
        self.get_component().get_t_next()
    }

    /// Sets the tine for the last and next state transitions of the inner DEVS [`Component`].
    #[inline]
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

impl<T: Atomic + DynRef> Simulator for T {
    #[inline]
    fn get_component(&self) -> &Component {
        Atomic::get_component(self)
    }

    #[inline]
    fn get_component_mut(&mut self) -> &mut Component {
        Atomic::get_component_mut(self)
    }

    #[inline]
    fn start(&mut self, t_start: f64) {
        Atomic::start(self);
        let ta = self.ta();
        self.set_sim_t(t_start, t_start + ta);
    }

    #[inline]
    fn stop(&mut self, t_stop: f64) {
        self.set_sim_t(t_stop, f64::INFINITY);
        Atomic::stop(self);
    }

    #[inline]
    fn collection(&mut self, t: f64) {
        if t >= self.get_t_next() {
            Atomic::lambda(self)
        }
    }

    fn transition(&mut self, t: f64) {
        let t_next = self.get_t_next();
        if !self.get_component().is_input_empty() {
            if t == t_next {
                Atomic::delta_conf(self);
            } else {
                let e = t - self.get_time();
                Atomic::delta_ext(self, e);
            }
        } else if t == t_next {
            Atomic::delta_int(self);
        } else {
            return;
        }
        let ta = Atomic::ta(self);
        self.set_sim_t(t, t + ta);
    }

    #[inline]
    fn clear_ports(&mut self) {
        let component = self.get_component_mut();
        component.clear_input();
        component.clear_output()
    }
}

impl Simulator for Coupled {
    #[inline]
    fn get_component(&self) -> &Component {
        &self.component
    }

    #[inline]
    fn get_component_mut(&mut self) -> &mut Component {
        &mut self.component
    }

    fn start(&mut self, t_start: f64) {
        let mut t_next = f64::INFINITY;
        for component in self.components.iter_mut() {
            component.start(t_start);
            let t = component.get_t_next();
            if t < t_next {
                t_next = t;
            }
        }
        self.set_sim_t(t_start, t_next);
    }

    #[inline]
    fn stop(&mut self, t_stop: f64) {
        self.components.iter_mut().for_each(|c| c.stop(t_stop));
        self.set_sim_t(t_stop, f64::INFINITY);
    }

    fn collection(&mut self, t: f64) {
        if t >= self.get_t_next() {
            self.components.iter_mut().for_each(|c| c.collection(t));
            self.ics
                .iter()
                .for_each(|(p_from, p_to)| p_from.propagate(&**p_to));
            self.eocs
                .iter()
                .for_each(|(p_from, p_to)| p_from.propagate(&**p_to));
        }
    }

    fn transition(&mut self, t: f64) {
        self.eics
            .iter()
            .for_each(|(p_from, p_to)| p_from.propagate(&**p_to));
        #[cfg(not(feature = "parallel"))]
        let iterator = self.components.iter_mut();
        #[cfg(feature = "parallel")]
        let iterator = self.components.par_iter_mut();
        let next_t = iterator
            .map(|c| {
                c.transition(t);
                c.get_t_next()
            })
            .min_by(|a, b| a.total_cmp(b))
            .unwrap_or(f64::INFINITY);
        self.set_sim_t(t, next_t);
    }

    #[inline]
    fn clear_ports(&mut self) {
        self.components.iter_mut().for_each(|c| c.clear_ports());
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
