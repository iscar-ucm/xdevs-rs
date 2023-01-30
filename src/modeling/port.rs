use crate::{DynRef, Shared};
use std::any::Any;
use std::cell::UnsafeCell;

/// DEVS ports. It does not consider message types nor port directions.
pub(crate) trait Port: DynRef {
    /// Port-to-any conversion.
    fn as_any(&self) -> &dyn Any;

    /// Returns `true` if the port does not contain any value.
    fn is_empty(&self) -> bool;

    /// It clears all the values in the port.
    fn clear(&self);

    /// Returns `true` if other port is compatible.
    fn is_compatible(&self, other: &dyn Port) -> bool;

    /// Propagates messages from the port to other receiving port.
    fn propagate(&self, port_to: &dyn Port);
}

/// Bag of DEVS messages.
/// It is just a wrapper of an [`UnsafeCell`] containing a vector of messages.
#[derive(Debug)]
pub(crate) struct Bag<T>(UnsafeCell<Vec<T>>);

impl<T> Bag<T> {
    /// Creates a new message bag.
    #[inline]
    pub(crate) fn new() -> Self {
        Self(UnsafeCell::new(Vec::new()))
    }

    /// Returns a reference to the vector of messages in the bag.
    ///
    /// # Safety:
    /// The DEVS simulation workflow already avoids multiple unsafe conditions.
    /// The xDEVS crate this only when propagating messages from this bag to other bags.
    /// On the other hand, only [`InPort`] structs are allowed to execute this method.
    /// All the [`InPort`] methods must **ONLY** be executed during an [`Atomic::delta_ext`] method.
    #[inline]
    pub(crate) unsafe fn borrow(&self) -> &Vec<T> {
        unsafe { &*self.0.get() }
    }

    /// Returns a mutable reference to the vector of messages in the bag.
    #[inline]
    #[allow(clippy::mut_from_ref)]
    pub(crate) unsafe fn borrow_mut(&self) -> &mut Vec<T> {
        unsafe { &mut *self.0.get() }
    }
}

#[cfg(feature = "par_any")]
unsafe impl<T: Send> Send for Bag<T> {}

#[cfg(feature = "par_any")]
unsafe impl<T: Sync> Sync for Bag<T> {}

/// Input port. It is just a wrapper of a shared [`Bag`].
/// This structure only allows reading messages from the underlying bag.
/// Thus, it cannot inject messages to the bag.
#[derive(Clone, Debug)]
pub struct InPort<T>(Shared<Bag<T>>);

impl<T> InPort<T> {
    /// Creates a new input port from a shared [`Bag`].
    pub(crate) fn new(bag: Shared<Bag<T>>) -> Self {
        Self(bag)
    }

    /// Returns `true` if the underlying bag is empty. Otherwise, it returns `false`.
    #[inline]
    pub fn is_empty(&self) -> bool {
        unsafe { self.0.borrow().is_empty() }
    }

    /// Returns a reference to the slice of messages of the underlying bag.
    #[inline]
    pub fn get_values(&self) -> &Vec<T> {
        unsafe { self.0.borrow() }
    }
}

/// Output port. It is just a wrapper of a shared [`Bag`].
/// This structure only injecting messages to the underlying bag.
/// Thus, it cannot read the messages in the bag.
#[derive(Clone, Debug)]
pub struct OutPort<T>(Shared<Bag<T>>);

impl<T> OutPort<T> {
    /// Creates a new input port from a shared [`Bag`].
    pub(crate) fn new(bag: Shared<Bag<T>>) -> Self {
        Self(bag)
    }

    /// Adds a new value to the output port.
    #[inline]
    pub fn add_value(&self, value: T) {
        unsafe { self.0.borrow_mut().push(value) };
    }
}

impl<T: Clone> OutPort<T> {
    /// Adds new values from a slice to the output port.
    #[inline]
    pub fn add_values(&self, values: &[T]) {
        unsafe { self.0.borrow_mut().extend_from_slice(values) };
    }
}

impl<T: DynRef + Clone> Port for Bag<T> {
    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[inline]
    fn is_empty(&self) -> bool {
        unsafe { self.borrow().is_empty() }
    }

    #[inline]
    fn clear(&self) {
        unsafe { self.borrow_mut().clear() };
    }

    #[inline]
    fn is_compatible(&self, other: &dyn Port) -> bool {
        other.as_any().downcast_ref::<Bag<T>>().is_some()
    }

    #[inline]
    fn propagate(&self, port_to: &dyn Port) {
        let port_to = port_to.as_any().downcast_ref::<Bag<T>>().unwrap();
        unsafe { port_to.borrow_mut().extend_from_slice(self.borrow()) };
    }
}
