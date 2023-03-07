use crate::DynRef;
use std::any::Any;
use std::cell::UnsafeCell;
use std::ops::Deref;

#[cfg(not(feature = "par_any"))]
use std::rc::Rc;
#[cfg(feature = "par_any")]
use std::sync::Arc;

#[cfg(not(feature = "par_any"))]
pub(super) type Shared<T> = Rc<T>;
#[cfg(feature = "par_any")]
pub(super) type Shared<T> = Arc<T>;

/// Trait implemented by DEVS ports. It does not consider message types nor port directions.
pub(crate) trait Port: DynRef {
    /// Port-to-any conversion.
    fn as_any(&self) -> &dyn Any;

    /// Returns `true` if the port does not contain any value.
    ///
    /// # Safety
    ///
    /// To do.
    unsafe fn is_empty(&self) -> bool;

    /// It clears all the values in the port.
    ///
    /// # Safety
    ///
    /// To do.
    unsafe fn clear(&self);

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
#[repr(transparent)]
pub(super) struct Bag<T>(UnsafeCell<Vec<T>>);

impl<T> Bag<T> {
    /// Creates a new message bag.
    #[inline]
    pub(super) fn new() -> Self {
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
    #[inline(always)]
    unsafe fn borrow(&self) -> &Vec<T> {
        &*self.get()
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
    #[allow(clippy::mut_from_ref)]
    #[inline(always)]
    unsafe fn borrow_mut(&self) -> &mut Vec<T> {
        &mut *self.get()
    }
}

impl<T> Deref for Bag<T> {
    type Target = UnsafeCell<Vec<T>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "par_any")]
// Safety: if all the invariants are met, then a bag can be safely shared among threads.
unsafe impl<T: Send> Send for Bag<T> {}

#[cfg(feature = "par_any")]
// Safety: if all the invariants are met, then a bag can be safely shared among threads.
unsafe impl<T: Sync> Sync for Bag<T> {}

impl<T: DynRef + Clone> Port for Bag<T> {
    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[inline]
    unsafe fn is_empty(&self) -> bool {
        self.borrow().is_empty()
    }

    #[inline]
    unsafe fn clear(&self) {
        self.borrow_mut().clear();
    }

    #[inline]
    fn is_compatible(&self, other: &dyn Port) -> bool {
        other.as_any().downcast_ref::<Bag<T>>().is_some()
    }

    #[inline]
    unsafe fn propagate(&self, port_to: &dyn Port) {
        let port_to = port_to.as_any().downcast_ref::<Bag<T>>().unwrap();
        port_to.borrow_mut().extend_from_slice(self.borrow());
    }
}

/// Input port. This structure only allows reading messages. Thus, it cannot inject messages.
#[derive(Debug)]
pub struct InPort<T>(Shared<Bag<T>>);

impl<T: Clone> InPort<T> {
    pub(super) fn new(bag: Shared<Bag<T>>) -> Self {
        Self(bag)
    }

    /// Returns `true` if the underlying bag is empty. Otherwise, it returns `false`.
    ///
    /// # Safety
    ///
    /// When calling this method, the caller must ensure that it fulfills all the following invariants:
    /// - The caller implements the [`super::Atomic`] trait.
    /// - This port is an input port of the caller.
    /// - The caller is inside the [`super::Atomic::delta_ext`] method.
    #[inline(always)]
    pub unsafe fn is_empty(&self) -> bool {
        self.0.borrow().is_empty()
    }

    /// Returns a reference to the slice of messages of the underlying bag.
    ///
    /// # Safety
    ///
    /// When calling this method, the caller must ensure that it fulfills all the following invariants:
    /// - The caller implements the [`super::Atomic`] trait.
    /// - This port is an input port of the caller.
    /// - The caller is inside the [`super::Atomic::delta_ext`] method.
    #[inline(always)]
    pub unsafe fn get_values(&self) -> &Vec<T> {
        self.0.borrow()
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
#[derive(Debug)]
pub struct OutPort<T: Clone>(Shared<Bag<T>>);

impl<T: Clone> OutPort<T> {
    pub(super) fn new(bag: Shared<Bag<T>>) -> Self {
        Self(bag)
    }

    /// Adds a new value to the output port.
    ///
    /// # Safety
    ///
    /// When calling this method, the caller must ensure that it fulfills all the following invariants:
    /// - The caller implements the [`super::Atomic`] trait.
    /// - This port is an output port of the caller.
    /// - The caller is inside the [`super::Atomic::lambda`] method.
    #[inline(always)]
    pub unsafe fn add_value(&self, value: T) {
        self.0.borrow_mut().push(value);
    }

    /// Adds new values from a slice to the output port.
    ///
    /// # Safety
    ///
    /// When calling this method, the caller must ensure that it fulfills all the following invariants:
    /// - The caller implements the [`super::Atomic`] trait.
    /// - This port is an output port of the caller.
    /// - The caller is inside the [`super::Atomic::lambda`] method.
    #[inline(always)]
    pub unsafe fn add_values(&self, values: &[T]) {
        self.0.borrow_mut().extend_from_slice(values);
    }
}
