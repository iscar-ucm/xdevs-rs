pub mod atomic;
pub mod coupled;

pub use atomic::Atomic;
pub use coupled::Coupled;

use crate::modeling::port::{Port, IN, OUT, AbstractPort, TypedPort};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Result};
use std::rc::Rc;

/// Generic DEVS model. Models must comprise a ['Component'] to implement the [`Simulator`] trait.
#[derive(Debug)]
pub struct Component {
    /// name of the DEVS component.
    name: String,
    /// Time for the last component state transition.
    t_last: f64,
    /// Time for the next component state transition.
    t_next: f64,
    /// Input port set of the DEVS component. Input port names must be unique.
    input_map: HashMap<String, Rc<dyn AbstractPort>>,
    /// Output port set of the DEVS component. Output port names must be unique.
    output_map: HashMap<String, Rc<dyn AbstractPort>>,
    /// Input port set of the DEVS component (serialized for better performance).
    input_vec: Vec<Rc<dyn AbstractPort>>,
    /// Output port set of the DEVS component (serialized for better performance).
    output_vec: Vec<Rc<dyn AbstractPort>>,
}

impl Component {
    /// It creates a new component with the provided name.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            t_last: 0.,
            t_next: f64::INFINITY,
            input_map: HashMap::new(),
            output_map: HashMap::new(),
            input_vec: Vec::new(),
            output_vec: Vec::new(),
        }
    }

    /// Returns name of the component.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Returns the time for the last component state transition.
    pub fn get_t_last(&self) -> f64 {
        self.t_last
    }

    /// Returns the time for the next component state transition.
    pub fn get_t_next(&self) -> f64 {
        self.t_next
    }

    /// Sets the time for the for the last and next component state transitions.
    pub(crate) fn set_sim_t(&mut self, t_last: f64, t_next: f64) {
        self.t_last = t_last;
        self.t_next = t_next;
    }

    /// Adds a new input port of type [`Port<T>`] to the component and returns a reference to it.
    /// It panics if there is already an input port with the same name.
    pub fn add_in_port<T: 'static + Clone + Debug>(&mut self, port_name: &str) -> Port<IN, T> {
        let port = Self::add_port(&mut self.input_map, &mut self.input_vec, port_name).unwrap_or_else(
            || {
                panic!(
                    "component {} already contains input port with name {}",
                    self.name, port_name
                )
            },
        );
        Port::<IN, T>(port)
    }

    /// Adds a new output port of type [`Port<T>`] to the component and returns a reference to it.
    /// It panics if there is already an output port with the same name.
    pub fn add_out_port<T: 'static + Clone + Debug>(&mut self, port_name: &str) -> Port<OUT, T> {
        let port = Self::add_port(&mut self.output_map, &mut self.output_vec, port_name).unwrap_or_else(
            || {
                panic!(
                    "component {} already contains output port with name {}",
                    self.name, port_name
                )
            },
        );
        Port::<OUT, T>(port)
    }

    /// Returns a pointer to an input port with the given name.
    /// If the component does not have any input port with this name, it panics.
    fn get_in_port(&self, port_name: &str) -> Rc<dyn AbstractPort> {
        self.input_map
            .get(port_name)
            .unwrap_or_else(|| {
                panic!(
                    "component {} does not contain input port with name {}",
                    self.name, port_name
                )
            })
            .clone()
    }

    /// Returns a pointer to an output port with the given name.
    /// If the component does not have any output port with this name, it panics.
    fn get_out_port(&self, port_name: &str) -> Rc<dyn AbstractPort> {
        self.output_map
            .get(port_name)
            .unwrap_or_else(|| {
                panic!(
                    "component {} does not contain output port with name {}",
                    self.name, port_name
                )
            })
            .clone()
    }

    /// Clears all the input and output ports of the model.
    pub(crate) fn clear_ports(&mut self) {
        self.input_vec.iter().for_each(|p| p.clear());
        self.output_vec.iter().for_each(|p| p.clear());
    }

    /// Returns true if all the input ports of the model are empty.
    pub fn is_input_empty(&self) -> bool {
        self.input_vec.iter().all(|p| p.is_empty())
    }

    /// Returns true if all the output ports of the model are empty.
    pub fn is_output_empty(&self) -> bool {
        self.output_vec.iter().all(|p| p.is_empty())
    }

    /// Helper function to add a port to a component regardless of whether it is input or output.
    fn add_port<T: 'static + Clone + Debug>(
        ports_map: &mut HashMap<String, Rc<dyn AbstractPort>>,
        ports_vec: &mut Vec<Rc<dyn AbstractPort>>,
        port_name: &str,
    ) -> Option<Rc<TypedPort<T>>> {
        if ports_map.contains_key(port_name) {
            return None;
        }
        let port = Rc::new(TypedPort::<T>::new(port_name));
        ports_map.insert(port_name.to_string(), port.clone());
        ports_vec.push(port.clone());
        Some(port)
    }
}

impl Display for Component {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "component component_a does not contain input port with name i32")]
    fn test_wrong_in_port() {
        Component::new("component_a").get_in_port("i32");
    }

    #[test]
    #[should_panic(expected = "component component_a does not contain output port with name i32")]
    fn test_wrong_out_port() {
        Component::new("component_a").get_out_port("i32");
    }

    #[test]
    #[should_panic(expected = "component component_a already contains input port with name i32")]
    fn test_duplicate_in_port() {
        let mut a = Component::new("component_a");
        let _port = a.add_in_port::<i32>("i32");
        assert_eq!(1, a.input_map.len());
        assert_eq!(0, a.output_map.len());
        let _port = a.add_in_port::<i32>("i32");
    }

    #[test]
    #[should_panic(expected = "component component_a already contains output port with name i32")]
    fn test_duplicate_out_port() {
        let mut a = Component::new("component_a");
        let _port = a.add_out_port::<i32>("i32");
        assert_eq!(0, a.input_map.len());
        assert_eq!(1, a.output_map.len());
        let _port = a.add_out_port::<f64>("i32");
    }

    #[test]
    fn test_component() {
        let mut a = Component::new("component_a");
        let in_i32 = a.add_in_port::<i32>("i32");
        let out_i32 = a.add_out_port::<i32>("i32");
        let out_f64 = a.add_out_port::<f64>("f64");

        assert_eq!("component_a", a.name);
        assert_eq!(1, a.input_map.len());
        assert_eq!(2, a.output_map.len());
        assert!(a.is_input_empty());
        assert!(a.is_output_empty());

        out_i32.add_value(1);
        out_f64.add_values(&vec![1.0, 2.0]);
        assert!(a.is_input_empty());
        assert!(!a.is_output_empty());
        {
            let port_dyn_ref = a.get_out_port("f64");
            assert!(!port_dyn_ref.is_empty());
        }

        a.clear_ports();
        assert!(a.is_input_empty());
        assert!(a.is_output_empty());

        in_i32.0.add_value(1);
        assert!(!a.is_input_empty());
        assert!(a.is_output_empty());
        {
            let port_dyn_ref = a.get_in_port("i32");
            assert!(!port_dyn_ref.is_empty());
        }

        a.clear_ports();
        assert!(a.is_input_empty());
        assert!(a.is_output_empty());
    }
}

