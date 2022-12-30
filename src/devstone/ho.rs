use super::{DEVStoneAtomic, DEVStoneSeeder, TestProbe};
use crate::modeling::Coupled;
use std::cell::RefCell;
use std::rc::Rc;

pub struct HO {
    pub coupled: Coupled,
}

impl HO {
    pub fn create(width: usize, depth: usize) -> Coupled {
        let mut coupled = Coupled::new("HO");
        let seeder = DEVStoneSeeder::new("seeder");
        let ho = Self::new(width, depth, None);
        let ho_name = ho.coupled.component.get_name().to_string();
        coupled.add_component(Box::new(seeder));
        coupled.add_component(Box::new(ho.coupled));
        coupled.add_ic("seeder", "output", &ho_name, "input_1");
        coupled.add_ic("seeder", "output", &ho_name, "input_2");
        coupled
    }

    fn _create_test(width: usize, depth: usize, probe: Rc<RefCell<TestProbe>>) -> Coupled {
        let mut coupled = Coupled::new("HO");
        let seeder = DEVStoneSeeder::new("seeder");
        let ho = Self::new(width, depth, Some(probe));
        let ho_name = ho.coupled.component.get_name().to_string();
        coupled.add_component(Box::new(seeder));
        coupled.add_component(Box::new(ho.coupled));
        coupled.add_ic("seeder", "output", &ho_name, "input_1");
        coupled.add_ic("seeder", "output", &ho_name, "input_2");
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
        // Next we create the model structure
        let name = format!("coupled_{}", depth);
        let mut coupled = Coupled::new(&name);
        coupled.add_in_port::<usize>("input_1");
        coupled.add_in_port::<usize>("input_2");
        coupled.add_out_port::<usize>("output_1");
        coupled.add_out_port::<usize>("output_2");
        // If this is the inner coupled model, we just add one atomic.
        if depth == 1 {
            let atomic = DEVStoneAtomic::new("inner_atomic", probe.clone());
            coupled.add_component(Box::new(atomic));
            coupled.add_eic("input_1", "inner_atomic", "input");
            coupled.add_eoc("inner_atomic", "output", "output_1");
            // Otherwise, we add a subcoupled and a set of atomics.
        } else {
            let subcoupled = Self::new(width, depth - 1, probe.clone());
            let subcoupled_name = subcoupled.coupled.component.get_name().to_string();
            coupled.add_component(Box::new(subcoupled.coupled));
            coupled.add_eic("input_1", &subcoupled_name, "input_1");
            coupled.add_eic("input_1", &subcoupled_name, "input_2");
            coupled.add_eoc(&subcoupled_name, "output_1", "output_1");
            for i in 1..width {
                let atomic_name = format!("atomic_{}", i);
                let atomic = DEVStoneAtomic::new(&atomic_name, probe.clone());
                coupled.add_component(Box::new(atomic));
                coupled.add_eic("input_2", &atomic_name, "input");
                if i > 1 {
                    let prev_atomic_name = format!("atomic_{}", i - 1);
                    coupled.add_ic(&prev_atomic_name, "output", &atomic_name, "input");
                }
                coupled.add_eoc(&atomic_name, "output", "output_2");
            }
        }
        // Before exiting, we update the probe if required
        if let Some(p) = probe {
            p.borrow_mut().n_eics += coupled.eics.len();
            p.borrow_mut().n_ics += coupled.ics.len();
            p.borrow_mut().n_eocs += coupled.eocs.len()
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
        (width + 1) * (depth - 1) + 1
    }

    fn expected_ics(width: usize, depth: usize) -> usize {
        match width > 2 {
            true => (width - 2) * (depth - 1),
            false => 0,
        }
    }

    fn expected_eocs(width: usize, depth: usize) -> usize {
        width * (depth - 1) + 1
    }

    fn expected_internals(width: usize, depth: usize) -> usize {
        (width - 1) * width / 2 * (depth - 1) + 1
    }

    #[test]
    fn test_ho() {
        for width in (1..50).step_by(5) {
            for depth in (1..50).step_by(5) {
                let probe = Rc::new(RefCell::new(TestProbe::default()));
                let coupled = HO::_create_test(width, depth, probe.clone());
                assert_eq!(expected_atomics(width, depth), probe.borrow().n_atomics);
                assert_eq!(expected_eics(width, depth), probe.borrow().n_eics);
                assert_eq!(expected_ics(width, depth), probe.borrow().n_ics);
                assert_eq!(expected_eocs(width, depth), probe.borrow().n_eocs);
                let mut simulator = RootCoordinator::new(coupled);
                simulator.simulate_time(f64::INFINITY);
                assert_eq!(expected_internals(width, depth), probe.borrow().n_internals);
                assert_eq!(expected_internals(width, depth), probe.borrow().n_externals);
                assert_eq!(expected_internals(width, depth), probe.borrow().n_events);
            }
        }
    }
}
