use super::port::{Bag, InPort, OutPort, Port};
use std::collections::HashMap;

/// DEVS component. Models must comprise a ['Component'] to fulfill the [`crate::simulation::Simulator`] trait.
pub struct Component {
    /// name of the DEVS component.
    name: String,
    /// Time of the last component state transition.
    t_last: f64,
    /// Time for the next component state transition.
    t_next: f64,
    /// Keys are port IDs, and values are indices of input ports in [Component::input_ports].
    input_map: HashMap<String, usize>,
    /// Keys are port IDs, and values are indices of output ports in [Component::output_ports].
    output_map: HashMap<String, usize>,
    /// Input port set of the DEVS component.
    in_ports: Vec<Box<dyn Port>>,
    /// Output port set of the DEVS component.
    out_ports: Vec<Box<dyn Port>>,
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
            in_ports: Vec::new(),
            out_ports: Vec::new(),
        }
    }

    /// Returns name of the component.
    #[inline]
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Returns the time for the last component state transition.
    #[inline]
    pub fn get_t_last(&self) -> f64 {
        self.t_last
    }

    /// Returns the time for the next component state transition.
    #[inline]
    pub fn get_t_next(&self) -> f64 {
        self.t_next
    }

    /// Sets the time for the for the last and next component state transitions.
    #[inline]
    pub(crate) fn set_sim_t(&mut self, t_last: f64, t_next: f64) {
        self.t_last = t_last;
        self.t_next = t_next;
    }

    /// Adds a new input port of type [`Port<Input, T>`] and returns a reference to it.
    /// It panics if there is already an input port with the same name.
    pub fn add_in_port<T: 'static + Clone>(&mut self, name: &str) -> InPort<T> {
        if self.input_map.contains_key(name) {
            panic!("component already contains input port with the name provided");
        }
        let bag = Box::new(Bag::<T>::new());
        let port = InPort::new(&bag);
        self.input_map
            .insert(name.to_string(), self.in_ports.len());
        self.in_ports.push(bag);
        port
    }

    /// Adds a new output port of type [`Port<Output, T>`] and returns a reference to it.
    /// It panics if there is already an output port with the same name.
    pub fn add_out_port<T: 'static + Clone>(&mut self, name: &str) -> OutPort<T> {
        if self.output_map.contains_key(name) {
            panic!("component already contains output port with the name provided");
        }
        let mut bag = Box::new(Bag::<T>::new());
        let port = OutPort::new(&mut bag);
        self.output_map
            .insert(name.to_string(), self.out_ports.len());
        self.out_ports.push(bag);
        port
    }

    /// Returns true if all the input ports of the model are empty.
    pub fn is_input_empty(&self) -> bool {
        self.in_ports.iter().all(|p| p.is_empty())
    }

    /// Returns true if all the output ports of the model are empty.
    pub fn is_output_empty(&self) -> bool {
        self.out_ports.iter().all(|p| p.is_empty())
    }

    /// Returns a reference to an input port with the given name.
    /// If the component does not have any input port with this name, it returns [`None`].
    pub(crate) fn get_in_port(&self, port_name: &str) -> Option<&dyn Port> {
        let i = *self.input_map.get(port_name)?;
        Some(self.in_ports.get(i)?)
    }

    /// Returns a mutable reference to an input port with the given name.
    /// If the component does not have any input port with this name, it returns [`None`].
    pub(crate) fn get_in_port_mut(&mut self, port_name: &str) -> Option<&mut dyn Port> {
        let i = *self.input_map.get(port_name)?;
        Some(self.in_ports.get_mut(i)?)
    }

    /// Returns a reference to an output port with the given name.
    /// If the component does not have any output port with this name, it returns [`None`].
    pub(crate) fn get_out_port(&self, port_name: &str) -> Option<&dyn Port> {
        let i = *self.output_map.get(port_name)?;
        Some(self.out_ports.get(i)?)
    }

    /// Returns a mutable reference to an output port with the given name.
    /// If the component does not have any output port with this name, it returns [`None`].
    pub(crate) fn get_out_port_mut(&mut self, port_name: &str) -> Option<&mut dyn Port> {
        let i = *self.output_map.get(port_name)?;
        Some(self.out_ports.get_mut(i)?)
    }

    /// Clears all the input ports of the model.
    #[inline]
    pub(crate) fn clear_input(&mut self) {
        self.in_ports.iter_mut().for_each(|p| p.clear());
    }

    /// Clears all the output ports of the model.
    #[inline]
    pub(crate) fn clear_output(&mut self) {
        self.out_ports.iter_mut().for_each(|p| p.clear());
    }
}
