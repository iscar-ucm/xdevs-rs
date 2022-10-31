pub mod modeling;
pub mod simulation;

pub use modeling::{Input, Output, Port};
pub use simulation::{RootCoordinator, Simulator};

use std::fmt::{Display, Formatter, Result};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::rc::Rc;

/// Hashable Rc pointer.
#[derive(Debug, Clone)]
struct RcHash<T: ?Sized>(Rc<T>);

impl<T: ?Sized> Deref for RcHash<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: ?Sized> PartialEq for RcHash<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
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
        (Rc::as_ptr(&self.0) as *const T).hash(state);
    }
}
