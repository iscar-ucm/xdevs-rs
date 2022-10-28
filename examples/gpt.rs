use xdevs::*;

#[derive(Debug)]
struct Generator {
    model: Model,
    sigma: f64,
    period: f64,
    count: usize,
    input: Shared<Port<bool>>,
    output: Shared<Port<usize>>,
}
impl Generator {
    fn new(name: &str, period: f64) -> Self {
        let mut component = Model::new(name);
        Self {
            sigma: 0.,
            period,
            count: 0,
            input: component.add_in_port::<bool>("input"),
            output: component.add_out_port::<usize>("output"),
            model: component,
        }
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
impl_atomic!(Generator); // TODO issue with private/public stuff


#[derive(Debug)]
struct Processor {
    model: Model,
    sigma: f64,
    time: f64,
    job: Option<usize>,
    input: Shared<Port<usize>>,
    output: Shared<Port<(usize, f64)>>,
}
impl Processor {
    fn new(name: &str, time: f64) -> Self {
        let mut component = Model::new(name);
        Self {
            sigma: f64::INFINITY,
            time,
            job: None,
            input: component.add_in_port::<usize>("input"),
            output: component.add_out_port::<(usize, f64)>("output"),
            model: component,
        }
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
impl_atomic!(Processor); // TODO issue with private/public stuff

#[derive(Debug)]
struct Transducer {
    model: Model,
    sigma: f64,
    input_g: Shared<Port<usize>>,
    input_p: Shared<Port<(usize, f64)>>,
    output: Shared<Port<bool>>,
}
impl Transducer {
    fn new(name: &str, time: f64) -> Self {
        let mut component = Model::new(name);
        Self {
            sigma: time,
            input_g: component.add_in_port::<usize>("input_g"),
            input_p: component.add_in_port::<(usize, f64)>("input_p"),
            output: component.add_out_port::<bool>("output"),
            model: component,
        }
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
            println!("generator sent job {} at time {}", job, self.model.simulator.t_last); // TODO
        }
        for (job, time) in self.input_p.get_values().iter() {
            println!("processor processed job {} after {} seconds at time {}", job, time, self.model.simulator.t_last); // TODO
        }
    }
    fn ta(&self) -> f64 {
        self.sigma
    }
}
impl_atomic!(Transducer); // TODO issue with private/public stuff


fn main() {
    let period = 3.;
    let time = 1.;
    let observation = 50.;
    let generator = Generator::new("generator", period);
    let processor = Processor::new("processor", time);
    let transducer = Transducer::new("transducer", observation);

    let mut coupled = Coupled::new("gpt");
    coupled.add_component(generator);
    coupled.add_component(processor);
    coupled.add_component(transducer);

    coupled.add_ic("generator", "output", "processor", "input");
    coupled.add_ic("generator", "output", "transducer", "input_g");
    coupled.add_ic("processor", "output", "transducer", "input_p");
    coupled.add_ic("transducer", "output", "generator", "input");

    let mut simulator = RootCoordinator::new(coupled);
    simulator.simulate_time(f64::INFINITY)
}