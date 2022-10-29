pub mod modeling;
pub mod simulation;

pub use modeling::atomic::Atomic;
pub use modeling::coupled::Coupled;
pub use modeling::port::Port;
pub use modeling::Component;
pub use simulation::{RootCoordinator, Simulator};

use std::fmt::{Display, Formatter, Result};
use std::hash::{Hash, Hasher};
use std::ops::Deref;

#[cfg(not(feature = "parallel"))]
use std::cell::RefCell;
#[cfg(not(feature = "parallel"))]
use std::rc::Rc;
#[cfg(feature = "parallel")]
use std::sync::{Arc, RwLock};

#[cfg(not(feature = "parallel"))]
pub type Shared<T> = Rc<T>;
#[cfg(not(feature = "parallel"))]
type Mutable<T> = RefCell<T>;

#[cfg(feature = "parallel")]
pub type Shared<T> = Arc<T>;
#[cfg(feature = "parallel")]
type Mutable<T> = RwLock<T>;

/// Handy wrapper for shared references that can be hashed by pointer value.
#[derive(Debug, Clone)]
struct RcHash<T: ?Sized>(Shared<T>);

impl<T: ?Sized> Deref for RcHash<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: ?Sized> PartialEq for RcHash<T> {
    fn eq(&self, other: &Self) -> bool {
        Shared::ptr_eq(&self.0, &other.0)
    }
}

impl<T: Display + ?Sized> Display for RcHash<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        Display::fmt(&self.0, f)
    }
}

impl<T: ?Sized> Eq for RcHash<T> {}

impl<T: ?Sized> Hash for RcHash<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (Shared::as_ptr(&self.0) as *const T).hash(state);
    }
}
