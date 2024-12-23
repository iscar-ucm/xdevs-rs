use crate::{
    modeling::{Atomic, Component, Coupled},
    DynRef,
};
#[cfg(feature = "par_any")]
use rayon::prelude::*;
use std::ops::{Deref, DerefMut};

pub mod rt;

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

    /// Clears input messages from the inner DEVS [`Component`]s.
    #[inline]
    fn clear_input(&mut self) {
        // Safety: simulator clearing its input
        unsafe { self.get_component_mut().clear_input() };
    }

    /// Clears output messages from the inner DEVS [`Component`]s.
    #[inline]
    fn clear_output(&mut self) {
        // Safety: simulator clearing its output
        unsafe { self.get_component_mut().clear_output() };
    }

    /// Removes all the messages from all the ports.
    #[inline]
    fn clear(&mut self) {
        let component = self.get_component_mut();
        // Safety: simulator clearing its ports
        unsafe {
            component.clear_input();
            component.clear_output();
        }
    }

    /// It starts the simulation, setting the initial time to t_start.
    fn start(&mut self, t_start: f64) -> f64;

    /// It stops the simulation, setting the last time to t_stop.
    fn stop(&mut self, t_stop: f64);

    /// Executes output functions and propagates messages according to ICs and EOCs.
    fn collection(&mut self, t: f64);

    /// Propagates messages according to EICs and executes model transition functions.
    fn transition(&mut self, t: f64) -> f64;
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

    fn start(&mut self, t_start: f64) -> f64 {
        Atomic::start(self);
        let t_next = t_start + self.ta();
        self.set_sim_t(t_start, t_next);
        t_next
    }

    fn stop(&mut self, t_stop: f64) {
        self.set_sim_t(t_stop, f64::INFINITY);
        Atomic::stop(self);
    }

    fn collection(&mut self, t: f64) {
        if t >= self.get_t_next() {
            Atomic::lambda(self)
        }
    }

    fn transition(&mut self, t: f64) -> f64 {
        let t_next = self.get_t_next();
        // Safety: simulator executing its transition function
        if !unsafe { self.get_component().is_input_empty() } {
            if t == t_next {
                Atomic::delta_conf(self);
                self.clear_output();
            } else {
                let e = t - self.get_t_last();
                Atomic::delta_ext(self, e);
            }
            self.clear_input();
        } else if t == t_next {
            Atomic::delta_int(self);
            self.clear_output();
        } else {
            return t_next;
        }
        let t_next = t + Atomic::ta(self);
        self.set_sim_t(t, t_next);
        t_next
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

    /// Iterates over all the subcomponents to call their [`Simulator::start`]
    /// method and obtain the next simulation time.
    ///
    /// If the feature `par_start` is activated, the iteration is parallelized.
    ///
    /// If the feature `par_couplings` is activated, the EICs, EOCs, and ICs are built
    /// for enabling parallel event propagation.
    fn start(&mut self, t_start: f64) -> f64 {
        #[cfg(feature = "par_start")]
        let iter = self.components.par_iter_mut();
        #[cfg(not(feature = "par_start"))]
        let iter = self.components.iter_mut();
        // we obtain the minimum next time of all the subcomponents
        let t_next = iter
            .map(|c| c.start(t_start))
            .min_by(|a, b| a.total_cmp(b))
            .unwrap_or(f64::INFINITY);
        // and set the inner component's last and next times
        self.set_sim_t(t_start, t_next);

        #[cfg(feature = "par_couplings")]
        {
            self.build_par_eics();
            self.build_par_eocs();
            self.build_par_ics();
        }

        t_next
    }

    /// Iterates over all the subcomponents to call their [`Simulator::stop`]
    /// method and obtain the next simulation time.
    ///
    /// If the feature `par_stop` is activated, the iteration is parallelized.
    fn stop(&mut self, t_stop: f64) {
        #[cfg(feature = "par_stop")]
        let iter = self.components.par_iter_mut();
        #[cfg(not(feature = "par_stop"))]
        let iter = self.components.iter_mut();
        iter.for_each(|c| c.stop(t_stop));
        // we set the inner component's last and next times accordingly
        self.set_sim_t(t_stop, f64::INFINITY);
    }

    /// Iterates over all the subcomponents to call their [`Simulator::collection`] method.
    /// Then, it iterates over all the EOCs and ICs and propagates messages accordingly.
    ///
    /// If the feature `par_collection` is activated, the iteration over subcomponents is parallelized.
    /// If the feature `par_couplings` is activated, the iteration is over couplings is parallelized.
    fn collection(&mut self, t: f64) {
        if t >= self.get_t_next() {
            #[cfg(feature = "par_collection")]
            let iter = self.components.par_iter_mut();
            #[cfg(not(feature = "par_collection"))]
            let iter = self.components.iter_mut();
            iter.for_each(|c| c.collection(t));

            #[cfg(feature = "par_couplings")]
            self.par_xxcs.par_iter().for_each(|coups| {
                coups.iter().for_each(|(port_to, port_from)| {
                    // Safety: coupled model propagating messages
                    unsafe { port_from.propagate(&**port_to) };
                });
            });

            #[cfg(not(feature = "par_couplings"))]
            {
                self.eocs.iter().for_each(|(port_to, port_from)| {
                    // Safety: coupled model propagating messages
                    unsafe { port_from.propagate(&**port_to) };
                });
                self.ics.iter().for_each(|(port_to, port_from)| {
                    // Safety: coupled model propagating messages
                    unsafe { port_from.propagate(&**port_to) };
                });
            }
        }
    }

    /// Iterates over all the EICs and propagates messages accordingly.
    /// Then, it iterates over all the subcomponents to:
    /// 1. Call their [`Simulator::transition`] method
    /// 2. Clear their ports
    /// 3. Obtain their next simulation time.
    ///
    /// If the feature `par_couplings` is activated, the iteration over EICs is parallelized.
    /// If the feature `par_transition` is activated, the iteration over subcomponents is parallelized.
    fn transition(&mut self, t: f64) -> f64 {
        // Safety: simulator checking if its input is empty
        let is_external = !unsafe { self.get_component().is_input_empty() };
        // Propagate messages according to EICs only if there are messages in the input ports
        if is_external {
            #[cfg(feature = "par_couplings")]
            self.par_eics.par_iter().for_each(|coups| {
                coups.iter().for_each(|(port_to, port_from)| {
                    // Safety: coupled model propagating messages
                    unsafe { port_from.propagate(&**port_to) };
                });
            });
            #[cfg(not(feature = "par_couplings"))]
            self.eics.iter().for_each(|(port_to, port_from)| {
                // Safety: coupled model propagating messages
                unsafe { port_from.propagate(&**port_to) };
            });
            self.clear_input();
        }
        let is_internal = t >= self.get_t_next();
        if is_internal {
            self.clear_output();
        }
        // Nested call only if there are messages in the input ports or if the time has come
        if is_external || is_internal {
            #[cfg(feature = "par_transition")]
            let iterator = self.components.par_iter_mut();
            #[cfg(not(feature = "par_transition"))]
            let iterator = self.components.iter_mut();
            let t_next = iterator
                .map(|c| c.transition(t))
                .min_by(|a, b| a.total_cmp(b))
                .unwrap_or(f64::INFINITY);
            self.set_sim_t(t, t_next);
        }
        self.get_t_next()
    }
}

/// Root coordinator for sequential simulations of DEVS models.
#[repr(transparent)]
pub struct RootCoordinator<T>(T);

impl<T: Simulator> RootCoordinator<T> {
    /// Creates a new root coordinator from a DEVS-compliant model.
    pub fn new(model: T) -> Self {
        Self(model)
    }

    /// Runs a simulation for a given period of time.
    pub fn simulate(&mut self, t_stop: f64) {
        let mut t_next = self.start(0.);
        while t_next < t_stop {
            self.collection(t_next);
            t_next = self.transition(t_next);
        }
        self.stop(t_next);
    }

    /// Runs a Real-Time (RT) simulation for a given period of time.
    ///
    /// # The `wait_event` closure
    ///
    /// RT behavior must be implemented in the `wait_event` closure.
    /// The `wait_event` closure behaves as a function that accepts two arguments:
    ///
    /// - `t_next: f64` - The next simulation time in which a model state transition is expected.
    /// - `component: &T` - A reference to the model under study.
    ///
    /// It must return the simulation time when the next event happens.
    ///
    ///  If no external event is received, the `wait_event` closure waits until the next internal
    /// event and returns `t_next`, which now is the current simulation time. We provide an
    /// [`rt::sleep`] closure that can be used as a `wait_event` closure for this scenario.
    ///
    /// When external events may occur, the behavior of the `wait_event` closure is more complex.
    /// Now, the `wait_event` closure must wait for external events without exceeding the time for
    /// the next internal event. If an external event is received, the `wait_event` closure must
    /// inject the events into the model's input ports and return the simulation time when the event
    /// was received. We provide an [`rt::wait_event`] closure that can be used as a `wait_event`
    ///
    /// Under no circumstances should the `wait_event` closure return a value higher than `t_next`.
    ///
    /// # The `output_handler` closure
    ///
    /// The `output_handler` closure behaves as a function that accepts a reference to the model
    /// under study and propagates the output messages to external entities.
    /// Every time the model  generates output messages, the `output_handler` closure is called.
    /// Implementation details of the `output_handler` closure depend on the specific use case.
    pub fn simulate_rt(
        &mut self,
        t_stop: f64,
        mut wait_event: impl FnMut(f64, &T) -> f64,
        mut output_handler: impl FnMut(&T),
    ) {
        let mut t = 0.;
        let mut t_next_internal = self.start(t);
        while t < t_stop {
            let t_until = f64::min(t_next_internal, t_stop);
            t = wait_event(t_until, self);
            if t >= t_next_internal {
                self.collection(t);
                output_handler(self);
            } else if unsafe { self.get_component().is_input_empty() } {
                continue; // avoid spurious external transitions
            }
            t_next_internal = self.transition(t);
        }
        self.stop(t_stop);
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
