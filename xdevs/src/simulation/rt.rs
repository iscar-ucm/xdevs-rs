use crate::simulation::Simulator;
use std::time::{Duration, SystemTime};

pub mod input;
pub mod output;

#[derive(Debug, Clone, Copy)]
pub struct RootCoordinatorConfig {
    pub time_scale: f64,
    pub max_jitter: Option<Duration>,
    pub output_capacity: Option<usize>,
    pub input_buffer: Option<usize>,
    pub input_window: Option<Duration>,
}

impl RootCoordinatorConfig {
    pub fn new(
        time_scale: f64,
        max_jitter: Option<Duration>,
        output_capacity: Option<usize>,
        input_buffer: Option<usize>,
        input_window: Option<Duration>,
    ) -> Self {
        Self {
            time_scale,
            max_jitter,
            output_capacity,
            input_buffer,
            input_window,
        }
    }
}

impl Default for RootCoordinatorConfig {
    fn default() -> Self {
        Self {
            time_scale: 1.,
            max_jitter: None,
            output_capacity: None,
            input_buffer: None,
            input_window: None,
        }
    }
}

#[derive(Debug)]
pub struct RootCoordinator<T> {
    model: T,
    time_scale: f64,
    max_jitter: Option<Duration>,
    output_queue: Option<output::OutputQueue>,
    input_queue: Option<input::InputQueue>,
}

impl<T: Simulator> RootCoordinator<T> {
    pub fn new(model: T, config: RootCoordinatorConfig) -> Self {
        let output_queue = config.output_capacity.map(output::OutputQueue::new);
        let input_queue = config
            .input_buffer
            .map(|buffer| input::InputQueue::new(buffer, config.input_window));
        Self {
            model,
            time_scale: config.time_scale,
            max_jitter: config.max_jitter,
            output_queue,
            input_queue,
        }
    }

    pub fn spawn_handler<H: Handler>(&mut self, handler: H) -> Vec<tokio::task::JoinHandle<()>> {
        let input_tx = self
            .input_queue
            .as_ref()
            .map(|input_handler| input_handler.subscribe());
        let output_rx = self
            .output_queue
            .as_ref()
            .map(|output_handler| output_handler.subscribe());
        // Safety: using run from the RootCoordinator::run_handler method
        Handler::spawn(handler, input_tx, output_rx)
    }

    pub async fn simulate(mut self, t_stop: f64) {
        tracing::info!("starting simulation");

        let mut last_vt = 0.;
        let mut next_vt = f64::min(self.model.start(last_vt), t_stop);

        let start_rt = SystemTime::now();
        let mut last_rt = start_rt;

        while last_vt < t_stop {
            tracing::debug!("simulation step from vt={last_vt} to {next_vt}");
            // Compute corresponding next_rt (None means infinity)
            let duration = match next_vt {
                f64::INFINITY => Duration::MAX,
                _ => Duration::from_secs_f64((next_vt - last_vt) * self.time_scale),
            };
            let next_rt = last_rt.checked_add(duration);

            // Use input handler if available, otherwise sleep
            match &mut self.input_queue {
                Some(input_handler) => input_handler.wait_event(next_rt, &self.model).await,
                None => {
                    tracing::debug!("sleeping for {duration:?}");
                    tokio::time::sleep(duration).await
                }
            };

            // Check the jitter and update last_rt and last_vt
            let t = SystemTime::now();
            let jitter = match next_rt {
                Some(next_rt) => t.duration_since(next_rt).ok(),
                None => None,
            };
            match jitter {
                Some(jitter) => {
                    tracing::debug!("jitter of {jitter:?}");
                    // t >= next_rt, check for the jitter
                    if let Some(max_jitter) = self.max_jitter {
                        if jitter > max_jitter {
                            tracing::error!("jitter too high: {jitter:?}");
                            panic!("jitter too high: {jitter:?}");
                        }
                    }
                    last_rt = next_rt.unwrap();
                    last_vt = next_vt;
                }
                None => {
                    // t < next_rt
                    last_rt = t;
                    let duration = last_rt.duration_since(start_rt).unwrap();
                    last_vt = duration.as_secs_f64() / self.time_scale;
                }
            };
            tracing::debug!("simulation step reached vt={last_vt}");

            if last_vt >= next_vt {
                self.model.collection(last_vt);
                if let Some(output_handler) = &self.output_queue {
                    output_handler.propagate_output(&self.model);
                }
            } else if unsafe { self.model.get_component().is_input_empty() } {
                tracing::warn!("spurious external transition. Ignoring.");
                continue;
            }
            next_vt = f64::min(self.model.transition(last_vt), t_stop);
            tracing::debug!("next simulation vt = {next_vt}");
        }
        self.model.stop(t_stop);

        tracing::info!("simulation completed");
    }
}

pub trait Handler {
    ///
    /// # Safety
    ///
    /// Do not call this method directly. Use [`RootCoordinator::spawn_handler`] instead.
    fn spawn(
        self,
        input_tx: Option<input::InputSender>,
        output_rx: Option<output::OutputReceiver>,
    ) -> Vec<tokio::task::JoinHandle<()>>;
}
