use super::{DEVStoneAtomic, DEVStoneSeeder, TestProbe};
use crate::modeling::Coupled;
use std::cell::RefCell;
use std::rc::Rc;

pub struct HOmod {
    pub coupled: Coupled,
}

impl HOmod {
    pub fn create(width: usize, depth: usize) -> Coupled {
        let mut coupled = Coupled::new("HOmod");
        let seeder = DEVStoneSeeder::new("seeder");
        let homod = Self::new(width, depth, None);
        let homod_name = homod.coupled.component.get_name().to_string();
        coupled.add_component(Box::new(seeder));
        coupled.add_component(Box::new(homod.coupled));
        coupled.add_ic("seeder", "output", &homod_name, "input_1");
        coupled.add_ic("seeder", "output", &homod_name, "input_2");
        coupled
    }

    fn _create_test(width: usize, depth: usize, probe: Rc<RefCell<TestProbe>>) -> Coupled {
        let mut coupled = Coupled::new("HOmod");
        let seeder = DEVStoneSeeder::new("seeder");
        let homod = Self::new(width, depth, Some(probe));
        let homod_name = homod.coupled.component.get_name().to_string();
        coupled.add_component(Box::new(seeder));
        coupled.add_component(Box::new(homod.coupled));
        coupled.add_ic("seeder", "output", &homod_name, "input_1");
        coupled.add_ic("seeder", "output", &homod_name, "input_2");
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
        coupled.add_out_port::<usize>("output");
        // If this is the inner coupled model, we just add one atomic.
        if depth == 1 {
            let atomic = DEVStoneAtomic::new("inner_atomic", probe.clone());
            coupled.add_component(Box::new(atomic));
            coupled.add_eic("input_1", "inner_atomic", "input");
            coupled.add_eoc("inner_atomic", "output", "output");
            // Otherwise, we add a subcoupled and a set of atomics.
        } else {
            let subcoupled = Self::new(width, depth - 1, probe.clone());
            let subcoupled_name = subcoupled.coupled.component.get_name().to_string();
            coupled.add_component(Box::new(subcoupled.coupled));
            coupled.add_eic("input_1", &subcoupled_name, "input_1");
            coupled.add_eoc(&subcoupled_name, "output", "output");
            let mut prev_row: Vec<String> = Vec::new();
            let mut current_row: Vec<String> = Vec::new();
            // First row
            for i in 1..width {
                let atomic_name = format!("atomic(1,{i}");
                prev_row.push(atomic_name.clone());
                let atomic = DEVStoneAtomic::new(&atomic_name, probe.clone());
                coupled.add_component(Box::new(atomic));
                coupled.add_eic("input_2", &atomic_name, "input");
                coupled.add_ic(&atomic_name, "output", &subcoupled_name, "input_2");
            }
            // Second row
            for i in 1..width {
                let atomic_name = format!("atomic(2,{i}");
                current_row.push(atomic_name.clone());
                let atomic = DEVStoneAtomic::new(&atomic_name, probe.clone());
                coupled.add_component(Box::new(atomic));
                if i == 1 {
                    coupled.add_eic("input_2", &atomic_name, "input");
                }
                for prev_name in &prev_row {
                    coupled.add_ic(&atomic_name, "output", prev_name, "input");
                }
            }
            // Rest of the tree
            for layer in 3..(width + 1) {
                prev_row = current_row;
                current_row = Vec::new();
                for i in 1..prev_row.len() {
                    let atomic_name = format!("atomic({layer},{i}");
                    current_row.push(atomic_name.clone());
                    let atomic = DEVStoneAtomic::new(&atomic_name, probe.clone());
                    coupled.add_component(Box::new(atomic));
                    if i == 1 {
                        coupled.add_eic("input_2", &atomic_name, "input");
                    }
                    coupled.add_ic(&atomic_name, "output", prev_row.get(i).unwrap(), "input");
                }
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
        (width - 1 + (width - 1) * width / 2) * (depth - 1) + 1
    }

    fn expected_eics(width: usize, depth: usize) -> usize {
        (2 * (width - 1) + 1) * (depth - 1) + 1
    }

    fn expected_ics(width: usize, depth: usize) -> usize {
        ((width - 1) * (width - 1) + (width - 1) * width / 2) * (depth - 1)
    }

    fn expected_eocs(_width: usize, depth: usize) -> usize {
        depth
    }

    fn expected_internals(width: usize, depth: usize) -> usize {
        let mut n = 1;
        for d in 1..depth {
            n += (1 + (d - 1) * (width - 1)) * (width - 1) * width / 2
                + (width - 1) * (width + (d - 1) * (width - 1));
        }
        n
    }

    fn expected_events(width: usize, depth: usize) -> usize {
        let mut n = 1;
        if width > 1 && depth > 1 {
            n += 2 * (width - 1);
            let mut aux = 0;
            for i in 2..depth {
                aux += 1 + (i - 1) * (width - 1);
            }
            n += aux * 2 * (width - 1) * (width - 1);
            n += (aux + 1) * ((width - 1) * (width - 1) + (width - 2) * (width - 1) / 2);
        }
        n
    }

    #[test]
    fn test_homod() {
        for width in (1..10).step_by(1) {
            for depth in (1..10).step_by(1) {
                let probe = Rc::new(RefCell::new(TestProbe::default()));
                let coupled = HOmod::_create_test(width, depth, probe.clone());
                assert_eq!(expected_atomics(width, depth), probe.borrow().n_atomics);
                assert_eq!(expected_eics(width, depth), probe.borrow().n_eics);
                assert_eq!(expected_ics(width, depth), probe.borrow().n_ics);
                assert_eq!(expected_eocs(width, depth), probe.borrow().n_eocs);
                let mut simulator = RootCoordinator::new(coupled);
                simulator.simulate_time(f64::INFINITY);
                assert_eq!(expected_internals(width, depth), probe.borrow().n_internals);
                assert_eq!(expected_internals(width, depth), probe.borrow().n_externals);
                assert_eq!(expected_events(width, depth), probe.borrow().n_events);
            }
        }
    }
}
