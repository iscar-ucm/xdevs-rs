use crate::{DynRef, Shared};
use std::any::Any;
use std::cell::UnsafeCell;

/// Trait implemented by DEVS ports. It does not consider message types nor port directions.
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
    ///
    /// # Safety
    ///
    /// The caller must ensure that it fulfills all the following invariants:
    /// - The caller is a [`super::Coupled`] model.
    /// - The caller is propagating messages according to its couplings.
    unsafe fn propagate(&self, port_to: &dyn Port);
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
    ///
    /// The caller must ensure that it fulfills all the following invariants:
    /// - The bag corresponds to an [`InPort`] of an implementer of the [`super::Atomic`] trait.
    /// - The implementer only calls this function inside the [`super::Atomic::delta_ext`] method.
    ///
    /// Alternatively, the [`super::Coupled`] struct can call this method when propagating messages.
    #[inline]
    pub(crate) unsafe fn borrow(&self) -> &Vec<T> {
        &*self.0.get()
    }

    /// Returns a mutable reference to the vector of messages in the bag.
    ///
    /// # Safety:
    ///
    /// The caller must ensure that it fulfills all the following invariants:
    /// - The bag corresponds to an [`OutPort`] of an implementer of the [`super::Atomic`] trait.
    /// - The implementer only calls this function inside the [`super::Atomic::lambda`] method.
    ///
    /// Alternatively, the [`super::Coupled`] struct can call this method when propagating messages.
    #[inline]
    #[allow(clippy::mut_from_ref)]
    pub(crate) unsafe fn borrow_mut(&self) -> &mut Vec<T> {
        &mut *self.0.get()
    }
}

#[cfg(feature = "par_any")]
// Safety: if all the invariants are met, then a bag can be safely shared among threads.
unsafe impl<T: Send> Send for Bag<T> {}

#[cfg(feature = "par_any")]
// Safety: if all the invariants are met, then a bag can be safely shared among threads.
unsafe impl<T: Sync> Sync for Bag<T> {}

/// Input port. This structure only allows reading messages. Thus, it cannot inject messages.
///
/// # Safety
///
/// When calling **any** of its method, the caller must ensure that it fulfills all the following invariants:
/// - The caller implements the [`super::Atomic`] trait.
/// - The caller created this port via the [`super::Component::add_in_port`] method of its inner [`super::Component`].
/// - The implementer only calls these methods inside the [`super::Atomic::delta_ext`] method.
#[derive(Clone, Debug)]
pub struct InPort<T>(Shared<Bag<T>>);

impl<T> InPort<T> {
    pub(crate) fn new(bag: Shared<Bag<T>>) -> Self {
        Self(bag)
    }

    /// Returns `true` if the underlying bag is empty. Otherwise, it returns `false`.
    ///
    /// # Safety
    ///
    /// The caller must fulfill all the invariants of the [`InPort`] struct.
    #[inline]
    pub fn is_empty(&self) -> bool {
        // Safety: we are executing a delta_ext function of the corresponding atomic model
        unsafe { self.0.borrow() }.is_empty()
    }

    /// Returns a reference to the slice of messages of the underlying bag.
    ///
    /// # Safety
    ///
    /// The caller must fulfill all the invariants of the [`InPort`] struct.
    #[inline]
    pub fn get_values(&self) -> &Vec<T> {
        // Safety: we are executing a delta_ext function of the corresponding atomic model
        unsafe { self.0.borrow() }
    }
}

/// Output port. This structure only injecting messages. Thus, it cannot read messages.
///
/// # Safety
///
/// When calling **any** of its method, the caller must ensure that it fulfills all the following invariants:
/// - The caller implements the [`super::Atomic`] trait.
/// - The caller created this port via the [`super::Component::add_out_port`] method of its inner [`super::Component`].
/// - The implementer only calls these methods inside the [`super::Atomic::lambda`] method.
#[derive(Clone, Debug)]
pub struct OutPort<T>(Shared<Bag<T>>);

impl<T> OutPort<T> {
    pub(crate) fn new(bag: Shared<Bag<T>>) -> Self {
        Self(bag)
    }

    /// Adds a new value to the output port.
    ///
    /// # Safety
    ///
    /// The caller must fulfill all the invariants of the [`OutPort`] struct.
    #[inline]
    pub fn add_value(&self, value: T) {
        // Safety: we are executing a lambda function of the corresponding atomic model
        unsafe { self.0.borrow_mut() }.push(value);
    }
}

impl<T: Clone> OutPort<T> {
    /// Adds new values from a slice to the output port.
    ///
    /// # Safety
    ///
    /// The caller must fulfill all the invariants of the [`OutPort`] struct.
    #[inline]
    pub fn add_values(&self, values: &[T]) {
        // Safety: we are executing a lambda function of the corresponding atomic model
        unsafe { self.0.borrow_mut() }.extend_from_slice(values);
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
    unsafe fn propagate(&self, port_to: &dyn Port) {
        let port_to = port_to.as_any().downcast_ref::<Bag<T>>().unwrap();
        unsafe { port_to.borrow_mut().extend_from_slice(self.borrow()) };
    }
}
