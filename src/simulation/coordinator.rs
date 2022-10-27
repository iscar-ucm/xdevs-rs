use super::{AbsSim, AbstractSimulator};
use crate::modeling::AsCoupled;

#[derive(Debug)]
pub(crate) struct Coordinator {
    abs_sim: AbsSim<dyn AsCoupled>,
    simulators: Vec<Box<dyn AbstractSimulator>>,
}

impl Coordinator {
    pub(crate) fn new(model: Box<dyn AsCoupled>, t: f64) -> Self {
        todo!()
    }
}

impl AbstractSimulator for Coordinator {
    fn start(&mut self) {
        todo!()
    }

    fn stop(&mut self) {
        todo!()
    }

    fn t_last(&self) -> f64 {
        todo!()
    }

    fn t_next(&self) -> f64 {
        todo!()
    }

    fn lambda(&mut self, t: f64) {
        todo!()
    }

    fn delta(&mut self, t: f64) {
        todo!()
    }

    fn clear(&mut self) {
        todo!()
    }
}