use super::port::{Coupling, InPort, OutPort};
use super::Component;
use crate::simulation::Simulator;
use std::collections::HashMap;

/// Coupled DEVS model.
pub struct Coupled {
    /// Component wrapped by the coupled model.
    pub(crate) component: Component,
    /// Keys are IDs of subcomponents, and values are indices of [`Coupled::comps_vec`].
    comps_map: HashMap<String, usize>,
    /// External input couplings.
    eic_map: HashMap<String, HashMap<String, usize>>,
    /// Internal couplings.
    ic_map: HashMap<String, HashMap<String, usize>>,
    /// External output couplings.
    eoc_map: HashMap<String, HashMap<String, usize>>,
    /// Components of the DEVS coupled model (serialized for better performance).
    pub(crate) components: Vec<Box<dyn Simulator>>,
    /// External input couplings (serialized for better performance).
    pub(crate) eics: Vec<Box<dyn Coupling>>,
    /// Internal couplings (serialized for better performance).
    pub(crate) ics: Vec<Box<dyn Coupling>>,
    /// External output couplings (serialized for better performance).
    pub(crate) eocs: Vec<Box<dyn Coupling>>,
}

impl Coupled {
    /// Creates a new coupled DEVS model.
    pub fn new(name: &str) -> Self {
        Self {
            component: Component::new(name),
            comps_map: HashMap::new(),
            eic_map: HashMap::new(),
            ic_map: HashMap::new(),
            eoc_map: HashMap::new(),
            components: Vec::new(),
            eics: Vec::new(),
            ics: Vec::new(),
            eocs: Vec::new(),
        }
    }

    /// Adds a new input port of type [`Port<Input, T>`] and returns a reference to it.
    /// It panics if there is already an input port with the same name.
    #[inline]
    pub fn add_in_port<T: 'static + Clone>(&mut self, name: &str) -> InPort<T> {
        self.component.add_in_port::<T>(name)
    }

    /// Adds a new output port of type [`Port<Output, T>`] and returns a reference to it.
    /// It panics if there is already an output port with the same name.
    #[inline]
    pub fn add_out_port<T: 'static + Clone>(&mut self, name: &str) -> OutPort<T> {
        self.component.add_out_port::<T>(name)
    }

    /// Adds a new component to the coupled model.
    /// If there is already a component with the same name as the new component, it panics.
    pub fn add_component<T: 'static + Simulator>(&mut self, component: Box<T>) {
        let component_name = component.get_name();
        if self.comps_map.contains_key(component_name) {
            panic!("coupled model already contains component with the name provided")
        }
        self.comps_map
            .insert(component_name.to_string(), self.components.len());
        self.components.push(component);
    }

    /// Returns a reference to a component with the provided name.
    /// If the coupled model does not contain any model with that name, it return [`None`].
    fn get_component(&self, name: &str) -> Option<&Component> {
        let index = *self.comps_map.get(name)?;
        Some(self.components.get(index)?.get_component())
    }

    /// Adds a new EIC to the model.
    /// You must provide the input port name of the coupled model,
    /// the receiving component name, and its input port name.
    /// This method panics if:
    /// - the origin port does not exist.
    /// - the destination component does not exist.
    /// - the destination port does not exist.
    /// - ports are not compatible.
    /// - coupling already exists.
    pub fn add_eic(&mut self, port_from: &str, component_to: &str, port_to: &str) {
        let p_from = self.component.get_in_port(port_from).expect("port_from does not exist");
        let comp_to = self.get_component(component_to).expect("component_to does not exist");
        let p_to = comp_to.get_in_port(port_to).expect("port_to does not exist");
        let coup = p_from.new_coupling(p_to).expect("ports are not compatible");

        let source_key = port_from.to_string();
        let destination_key = component_to.to_string() + "-" + port_to;
        let coups = self.eic_map.entry(source_key).or_default();
        if coups.contains_key(&destination_key) {
            panic!("coupling already exists");
        }
        coups.insert(destination_key, self.eics.len());
        self.eics.push(coup);
    }

    /// Adds a new IC to the model.
    /// You must provide the sending component name, its output port name,
    /// the receiving component name, and its input port name.
    /// This method panics if:
    /// - the origin component does not exist.
    /// - the origin port does not exist.
    /// - the destination component does not exist.
    /// - the destination port does not exist.
    /// - ports are not compatible.
    /// - coupling already exists.
    pub fn add_ic(
        &mut self,
        component_from: &str,
        port_from: &str,
        component_to: &str,
        port_to: &str,
    ) {
        let comp_from = self.get_component(component_from).expect("component_from does not exist");
        let p_from = comp_from.get_out_port(port_from).expect("port_from does not exist");
        let comp_to = self.get_component(component_to).expect("component_to does not exist");
        let p_to = comp_to.get_in_port(port_to).expect("port_to does not exist");
        let coup = p_from.new_coupling(p_to).expect("ports are not compatible");

        let source_key = component_from.to_string() + "-" + port_from;
        let destination_key = component_to.to_string() + "-" + port_to;
        let coups = self.ic_map.entry(source_key).or_default();
        if coups.contains_key(&destination_key) {
            panic!("coupling already exists");
        }
        coups.insert(destination_key, self.ics.len());
        self.ics.push(coup);
    }

    /// Adds a new EOC to the model.
    /// You must provide the sending component name, its output port name,
    /// and the output port name of the coupled model.
    /// This method panics if:
    /// - the origin component does not exist.
    /// - the origin port does not exist.
    /// - the destination port does not exist.
    /// - ports are not compatible.
    /// - coupling already exists.
    pub fn add_eoc(&mut self, component_from: &str, port_from: &str, port_to: &str) {

        let comp_from = self.get_component(component_from).expect("component_from does not exist");
        let p_from = comp_from.get_out_port(port_from).expect("port_from does not exist");
        let p_to = self.component.get_out_port(port_to).expect("port_to does not exist");
        let coup = p_from.new_coupling(p_to).expect("ports are not compatible");

        let source_key = component_from.to_string() + "-" + port_from;
        let destination_key = port_to.to_string();
        let coups = self.eoc_map.entry(source_key).or_default();
        if coups.contains_key(&destination_key) {
            panic!("coupling already exists");
        }
        coups.insert(destination_key, self.eocs.len());
        self.eocs.push(coup);
    }
}
