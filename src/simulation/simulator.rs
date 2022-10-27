use super::{AbsSim, AbstractSimulator};
use crate::modeling::AsAtomic;

#[derive(Debug)]
pub(crate) struct Simulator(AbsSim<dyn AsAtomic>);

impl Simulator {
    pub(crate) fn new(model: Box<dyn AsAtomic>, t: f64) -> Self {
        let mut abs_sim = AbsSim::new(model, t);
        abs_sim.t_next = t + abs_sim.model.ta();
        Self(abs_sim)
    }
}

impl AbstractSimulator for Simulator {
    fn start(&mut self) {}

    fn stop(&mut self) {}

    fn t_last(&self) -> f64 {
        self.0.t_last
    }

    fn t_next(&self) -> f64 {
        self.0.t_next
    }

    fn lambda(&mut self, t: f64) {
        if t >= self.0.t_next {
            self.0.model.lambda();
        }
    }

    fn delta(&mut self, t: f64) {
        if !self.0.input_empty() {
            if t == self.0.t_next {
                self.0.model.delta_conf();
            } else {
                let e = t - self.0.t_last;
                self.0.model.delta_ext(e);
            }
        } else if t == self.0.t_next {
        } else {
            return;
        }
        self.0.t_last = t;
        self.0.t_next = t + self.0.model.ta();
    }

    fn clear(&mut self) {
        self.0.clear_input();
        self.0.clear_output();
    }
}
