pub mod devstone;
pub mod modeling;
pub mod simulation;

/// Helper traits for avoiding too big trait constraits.
#[cfg(not(feature = "par_any"))]
pub trait DynRef: 'static {}
/// Helper traits for avoiding too big trait constraits.
#[cfg(feature = "par_any")]
pub trait DynRef: 'static + Sync + Send {}

#[cfg(not(feature = "par_any"))]
impl<T: 'static + ?Sized> DynRef for T {}
#[cfg(feature = "par_any")]
impl<T: 'static + Sync + Send + ?Sized> DynRef for T {}
