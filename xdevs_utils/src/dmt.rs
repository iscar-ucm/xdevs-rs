use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Node {
    pub component: Option<String>,
    pub port: String,
}

pub enum CouplingType {
    EIC,
    IC,
    EOC,
    Invalid,
}

pub struct Coupling {
    src: Node,
    dst: Node,
}

impl Coupling {
    pub fn eic<R: Into<String>, S: Into<String>, T: Into<String>>(
        port_from: R,
        component_to: S,
        port_to: T,
    ) -> Self {
        Self {
            src: Node {
                component: None,
                port: port_from.into(),
            },
            dst: Node {
                component: Some(component_to.into()),
                port: port_to.into(),
            },
        }
    }

    pub fn ic<Q: Into<String>, R: Into<String>, S: Into<String>, T: Into<String>>(
        component_from: Q,
        port_from: R,
        component_to: S,
        port_to: T,
    ) -> Self {
        Self {
            src: Node {
                component: Some(component_from.into()),
                port: port_from.into(),
            },
            dst: Node {
                component: Some(component_to.into()),
                port: port_to.into(),
            },
        }
    }

    pub fn eoc<Q: Into<String>, R: Into<String>, S: Into<String>>(
        component_from: Q,
        port_from: R,
        port_to: S,
    ) -> Self {
        Self {
            src: Node {
                component: Some(component_from.into()),
                port: port_from.into(),
            },
            dst: Node {
                component: None,
                port: port_to.into(),
            },
        }
    }

    fn get_type(&self) -> CouplingType {
        if self.src.component.is_none() && self.dst.component.is_some() {
            CouplingType::EIC
        } else if self.src.component.is_some() && self.dst.component.is_some() {
            CouplingType::IC
        } else if self.src.component.is_some() && self.dst.component.is_none() {
            CouplingType::EOC
        } else {
            CouplingType::Invalid
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Components {
    pub components: HashMap<String, DevsModel>,
    pub couplings: HashMap<Node, HashSet<Node>>,
}

impl Components {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            couplings: HashMap::new(),
        }
    }

    fn add_component(&mut self, component: DevsModel) -> Option<DevsModel> {
        self.components.insert(component.name.clone(), component)
    }
}

#[derive(Debug)]
pub struct DevsModel {
    pub name: String,
    pub input: HashSet<String>,
    pub output: HashSet<String>,
    pub components: Option<Components>,
}

impl PartialEq for DevsModel {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for DevsModel {}

impl Hash for DevsModel {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl DevsModel {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            input: HashSet::new(),
            output: HashSet::new(),
            components: None,
        }
    }

    pub fn add_input<S: Into<String>>(&mut self, input: S) -> bool {
        self.input.insert(input.into())
    }

    pub fn has_input(&self, input: &str) -> bool {
        self.input.contains(input)
    }

    pub fn add_output<S: Into<String>>(&mut self, output: S) -> bool {
        self.output.insert(output.into())
    }

    pub fn has_output(&self, output: &str) -> bool {
        self.output.contains(output)
    }

    pub fn add_component(&mut self, component: DevsModel) -> Option<DevsModel> {
        if self.components.is_none() {
            self.components = Some(Components::new());
        }
        self.components.as_mut().unwrap().add_component(component)
    }

    pub fn get_component(&self, name: &str) -> Option<&DevsModel> {
        self.components
            .as_ref()
            .and_then(|c| c.components.get(name))
    }

    pub fn add_coupling(&mut self, coupling: Coupling) -> Result<bool, CouplingError> {
        if self.components.is_none() {
            return Err(CouplingError::NoComponents);
        }
        match coupling.get_type() {
            CouplingType::EIC => {
                if !self.has_input(&coupling.src.port) {
                    return Err(CouplingError::InvalidPort);
                }
                match self.get_component(coupling.dst.component.as_ref().unwrap()) {
                    Some(component) => {
                        if component.has_input(&coupling.dst.port) {
                            let res = self
                                .components
                                .as_mut()
                                .unwrap()
                                .couplings
                                .entry(coupling.src)
                                .or_default()
                                .insert(coupling.dst);
                            Ok(res)
                        } else {
                            Err(CouplingError::InvalidPort)
                        }
                    }
                    None => Err(CouplingError::InvalidComponent),
                }
            }
            CouplingType::IC => {
                match (
                    self.get_component(coupling.src.component.as_ref().unwrap()),
                    self.get_component(coupling.dst.component.as_ref().unwrap()),
                ) {
                    (Some(src), Some(dst)) => {
                        if src.has_output(&coupling.src.port) && dst.has_input(&coupling.dst.port) {
                            let res = self
                                .components
                                .as_mut()
                                .unwrap()
                                .couplings
                                .entry(coupling.src)
                                .or_default()
                                .insert(coupling.dst);
                            Ok(res)
                        } else {
                            Err(CouplingError::InvalidPort)
                        }
                    }
                    _ => Err(CouplingError::InvalidComponent),
                }
            }
            CouplingType::EOC => {
                match self.get_component(coupling.src.component.as_ref().unwrap()) {
                    Some(component) => {
                        if component.has_output(&coupling.src.port) {
                            let res = self
                                .components
                                .as_mut()
                                .unwrap()
                                .couplings
                                .entry(coupling.src)
                                .or_default()
                                .insert(coupling.dst);
                            Ok(res)
                        } else {
                            Err(CouplingError::InvalidPort)
                        }
                    }
                    None => Err(CouplingError::InvalidComponent),
                }
            }
            CouplingType::Invalid => Err(CouplingError::InvalidComponent),
        }
    }

    pub fn is_coupled(&self) -> bool {
        self.components.is_some()
    }
}

#[derive(Debug)]
pub enum CouplingError {
    NoComponents,
    InvalidPort,
    InvalidComponent,
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn efp() {
        let mut p = DevsModel::new("processor");
        p.add_input("input_req");
        p.add_output("output_res");

        let mut ef = DevsModel::new("ef");
        ef.add_input("input_res");
        ef.add_output("output_req");

        let mut efp = DevsModel::new("efp");
        assert_eq!(efp.add_component(p), None);
        assert_eq!(efp.add_component(ef), None);

        efp.add_coupling(Coupling::ic("ef", "output_req", "processor", "input_req"))
            .unwrap();
        efp.add_coupling(Coupling::ic("processor", "output_res", "ef", "input_res"))
            .unwrap();

        println!("{:?}", efp);
    }
}
