use std::any::Any;
use std::ops::{Deref, DerefMut};

/// DEVS coupling.
pub(crate) trait Coupling {
    /// It propagates the messages from a source to a destination port.
    fn propagate(&self);
}

/// Intermediate trait for creating couplings without breaking borrowing rules.
pub(crate) trait HalfCoupling {
    /// It completes a coupling fiven a source port. If ports are incompatible, it returns ['None'].
    fn new_coupling(&self, from: &dyn Port) -> Option<Box<dyn Coupling>>;
}

/// DEVS ports. It does not consider message types nor port directions.
pub(crate) trait Port {
    /// Port-to-any conversion.
    fn as_any(&self) -> &dyn Any;

    /// Returns `true` if the port does not contain any value.
    fn is_empty(&self) -> bool;

    /// It clears all the values in the port.
    fn clear(&mut self);

    /// Creates a new coupling from other port to this port.
    /// If ports are incompatible, it returns [`None`].
    fn half_coupling(&mut self) -> Box<dyn HalfCoupling>;
}

///  Message bag. This is the primary artifact for implementing DEVS ports.
#[derive(Debug)]
pub(crate) struct Bag<T>(Vec<T>);

impl<T> Bag<T> {
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }
}

impl<T> Deref for Bag<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Bag<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Input port. It is just a wrapper to a constant pointer to a [`Bag`].
/// This structure only allows reading messages from the underlying bag.
/// Thus, it cannot inject messages to the bag. This is key to ensure safety.
#[derive(Debug)]
pub struct InPort<T>(*const Bag<T>);

impl<T: 'static + Clone> InPort<T> {
    pub(crate) fn new(bag: &Bag<T>) -> Self {
        Self(bag)
    }

    /// Returns `true` if the underlying bag is empty. Otherwise, it returns `false`.
    ///
    /// # Safety
    ///
    /// Users must only use this method while on a transition function of
    /// an atomic model. Otherwise, it may lead to undefined behavior.
    #[inline]
    pub fn is_empty(&self) -> bool {
        unsafe { (*self.0).is_empty() }
    }

    /// Returns a reference to the slice of messages of the underlying bag.
    ///
    /// # Safety
    ///
    /// Users must only use this method while on a transition function of
    /// an atomic model. Otherwise, it may lead to undefined behavior.
    #[inline]
    pub fn get_values(&self) -> &[T] {
        unsafe { &(*self.0) }
    }
}

/// Output port. It is just a wrapper to a mutable pointer to a [`Bag`].
/// This structure only injecting messages to the underlying bag.
/// Thus, it cannot read the messages in the bag. This is key to ensure safety.
#[derive(Debug)]
pub struct OutPort<T>(*mut Bag<T>);

impl<T: 'static + Clone> OutPort<T> {
    pub(crate) fn new(bag: &mut Bag<T>) -> Self {
        Self(bag)
    }

    /// Adds a new value to the output port.
    ///
    /// # Safety
    ///
    /// Users must only use this method while on an output function of
    /// an atomic model. Otherwise, it may lead to undefined behavior.
    #[inline]
    pub fn add_value(&self, value: T) {
        unsafe {
            (*self.0).push(value);
        }
    }

    /// Adds new values from a slice to the output port.
    ///
    /// # Safety
    ///
    /// Users must only use this method while on an output function of
    /// an atomic model. Otherwise, it may lead to undefined behavior.
    #[inline]
    pub fn add_values(&self, values: &[T]) {
        unsafe {
            (*self.0).extend_from_slice(values);
        }
    }
}

/// Coupling edge between a destination port and a source port.
struct Edge<T> {
    /// Source port.
    port_from: InPort<T>,
    /// Destination port.
    port_to: OutPort<T>,
}

impl<T: 'static + Clone> HalfCoupling for OutPort<T> {
    fn new_coupling(&self, from: &dyn Port) -> Option<Box<dyn Coupling>> {
        let port_from = InPort::new(from.as_any().downcast_ref::<Bag<T>>()?);
        let port_to = OutPort(self.0);
        Some(Box::new(Edge{port_from, port_to}))
    }
}

impl<T: HalfCoupling> HalfCoupling for Box<T> {
    #[inline]
    fn new_coupling(&self, from: &dyn Port) -> Option<Box<dyn Coupling>> {
        (**self).new_coupling(from)
    }
}

impl<T: 'static + Clone> Coupling for Edge<T> {
    #[inline]
    fn propagate(&self) {
        self.port_to.add_values(self.port_from.get_values());
    }
}

impl<T: 'static + Clone> Port for Bag<T> {
    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    fn clear(&mut self) {
        self.0.clear();
    }

    #[inline]
    fn half_coupling(&mut self) -> Box<dyn HalfCoupling> {
        Box::new(OutPort::new(self))
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
    fn half_coupling(&mut self) -> Box<dyn HalfCoupling> {
        (**self).half_coupling()
    }
}
