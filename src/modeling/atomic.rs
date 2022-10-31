use super::component::Component;
use std::fmt::Debug;

/// Interface for atomic DEVS models.
pub trait Atomic: Debug {
    /// Returns reference to inner component.
    fn get_component(&self) -> &Component;

    /// Returns mutable reference to inner component.
    fn get_component_mut(&mut self) -> &mut Component;

    /// Returns current simulation time.
    fn get_time(&self) -> f64 {
        self.get_component().get_t_last()
    }

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

/*
pub fn start<T: AtomicModel + Simulator>(this: &mut T, t_start: f64) {
    let ta = this.ta();
    this.set_sim_t(t_start, t_start + ta);
}

pub fn stop<T: AtomicModel + Simulator>(this: &mut T, t_stop: f64) {
    this.set_sim_t(t_stop, f64::INFINITY);
}

pub fn collection<T: AtomicModel + Simulator>(this: &mut T, t: f64) {
    if t >= this.get_t_next() {
        this.lambda();
    }
}

pub fn transition<T: AtomicModel + Simulator>(this: &mut T, t: f64) {
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

pub fn clear_ports<T: AtomicModel + Simulator>(this: &mut T) {
    this.get_component_mut().clear_ports();
}
 */

/// Helper macro to implement the [`Atomic`] trait.
#[macro_export]
macro_rules! impl_atomic {
    ($($ATOMIC:ident),+) => {
        $(
            impl $crate::modeling::atomic::Atomic for $ATOMIC {
                fn get_component(&self) -> &Component { &self.component }
                fn get_component_mut(&mut self) -> &mut Component { &mut self.component }
                fn lambda(&self) { self.lambda(); }
                fn delta_int(&mut self) { self.delta_int() }
                fn delta_ext(&mut self, e: f64) { self. delta_ext(e) }
                fn ta(&self) -> f64 { self.ta() }
            }
        )+
    }
}
