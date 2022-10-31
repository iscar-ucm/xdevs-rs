use xdevs::modeling::*;
use xdevs::simulation::*;

#[derive(Debug)]
struct Generator {
    component: Component,
    sigma: f64,
    period: f64,
    count: usize,
    input: Port<Input, bool>,
    output: Port<Output, usize>,
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
        self.output.add_value(self.count);
    }
    fn delta_int(&mut self) {
        self.count += 1;
        self.sigma = self.period;
    }
    fn delta_ext(&mut self, e: f64) {
        self.sigma -= e;
        if !self.input.is_empty() {
            self.sigma = f64::INFINITY;
        }
    }
    fn ta(&self) -> f64 {
        self.sigma
    }
}

#[derive(Debug)]
struct Processor {
    component: Component,
    sigma: f64,
    time: f64,
    job: Option<usize>,
    input: Port<Input, usize>,
    output: Port<Output, (usize, f64)>,
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
            self.output.add_value((job, self.time));
        }
    }

    fn delta_int(&mut self) {
        self.sigma = f64::INFINITY;
        self.job = None;
    }

    fn delta_ext(&mut self, e: f64) {
        self.sigma -= e;
        if self.job.is_none() {
            self.job = Some(*self.input.get_values().get(0).unwrap());
            self.sigma = self.time;
        }
    }

    fn ta(&self) -> f64 {
        self.sigma
    }
}

#[derive(Debug)]
struct Transducer {
    component: Component,
    sigma: f64,
    input_g: Port<Input, usize>,
    input_p: Port<Input, (usize, f64)>,
    output: Port<Output, bool>,
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
        self.output.add_value(true);
    }

    fn delta_int(&mut self) {
        self.sigma = f64::INFINITY;
        println!("TRANSDUCER FINISHED");
    }

    fn delta_ext(&mut self, e: f64) {
        self.sigma -= e;
        for job in self.input_g.get_values().iter() {
            println!("generator sent job {} at time {}", job, self.get_time());
        }
        for (job, time) in self.input_p.get_values().iter() {
            println!(
                "processor processed job {} after {} seconds at time {}",
                job,
                time,
                self.get_time()
            );
        }
    }

    fn ta(&self) -> f64 {
        self.sigma
    }
}

#[derive(Debug)]
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
    let period = 3.;
    let time = 1.;
    let observation = 50.;

    // let coupled = create_gpt(period, time, observation);
    let coupled = create_efp(period, time, observation);

    let mut simulator = RootCoordinator::new(coupled);
    simulator.simulate_time(f64::INFINITY)
}
