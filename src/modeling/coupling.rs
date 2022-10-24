use super::PortInterface;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

/// Coupling bridge. This is an auxiliary struct for coupled models.
#[derive(Debug)]
pub(crate) struct CouplingBridge {
    component_name: String,
    port_ref: Rc<dyn PortInterface>,
}

/// Type alias for coupling hash maps in coupled models.
pub(crate) type CouplingsHashMap = HashMap<CouplingBridge, HashSet<CouplingBridge>>;

impl CouplingBridge {
    /// It creates a new coupling bridge.
    pub(crate) fn new(component_name: &str, port_ref: Rc<dyn PortInterface>) -> Self {
        CouplingBridge {
            component_name: component_name.to_string(),
            port_ref,
        }
    }
}

impl Hash for CouplingBridge {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.component_name.hash(state);
        self.port_ref.get_name().hash(state);
    }
}

impl PartialEq<Self> for CouplingBridge {
    fn eq(&self, other: &Self) -> bool {
        self.component_name == other.component_name
            && self.port_ref.get_name() == other.port_ref.get_name()
    }
}

impl Eq for CouplingBridge {}
