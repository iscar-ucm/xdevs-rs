use std::any::Any;
use std::ops::Deref;
use std::rc::Rc;
use std::cell::RefCell;

pub trait Message: 'static + Clone {}

impl<T: 'static + Clone> Message for T {}

/// DEVS coupling.
pub(crate) trait Coupling {
    /// It propagates the messages from a source to a destination port.
    fn propagate(&self);
}

/// DEVS ports. It does not consider message types nor port directions.
pub(crate) trait Port {
    /// Port-to-any conversion.
    fn as_any(&self) -> &dyn Any;

    /// Returns `true` if the port does not contain any value.
    fn is_empty(&self) -> bool;

    /// It clears all the values in the port.
    fn clear(&mut self);

    /// Returns `true` if other port is compatible.
    fn is_compatible(&self, other: &dyn Port) -> bool;

    fn propagate(&self, port_to: &dyn Port);

    /// Creates a new coupling from this prot to other port.
    /// If ports are incompatible, it returns [`None`].
    fn new_coupling(&self, other: &dyn Port) -> Option<Box<dyn Coupling>>;
}

pub(crate) type Bag<T> = Rc<RefCell<Vec<T>>>;

pub(crate) fn new_bag<T>() -> Bag<T> {
    Rc::new(RefCell::new(Vec::new()))
}

/// Input port. It is just a wrapper of a [`Bag`].
/// This structure only allows reading messages from the underlying bag.
/// Thus, it cannot inject messages to the bag.
#[derive(Debug)]
pub struct InPort<T>(Bag<T>);

impl<T> InPort<T> {
    pub(crate) fn new(bag: Bag<T>) -> Self {
        Self(bag)
    }

    /// Returns `true` if the underlying bag is empty. Otherwise, it returns `false`.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.borrow().is_empty()
    }

    /// Returns a reference to the slice of messages of the underlying bag.
    #[inline]
    pub fn get_values(&self) -> impl Deref<Target=Vec<T>> + '_ {
        self.0.borrow()
    }
}

/// Output port. It is just a wrapper of a [`Bag`].
/// This structure only injecting messages to the underlying bag.
/// Thus, it cannot read the messages in the bag.
#[derive(Debug)]
pub struct OutPort<T>(Bag<T>);

impl<T> OutPort<T> {
    pub(crate) fn new(bag: Bag<T>) -> Self {
        Self(bag)
    }

    /// Adds a new value to the output port.
    #[inline]
    pub fn add_value(&self, value: T) {
        self.0.borrow_mut().push(value);
    }
}

impl<T: Clone> OutPort<T> {
    /// Adds new values from a slice to the output port.
    #[inline]
    pub fn add_values(&self, values: &[T]) {
        self.0.borrow_mut().extend_from_slice(values);
    }
}

/// Coupling edge between a destination port and a source port.
struct Edge<T> {
    /// Source port.
    port_from: InPort<T>,
    /// Destination port.
    port_to: OutPort<T>,
}

impl<T: Clone> Coupling for Edge<T> {
    #[inline]
    fn propagate(&self) {
        self.port_to.add_values(&self.port_from.get_values());
    }
}

impl<T: 'static + Clone> Port for Bag<T> {
    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.borrow().is_empty()
    }

    #[inline]
    fn clear(&mut self) {
        self.borrow_mut().clear();
    }

    #[inline]
    fn is_compatible(&self, other: &dyn Port) -> bool {
        other.as_any().downcast_ref::<Bag<T>>().is_some()
    }

    fn propagate(&self, port_to: &dyn Port) {
        let port_to = port_to.as_any().downcast_ref::<Bag<T>>().unwrap();
        port_to.borrow_mut().extend_from_slice(&self.borrow());
    }

    #[inline]
    fn new_coupling(&self, other: &dyn Port) -> Option<Box<dyn Coupling>> {
        let other = other.as_any().downcast_ref::<Bag<T>>()?;
        Some(Box::new(Edge{ port_from: InPort(self.clone()), port_to: OutPort(other.clone()) }))
    }
}

impl<T: Port + ?Sized> Port for Box<T> {
    #[inline]
    fn as_any(&self) -> &dyn Any {
        (**self).as_any()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        (**self).is_empty()
    }

    #[inline]
    fn clear(&mut self) {
        (**self).clear();
    }

    #[inline]
    fn is_compatible(&self, other: &dyn Port) -> bool {
        (**self).is_compatible(other)
    }

    fn propagate(&self, port_to: &dyn Port) {
        (**self).propagate(port_to)
    }

    #[inline]
    fn new_coupling(&self, other: &dyn Port) -> Option<Box<dyn Coupling>> {
        (**self).new_coupling(other)
    }
}
