use crate::simulation::Simulator;
use std::time::{Duration, SystemTime};

#[cfg(feature = "async_rt")]
pub mod input;
#[cfg(feature = "async_rt")]
pub mod output;

/// It computes and waits for the next wall-clock time corresponding to the next state transition of the model.
///
/// An input handler function waits for external events without exceeding the time for the next internal event.
/// Finally, it checks that the wall-clock drift does not exceed the maximum jitter allowed (if any) and panics if it does.
///
/// # Arguments
///
/// * `time_scale` - The time scale factor between virtual and wall-clock time.
///   A value of `1.0` implies that virtual time and wall-clock time use the same scale.
/// * `max_jitter` - The maximum allowed jitter. If `None`, no jitter check is performed.
///   If `Some(value)`, the simulation panics if the wall-clock drift exceeds `value`.
/// * `input_handler` - The function to handle incoming external events. This function expects two arguments:
///   - `next_t: [Option<SystemTime>]` - System time of the next internal event.
///     If `None`, it means that the next simulation time is infinity.
///     The input handler function may return earlier if an input event is received.
///     Note, however, that it must **NOT** return after, as it would result in an incorrect real-time implementation.
///   - `model: &T` - Reference to the top-most model under simulation.
///    
/// # Returns
///
/// A closure that takes the expected next virtual time and a mutable reference to the bag
/// and returns the next virtual time after injecting input events (if any).
///
/// # Example
///
/// ```ignore
/// xdevs::simulator::std::wait_event(0., 1., Some(Duration::from_millis(50)), some_input_handler);
/// ```
pub fn wait_event<T: Simulator>(
    time_scale: f64,
    max_jitter: Option<Duration>,
    mut input_handler: impl FnMut(Option<SystemTime>, &T),
) -> impl FnMut(f64, &T) -> f64 {
    let mut last_vt = 0.;
    let mut last_rt = SystemTime::now();
    let start_rt = last_rt;

    move |next_vt, component| -> f64 {
        // Compute wall-clock time for the next state transition
        // If next_rt is None, it means that next simulation time is infinity
        let duration = match next_vt {
            f64::INFINITY => Duration::MAX,
            _ => Duration::from_secs_f64((next_vt - last_vt) * time_scale),
        };
        let next_rt = last_rt.checked_add(duration);
        // Use input_handler to wait for external events
        input_handler(next_rt, component);

        // Check the jitter and update last_rt and last_vt
        let t = SystemTime::now();
        let jitter = match next_rt {
            Some(next_rt) => t.duration_since(next_rt).ok(),
            None => None,
        };
        match jitter {
            Some(jitter) => {
                // t >= next_rt, check for the jitter
                if let Some(max_jitter) = max_jitter {
                    if jitter > max_jitter {
                        panic!("jitter too high: {:?}", jitter);
                    }
                }
                last_rt = next_rt.unwrap();
                last_vt = next_vt;
            }
            None => {
                // t < next_rt
                last_rt = t;
                let duration = last_rt.duration_since(start_rt).unwrap();
                last_vt = duration.as_secs_f64() / time_scale;
            }
        };

        last_vt
    }
}

/// Basic `wait_event` closure for RT simulation. It sleeps until the next state transition.
///
/// # Arguments
///
/// * `time_scale` - The time scale factor between virtual and wall-clock time.
///   A value of `1.0` implies that virtual time and wall-clock time use the same scale.
/// * `max_jitter` - The maximum allowed jitter. If `None`, no jitter check is performed.
///   If `Some(value)`, the simulation panics if the wall-clock drift exceeds `value`.
pub fn sleep<T: Simulator>(
    time_scale: f64,
    max_jitter: Option<std::time::Duration>,
) -> impl FnMut(f64, &T) -> f64 {
    wait_event(time_scale, max_jitter, |next_t, _| {
        let duration = match next_t {
            Some(next_t) => next_t.duration_since(SystemTime::now()).unwrap_or_default(),
            None => Duration::MAX,
        };
        std::thread::sleep(duration);
    })
}

#[cfg(feature = "async_rt")]
#[derive(Debug)]
pub struct RootCoordinator<T> {
    model: T,
    time_scale: f64,
    max_jitter: Option<Duration>,
    output_queue: Option<output::OutputQueue>,
    input_queue: Option<input::InputQueue>,
    locked: bool,
}

#[cfg(feature = "async_rt")]
impl<T: Simulator> RootCoordinator<T> {
    pub fn new(model: T, time_scale: f64, max_jitter: Option<Duration>) -> Self {
        Self {
            model,
            time_scale,
            max_jitter,
            output_queue: None,
            input_queue: None,
            locked: false,
        }
    }

    pub fn create_output_queue(&mut self, capacity: usize) {
        if self.locked {
            panic!("Handlers already created");
        }
        if self.output_queue.is_some() {
            panic!("Output queue already created");
        }
        self.output_queue = Some(output::OutputQueue::new(capacity));
    }

    pub fn create_input_queue(&mut self, buffer: usize, window: Option<Duration>) {
        if self.locked {
            panic!("Handlers already created");
        }
        if self.input_queue.is_some() {
            panic!("Input queue already created");
        }
        self.input_queue = Some(input::InputQueue::new(buffer, window));
    }

    pub fn spawn_handler<H: Handler>(&mut self, handler: H) -> Vec<tokio::task::JoinHandle<()>> {
        self.locked = true;
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

#[cfg(feature = "async_rt")]
pub trait Handler: Send {
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
