pub mod devstone;
pub mod modeling;
pub mod simulation;

#[cfg(not(feature = "par_any"))]
use std::rc::Rc;
#[cfg(feature = "par_any")]
use std::sync::Arc;

#[cfg(not(feature = "par_any"))]
type Shared<T> = Rc<T>;
#[cfg(feature = "par_any")]
type Shared<T> = Arc<T>;

#[cfg(not(feature = "par_any"))]
pub trait DynRef: 'static {}
#[cfg(feature = "par_any")]
pub trait DynRef: 'static + Sync + Send {}

#[cfg(not(feature = "par_any"))]
impl<T: 'static + ?Sized> DynRef for T {}
#[cfg(feature = "par_any")]
impl<T: 'static + Sync + Send + ?Sized> DynRef for T {}
