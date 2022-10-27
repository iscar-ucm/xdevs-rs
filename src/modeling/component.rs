use super::{AsPort, Port, Shared};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

/// DEVS component.
#[derive(Clone, Debug)]
pub struct Component {
    /// name of the DEVS component.
    pub(crate) name: String,
    /// Input port set of the DEVS component.
    /// Input ports are arranged in a [`HashMap`] which keys are the port names.
    /// Thus, input port names must be unique.
    in_ports: HashMap<String, Shared<dyn AsPort>>,
    /// Output port set of the DEVS component.
    /// Output ports are arranged in a [`HashMap`] which keys are the port names.
    /// Thus, output port names must be unique.
    out_ports: HashMap<String, Shared<dyn AsPort>>,
}

/// Struct containing features that are common to all DEVS components.
impl Component {
    /// It creates a new component with the provided name. By default, parent component is null.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            in_ports: HashMap::new(),
            out_ports: HashMap::new(),
        }
    }

    /// Helper function to add a port to a component regardless of whether it is input or output.
    fn add_port<T: 'static + Clone + Debug + Display>(
        ports_map: &mut HashMap<String, Shared<dyn AsPort>>,
        port_name: &str,
    ) -> Option<Port<T>> {
        if ports_map.contains_key(port_name) {
            return None;
        }
        let port = Port::<T>::new(port_name);
        ports_map.insert(port_name.to_string(), Shared::new(port.clone()));
        Some(port)
    }

    /// Adds a new input port of type [`Port<T>`] to the component and returns a reference to it.
    /// It panics if there is already an input port with the same name.
    fn _add_in_port<T: 'static + Clone + Debug + Display>(&mut self, port_name: &str) -> Port<T> {
        Self::add_port(&mut self.in_ports, port_name).unwrap_or_else(|| {
            panic!(
                "component {} already contains input port with name {}",
                self.name, port_name
            )
        })
    }

    /// Adds a new output port of type [`Port<T>`] to the component and returns a reference to it.
    /// It panics if there is already an output port with the same name.
    fn _add_out_port<T: 'static + Clone + Debug + Display>(&mut self, port_name: &str) -> Port<T> {
        Self::add_port(&mut self.out_ports, port_name).unwrap_or_else(|| {
            panic!(
                "component {} already contains output port with name {}",
                self.name, port_name
            )
        })
    }

    /// Returns `true` if all the input ports of the component are empty.
    pub(crate) fn is_input_empty(&self) -> bool {
        self.in_ports.values().all(|port| port.is_empty())
    }

    /// Returns `true` if all the output ports are empty.
    pub(crate) fn is_output_empty(&self) -> bool {
        self.out_ports.values().all(|port| port.is_empty())
    }

    /// Clears all the input ports of the component.
    pub(crate) fn clear_in_ports(&mut self) {
        self.in_ports.values().for_each(|port| port.clear());
    }

    /// Clears all the output ports of the component.
    pub(crate) fn clear_out_ports(&mut self) {
        self.out_ports.values().for_each(|port| port.clear());
    }

    /// Returns a pointer to an input port with the given name.
    /// If the component does not have any input port with this name, it returns [`None`].
    fn try_get_in_port(&self, port_name: &str) -> Option<Shared<dyn AsPort>> {
        Some(Shared::clone(self.as_component().in_ports.get(port_name)?))
    }

    /// Returns a pointer to an input port with the given name.
    /// If the component does not have any input port with this name, it panics.
    pub(crate) fn get_in_port(&self, port_name: &str) -> Shared<dyn AsPort> {
        self.try_get_in_port(port_name).unwrap_or_else(|| {
            panic!(
                "component {} does not contain input port with name {}",
                self.name, port_name
            )
        })
    }

    /// Returns a pointer to an output port with the given name.
    /// If the component does not have any output port with this name, it returns [`None`].
    fn try_get_out_port(&self, port_name: &str) -> Option<Shared<dyn AsPort>> {
        Some(Shared::clone(self.as_component().out_ports.get(port_name)?))
    }

    /// Returns a pointer to an output port with the given name.
    /// If the component does not have any output port with this name, it panics.
    pub(crate) fn get_out_port(&self, port_name: &str) -> Shared<dyn AsPort> {
        self.try_get_out_port(port_name).unwrap_or_else(|| {
            panic!(
                "component {} does not contain output port with name {}",
                self.name, port_name
            )
        })
    }
}

impl Display for Component {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.name)
    }
}

impl AsComponent for Component {
    fn as_component(&self) -> &Component {
        self
    }
    fn as_component_mut(&mut self) -> &mut Component {
        self
    }
}

/// Interface for DEVS components.
pub trait AsComponent: Debug {
    /// All the DEVS component must own a [`Component`] struct.
    /// This method returns a reference to this inner [`Component`].
    fn as_component(&self) -> &Component;

    /// All the DEVS component must own a [`Component`] struct.
    /// This method returns a mutable reference to this inner [`Component`].
    fn as_component_mut(&mut self) -> &mut Component;

    /// Returns the name of the component.
    fn get_name(&self) -> &str {
        &self.as_component().name
    }

    /// Adds a new input port of type [`Port<T>`] to the component and returns a reference to it.
    /// It panics if there is already an input port with the same name.
    ///
    /// # Examples
    /// ```
    /// use xdevs::modeling::{AsComponent, Component};
    /// let mut component = Component::new("component");
    /// let port_1 = component.add_in_port::<i32>("port_1");
    /// // let port_2 = component.add_in_port::<i32>("port_1");  // This panics, as there is already an output port with this name
    /// let port_2 = component.add_in_port::<i64>("port_2");  // A component can have output ports of different types
    /// ```
    fn add_in_port<T>(&mut self, port_name: &str) -> Port<T>
    where
        T: 'static + Clone + Debug + Display,
        Self: Sized,
    {
        self.as_component_mut()._add_in_port(port_name)
    }

    /// Adds a new output port of type [`Port<T>`] to the component and returns a reference to it.
    /// It panics if there is already an output port with the same name.
    ///
    /// # Examples
    /// ```
    /// use xdevs::modeling::{AsComponent, Component};
    /// let mut component = Component::new("component");
    /// let port_1 = component.add_out_port::<i32>("port_1");
    /// // let port_2 = component.add_out_port::<i32>("port_1");  // This panics, as there is already an output port with this name
    /// let port_2 = component.add_out_port::<i64>("port_2");  // A component can have output ports of different types
    /// ```
    fn add_out_port<T>(&mut self, port_name: &str) -> Port<T>
    where
        T: 'static + Clone + Debug + Display,
        Self: Sized,
    {
        self.as_component_mut()._add_out_port(port_name)
    }

    /// It performs all the required actions before a simulation. By default, it does nothing.
    fn initialize(&mut self) {}

    /// It performs all the required actions after a simulation. By default, it does nothing.
    fn exit(&mut self) {}
}

/// Helper macro to implement the AsComponent trait.
/// You can use this macro with any struct containing a field `component` of type [`Component`].
/// TODO try to use the derive stuff (it will be more elegant).
#[macro_export]
macro_rules! impl_component {
    ($($COMPONENT:ident),+) => {
        $(
            impl AsComponent for $COMPONENT {
                fn as_component(&self) -> &Component {
                    &self.component
                }
                fn as_component_mut(&mut self) -> &mut Component {
                    &mut self.component
                }
            }
        )+
    }
}
pub use impl_component;

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
        assert_eq!(1, a.in_ports.len());
        assert_eq!(0, a.out_ports.len());
        let _port = a.add_in_port::<i32>("i32");
    }

    #[test]
    #[should_panic(expected = "component component_a already contains output port with name i32")]
    fn test_duplicate_out_port() {
        let mut a = Component::new("component_a");
        let _port = a.add_out_port::<i32>("i32");
        assert_eq!(0, a.in_ports.len());
        assert_eq!(1, a.out_ports.len());
        let _port = a.add_out_port::<f64>("i32");
    }

    #[test]
    fn test_component() {
        let mut a = Component::new("component_a");
        let in_i32 = a.add_in_port::<i32>("i32");
        let out_i32 = a.add_out_port::<i32>("i32");
        let out_f64 = a.add_out_port::<f64>("f64");

        assert_eq!("component_a", a.name);
        assert_eq!(1, a.in_ports.len());
        assert_eq!(2, a.out_ports.len());
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

        a.clear_out_ports();
        assert!(a.is_input_empty());
        assert!(a.is_output_empty());

        in_i32.add_value(1);
        assert!(!a.is_input_empty());
        assert!(a.is_output_empty());
        {
            let port_dyn_ref = a.get_in_port("i32");
            assert!(!port_dyn_ref.is_empty());
        }

        a.clear_in_ports();
        assert!(a.is_input_empty());
        assert!(a.is_output_empty());
    }
}
