pub mod atomic;
pub mod coupled;
pub mod port;

use crate::Shared;
use port::{AsPort, Port};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Result};

/// Basic simulation clock. All models have one of these inside to keep track of time.
#[derive(Clone, Debug)]
pub struct Clock {
    /// Time for the latest model state transition.
    pub t_last: f64,
    /// Time for the next model state transition.
    pub t_next: f64,
}

impl Clock {
    /// It creates a new simulator with default values.
    pub fn new() -> Self {
        Self {
            t_last: 0.,
            t_next: f64::INFINITY,
        }
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}

/// Generic DEVS model.
#[derive(Debug)]
pub struct Model {
    /// name of the DEVS component.
    name: String,
    /// Input port set of the DEVS component. Input port names must be unique.
    input_map: HashMap<String, Shared<dyn AsPort>>,
    /// Output port set of the DEVS component. Output port names must be unique.
    output_map: HashMap<String, Shared<dyn AsPort>>,
    /// Input port set of the DEVS component (serialized for better performance).
    input_vec: Vec<Shared<dyn AsPort>>,
    /// Output port set of the DEVS component (serialized for better performance).
    output_vec: Vec<Shared<dyn AsPort>>,
    /// Simulation-related stuff.
    clock: Clock,
}

/// Helper function to add a port to a component regardless of whether it is input or output.
fn add_port<T: 'static + Clone + Debug>(
    ports_map: &mut HashMap<String, Shared<dyn AsPort>>,
    ports_vec: &mut Vec<Shared<dyn AsPort>>,
    port_name: &str,
) -> Option<Shared<Port<T>>> {
    if ports_map.contains_key(port_name) {
        return None;
    }
    let port = Shared::new(Port::<T>::new(port_name));
    ports_map.insert(port_name.to_string(), port.clone());
    ports_vec.push(port.clone());
    Some(port)
}

impl Model {
    /// It creates a new component with the provided name.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            input_map: HashMap::new(),
            output_map: HashMap::new(),
            input_vec: Vec::new(),
            output_vec: Vec::new(),
            clock: Clock::new(),
        }
    }

    /// Returns name of the component.
    fn get_name(&self) -> &str {
        &self.name
    }

    pub(crate) fn get_t_next(&self) -> f64 {
        self.clock.t_next
    }

    /// Adds a new input port of type [`Port<T>`] to the component and returns a reference to it.
    /// It panics if there is already an input port with the same name.
    pub fn add_in_port<T: 'static + Clone + Debug>(&mut self, port_name: &str) -> Shared<Port<T>> {
        add_port(&mut self.input_map, &mut self.input_vec, port_name).unwrap_or_else(|| {
            panic!(
                "component {} already contains input port with name {}",
                self.name, port_name
            )
        })
    }

    /// Adds a new output port of type [`Port<T>`] to the component and returns a reference to it.
    /// It panics if there is already an output port with the same name.
    pub fn add_out_port<T: 'static + Clone + Debug>(&mut self, port_name: &str) -> Shared<Port<T>> {
        add_port(&mut self.output_map, &mut self.output_vec, port_name).unwrap_or_else(|| {
            panic!(
                "component {} already contains output port with name {}",
                self.name, port_name
            )
        })
    }

    /// Returns a pointer to an input port with the given name.
    /// If the component does not have any input port with this name, it panics.
    fn get_in_port(&self, port_name: &str) -> Shared<dyn AsPort> {
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
    fn get_out_port(&self, port_name: &str) -> Shared<dyn AsPort> {
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

    fn clear_ports(&mut self) {
        self.input_vec.iter().for_each(|p| p.clear());
        self.output_vec.iter().for_each(|p| p.clear());
    }

    /// Returns true if all the input ports of the model are empty.
    fn is_input_empty(&self) -> bool {
        self.input_vec.iter().all(|p| p.is_empty())
    }

    /// Returns true if all the output ports of the model are empty.
    fn is_output_empty(&self) -> bool {
        self.output_vec.iter().all(|p| p.is_empty())
    }

    /// Sets last and next times for the internal clock.
    fn set_clock(&mut self, t_last: f64, t_next: f64) {
        self.clock.t_last = t_last;
        self.clock.t_next = t_next;
    }
}

impl Display for Model {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.name)
    }
}

/// Interface for DEVS components.
pub trait AsModel: Debug {
    /// All the DEVS component must own a [`Model`] struct.
    /// This method returns a reference to this inner [`Model`].
    fn as_model(&self) -> &Model;

    /// All the DEVS component must own a [`Model`] struct.
    /// This method returns a mutable reference to this inner [`Model`].
    fn as_model_mut(&mut self) -> &mut Model;

    /// It starts the simulation, setting the initial time to t_start.
    fn start_simulation(&mut self, t_start: f64);

    /// It stops the simulation, setting the last time to t_stop.
    fn stop_simulation(&mut self, t_stop: f64);

    /// Executes output functions and propagates messages according to ICs and EOCs.
    fn lambda(&mut self, t: f64);

    /// Propagates messages according to EICs and executes model transition functions.
    fn delta(&mut self, t: f64);

    /// Returns the name of the component.
    fn get_name(&self) -> &str {
        self.as_model().get_name()
    }

    /// Adds a new input port of type [`Port<T>`] to the component and returns a reference to it.
    /// It panics if there is already an input port with the same name.
    fn add_in_port<T>(&mut self, port_name: &str) -> Shared<Port<T>>
    where
        T: 'static + Clone + Debug + Display,
        Self: Sized,
    {
        self.as_model_mut().add_in_port(port_name)
    }

    /// Adds a new output port of type [`Port<T>`] to the component and returns a reference to it.
    /// It panics if there is already an output port with the same name.
    fn add_out_port<T>(&mut self, port_name: &str) -> Shared<Port<T>>
    where
        T: 'static + Clone + Debug + Display,
        Self: Sized,
    {
        self.as_model_mut().add_out_port(port_name)
    }

    /// Returns true if all the input ports of the model are empty.
    fn is_input_empty(&self) -> bool {
        self.as_model().is_input_empty()
    }

    /// Returns true if all the output ports of the model are empty.
    fn is_output_empty(&self) -> bool {
        self.as_model().is_output_empty()
    }

    /// Removes all the messages from all the ports.
    fn clear_ports(&mut self) {
        self.as_model_mut().clear_ports();
    }

    /// It returns the time of the latest model state transition.
    fn get_time(&self) -> f64 {
        self.as_model().clock.t_last
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "component component_a does not contain input port with name i32")]
    fn test_wrong_in_port() {
        Model::new("component_a").get_in_port("i32");
    }

    #[test]
    #[should_panic(expected = "component component_a does not contain output port with name i32")]
    fn test_wrong_out_port() {
        Model::new("component_a").get_out_port("i32");
    }

    #[test]
    #[should_panic(expected = "component component_a already contains input port with name i32")]
    fn test_duplicate_in_port() {
        let mut a = Model::new("component_a");
        let _port = a.add_in_port::<i32>("i32");
        assert_eq!(1, a.input_map.len());
        assert_eq!(0, a.output_map.len());
        let _port = a.add_in_port::<i32>("i32");
    }

    #[test]
    #[should_panic(expected = "component component_a already contains output port with name i32")]
    fn test_duplicate_out_port() {
        let mut a = Model::new("component_a");
        let _port = a.add_out_port::<i32>("i32");
        assert_eq!(0, a.input_map.len());
        assert_eq!(1, a.output_map.len());
        let _port = a.add_out_port::<f64>("i32");
    }

    #[test]
    fn test_component() {
        let mut a = Model::new("component_a");
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

        in_i32.add_value(1);
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
