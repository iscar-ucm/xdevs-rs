pub mod atomic;
pub mod component;
pub mod coupled;
pub mod port;

pub use atomic::AsAtomic;
pub use component::{impl_component, AsComponent, Component};
pub use coupled::{impl_coupled, AsCoupled, Coupled};
pub(crate) use port::AsPort;
pub use port::Port;
use std::fmt::{Display, Formatter, Result};
use std::hash::{Hash, Hasher};

#[cfg(not(feature = "parallel"))]
use std::rc::Rc;
#[cfg(feature = "parallel")]
use std::sync::Arc;

#[cfg(not(feature = "parallel"))]
pub type Shared<T> = Rc<T>;
#[cfg(feature = "parallel")]
pub type Shared<T> = Arc<T>;

#[derive(Debug, Clone)]
struct RcHash<T: ?Sized>(Shared<T>);

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
