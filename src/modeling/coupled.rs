use super::port::Port;
use super::{Component, InPort, OutPort};
use crate::simulation::Simulator;
use crate::{DynRef, Shared};
use std::collections::HashMap;

/// Coupled DEVS model.
pub struct Coupled {
    /// Component wrapped by the coupled model.
    pub(crate) component: Component,
    /// Components map. Keys are components' IDs.
    comps_map: HashMap<String, usize>,
    /// External input couplings map.
    eic_map: HashMap<String, HashMap<String, usize>>,
    /// Internal couplings map.
    ic_map: HashMap<String, HashMap<String, usize>>,
    /// External output couplings map.
    eoc_map: HashMap<String, HashMap<String, usize>>,
    /// Components of the DEVS coupled model (serialized for better performance).
    pub(crate) components: Vec<Box<dyn Simulator>>,
    /// External input and internal couplings (serialized for better performance).
    pub(crate) xics: Vec<(Shared<dyn Port>, Shared<dyn Port>)>,
    /// External output couplings (serialized for better performance).
    pub(crate) eocs: Vec<(Shared<dyn Port>, Shared<dyn Port>)>,
    #[cfg(feature = "par_xic")]
    xic_map: HashMap<String, Vec<usize>>,
    #[cfg(feature = "par_xic")]
    pub(crate) par_xics: Vec<Vec<usize>>,
    #[cfg(feature = "par_eoc")]
    pub(crate) par_eocs: Vec<Vec<usize>>,
}

impl Coupled {
    /// Creates a new coupled DEVS model with the provided name.
    pub fn new(name: &str) -> Self {
        Self {
            component: Component::new(name),
            comps_map: HashMap::new(),
            eic_map: HashMap::new(),
            ic_map: HashMap::new(),
            eoc_map: HashMap::new(),
            components: Vec::new(),
            xics: Vec::new(),
            eocs: Vec::new(),
            #[cfg(feature = "par_xic")]
            xic_map: HashMap::new(),
            #[cfg(feature = "par_xic")]
            par_xics: Vec::new(),
            #[cfg(feature = "par_eoc")]
            par_eocs: Vec::new(),
        }
    }

    /// Returns the number of components in the coupled model.
    pub fn n_components(&self) -> usize {
        self.components.len()
    }

    /// Returns the number of external input couplings in the coupled model.
    pub fn n_eics(&self) -> usize {
        self.eic_map.values().map(|eics| eics.len()).sum()
    }

    /// Returns the number of internal couplings in the coupled model.
    pub fn n_ics(&self) -> usize {
        self.ic_map.values().map(|ics| ics.len()).sum()
    }

    /// Returns the number of external output couplings in the coupled model.
    pub fn n_eocs(&self) -> usize {
        self.eoc_map.values().map(|eocs| eocs.len()).sum()
    }

    /// Adds a new input port of type `T` and returns a reference to it.
    /// It panics if there is already an input port with the same name.
    #[inline]
    pub fn add_in_port<T: DynRef + Clone>(&mut self, name: &str) -> InPort<T> {
        self.component.add_in_port::<T>(name)
    }

    /// Adds a new output port of type `T` and returns a reference to it.
    /// It panics if there is already an output port with the same name.
    #[inline]
    pub fn add_out_port<T: DynRef + Clone>(&mut self, name: &str) -> OutPort<T> {
        self.component.add_out_port::<T>(name)
    }

    /// Adds a new component to the coupled model.
    /// If there is already a component with the same name as the new component, it panics.
    pub fn add_component<T: Simulator>(&mut self, component: Box<T>) {
        let component_name = component.get_name();
        if self.comps_map.contains_key(component_name) {
            panic!("coupled model already contains component with the name provided")
        }
        self.comps_map
            .insert(component_name.to_string(), self.components.len());
        self.components.push(component);
    }

    /// Returns a reference to a component with the provided name.
    /// If the coupled model does not contain any model with that name, it returns [`None`].
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
        let p_from = self
            .component
            .get_in_port(port_from)
            .expect("port_from does not exist");
        let comp_to = self
            .get_component(component_to)
            .expect("component_to does not exist");
        let p_to = comp_to
            .get_in_port(port_to)
            .expect("port_to does not exist");
        if !p_from.is_compatible(&*p_to) {
            panic!("ports are not compatible")
        }
        let source_key = port_from.to_string();
        let destination_key = component_to.to_string() + "-" + port_to;
        let coups = self.eic_map.entry(destination_key).or_default();
        if coups.contains_key(&source_key) {
            panic!("coupling already exists");
        }
        coups.insert(source_key, self.xics.len());
        #[cfg(feature = "par_xic")]
        {
            let destination_key = component_to.to_string() + "-" + port_to;
            self.xic_map
                .entry(destination_key)
                .or_default()
                .push(self.xics.len());
        }
        self.xics.push((p_to, p_from));
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
        let comp_from = self
            .get_component(component_from)
            .expect("component_from does not exist");
        let p_from = comp_from
            .get_out_port(port_from)
            .expect("port_from does not exist");
        let comp_to = self
            .get_component(component_to)
            .expect("component_to does not exist");
        let p_to = comp_to
            .get_in_port(port_to)
            .expect("port_to does not exist");
        if !p_from.is_compatible(&*p_to) {
            panic!("ports are not compatible")
        }
        let source_key = component_from.to_string() + "-" + port_from;
        let destination_key = component_to.to_string() + "-" + port_to;
        let coups = self.ic_map.entry(destination_key).or_default();
        if coups.contains_key(&source_key) {
            panic!("coupling already exists");
        }
        coups.insert(source_key, self.xics.len());
        #[cfg(feature = "par_xic")]
        {
            let destination_key = component_to.to_string() + "-" + port_to;
            self.xic_map
                .entry(destination_key)
                .or_default()
                .push(self.xics.len());
        }
        self.xics.push((p_to, p_from));
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
        let comp_from = self
            .get_component(component_from)
            .expect("component_from does not exist");
        let p_from = comp_from
            .get_out_port(port_from)
            .expect("port_from does not exist");
        let p_to = self
            .component
            .get_out_port(port_to)
            .expect("port_to does not exist");
        if !p_from.is_compatible(&*p_to) {
            panic!("ports are not compatible")
        }
        let source_key = component_from.to_string() + "-" + port_from;
        let destination_key = port_to.to_string();
        let coups = self.eoc_map.entry(destination_key).or_default();
        if coups.contains_key(&source_key) {
            panic!("coupling already exists");
        }
        coups.insert(source_key, self.eocs.len());
        self.eocs.push((p_to, p_from));
    }

    #[cfg(any(ffeature = "par_eoc"))]
    fn flatten_map(map: &HashMap<String, HashMap<String, usize>>) -> Vec<Vec<usize>> {
        map.values()
            .map(|m| m.values().copied().collect())
            .collect()
    }

    #[cfg(feature = "par_xic")]
    pub(crate) fn build_par_xics(&mut self) {
        self.par_xics = self.xic_map.values().cloned().collect();
    }

    #[cfg(feature = "par_eoc")]
    pub(crate) fn build_par_eocs(&mut self) {
        self.par_eocs = self
            .eoc_map
            .values()
            .map(|m| m.values().copied().collect())
            .collect();
    }
}
