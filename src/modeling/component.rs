use super::port::{Bag, InPort, OutPort, Port};
use crate::{DynRef, Shared};
use std::collections::HashMap;

/// DEVS component. Models must comprise a component to fulfill the [`crate::simulation::Simulator`] trait.
pub struct Component {
    /// name of the DEVS component.
    name: String,
    /// Time of the last component state transition.
    t_last: f64,
    /// Time for the next component state transition.
    t_next: f64,
    /// Input ports map. Keys are the port IDs.
    in_map: HashMap<String, usize>,
    /// Output ports map. Keys are the port IDs.
    out_map: HashMap<String, usize>,
    /// Input port set of the DEVS component (serialized for better performance).
    in_ports: Vec<Shared<dyn Port>>,
    /// Output port set of the DEVS component (serialized for better performance).
    out_ports: Vec<Shared<dyn Port>>,
}

impl Component {
    /// It creates a new component with the provided name.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            t_last: 0.,
            t_next: f64::INFINITY,
            in_map: HashMap::new(),
            out_map: HashMap::new(),
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

    /// Adds a new input port of type `T` and returns a reference to it.
    /// It panics if there is already an input port with the same name.
    pub fn add_in_port<T: DynRef + Clone>(&mut self, name: &str) -> InPort<T> {
        if self.in_map.contains_key(name) {
            panic!("component already contains input port with the name provided");
        }
        self.in_map.insert(name.to_string(), self.in_ports.len());
        let bag = Shared::new(Bag::new());
        self.in_ports.push(bag.clone());
        InPort::new(bag)
    }

    /// Adds a new output port of type `T` and returns a reference to it.
    /// It panics if there is already an output port with the same name.
    pub fn add_out_port<T: DynRef + Clone>(&mut self, name: &str) -> OutPort<T> {
        if self.out_map.contains_key(name) {
            panic!("component already contains output port with the name provided");
        }
        self.out_map.insert(name.to_string(), self.out_ports.len());
        let bag = Shared::new(Bag::new());
        self.out_ports.push(bag.clone());
        OutPort::new(bag)
    }

    /// Returns `true` if all the input ports of the model are empty.
    #[inline]
    pub fn is_input_empty(&self) -> bool {
        self.in_ports.iter().all(|p| p.is_empty())
    }

    /// Returns `true` if all the output ports of the model are empty.
    #[inline]
    pub fn is_output_empty(&self) -> bool {
        self.out_ports.iter().all(|p| p.is_empty())
    }

    /// Returns a reference to an input port with the given name.
    /// If the component does not have any input port with this name, it returns [`None`].
    pub(crate) fn get_in_port(&self, port_name: &str) -> Option<Shared<dyn Port>> {
        let i = *self.in_map.get(port_name)?;
        Some(self.in_ports.get(i)?.clone())
    }

    /// Returns a reference to an output port with the given name.
    /// If the component does not have any output port with this name, it returns [`None`].
    pub(crate) fn get_out_port(&self, port_name: &str) -> Option<Shared<dyn Port>> {
        let i = *self.out_map.get(port_name)?;
        Some(self.out_ports.get(i)?.clone())
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
