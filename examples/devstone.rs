use std::env;
use xdevs::devstone::*;
use xdevs::modeling::*;
use xdevs::simulation::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    let model_type = args
        .get(1)
        .expect("first argument must select the model type");
    let width = args
        .get(2)
        .expect("second argument must select the width")
        .parse()
        .expect("width could not be parsed");
    let depth = args
        .get(3)
        .expect("third argument must select the depth")
        .parse()
        .expect("depth could not be parsed");

    let mut coupled: Coupled;
    if model_type == "HI" {
        coupled = HI::new(width, depth);
    } else {
        panic!()
    }
    let mut simulator = RootCoordinator::new(coupled);
    simulator.simulate_time(f64::INFINITY)
}
