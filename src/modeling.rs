pub mod atomic;
pub mod model;
pub mod coupled;
pub mod port;

pub use atomic::AsAtomic;
pub use model::{impl_model, AsModel, Model};
pub use coupled::{impl_coupled, AsCoupled, Coupled};
pub(crate) use port::AsPort;
pub use port::Port;

#[derive(Debug)]
pub enum Component {
    Atomic(Box<dyn AsAtomic>),
    Coupled(Box<dyn AsCoupled>),
}

impl AsModel for Component {
    fn to_component(self) -> Component {
        self
    }

    fn as_model(&self) -> &Model {
        match self {
            Component::Atomic(a) => a.as_model(),
            Component::Coupled(a) => a.as_model(),
        }
    }

    fn as_model_mut(&mut self) -> &mut Model {
        match self {
            Component::Atomic(a) => a.as_model_mut(),
            Component::Coupled(a) => a.as_model_mut(),
        }
    }
}
