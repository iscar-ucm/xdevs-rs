pub mod hi;
pub mod ho;
pub mod homod;
pub mod li;

use crate::modeling::*;
use std::cell::RefCell;
use std::rc::Rc;

pub use hi::HI;
pub use ho::HO;
pub use homod::HOmod;
pub use li::LI;

#[derive(Debug, Default, Copy, Clone)]
struct TestProbe {
    n_atomics: usize,
    n_eics: usize,
    n_ics: usize,
    n_eocs: usize,
    n_internals: usize,
    n_externals: usize,
    n_events: usize,
}

struct DEVStoneAtomic {
    component: Component,
    sigma: f64,
    n_internals: usize,
    n_externals: usize,
    n_events: usize,
    probe: Option<Rc<RefCell<TestProbe>>>,
    input: InPort<usize>,
    output: OutPort<usize>,
}

impl DEVStoneAtomic {
    pub fn new(name: &str, probe: Option<Rc<RefCell<TestProbe>>>) -> Self {
        let mut component = Component::new(name);
        let input = component.add_in_port("input");
        let output = component.add_out_port("output");
        if let Some(p) = &probe {
            p.borrow_mut().n_atomics += 1;
        }
        Self {
            sigma: f64::INFINITY,
            n_internals: 0,
            n_externals: 0,
            n_events: 0,
            component,
            probe,
            input,
            output,
        }
    }
}

impl Atomic for DEVStoneAtomic {
    #[inline]
    fn get_component(&self) -> &Component {
        &self.component
    }

    #[inline]
    fn get_component_mut(&mut self) -> &mut Component {
        &mut self.component
    }

    #[inline]
    fn stop(&mut self) {
        if let Some(t) = &self.probe {
            let mut x = t.borrow_mut();
            x.n_internals += self.n_internals;
            x.n_externals += self.n_externals;
            x.n_events += self.n_events;
        }
    }

    #[inline]
    fn lambda(&self) {
        self.output.add_value(self.n_events);
    }

    #[inline]
    fn delta_int(&mut self) {
        self.n_internals += 1;
        self.sigma = f64::INFINITY;
    }

    fn delta_ext(&mut self, _e: f64) {
        if self.probe.is_some() {
            self.n_externals += 1;
            self.n_events += self.input.get_values().len();
        }
        self.sigma = 0.;
    }

    #[inline]
    fn ta(&self) -> f64 {
        self.sigma
    }
}

struct DEVStoneSeeder {
    component: Component,
    sigma: f64,
    output: OutPort<usize>,
}

impl DEVStoneSeeder {
    pub fn new(name: &str) -> Self {
        let mut component = Component::new(name);
        let output = component.add_out_port("output");
        Self {
            sigma: 0.,
            component,
            output,
        }
    }
}

impl Atomic for DEVStoneSeeder {
    #[inline]
    fn get_component(&self) -> &Component {
        &self.component
    }

    #[inline]
    fn get_component_mut(&mut self) -> &mut Component {
        &mut self.component
    }

    #[inline]
    fn lambda(&self) {
        self.output.add_value(0);
    }

    #[inline]
    fn delta_int(&mut self) {
        self.sigma = f64::INFINITY;
    }

    #[inline]
    fn delta_ext(&mut self, _e: f64) {
        self.sigma = f64::INFINITY;
    }

    #[inline]
    fn ta(&self) -> f64 {
        self.sigma
    }
}
