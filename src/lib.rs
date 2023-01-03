pub mod devstone;
pub mod modeling;
pub mod simulation;

#[cfg(not(feature = "parallel"))]
use std::cell::RefCell;
#[cfg(not(feature = "parallel"))]
use std::rc::Rc;
#[cfg(feature = "parallel")]
use std::sync::{Arc, RwLock};

#[cfg(not(feature = "parallel"))]
type Shared<T> = Rc<T>;
#[cfg(not(feature = "parallel"))]
type Mutable<T> = RefCell<T>;
#[cfg(feature = "parallel")]
type Shared<T> = Arc<T>;
#[cfg(feature = "parallel")]
type Mutable<T> = RwLock<T>;

#[cfg(not(feature = "parallel"))]
pub trait DynRef: 'static {}
#[cfg(feature = "parallel")]
pub trait DynRef: 'static + Sync + Send {}

#[cfg(not(feature = "parallel"))]
impl<T: 'static + ?Sized> DynRef for T {}
#[cfg(feature = "parallel")]
impl<T: 'static + Sync + Send + ?Sized> DynRef for T {}
