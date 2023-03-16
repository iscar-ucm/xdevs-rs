extern crate core;

use std::env;
use xdevs::modeling::*;
use xdevs::simulation::*;

struct Generator {
    component: Component,
    sigma: f64,
    period: f64,
    count: usize,
    input: InPort<bool>,
    output: OutPort<usize>,
}
impl Generator {
    fn new(name: &str, period: f64) -> Self {
        let mut component = Component::new(name);
        let input = component.add_in_port::<bool>("input");
        let output = component.add_out_port::<usize>("output");
        Self {
            sigma: 0.,
            period,
            count: 0,
            input,
            output,
            component,
        }
    }
}
impl Atomic for Generator {
    fn get_component(&self) -> &Component {
        &self.component
    }
    fn get_component_mut(&mut self) -> &mut Component {
        &mut self.component
    }
    fn lambda(&self) {
        // Safety: adding message on atomic model's output port at lambda
        unsafe { self.output.add_value(self.count) };
    }
    fn delta_int(&mut self) {
        self.count += 1;
        self.sigma = self.period;
    }
    fn delta_ext(&mut self, e: f64) {
        self.sigma -= e;
        // Safety: reading messages on atomic model's input port at delta_ext
        if !unsafe { self.input.is_empty() } {
            self.sigma = f64::INFINITY;
        }
    }
    fn ta(&self) -> f64 {
        self.sigma
    }
}

struct Processor {
    component: Component,
    sigma: f64,
    time: f64,
    job: Option<usize>,
    input: InPort<usize>,
    output: OutPort<(usize, f64)>,
}
impl Processor {
    fn new(name: &str, time: f64) -> Self {
        let mut component = Component::new(name);
        let input = component.add_in_port::<usize>("input");
        let output = component.add_out_port::<(usize, f64)>("output");
        Self {
            sigma: f64::INFINITY,
            time,
            job: None,
            input,
            output,
            component,
        }
    }
}
impl Atomic for Processor {
    fn get_component(&self) -> &Component {
        &self.component
    }

    fn get_component_mut(&mut self) -> &mut Component {
        &mut self.component
    }

    fn lambda(&self) {
        if let Some(job) = self.job {
            // Safety: adding message on atomic model's output port at lambda
            unsafe { self.output.add_value((job, self.time)) };
        }
    }

    fn delta_int(&mut self) {
        self.sigma = f64::INFINITY;
        self.job = None;
    }

    fn delta_ext(&mut self, e: f64) {
        self.sigma -= e;
        if self.job.is_none() {
            // Safety: reading messages on atomic model's input port at delta_ext
            self.job = Some(*unsafe { self.input.get_values() }.first().unwrap());
            self.sigma = self.time;
        }
    }

    fn ta(&self) -> f64 {
        self.sigma
    }
}

struct Transducer {
    component: Component,
    sigma: f64,
    input_g: InPort<usize>,
    input_p: InPort<(usize, f64)>,
    output: OutPort<bool>,
}

impl Transducer {
    fn new(name: &str, time: f64) -> Self {
        let mut component = Component::new(name);
        let input_g = component.add_in_port::<usize>("input_g");
        let input_p = component.add_in_port::<(usize, f64)>("input_p");
        let output = component.add_out_port::<bool>("output");
        Self {
            sigma: time,
            input_g,
            input_p,
            output,
            component,
        }
    }
}
impl Atomic for Transducer {
    fn get_component(&self) -> &Component {
        &self.component
    }

    fn get_component_mut(&mut self) -> &mut Component {
        &mut self.component
    }

    fn lambda(&self) {
        // Safety: adding message on atomic model's output port at lambda
        unsafe { self.output.add_value(true) };
    }

    fn delta_int(&mut self) {
        self.sigma = f64::INFINITY;
        println!("TRANSDUCER FINISHED");
    }

    fn delta_ext(&mut self, e: f64) {
        self.sigma -= e;
        let t = self.component.get_t_last() + e;
        // Safety: reading messages on atomic model's input port at delta_ext
        for job in unsafe { self.input_g.get_values() }.iter() {
            println!("generator sent job {job} at time {t}");
        }
        // Safety: reading messages on atomic model's input port at delta_ext
        for (job, time) in unsafe { self.input_p.get_values() }.iter() {
            println!("processor processed job {job} after {time} seconds at time {t}");
        }
    }

    fn ta(&self) -> f64 {
        self.sigma
    }
}

struct ExperimentalFrame {
    coupled: Coupled,
}
impl ExperimentalFrame {
    fn new(name: &str, period: f64, observation: f64) -> Self {
        let mut coupled = Coupled::new(name);
        coupled.add_in_port::<(usize, f64)>("input");
        coupled.add_out_port::<usize>("output");

        let generator = Generator::new("generator", period);
        let transducer = Transducer::new("transducer", observation);

        coupled.add_component(Box::new(generator));
        coupled.add_component(Box::new(transducer));

        coupled.add_eic("input", "transducer", "input_p");
        coupled.add_ic("generator", "output", "transducer", "input_g");
        coupled.add_ic("transducer", "output", "generator", "input");
        coupled.add_eoc("generator", "output", "output");

        Self { coupled }
    }
}

fn create_gpt(period: f64, time: f64, observation: f64) -> Coupled {
    let generator = Generator::new("generator", period);
    let processor = Processor::new("processor", time);
    let transducer = Transducer::new("transducer", observation);

    let mut gpt = Coupled::new("gpt");
    gpt.add_component(Box::new(generator));
    gpt.add_component(Box::new(processor));
    gpt.add_component(Box::new(transducer));

    gpt.add_ic("generator", "output", "processor", "input");
    gpt.add_ic("generator", "output", "transducer", "input_g");
    gpt.add_ic("processor", "output", "transducer", "input_p");
    gpt.add_ic("transducer", "output", "generator", "input");

    gpt
}

fn create_efp(period: f64, time: f64, observation: f64) -> Coupled {
    let mut efp = Coupled::new("efp");

    let ef = ExperimentalFrame::new("ef", period, observation);
    let processor = Processor::new("processor", time);

    efp.add_component(Box::new(ef.coupled));
    efp.add_component(Box::new(processor));

    efp.add_ic("ef", "output", "processor", "input");
    efp.add_ic("processor", "output", "ef", "input");
    efp
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let model_type = match args.get(1) {
        Some(model) => model.clone(),
        None => String::from("gpt"),
    }
    .to_lowercase();
    let period = 3.;
    let time = 1.;
    let observation = 50.;

    let coupled = match model_type.as_str() {
        "gpt" => create_gpt(period, time, observation),
        "efp" => create_efp(period, time, observation),
        _ => panic!("unknown model type. It must be either \"gpt\" or \"efp\""),
    };
    let mut simulator = RealTimeCoordinator::new(coupled, 1., 0.);
    simulator.simulate(f64::INFINITY)
}
