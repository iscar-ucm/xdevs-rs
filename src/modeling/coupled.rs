use super::{AsModel, AsPort, Model, Component};
use crate::{RcHash, Shared};
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display, Result, Formatter};

type CouplingsMap = HashMap<RcHash<dyn AsPort>, HashSet<RcHash<dyn AsPort>>>;

/// Coupled DEVS model.
#[derive(Debug)]
pub struct Coupled {
    /// Component wrapped by the coupled model.
    component: Model,
    /// Components set of the DEVS coupled model.
    /// Components are arranged in a [`HashMap`] which keys are the component names.
    /// Thus, component names must be unique.
    pub(crate) components: HashMap<String, Component>,
    /// External input couplings.
    pub(crate) eic: CouplingsMap,
    /// Internal couplings.
    pub(crate) ic: CouplingsMap,
    /// External output couplings.
    pub(crate) eoc: CouplingsMap,
}

impl Coupled {
    /// Creates a new coupled DEVS model.
    pub fn new(name: &str) -> Self {
        Self {
            component: Model::new(name),
            components: HashMap::new(),
            eic: HashMap::new(),
            ic: HashMap::new(),
            eoc: HashMap::new(),
        }
    }

    fn _add_component<T: 'static + AsModel>(&mut self, component: T) {
        let component_name = component.get_name();
        if self.components.contains_key(component_name) {
            panic!(
                "coupled model {} already contains component with name {}",
                self.component, component_name
            )
        }
        self.components
            .insert(component_name.to_string(), AsModel::to_component(component));
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
            .entry(RcHash(port_to))
            .or_insert_with(HashSet::new);
        ports_from.insert(RcHash(port_from))
    }
}

impl Display for Coupled {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.component.get_name())
    }
}

impl AsCoupled for Coupled {
    fn to_coupled(self) -> Coupled {
        self
    }

    fn as_coupled(&self) -> &Coupled {
        self
    }

    fn as_coupled_mut(&mut self) -> &mut Coupled {
        self
    }
}

pub trait AsCoupled: AsModel {
    fn to_coupled(self) -> Coupled;
    /// Method to return a reference to the inner coupled model.
    fn as_coupled(&self) -> &Coupled;

    /// Method to return a mutable reference to the inner coupled model.
    fn as_coupled_mut(&mut self) -> &mut Coupled;

    /// Adds a new component to the coupled model.
    /// If there is already a component with the same name as the new component, it panics.
    ///
    /// # Examples
    /// ```
    /// use xdevs::modeling::{AsCoupled, Coupled};
    ///
    /// let mut top_coupled = Coupled::new("top_coupled");
    /// top_coupled.add_component(Coupled::new("component"));
    /// // top_coupled.add_component(Coupled::new("component"));  // this panics (duplicate name)
    /// ```
    fn add_component<T: AsModel + 'static>(&mut self, component: T)
    where
        Self: Sized,
    {
        self.as_coupled_mut()._add_component(component);
    }

    /// Returns a dynamic reference to a component with the provided name.
    /// If the coupled model does not contain any model with that name, it returns [`None`].
    ///
    /// # Examples
    /// ```
    /// use xdevs::modeling::{AsCoupled, Coupled};
    ///
    /// let mut top_coupled = Coupled::new("top_coupled");
    /// assert!(top_coupled.try_get_component("component_1").is_none());
    /// top_coupled.add_component(Coupled::new("component_1"));
    /// assert!(top_coupled.try_get_component("component_1").is_some());
    /// ```
    fn try_get_component(&self, component_name: &str) -> Option<&Component> {
        Some(self.as_coupled().components.get(component_name)?)
    }

    /// Returns a dynamic reference to a component with the provided name.
    /// If the coupled model does not contain any model with that name, it panics.
    ///
    /// # Examples
    /// ```
    /// use xdevs::modeling::{AsCoupled, Coupled};
    ///
    /// let mut top_coupled = Coupled::new("top_coupled");
    /// // let _component = top_coupled.get_component("component_1");  // this panics (no component)
    /// top_coupled.add_component(Coupled::new("component_1"));
    /// let _component = top_coupled.get_component("component_1");
    /// ```
    fn get_component(&self, component_name: &str) -> &Component {
        self.try_get_component(component_name).unwrap_or_else(|| {
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
    /// use xdevs::modeling::{AsModel, AsCoupled, Coupled};
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
    fn add_eic(&mut self, port_from_name: &str, component_to_name: &str, port_to_name: &str) {
        let port_from = self.as_model().get_in_port(port_from_name);
        let component = self.get_component(component_to_name);
        let port_to = component.as_model().get_in_port(port_to_name);
        if !Coupled::add_coupling(&mut self.as_coupled_mut().eic, port_from, port_to) {
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
    /// use xdevs::modeling::{AsModel, AsCoupled, Coupled};
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
    fn add_ic(
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
        if !Coupled::add_coupling(&mut self.as_coupled_mut().ic, port_from, port_to) {
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
    /// use xdevs::modeling::{AsModel, AsCoupled, Coupled};
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
    fn add_eoc(&mut self, component_from_name: &str, port_from_name: &str, port_to_name: &str) {
        let component = self.get_component(component_from_name).as_model();
        let port_from = component.get_out_port(port_from_name);
        let port_to = self.as_model().get_out_port(port_to_name);
        if !Coupled::add_coupling(&mut self.as_coupled_mut().eoc, port_from, port_to) {
            panic!(
                "EOC coupling <{}::{}->{}> is already defined",
                component_from_name, port_from_name, port_to_name
            );
        }
    }
}

impl<T: AsCoupled + 'static> AsModel for T {
    fn to_component(self) -> Component {
        Component::Coupled(Box::new(self))
    }

    fn as_model(&self) -> &Model {
        &self.as_coupled().component
    }

    fn as_model_mut(&mut self) -> &mut Model {
        &mut self.as_coupled_mut().component
    }
}

/// Helper macro to implement the AsCoupled trait.
/// You can use this macro with any struct containing a field `coupled` of type [`Coupled`].
/// TODO try to use the derive stuff (it will be more elegant).
#[macro_export]
macro_rules! impl_coupled {
    ($($COUPLED:ident),+) => {
        $(
            impl AsCoupled for $COUPLED {
                fn to_coupled(self) -> Coupled {
                    self.coupled
                }
                fn as_coupled(&self) -> &Coupled {
                    &self.coupled
                }
                fn as_coupled_mut(&mut self) -> &mut Coupled {
                    &mut self.coupled
                }
            }
        )+
    }
}
pub use impl_coupled;

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
        assert!(top_coupled.try_get_component("component_1").is_none());
        top_coupled.add_component(Coupled::new("component_1"));
        assert!(top_coupled.try_get_component("component_1").is_some());
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
