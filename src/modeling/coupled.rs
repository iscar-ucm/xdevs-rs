use crate::modeling::port::AsPort;
use crate::{AsModel, Model, RcHash, Shared};
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display, Formatter, Result};

type CouplingsMap = HashMap<RcHash<dyn AsPort>, HashSet<RcHash<dyn AsPort>>>;

/// Coupled DEVS model.
#[derive(Debug)]
pub struct Coupled {
    /// Component wrapped by the coupled model.
    model: Model,
    /// Components set of the DEVS coupled model.
    components: HashMap<String, Box<dyn AsModel>>,
    /// External input couplings.
    eic: CouplingsMap,
    /// Internal couplings.
    ic: CouplingsMap,
    /// External output couplings.
    eoc: CouplingsMap,
}

impl Coupled {
    /// Creates a new coupled DEVS model.
    pub fn new(name: &str) -> Self {
        Self {
            model: Model::new(name),
            components: HashMap::new(),
            eic: HashMap::new(),
            ic: HashMap::new(),
            eoc: HashMap::new(),
        }
    }

    /// Returns false if coupling was already defined
    fn add_coupling(
        couplings: &mut CouplingsMap,
        port_from: Shared<dyn AsPort>,
        port_to: Shared<dyn AsPort>,
    ) -> bool {
        if !port_from.is_compatible(&*port_to) {
            panic!("ports {} and {} are incompatible", port_from, port_to);
        }
        let ports_from = couplings
            .entry(RcHash(port_to.clone()))
            .or_insert_with(HashSet::new);
        ports_from.insert(RcHash(port_from.clone()))
    }

    /// Adds a new component to the coupled model.
    /// If there is already a component with the same name as the new component, it panics.
    ///
    /// # Examples
    /// ```
    /// use xdevs::Coupled;
    ///
    /// let mut top_coupled = Coupled::new("top_coupled");
    /// top_coupled.add_component(Coupled::new("component"));
    /// // top_coupled.add_component(Coupled::new("component"));  // this panics (duplicate name)
    /// ```
    pub fn add_component<T: 'static + AsModel>(&mut self, component: T) {
        let component_name = component.get_name();
        if self.components.contains_key(component_name) {
            panic!(
                "coupled model {} already contains component with name {}",
                self.model, component_name
            )
        }
        self.components
            .insert(component_name.to_string(), Box::new(component));
    }

    /// Returns a dynamic reference to a component with the provided name.
    /// If the coupled model does not contain any model with that name, it panics.
    fn get_component(&self, component_name: &str) -> &Box<dyn AsModel> {
        self.components.get(component_name).unwrap_or_else(|| {
            panic!(
                "coupled model {} does not contain component with name {}",
                self.get_name(),
                component_name
            )
        })
    }

    /// Adds a new EIC to the model.
    /// You must provide the input port name of the coupled model,
    /// the receiving component name, and its input port name.
    /// This method panics if:
    /// - the origin port does not exist.
    /// - the destination component does not exist.
    /// - the destination port does not exist.
    /// - ports are not compatible.
    /// - coupling already exist.
    ///
    /// # Examples
    /// ```
    /// use xdevs::{AsModel, Coupled};
    ///
    /// let mut component = Coupled::new("component");
    /// component.add_in_port::<i32>("input");
    /// let mut top_coupled = Coupled::new("top_coupled");
    /// top_coupled.add_in_port::<i32>("input");
    /// top_coupled.add_component(component);
    ///
    /// top_coupled.add_eic("input", "component", "input");
    /// // top_coupled.add_eic("input", "component", "input");  // this panics (duplicate coupling)
    /// ```
    pub fn add_eic(&mut self, port_from_name: &str, component_to_name: &str, port_to_name: &str) {
        let port_from = self.as_model().get_in_port(port_from_name);
        let component = self.get_component(component_to_name);
        let port_to = component.as_model().get_in_port(port_to_name);
        if !Self::add_coupling(&mut self.eic, port_from, port_to) {
            panic!(
                "EIC coupling {}->{}::{} is already defined",
                port_from_name, component_to_name, port_to_name
            )
        }
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
    /// - coupling already exist.
    ///
    /// # Examples
    /// ```
    /// use xdevs::{AsModel, Coupled};
    ///
    /// let mut component_1 = Coupled::new("component_1");
    /// component_1.add_out_port::<i32>("output");
    /// let mut component_2 = Coupled::new("component_2");
    /// component_2.add_in_port::<i32>("input");
    /// let mut top_coupled = Coupled::new("top_coupled");
    /// top_coupled.add_component(component_1);
    /// top_coupled.add_component(component_2);
    ///
    /// top_coupled.add_ic("component_1", "output", "component_2", "input");
    /// // top_coupled.add_ic("component_1", "output", "component_2", "input");  // this panics (duplicate coupling)
    /// ```
    pub fn add_ic(
        &mut self,
        component_from_name: &str,
        port_from_name: &str,
        component_to_name: &str,
        port_to_name: &str,
    ) {
        let component_from = self.get_component(component_from_name).as_model();
        let port_from = component_from.get_out_port(port_from_name);
        let component_to = self.get_component(component_to_name);
        let port_to = component_to.as_model().get_in_port(port_to_name);
        if !Self::add_coupling(&mut self.ic, port_from, port_to) {
            panic!(
                "IC coupling {}::{}->{}::{} is already defined",
                component_from_name, port_from_name, component_to_name, port_to_name
            );
        }
    }

    /// Adds a new EOC to the model.
    /// You must provide the sending component name, its output port name,
    /// and the output port name of the coupled model.
    /// This method panics if:
    /// - the origin component does not exist.
    /// - the origin port does not exist.
    /// - the destination port does not exist.
    /// - ports are not compatible.
    /// - coupling already exist.
    ///
    /// # Examples
    /// ```
    /// use xdevs::{AsModel, Coupled};
    ///
    /// let mut component = Coupled::new("component");
    /// component.add_out_port::<i32>("output");
    /// let mut top_coupled = Coupled::new("top_coupled");
    /// top_coupled.add_out_port::<i32>("output");
    /// top_coupled.add_component(component);
    ///
    /// top_coupled.add_eoc("component", "output", "output");
    /// // top_coupled.add_eoc("component", "output", "output");  // this panics (duplicate coupling)
    /// ```
    pub fn add_eoc(&mut self, component_from_name: &str, port_from_name: &str, port_to_name: &str) {
        let component = self.get_component(component_from_name).as_model();
        let port_from = component.get_out_port(port_from_name);
        let port_to = self.as_model().get_out_port(port_to_name);
        if !Self::add_coupling(&mut self.eoc, port_from, port_to) {
            panic!(
                "EOC coupling <{}::{}->{}> is already defined",
                component_from_name, port_from_name, port_to_name
            );
        }
    }
}

impl Display for Coupled {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.get_name())
    }
}

impl AsModel for Coupled {
    fn as_model(&self) -> &Model {
        &self.model
    }

    fn as_model_mut(&mut self) -> &mut Model {
        &mut self.model
    }

    fn start_simulation(&mut self, t_start: f64) {
        let mut t_next = f64::INFINITY;
        for component in self.components.values_mut() {
            component.start_simulation(t_start);
            let t = component.as_model().get_t_next();
            if t < t_next {
                t_next = t;
            }
        }
        self.model.set_clock(t_start, t_next);
    }

    fn stop_simulation(&mut self, t_stop: f64) {
        self.components
            .values_mut()
            .for_each(|c| c.stop_simulation(t_stop)); // TODO parallel?
        self.model.set_clock(t_stop, f64::INFINITY);
    }

    fn lambda(&mut self, t: f64) {
        // TODO parallel?
        if t >= self.model.get_t_next() {
            self.components.values_mut().for_each(|c| c.lambda(t));
            for (port_to, ports_from) in self.ic.iter() {
                ports_from
                    .iter()
                    .for_each(|port_from| port_to.propagate(&**port_from))
            }
            for (port_to, ports_from) in self.eoc.iter() {
                ports_from
                    .iter()
                    .for_each(|port_from| port_to.propagate(&**port_from))
            }
        }
    }

    fn delta(&mut self, t: f64) {
        for (port_to, ports_from) in self.eic.iter() {
            ports_from
                .iter()
                .for_each(|port_from| port_to.propagate(&**port_from))
        }
        self.components.values_mut().for_each(|c| c.delta(t));
        self.model.clock.t_last = t;
        let mut next_t = f64::INFINITY;
        for component in self.components.values() {
            let t = component.as_model().get_t_next();
            if t < next_t {
                next_t = t;
            }
        }
        self.model.clock.t_next = next_t;
    }

    fn clear_ports(&mut self) {
        self.components.values_mut().for_each(|c| c.clear_ports());
        self.model.clear_ports();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(
        expected = "coupled model top_coupled already contains component with name component"
    )]
    fn test_duplicate_component() {
        let mut top_coupled = Coupled::new("top_coupled");
        top_coupled.add_component(Coupled::new("component"));
        top_coupled.add_component(Coupled::new("component"));
    }

    #[test]
    #[should_panic(
        expected = "coupled model top_coupled does not contain component with name component_2"
    )]
    fn test_get_component() {
        let mut top_coupled = Coupled::new("top_coupled");
        top_coupled.add_component(Coupled::new("component_1"));
        assert_eq!(
            "component_1",
            top_coupled.get_component("component_1").get_name()
        );
        assert_eq!(
            top_coupled.get_component("component_1").get_name(),
            top_coupled.get_component("component_1").get_name()
        );
        top_coupled.get_component("component_2");
    }

    #[test]
    #[should_panic(
        expected = "component top_coupled does not contain input port with name bad_input"
    )]
    fn test_eic_bad_port_from() {
        let mut top_coupled = Coupled::new("top_coupled");
        top_coupled.add_eic("bad_input", "bad_component", "bad_output");
    }

    #[test]
    #[should_panic(
        expected = "coupled model top_coupled does not contain component with name bad_component"
    )]
    fn test_eic_bad_component_to() {
        let mut top_coupled = Coupled::new("top_coupled");
        top_coupled.add_in_port::<i32>("input");
        top_coupled.add_eic("input", "bad_component", "bad_output");
    }

    #[test]
    #[should_panic(
        expected = "component component does not contain input port with name bad_output"
    )]
    fn test_eic_bad_port_to() {
        let mut top_coupled = Coupled::new("top_coupled");
        top_coupled.add_in_port::<i32>("input");
        top_coupled.add_component(Coupled::new("component"));
        top_coupled.add_eic("input", "component", "bad_output");
    }

    #[test]
    #[should_panic(expected = "ports input<i32> and input<i64> are incompatible")]
    fn test_eic_bad_types() {
        let mut top_coupled = Coupled::new("top_coupled");
        top_coupled.add_in_port::<i32>("input");
        let mut component = Coupled::new("component");
        component.add_in_port::<i64>("input");
        top_coupled.add_component(component);
        top_coupled.add_eic("input", "component", "input");
    }

    #[test]
    #[should_panic(expected = "EIC coupling input->component::input is already defined")]
    fn test_eic() {
        let mut top_coupled = Coupled::new("top_coupled");
        top_coupled.add_in_port::<i32>("input");
        let mut component = Coupled::new("component");
        component.add_in_port::<i32>("input");
        top_coupled.add_component(component);
        top_coupled.add_eic("input", "component", "input");
        top_coupled.add_eic("input", "component", "input");
    }
}
