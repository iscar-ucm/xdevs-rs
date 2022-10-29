use crate::Simulator;
use std::fmt::Debug;

/// Interface for atomic DEVS models.
pub trait Atomic: Debug {
    /// Output function of the atomic DEVS model.
    fn lambda(&self);

    /// Internal transition function of the atomic DEVS model.
    fn delta_int(&mut self);

    /// External transition function of the atomic DEVS model.
    /// `e` corresponds to the elapsed time since the last state transition of the model.
    fn delta_ext(&mut self, e: f64);

    /// Time advance function of the atomic DEVS model.
    fn ta(&self) -> f64;

    /// Confluent transition function of the atomic DEVS model.
    /// By default, it first triggers [`Atomic::delta_int`].
    /// Then, it triggers [`Atomic::delta_ext`] with elapsed time 0.
    fn delta_conf(&mut self) {
        self.delta_int();
        self.delta_ext(0.);
    }
}

pub fn start<T: Atomic + Simulator>(this: &mut T, t_start: f64) {
    let ta = this.ta();
    this.set_sim_t(t_start, t_start + ta);
}

pub fn stop<T: Atomic + Simulator>(this: &mut T, t_stop: f64) {
    this.set_sim_t(t_stop, f64::INFINITY);
}

pub fn collection<T: Atomic + Simulator>(this: &mut T, t: f64) {
    if t >= this.get_t_next() {
        this.lambda();
    }
}

pub fn transition<T: Atomic + Simulator>(this: &mut T, t: f64) {
    let t_next = this.get_t_next();
    if !this.get_component().is_input_empty() {
        if t == t_next {
            this.delta_conf();
        } else {
            let e = t - this.get_time();
            this.delta_ext(e);
        }
    } else if t == t_next {
        this.delta_int();
    } else {
        return;
    }
    let ta = this.ta();
    this.set_sim_t(t, t + ta)
}

pub fn clear_ports<T: Atomic + Simulator>(this: &mut T) {
    this.get_component_mut().clear_ports();
}

/// Helper macro to implement the [`Atomic`] trait. TODO try to use the derive stuff
#[macro_export]
macro_rules! impl_atomic {
    ($($ATOMIC:ident),+) => {
        $(
            impl Atomic for $ATOMIC {
                fn lambda(&self) { self.lambda(); }
                fn delta_int(&mut self) { self.delta_int() }
                fn delta_ext(&mut self, e: f64) { self. delta_ext(e) }
                fn ta(&self) -> f64 { self.ta() }
            }
            impl Simulator for $ATOMIC {
                fn get_component(&self) -> &Component { &self.component }
                fn get_component_mut(&mut self) -> &mut Component { &mut self.component }
                fn start(&mut self, t_start: f64) { $crate::modeling::atomic::start(self, t_start); }
                fn stop(&mut self, t_stop: f64) { $crate::modeling::atomic::stop(self, t_stop); }
                fn collection(&mut self, t: f64) { $crate::modeling::atomic::collection(self, t); }
                fn transition(&mut self, t: f64) { $crate::modeling::atomic::transition(self, t); }
                fn clear_ports(&mut self) { $crate::modeling::atomic::clear_ports(self); }
            }
        )+
    }
}
