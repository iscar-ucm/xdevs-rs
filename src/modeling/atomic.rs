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
    /// Then, it triggers [`Atomic::delta_ext`] with the elapsed time set to 0.
    fn delta_conf(&mut self) {
        self.delta_int();
        self.delta_ext(0.);
    }
}

/// Helper macro to implement the [`Atomic`] trait using features from its inner [`Component`].
/// To implement the [`Atomic`] trait for your struct `MyAtomic`, call `impl_atomic!(MyAtomic)`.
/// You must ensure the following conditions:
/// - `MyAtomic` has a field `component` of type [`Component`].
/// - `MyAtomic` has the following methods:
///     - `lambda(&self)` (i.e., output function).
///     - `delta_int(&mut self)` (i.e., internal transition function).
///     - `delta_ext(&mut self, e: f64)` (i.e., external transition function).
///     - `ta(&self)` (i.e., time advance function).
/// Currently, the confluent transition function can only be the default.
#[macro_export]
macro_rules! impl_atomic {
    ($ATOMIC:ident) => {
        impl $crate::modeling::atomic::Atomic for $ATOMIC {
            fn get_component(&self) -> &Component {
                &self.component
            }
            fn get_component_mut(&mut self) -> &mut Component {
                &mut self.component
            }
            fn lambda(&self) {
                self.lambda();
            }
            fn delta_int(&mut self) {
                self.delta_int()
            }
            fn delta_ext(&mut self, e: f64) {
                self.delta_ext(e)
            }
            fn ta(&self) -> f64 {
                self.ta()
            }
        }
    };
}
