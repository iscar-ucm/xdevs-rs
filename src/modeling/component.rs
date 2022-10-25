use super::{Port, PortInterface};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::ptr::null;
use std::rc::Rc;

/// DEVS component.
#[derive(Debug)]
pub struct Component {
    /// name of the DEVS component.
    name: String,
    /// Raw pointer to parent component.
    parent: *const Component,
    /// Input port set of the DEVS component.
    /// Input ports are arranged in a [`HashMap`] which keys are the port names.
    /// Thus, input port names must be unique.
    in_ports: HashMap<String, Rc<dyn PortInterface>>,
    /// Output port set of the DEVS component.
    /// Output ports are arranged in a [`HashMap`] which keys are the port names.
    /// Thus, output port names must be unique.
    out_ports: HashMap<String, Rc<dyn PortInterface>>,
    /// Serialized version of the input port set of the DEVS component.
    /// It is faster to iterate over a vector than over the values of a hash map.
    _in_ports: Vec<Rc<dyn PortInterface>>,
    /// Serialized version of the output port set of the DEVS component.
    /// It is faster to iterate over a vector than over the values of a hash map.
    _out_ports: Vec<Rc<dyn PortInterface>>,
}

/// Struct containing features that are common to all DEVS components.
impl Component {
    /// It creates a new component with the provided name. By default, parent component is null.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            parent: null(),
            in_ports: HashMap::new(),
            out_ports: HashMap::new(),
            _in_ports: Vec::new(),
            _out_ports: Vec::new(),
        }
    }

    /// Helper function to add a port to a component regardless of whether it is input or output.
    fn add_port<T: 'static + Copy + Debug + Display>(
        this: *const Self,
        ports_map: &mut HashMap<String, Rc<dyn PortInterface>>,
        ports_vec: &mut Vec<Rc<dyn PortInterface>>,
        port_name: &str,
    ) -> Option<Rc<Port<T>>> {
        if ports_map.contains_key(port_name) {
            return None;
        }
        let port_ref = Rc::new(Port::<T>::new(port_name, this));
        let port_dyn_ref = port_ref.clone() as Rc<dyn PortInterface>;
        ports_map.insert(port_name.to_string(), port_dyn_ref.clone());
        ports_vec.push(port_dyn_ref);
        Some(port_ref)
    }

    /// Adds a new input port of type [`Port<T>`] to the component and returns a reference to it.
    /// It panics if there is already an input port with the same name.
    fn _add_in_port<T: 'static + Copy + Debug + Display>(
        &mut self,
        port_name: &str,
    ) -> Rc<Port<T>> {
        Self::add_port(self, &mut self.in_ports, &mut self._in_ports, port_name).unwrap_or_else(
            || {
                panic!(
                    "component {} already contains input port with name {}",
                    self.name, port_name
                )
            },
        )
    }

    /// Adds a new output port of type [`Port<T>`] to the component and returns a reference to it.
    /// It panics if there is already an output port with the same name.
    fn _add_out_port<T: 'static + Copy + Debug + Display>(
        &mut self,
        port_name: &str,
    ) -> Rc<Port<T>> {
        Self::add_port(self, &mut self.out_ports, &mut self._out_ports, port_name).unwrap_or_else(
            || {
                panic!(
                    "component {} already contains output port with name {}",
                    self.name, port_name
                )
            },
        )
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

    /// Returns a raw pointer to parent component.
    fn get_parent(&self) -> *const Component {
        self.as_component().parent
    }

    /// Sets the raw pointer to parent component.
    fn set_parent(&mut self, parent: *const Component) {
        self.as_component_mut().parent = parent;
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
    fn add_in_port<T: 'static + Copy + Debug + Display>(&mut self, port_name: &str) -> Rc<Port<T>>
    where
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
    fn add_out_port<T: 'static + Copy + Debug + Display>(&mut self, port_name: &str) -> Rc<Port<T>>
    where
        Self: Sized,
    {
        self.as_component_mut()._add_out_port(port_name)
    }

    /// It performs all the required actions before a simulation. By default, it does nothing.
    fn initialize(&mut self) {}

    /// It performs all the required actions after a simulation. By default, it does nothing.
    fn exit(&mut self) {}

    /// Returns `true` if all the input ports of the component are empty.
    ///
    /// # Examples
    /// ```
    /// use xdevs::modeling::{AsComponent, Component};
    /// let mut component = Component::new("component");
    /// let port_1 = component.add_in_port::<i32>("port_1");
    /// assert!(component.is_input_empty());
    /// port_1.add_value(2);
    /// assert!(!component.is_input_empty());
    /// ```
    fn is_input_empty(&self) -> bool {
        self.as_component()
            ._in_ports
            .iter()
            .all(|port| port.is_empty())
    }

    /// Returns `true` if all the output ports are empty.
    ///
    /// # Examples
    /// ```
    /// use xdevs::modeling::{AsComponent, Component};
    /// let mut component = Component::new("component");
    /// let port_1 = component.add_out_port::<i32>("port_1");
    /// assert!(component.is_output_empty());
    /// port_1.add_value(2);
    /// assert!(!component.is_output_empty());
    /// ```
    fn is_output_empty(&self) -> bool {
        self.as_component()
            ._out_ports
            .iter()
            .all(|port| port.is_empty())
    }

    /// Clears all the input ports of the component.
    ///
    /// # Examples
    /// ```
    /// use xdevs::modeling::{AsComponent, Component};
    /// let mut component = Component::new("component");
    /// let port_1 = component.add_in_port::<i32>("port_1");
    /// port_1.add_value(2);
    /// assert!(!component.is_input_empty());
    /// component.clear_in_ports();
    /// assert!(component.is_input_empty());
    /// ```
    fn clear_in_ports(&mut self) {
        self.as_component_mut()
            ._in_ports
            .iter()
            .for_each(|port| port.clear());
    }

    /// Clears all the output ports of the component.
    ///
    /// # Examples
    /// ```
    /// use xdevs::modeling::{AsComponent, Component};
    /// let mut component = Component::new("component");
    /// let port_1 = component.add_out_port::<i32>("port_1");
    /// port_1.add_value(2);
    /// assert!(!component.is_output_empty());
    /// component.clear_out_ports();
    /// assert!(component.is_output_empty());
    /// ```
    fn clear_out_ports(&mut self) {
        self.as_component_mut()
            ._out_ports
            .iter()
            .for_each(|port| port.clear());
    }

    /// Returns a pointer to an input port with the given name.
    /// If the component does not have any input port with this name, it returns [`None`].
    ///
    /// # Examples
    /// ```
    /// use xdevs::modeling::{AsComponent, Component, Port, PortInterface};
    /// let mut component = Component::new("component");
    /// let port_1 = component.add_in_port::<i32>("port_1");
    /// let port_try = component.try_get_in_port("port_1");
    /// assert!(port_try.is_some());
    /// assert!(component.try_get_in_port("port_2").is_none());
    /// ```
    fn try_get_in_port(&self, port_name: &str) -> Option<Rc<dyn PortInterface>> {
        Some(self.as_component().in_ports.get(port_name)?.clone())
    }

    /// Returns a pointer to an input port with the given name.
    /// If the component does not have any input port with this name, it panics.
    ///
    /// # Examples
    /// ```
    /// use xdevs::modeling::{AsComponent, Component, PortInterface};
    /// let mut component = Component::new("component");
    /// let port_1 = component.add_in_port::<i32>("port_1");
    /// // assert!(component.get_in_port("port_2").is_none());  // this panics, as port_2 does not exist.
    /// ```
    fn get_in_port(&self, port_name: &str) -> Rc<dyn PortInterface> {
        self.try_get_in_port(port_name).unwrap_or_else(|| {
            panic!(
                "component {} does not contain input port with name {}",
                self.get_name(),
                port_name
            )
        })
    }

    /// Returns a pointer to an output port with the given name.
    /// If the component does not have any output port with this name, it returns [`None`].
    ///
    /// # Examples
    /// ```
    /// use xdevs::modeling::{AsComponent, Component, PortInterface};
    /// let mut component = Component::new("component");
    /// let port_1 = component.add_out_port::<i32>("port_1");
    /// let port_try = component.try_get_out_port("port_1");
    /// assert!(port_try.is_some());
    /// assert!(component.try_get_out_port("port_2").is_none());
    /// ```
    fn try_get_out_port(&self, port_name: &str) -> Option<Rc<dyn PortInterface>> {
        Some(self.as_component().out_ports.get(port_name)?.clone())
    }

    /// Returns a pointer to an output port with the given name.
    /// If the component does not have any output port with this name, it panics.
    ///
    /// # Examples
    /// ```
    /// use xdevs::modeling::{AsComponent, Component, PortInterface};
    /// let mut component = Component::new("component");
    /// let port_1 = component.add_out_port::<i32>("port_1");
    /// // assert!(component.get_out_port("port_2").is_none());  // this panics, as port_2 does not exist.
    /// ```
    fn get_out_port(&self, port_name: &str) -> Rc<dyn PortInterface> {
        self.try_get_out_port(port_name).unwrap_or_else(|| {
            panic!(
                "component {} does not contain output port with name {}",
                self.get_name(),
                port_name
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

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
        assert_eq!(3, Rc::strong_count(&in_i32));

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
