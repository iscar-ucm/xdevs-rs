mod simulator;
mod coordinator;  // TODO hay que diferenciar con paralelo
use std::fmt::Debug;

use crate::{
    modeling::{AsModel, AsPort, Component},
    Shared,
};
use simulator::Simulator;
use coordinator::Coordinator;

#[derive(Debug)]
struct AbsSim<T: ?Sized> {
    model: Box<T>,
    t_last: f64,
    t_next: f64,
    in_ports: Vec<Shared<dyn AsPort>>,
    out_ports: Vec<Shared<dyn AsPort>>,
}

impl<T: AsModel + ?Sized> AbsSim<T> {
    fn new(model: Box<T>, t: f64) -> Self {
        // We serialize the input and output ports for better simulation performance
        let mut in_ports = vec![];
        for port in model.as_model().in_ports.values() {
            in_ports.push(port.clone());
        }
        let mut out_ports = vec![];
        for port in model.as_model().out_ports.values() {
            out_ports.push(port.clone());
        }
        Self {
            model,
            t_last: t,
            t_next: f64::INFINITY,
            in_ports,
            out_ports,
        }
    }

    fn input_empty(&self) -> bool {
        self.in_ports.iter().all(|p| p.is_empty())
    }

    fn output_empty(&self) -> bool {
        self.out_ports.iter().all(|p| p.is_empty())
    }

    fn clear_input(&mut self) {
        self.in_ports.iter().for_each(|p| p.clear());
    }

    fn clear_output(&mut self) {
        self.out_ports.iter().for_each(|p| p.clear());
    }
}

trait AbstractSimulator: Debug {
    fn start(&mut self);
    fn stop(&mut self);
    fn t_last(&self) -> f64;
    fn t_next(&self) -> f64;
    fn lambda(&mut self, t: f64);
    fn delta(&mut self, t: f64);
    fn clear(&mut self);
}


pub fn new_simulator(this: Component, t: f64) -> Box<dyn AbstractSimulator> {
    match this {
        Component::Atomic(a) => Box::new(Simulator::new(a, t)),
        Component::Coupled(c) => Box::new(Coordinator::new(c, t)),
    }
}
