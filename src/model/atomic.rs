use crate::AsModel;

/// Interface for atomic DEVS models.
pub trait AsAtomic: AsModel {
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
    /// By default, it first triggers [`AsAtomic::delta_int`].
    /// Then, it triggers [`AsAtomic::delta_ext`] with elapsed time 0.
    fn delta_conf(&mut self) {
        self.delta_int();
        self.delta_ext(0.);
    }
}

/// Helper macro to implement the AsModel trait.
/// You can use this macro with any struct containing a field `model` of type [`Model`].
/// TODO try to use the derive stuff (it will be more elegant).
#[macro_export]
macro_rules! impl_atomic {
    ($($ATOMIC:ident),+) => {
        // use $crate::{AbstractSimulator, AsAtomic, AsModel};
        $(
            impl AsModel for $ATOMIC {
                fn as_model(&self) -> &Model { &self.model }
                fn as_model_mut(&mut self) -> &mut Model { &mut self.model }
            }
            impl AsAtomic for $ATOMIC {
                fn lambda(&self) { self.lambda(); }
                fn delta_int(&mut self) { self.delta_int() }
                fn delta_ext(&mut self, e: f64) { self. delta_ext(e) }
                fn ta(&self) -> f64 { self.ta() }
            }
            impl AbstractSimulator for $ATOMIC {
                fn start(&mut self, t_start: f64) {
                    self.as_model_mut().reset_simulator(t_start);
                    self.as_model_mut().simulator.t_next = t_start + self.ta()
                }

                fn stop(&mut self, t_stop: f64) {
                    self.as_model_mut().reset_simulator(t_stop);
                }

                fn t_last(&self) -> f64 {
                    self.as_model().simulator.t_last
                }

                fn t_next(&self) -> f64 {
                    self.as_model().simulator.t_next
                }

                fn collection(&mut self, t: f64) {
                    if t >= self.t_next() {
                        self.lambda();
                    }
                }

                fn transition(&mut self, t: f64) {
                    if !self.is_input_empty() {
                        if t == self.t_next() {
                            self.delta_conf();
                        } else {
                            let e = t - self.t_last();
                            self.delta_ext(e);
                        }
                    } else if t == self.t_next() {
                        self.delta_int();
                    } else {
                        return;
                    }
                    let ta = self.ta();
                    let simulator = &mut self.as_model_mut().simulator;
                    simulator.t_last = t;
                    simulator.t_next = t + ta;
                }

                fn clear(&mut self) {
                    self.clear_in_ports();
                    self.clear_out_ports();
                }
            }
        )+
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[derive(Debug)]
    struct TestAtomic {
        // We need to have a Component for composition.
        model: Model,
        // We add all the state-related fields.
        n_delta_int: i32,
        n_delta_ext: i32,
        sigma: f64,
        // We add all the in/out ports of the model.
        in_port: Shared<Port<i32>>,
        out_port: Shared<Port<i32>>,
    }

    impl TestAtomic {
        fn new(name: &str) -> Self {
            let mut component = Model::new(name);
            Self {
                n_delta_ext: 0,
                n_delta_int: 0,
                sigma: f64::INFINITY,
                in_port: component.add_in_port::<i32>("in_port"),
                out_port: component.add_out_port::<i32>("out_port"),
                model: component,
            }
        }

        fn add_input_value(&self, value: i32) {
            self.in_port.add_value(value);
        }

        fn lambda(&self) {
            self.out_port.add_value(self.n_delta_ext + self.n_delta_int);
        }

        fn delta_int(&mut self) {
            self.n_delta_int += 1;
            self.sigma = f64::INFINITY;
        }

        fn delta_ext(&mut self, _e: f64) {
            self.n_delta_ext += 1;
            let mut new_sigma = 0;
            self.in_port
                .get_values()
                .iter()
                .for_each(|msg| new_sigma += msg);
            self.sigma = new_sigma as f64;
        }

        fn ta(&self) -> f64 {
            self.sigma
        }
    }
    impl_atomic!(TestAtomic); // it automatically implements the AsModel and AsAtomic traits

    #[test]
    fn test_component() {
        let mut atomic = TestAtomic::new("atomic");
        assert_eq!(0, atomic.n_delta_int);
        assert_eq!(0, atomic.n_delta_ext);
        assert_eq!(f64::INFINITY, atomic.sigma);

        atomic.add_input_value(0);
        atomic.delta_ext(0.);
        assert_eq!(0, atomic.n_delta_int);
        assert_eq!(1, atomic.n_delta_ext);
        assert_eq!(0., atomic.sigma);
        atomic.clear_in_ports();

        atomic.lambda();
        assert_eq!(0, atomic.n_delta_int);
        assert_eq!(1, atomic.n_delta_ext);
        assert_eq!(0., atomic.sigma);
        assert_eq!(1, atomic.out_port.len());
        assert_eq!(1, *atomic.out_port.get_values().get(0).unwrap());

        atomic.delta_int();
        assert_eq!(1, atomic.n_delta_int);
        assert_eq!(1, atomic.n_delta_ext);
        assert_eq!(f64::INFINITY, atomic.sigma);
        atomic.clear_out_ports();
        assert_eq!(0, atomic.out_port.len());

        atomic.add_input_value(0);
        atomic.add_input_value(1);
        atomic.add_input_value(2);
        atomic.delta_ext(0.);
        assert_eq!(1, atomic.n_delta_int);
        assert_eq!(2, atomic.n_delta_ext);
        assert_eq!(3., atomic.sigma);
        atomic.clear_in_ports();

        atomic.lambda();
        assert_eq!(1, atomic.n_delta_int);
        assert_eq!(2, atomic.n_delta_ext);
        assert_eq!(3., atomic.sigma);
        assert_eq!(1, atomic.out_port.len());
        assert_eq!(3, *atomic.out_port.get_values().get(0).unwrap());

        atomic.delta_int();
        assert_eq!(2, atomic.n_delta_int);
        assert_eq!(2, atomic.n_delta_ext);
        assert_eq!(f64::INFINITY, atomic.sigma);
        atomic.clear_out_ports();
        assert_eq!(0, atomic.out_port.len());
    }
}
