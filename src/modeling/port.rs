use super::Component;
use std::any::{type_name, Any};
use std::cell::{Ref, RefCell};
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

/// Interface for DEVS ports.
pub trait PortInterface: Debug + Display {
    /// Returns the name of the port.
    fn get_name(&self) -> &str;

    /// Returns raw pointer to parent component of the port.
    fn get_parent(&self) -> *const Component;

    /// Port-to-any conversion.
    fn as_any(&self) -> &dyn Any;

    /// Returns `true` if the port does not contain any value.
    fn is_empty(&self) -> bool;

    /// It clears all the values in the port.
    fn clear(&self);

    /// Checks if one port is compatible with other.
    fn is_compatible(&self, other: &dyn PortInterface) -> bool;

    /// Propagates values from other port to the port.
    fn propagate(&self, other: &dyn PortInterface);
}

/// DEVS port.
#[derive(Debug)]
pub struct Port<T: 'static + Copy + Debug + Display> {
    /// Name of the port.
    name: String,
    /// Pointer to parent component.
    parent: *const Component,
    /// Bag of values in the port. It is wrapped by a [`RefCell`] to provide interior mutability.
    values: RefCell<Vec<T>>,
}

impl<T: 'static + Copy + Debug + Display> Port<T> {
    /// Creates a new port with the given name. The port must belong to a DEVS component.
    pub fn new(name: &str, parent: *const Component) -> Self {
        Self {
            name: name.to_string(),
            parent,
            values: RefCell::new(vec![]),
        }
    }

    /// Adds a new value to the port.
    ///
    /// # Examples
    /// ```
    /// use xdevs::modeling::Port;
    /// let mut port = Port::<i32>::new("my_port", std::ptr::null());
    /// port.add_value(1);
    /// ```
    pub fn add_value(&self, value: T) {
        self.values.borrow_mut().push(value);
    }

    /// Adds multiple values to the port.
    ///
    /// # Examples
    /// ```
    /// use xdevs::modeling::Port;
    /// let mut port = Port::<i32>::new("my_port", std::ptr::null());
    /// let values = vec![1, 2, 3];
    /// port.add_values(&values);
    pub fn add_values(&self, values: &[T]) {
        self.values.borrow_mut().extend(values);
    }

    /// Checks if the port contains any message.
    ///
    /// # Examples
    /// ```
    /// use xdevs::modeling::Port;
    /// let mut port = Port::<i32>::new("my_port", std::ptr::null());
    /// assert!(port.is_empty());
    /// port.add_value(2);
    /// assert!(!port.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.values.borrow().is_empty()
    }

    /// Returns the number of messages in the port.
    ///
    /// # Examples
    /// ```
    /// use xdevs::modeling::Port;
    /// let mut port = Port::<i32>::new("my_port", std::ptr::null());
    /// assert_eq!(0, port.len());
    /// port.add_value(2);
    /// assert_eq!(1, port.len());
    /// ```
    pub fn len(&self) -> usize {
        self.values.borrow().len()
    }

    /// It returns a [`Ref`] to the values in the port.
    ///
    /// # Examples
    /// ```
    /// use xdevs::modeling::Port;
    /// let mut port = Port::<i32>::new("my_port", std::ptr::null());
    /// for i in 0..10 {
    ///     port.add_value(i);  // We add 10 values from 0 to 9.
    /// }
    /// let mut i = 0;
    /// let vals = port.get_values();
    /// assert_eq!(10, vals.len());
    /// for val in vals.iter() {
    ///     assert_eq!(i, *val);
    ///     i += 1;
    /// }
    /// ```
    pub fn get_values(&self) -> Ref<Vec<T>> {
        self.values.borrow()
    }

    /// Converts a reference to [`PortInterface`] trait to a reference to [`Port<T>`].
    /// If this conversion is not possible, it returns [`None`].
    /// # Examples
    /// ```
    /// use xdevs::modeling::Port;
    /// let port_a = Port::<i32>::new("port_a", std::ptr::null());  // Port implements the PortInterface trait
    /// assert!(Port::<i32>::try_upgrade(&port_a).is_some());
    /// assert!(Port::<i64>::try_upgrade(&port_a).is_none());
    /// ```
    pub fn try_upgrade(port: &dyn PortInterface) -> Option<&Port<T>> {
        port.as_any().downcast_ref::<Port<T>>()
    }

    /// Converts a reference to [`PortInterface`] trait to a reference to [`Port<T>`].
    /// If this conversion is not possible, it panics.
    pub fn upgrade(port: &dyn PortInterface) -> &Port<T> {
        Port::<T>::try_upgrade(port).unwrap_or_else(|| {
            panic!(
                "port {} is incompatible with value type {}",
                port,
                type_name::<T>()
            )
        })
    }

    /// Checks if a reference to [`PortInterface`] can be upgraded to a reference to [`Port<T>`].
    ///
    /// # Examples
    /// ```
    /// use xdevs::modeling::Port;
    /// let mut port_a = Port::<i32>::new("port_a", std::ptr::null());  // Port implements the PortInterface trait
    /// assert!(Port::<i32>::is_compatible(&port_a));
    /// assert!(!Port::<i64>::is_compatible(&port_a));
    /// ```
    pub fn is_compatible(port: &dyn PortInterface) -> bool {
        Port::<T>::try_upgrade(port).is_some()
    }
}

impl<T: 'static + Copy + Debug + Display> Display for Port<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}<{}>", self.name, type_name::<T>())
    }
}

impl<T: 'static + Copy + Debug + Display> PortInterface for Port<T> {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_parent(&self) -> *const Component {
        self.parent
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn is_empty(&self) -> bool {
        self.values.borrow().is_empty()
    }

    fn clear(&self) {
        self.values.borrow_mut().clear();
    }

    fn is_compatible(&self, other: &dyn PortInterface) -> bool {
        Port::<T>::is_compatible(other)
    }

    fn propagate(&self, port_from: &dyn PortInterface) {
        self.values
            .borrow_mut()
            .extend(&*Port::<T>::upgrade(port_from).values.borrow());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr::null;

    #[test]
    fn test_port() {
        let port_a = Port::new("port_a", null());
        assert_eq!("port_a", port_a.get_name());
        assert_eq!("port_a<usize>", port_a.to_string());
        assert!(port_a.is_empty());
        assert_eq!(0, port_a.len());

        port_a.add_value(0);
        assert!(!port_a.is_empty());
        assert_eq!(1, port_a.len());

        port_a.clear();
        assert!(port_a.is_empty());
        assert_eq!(0, port_a.len());

        for i in 0..10 {
            port_a.add_value(i);
            assert!(port_a.get_values().get(i).is_some());
            assert!(port_a.get_values().get(i + 1).is_none());
            assert_eq!(i + 1, port_a.len());
        }

        let mut i = 0;
        let vals = port_a.get_values();
        for event in vals.iter() {
            assert_eq!(&i, event);
            i += 1;
        }
    }

    #[test]
    fn test_port_trait() {
        let port_a = Port::<i32>::new("port_a", null());

        assert!(Port::<i32>::is_compatible(&port_a));
        assert!(!Port::<i64>::is_compatible(&port_a));
        assert!(Port::<i32>::try_upgrade(&port_a).is_some());
        assert!(Port::<i64>::try_upgrade(&port_a).is_none());
    }

    #[test]
    #[should_panic(expected = "port port_a<i32> is incompatible with value type i64")]
    fn test_port_upgrade_panics() {
        let port_a = Port::<i32>::new("port_a", null());
        Port::<i64>::upgrade(&port_a);
    }

    #[test]
    fn test_propagate() {
        let port_a = Port::new("port_a", null());
        let port_b = Port::new("port_b", null());

        for i in 0..10 {
            port_a.add_value(i);
            port_b.add_value(10 + i);
        }

        port_a.propagate(&port_b);
        assert_eq!(20, port_a.len());
        assert_eq!(10, port_b.len());

        port_b.add_value(20);
        assert_eq!(20, port_a.len());
        assert_eq!(11, port_b.len());

        port_a.clear();
        assert_eq!(0, port_a.len());
        assert_eq!(11, port_b.len());

        port_a.propagate(&port_b);
        assert_eq!(11, port_a.len());

        port_a.clear();
        port_b.clear();
    }
}
