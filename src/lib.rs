pub mod devstone;
pub mod modeling;
pub mod simulation;

/// Helper trait for avoiding verbose trait constraints.
#[cfg(not(feature = "rayon"))]
pub trait DynRef: 'static {}
/// Helper trait for avoiding verbose trait constraints.
#[cfg(feature = "rayon")]
pub trait DynRef: 'static + Sync + Send {}

#[cfg(not(feature = "rayon"))]
impl<T: 'static + ?Sized> DynRef for T {}
#[cfg(feature = "rayon")]
impl<T: 'static + Sync + Send + ?Sized> DynRef for T {}
