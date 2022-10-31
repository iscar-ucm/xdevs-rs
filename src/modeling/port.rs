use super::Component;
use std::any::{type_name, Any};
use std::cell::RefCell;
use std::fmt::{Debug, Display, Formatter, Result};
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;

/// Interface for DEVS ports. This trait does not consider message types.
pub(crate) trait AbstractPort: Any + Debug + Display {
    /// Port-to-any conversion.
    fn as_any(&self) -> &dyn Any;

    /// Returns the name of the port.
    fn get_name(&self) -> &str;

    /// Returns pointer to parent component of the port.
    fn get_parent(&self) -> *const Component;

    /// Returns `true` if the port does not contain any value.
    fn is_empty(&self) -> bool;

    /// It clears all the values in the port.
    fn clear(&self);

    /// Checks if one port is compatible with other.
    fn is_compatible(&self, other: &dyn AbstractPort) -> bool;

    /// Propagates values from other port to the port.
    fn propagate(&self, other: &dyn AbstractPort);
}

/// Directionless DEVS port with an associated message type.
#[derive(Debug)]
pub(crate) struct TypedPort<T> {
    /// Name of the port.
    name: String,
    /// Pointer to parent component of the port.
    parent: *const Component,
    /// Message bag.
    pub(crate) bag: Rc<RefCell<Vec<T>>>,
}

impl<T> TypedPort<T> {
    /// Constructor function.
    pub fn new(name: &str, parent: *const Component) -> Self {
        Self {
            name: name.to_string(),
            parent,
            bag: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// It checks if the message bag of the port is empty.
    pub fn is_empty(&self) -> bool {
        self.bag.borrow().is_empty()
    }

    /// It returns a reference to the message bag of the port.
    pub fn get_values(&self) -> impl Deref<Target = Vec<T>> + '_ {
        self.bag.borrow()
    }

    /// It adds a new value to the message bag of the port.
    pub fn add_value(&self, value: T) {
        self.bag.borrow_mut().push(value);
    }
}

impl<T: Clone> TypedPort<T> {
    /// It adds multiple values to the message bag of the port.
    pub fn add_values(&self, values: &[T]) {
        self.bag.borrow_mut().extend_from_slice(values);
    }
}

impl<T: 'static> TypedPort<T> {
    /// Tries to convert a trait object [`AsPort`] to a reference to [`Port<T>`].
    pub(crate) fn try_upgrade(port: &dyn AbstractPort) -> Option<&TypedPort<T>> {
        port.as_any().downcast_ref::<TypedPort<T>>()
    }

    /// Converts a trait object [`AsPort`] to a reference to [`Port<T>`].
    /// It panics if this conversion is not possible.
    pub(crate) fn upgrade(port: &dyn AbstractPort) -> &TypedPort<T> {
        TypedPort::<T>::try_upgrade(port).unwrap_or_else(|| {
            panic!(
                "port {} is incompatible with value type {}",
                port,
                type_name::<T>()
            )
        })
    }

    /// Checks if a trait object [`AsPort`] can be upgraded to a reference to [`Port<T>`].
    pub(crate) fn is_compatible(port: &dyn AbstractPort) -> bool {
        TypedPort::<T>::try_upgrade(port).is_some()
    }
}

impl<T> Display for TypedPort<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}<{}>", self.name, type_name::<T>())
    }
}

impl<T: 'static + Clone + Debug> AbstractPort for TypedPort<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_parent(&self) -> *const Component {
        self.parent
    }

    fn is_empty(&self) -> bool {
        self.bag.borrow().is_empty()
    }

    fn clear(&self) {
        self.bag.borrow_mut().clear();
    }

    fn is_compatible(&self, other: &dyn AbstractPort) -> bool {
        TypedPort::<T>::is_compatible(other)
    }

    fn propagate(&self, port_from: &dyn AbstractPort) {
        self.bag
            .borrow_mut()
            .extend_from_slice(&*TypedPort::<T>::upgrade(port_from).bag.borrow());
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Input();

#[derive(Clone, Copy, Debug)]
pub struct Output();

/// Directive DEVS port with an associated message type.
/// This struct is useful for two reasons. First, it hides the Rc stuff from the users.
/// Second, it constraints the methods available depending on their direction.
#[derive(Clone, Debug)]
pub struct Port<D, T>(pub(crate) Rc<TypedPort<T>>, PhantomData<D>);

impl<D, T> Port<D, T> {
    pub(crate) fn new(port: Rc<TypedPort<T>>) -> Self {
        Self(port, PhantomData::default())
    }
    pub fn get_name(&self) -> &str {
        &self.0.name
    }
}

/// For input ports, we can only check if they are empty and read their values.
impl<T> Port<Input, T> {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get_values(&self) -> impl Deref<Target = Vec<T>> + '_ {
        self.0.get_values()
    }
}

/// For output ports, we can only add new values.
impl<T> Port<Output, T> {
    /// It adds a new value to the message bag of the port.
    pub fn add_value(&self, value: T) {
        self.0.add_value(value);
    }
}

impl<T: Clone> Port<Output, T> {
    /// It adds a new value to the message bag of the port.
    pub fn add_values(&self, value: &[T]) {
        self.0.add_values(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port() {
        let port_a = TypedPort::new("port_a", std::ptr::null());
        assert_eq!("port_a", port_a.get_name());
        assert_eq!("port_a<usize>", port_a.to_string());
        assert!(port_a.is_empty());
        assert_eq!(0, port_a.get_values().len());

        port_a.add_value(0);
        assert!(!port_a.is_empty());
        assert_eq!(1, port_a.get_values().len());

        port_a.clear();
        assert!(port_a.is_empty());
        assert_eq!(0, port_a.get_values().len());

        for i in 0..10 {
            port_a.add_value(i);
            assert!(port_a.get_values().get(i).is_some());
            assert!(port_a.get_values().get(i + 1).is_none());
            assert_eq!(i + 1, port_a.get_values().len());
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
        let port_a = TypedPort::<i32>::new("port_a", std::ptr::null());

        assert!(TypedPort::<i32>::is_compatible(&port_a));
        assert!(!TypedPort::<i64>::is_compatible(&port_a));
        assert!(TypedPort::<i32>::try_upgrade(&port_a).is_some());
        assert!(TypedPort::<i64>::try_upgrade(&port_a).is_none());
    }

    #[test]
    #[should_panic(expected = "port port_a<i32> is incompatible with value type i64")]
    fn test_port_upgrade_panics() {
        let port_a = TypedPort::<i32>::new("port_a", std::ptr::null());
        TypedPort::<i64>::upgrade(&port_a);
    }

    #[test]
    fn test_propagate() {
        let port_a = TypedPort::new("port_a", std::ptr::null());
        let port_b = TypedPort::new("port_b", std::ptr::null());

        for i in 0..10 {
            port_a.add_value(i);
            port_b.add_value(10 + i);
        }

        port_a.propagate(&port_b);
        assert_eq!(20, port_a.get_values().len());
        assert_eq!(10, port_b.get_values().len());

        port_b.add_value(20);
        assert_eq!(20, port_a.get_values().len());
        assert_eq!(11, port_b.get_values().len());

        port_a.clear();
        assert_eq!(0, port_a.get_values().len());
        assert_eq!(11, port_b.get_values().len());

        port_a.propagate(&port_b);
        assert_eq!(11, port_a.get_values().len());

        port_a.clear();
        port_b.clear();
    }
}
