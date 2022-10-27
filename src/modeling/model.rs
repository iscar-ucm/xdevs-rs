use super::{AsPort, Component, Port};
use crate::Shared;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

/// Generic DEVS model.
#[derive(Clone, Debug)]
pub struct Model {
    /// name of the DEVS component.
    name: String,
    /// Input port set of the DEVS component. Input port names must be unique.
    pub(crate) in_ports: HashMap<String, Shared<dyn AsPort>>,
    /// Output port set of the DEVS component. Output port names must be unique.
    pub(crate) out_ports: HashMap<String, Shared<dyn AsPort>>,
}

impl Model {
    /// It creates a new component with the provided name.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            in_ports: HashMap::new(),
            out_ports: HashMap::new(),
        }
    }

    /// Returns name of the component.
    pub fn get_name(&self) -> &str {
        &self.name
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
    pub(crate) fn add_in_port<T: 'static + Clone + Debug + Display>(&mut self, port_name: &str) -> Port<T> {
        Self::add_port(&mut self.in_ports, port_name).unwrap_or_else(|| {
            panic!(
                "component {} already contains input port with name {}",
                self.name, port_name
            )
        })
    }

    /// Adds a new output port of type [`Port<T>`] to the component and returns a reference to it.
    /// It panics if there is already an output port with the same name.
    pub(crate) fn add_out_port<T: 'static + Clone + Debug + Display>(&mut self, port_name: &str) -> Port<T> {
        Self::add_port(&mut self.out_ports, port_name).unwrap_or_else(|| {
            panic!(
                "component {} already contains output port with name {}",
                self.name, port_name
            )
        })
    }

    /// Returns a pointer to an input port with the given name.
    /// If the component does not have any input port with this name, it returns [`None`].
    fn try_get_in_port(&self, port_name: &str) -> Option<Shared<dyn AsPort>> {
        Some(Shared::clone(self.in_ports.get(port_name)?))
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
        Some(Shared::clone(self.out_ports.get(port_name)?))
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

    pub fn clear_in_ports(&mut self) {
        self.in_ports.values().for_each(|p| p.clear())
    }

    pub fn clear_out_ports(&mut self) {
        self.out_ports.values().for_each(|p| p.clear())
    }

    pub fn is_input_empty(&self) -> bool {
        self.in_ports.values().all(|p| p.is_empty())
    }

    pub fn is_output_empty(&self) -> bool {
        self.out_ports.values().all(|p| p.is_empty())
    }
}

impl Display for Model {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.name)
    }
}

/// Interface for DEVS components.
pub trait AsModel: Debug {
    fn to_component(self) -> Component;
    /// All the DEVS component must own a [`Model`] struct.
    /// This method returns a reference to this inner [`Model`].
    fn as_model(&self) -> &Model;

    /// All the DEVS component must own a [`Model`] struct.
    /// This method returns a mutable reference to this inner [`Model`].
    fn as_model_mut(&mut self) -> &mut Model;

    /// Returns the name of the component.
    fn get_name(&self) -> &str {
        &self.as_model().name
    }

    /// Adds a new input port of type [`Port<T>`] to the component and returns a reference to it.
    /// It panics if there is already an input port with the same name.
    fn add_in_port<T>(&mut self, port_name: &str) -> Port<T>
    where
        T: 'static + Clone + Debug + Display,
        Self: Sized,
    {
        self.as_model_mut().add_in_port(port_name)
    }

    /// Adds a new output port of type [`Port<T>`] to the component and returns a reference to it.
    /// It panics if there is already an output port with the same name.
    fn add_out_port<T>(&mut self, port_name: &str) -> Port<T>
    where
        T: 'static + Clone + Debug + Display,
        Self: Sized,
    {
        self.as_model_mut().add_out_port(port_name)
    }
}

/// Helper macro to implement the AsModel trait.
/// You can use this macro with any struct containing a field `model` of type [`Model`].
/// TODO try to use the derive stuff (it will be more elegant).
#[macro_export]
macro_rules! impl_model {
    ($($MODEL:ident),+) => {
        $(
            impl AsModel for $MODEL {
                fn to_component(self) -> Component {
                    Component::Atomic(Box::new(self))
                }
                fn as_model(&self) -> &Model {
                    &self.model
                }
                fn as_model_mut(&mut self) -> &mut Model {
                    &mut self.model
                }
            }
        )+
    }
}
pub use impl_model;

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
        assert_eq!(1, a.in_ports.len());
        assert_eq!(0, a.out_ports.len());
        let _port = a.add_in_port::<i32>("i32");
    }

    #[test]
    #[should_panic(expected = "component component_a already contains output port with name i32")]
    fn test_duplicate_out_port() {
        let mut a = Model::new("component_a");
        let _port = a.add_out_port::<i32>("i32");
        assert_eq!(0, a.in_ports.len());
        assert_eq!(1, a.out_ports.len());
        let _port = a.add_out_port::<f64>("i32");
    }

    #[test]
    fn test_component() {
        let mut a = Model::new("component_a");
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
