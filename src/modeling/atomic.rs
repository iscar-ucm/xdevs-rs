use super::AsComponent;

/// Interface for atomic DEVS models.
pub trait AtomicInterface: AsComponent {
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
    /// By default, it first triggers [`AtomicInterface::delta_int`].
    /// Then, it triggers [`AtomicInterface::delta_ext`] with elapsed time 0.
    fn delta_conf(&mut self) {
        self.delta_int();
        self.delta_ext(0.);
    }
}

#[cfg(test)]
mod tests {
    use crate::modeling::*;
    use std::rc::Rc;

    #[derive(Debug)]
    struct TestAtomic {
        // We need to have a Component for composition.
        component: Component,
        // We add all the state-related fields.
        n_delta_int: i32,
        n_delta_ext: i32,
        sigma: f64,
        // We add all the in/out ports of the model.
        in_port: Rc<Port<i32>>,
        out_port: Rc<Port<i32>>,
    }

    impl TestAtomic {
        fn new(name: &str) -> Self {
            let mut component = Component::new(name);
            Self {
                n_delta_ext: 0,
                n_delta_int: 0,
                sigma: f64::INFINITY,
                in_port: component.add_in_port::<i32>("in_port"),
                out_port: component.add_out_port::<i32>("out_port"),
                component,
            }
        }

        fn add_input_value(&self, value: i32) {
            self.in_port.add_value(value);
        }
    }

    impl_component!(TestAtomic); // impl_component automatically implements the AsComponent

    impl AtomicInterface for TestAtomic {
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
