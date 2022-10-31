use super::{DEVStoneAtomic, DEVStoneSeeder, TestProbe};
use crate::modeling::Coupled;
use crate::*;
use std::cell::RefCell;

#[derive(Debug)]
pub struct LI {
    pub coupled: Coupled,
}

impl LI {
    pub fn create(width: usize, depth: usize) -> Coupled {
        let mut coupled = Coupled::new("LI");
        let seeder = DEVStoneSeeder::new("seeder");
        let li = Self::new(width, depth, None);
        let li_name = li.coupled.component.get_name().to_string();
        coupled.add_component(Box::new(seeder));
        coupled.add_component(Box::new(li.coupled));
        coupled.add_ic("seeder", "output", &li_name, "input");
        coupled
    }

    fn _create_test(width: usize, depth: usize, probe: Rc<RefCell<TestProbe>>) -> Coupled {
        let mut coupled = Coupled::new("LI");
        let seeder = DEVStoneSeeder::new("seeder");
        let li = Self::new(width, depth, Some(probe));
        let li_name = li.coupled.component.get_name().to_string();
        coupled.add_component(Box::new(seeder));
        coupled.add_component(Box::new(li.coupled));
        coupled.add_ic("seeder", "output", &li_name, "input");
        coupled
    }

    fn new(width: usize, depth: usize, probe: Option<Rc<RefCell<TestProbe>>>) -> Self {
        // First we check the input parameters
        if width < 1 {
            panic!("width must be greater than 1")
        }
        if depth < 1 {
            panic!("depth must be greater than 1")
        }
        // Next we create the inner coupled model
        let name = format!("coupled_{}", depth);
        let mut coupled = Coupled::new(&name);
        coupled.add_in_port::<usize>("input");
        coupled.add_out_port::<usize>("output");
        // If this is the inner coupled model, we just add one atomic.
        if depth == 1 {
            let atomic = DEVStoneAtomic::new("inner_atomic", probe.clone());
            coupled.add_component(Box::new(atomic));
            coupled.add_eic("input", "inner_atomic", "input");
            coupled.add_eoc("inner_atomic", "output", "output");
        // Otherwise, we add a subcoupled and a set of atomics.
        } else {
            let subcoupled = Self::new(width, depth - 1, probe.clone());
            let subcoupled_name = subcoupled.coupled.component.get_name().to_string();
            coupled.add_component(Box::new(subcoupled.coupled));
            coupled.add_eic("input", &subcoupled_name, "input");
            coupled.add_eoc(&subcoupled_name, "output", "output");
            for i in 1..width {
                let atomic_name = format!("atomic_{}", i);
                let atomic = DEVStoneAtomic::new(&atomic_name, probe.clone());
                coupled.add_component(Box::new(atomic));
                coupled.add_eic("input", &atomic_name, "input");
            }
        }
        // Before exiting, we update the probe if required
        if let Some(p) = probe {
            p.borrow_mut().n_eics += coupled.eic_vec.len();
            p.borrow_mut().n_ics += coupled.ic_vec.len();
            p.borrow_mut().n_eocs += coupled.eoc_vec.len()
        }
        Self { coupled }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::*;

    fn expected_atomics(width: usize, depth: usize) -> usize {
        (width - 1) * (depth - 1) + 1
    }

    fn expected_eics(width: usize, depth: usize) -> usize {
        width * (depth - 1) + 1
    }

    #[test]
    fn test_li() {
        for width in (1..50).step_by(5) {
            for depth in (1..50).step_by(5) {
                let probe = Rc::new(RefCell::new(TestProbe::new()));
                let coupled = LI::_create_test(width, depth, probe.clone());
                assert_eq!(expected_atomics(width, depth), probe.borrow().n_atomics);
                assert_eq!(expected_eics(width, depth), probe.borrow().n_eics);
                assert_eq!(0, probe.borrow().n_ics);
                assert_eq!(depth, probe.borrow().n_eocs);
                let mut simulator = RootCoordinator::new(coupled);
                simulator.simulate_time(f64::INFINITY);
                assert_eq!(expected_atomics(width, depth), probe.borrow().n_internals);
                assert_eq!(expected_atomics(width, depth), probe.borrow().n_externals);
                assert_eq!(expected_atomics(width, depth), probe.borrow().n_events);
            }
        }
    }
}
