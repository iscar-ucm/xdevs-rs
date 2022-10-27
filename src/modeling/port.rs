use super::Shared;
use std::any::{type_name, Any};
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::ops::{Deref, DerefMut};

#[cfg(not(feature = "parallel"))]
use std::cell::RefCell;
#[cfg(feature = "parallel")]
use std::sync::RwLock;

#[cfg(not(feature = "parallel"))]
type Mutable<T> = RefCell<T>;
#[cfg(feature = "parallel")]
type Mutable<T> = RwLock<T>;

/// DEVS port struct.
#[derive(Clone, Debug)]
pub struct Port<T> {
    /// name of the port.
    name: String,
    /// Message bag.
    bag: Shared<Mutable<Vec<T>>>,
}

impl<T> Port<T> {
    /// Constructor function.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            bag: Shared::new(Mutable::new(Vec::new())),
        }
    }

    /// Helper function to get a reference to the message bag.
    fn borrow_bag(&self) -> impl Deref<Target = Vec<T>> + '_ {
        #[cfg(not(feature = "parallel"))]
        return self.bag.borrow();
        #[cfg(feature = "parallel")]
        return self.bag.read().unwrap();
    }

    /// Helper function to get a mutable reference to the message bag.
    fn borrow_bag_mut(&self) -> impl DerefMut<Target = Vec<T>> + '_ {
        #[cfg(not(feature = "parallel"))]
        return self.bag.borrow_mut();
        #[cfg(feature = "parallel")]
        return self.bag.write().unwrap();
    }

    /// It checks if the message bag of the port is empty.
    pub fn is_empty(&self) -> bool {
        self.borrow_bag().is_empty()
    }

    /// It returns a reference to the message bag of the port.
    pub fn get_values(&self) -> impl Deref<Target = Vec<T>> + '_ {
        self.borrow_bag()
    }

    /// It returns the number of messages in the bag of the port.
    pub fn len(&self) -> usize {
        self.borrow_bag().len()
    }

    /// It adds a new value to the message bag of the port.
    pub fn add_value(&self, value: T) {
        self.borrow_bag_mut().push(value);
    }
}

impl<T: Clone> Port<T> {
    /// It adds multiple values to the message bag of the port.
    pub fn add_values(&self, values: &[T]) {
        self.borrow_bag_mut().extend_from_slice(values);
    }
}

impl<T: 'static> Port<T> {
    /// Tries to convert a trait object [`AsPort`] to a reference to [`Port<T>`].
    pub(crate) fn try_upgrade(port: &dyn AsPort) -> Option<&Port<T>> {
        port.as_any().downcast_ref::<Port<T>>()
    }

    /// Converts a trait object [`AsPort`] to a reference to [`Port<T>`].
    /// It panics if this conversion is not possible.
    pub(crate) fn upgrade(port: &dyn AsPort) -> &Port<T> {
        Port::<T>::try_upgrade(port).unwrap_or_else(|| {
            panic!(
                "port {} is incompatible with value type {}",
                port,
                type_name::<T>()
            )
        })
    }

    /// Checks if a trait object [`AsPort`] can be upgraded to a reference to [`Port<T>`].
    pub(crate) fn is_compatible(port: &dyn AsPort) -> bool {
        Port::<T>::try_upgrade(port).is_some()
    }
}

impl<T> Display for Port<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}<{}>", self.name, type_name::<T>())
    }
}

/// Interface for DEVS ports.
pub(crate) trait AsPort: Debug + Display {
    /// Port-to-any conversion.
    fn as_any(&self) -> &dyn Any;

    /// Returns the name of the port.
    fn get_name(&self) -> &str;

    /// Returns `true` if the port does not contain any value.
    fn is_empty(&self) -> bool;

    /// It clears all the values in the port.
    fn clear(&self);

    /// Checks if one port is compatible with other.
    fn is_compatible(&self, other: &dyn AsPort) -> bool;

    /// Propagates values from other port to the port.
    fn propagate(&self, other: &dyn AsPort);
}

impl<T: 'static + Clone + Debug> AsPort for Port<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn is_empty(&self) -> bool {
        self.borrow_bag().is_empty()
    }

    fn clear(&self) {
        self.borrow_bag_mut().clear();
    }

    fn is_compatible(&self, other: &dyn AsPort) -> bool {
        Port::<T>::is_compatible(other)
    }

    fn propagate(&self, port_from: &dyn AsPort) {
        self.borrow_bag_mut()
            .extend_from_slice(&*Port::<T>::upgrade(port_from).borrow_bag());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port() {
        let port_a = Port::new("port_a");
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
        let port_a = Port::<i32>::new("port_a");

        assert!(Port::<i32>::is_compatible(&port_a));
        assert!(!Port::<i64>::is_compatible(&port_a));
        assert!(Port::<i32>::try_upgrade(&port_a).is_some());
        assert!(Port::<i64>::try_upgrade(&port_a).is_none());
    }

    #[test]
    #[should_panic(expected = "port port_a<i32> is incompatible with value type i64")]
    fn test_port_upgrade_panics() {
        let port_a = Port::<i32>::new("port_a");
        Port::<i64>::upgrade(&port_a);
    }

    #[test]
    fn test_propagate() {
        let port_a = Port::new("port_a");
        let port_b = Port::new("port_b");

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
