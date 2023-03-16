use super::Simulator;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

/// Root coordinator for sequential simulations of DEVS models.
pub struct RootCoordinator<T> {
    model: T,
    scale: f64,
    max_jitter: Option<Duration>,
}

impl<T: Simulator> RootCoordinator<T> {
    /// Creates a new root coordinator from a DEVS-compliant model.
    pub fn new(model: T, scale: f64, max_jitter: f64) -> Self {
        let max_jitter = match max_jitter > 0. {
            true => Some(Duration::from_secs_f64(max_jitter)),
            false => None,
        };
        Self {
            model,
            scale,
            max_jitter,
        }
    }

    pub fn new_default(model: T) -> Self {
        Self::new(model, 1., 0.)
    }

    /// Runs a simulation for a given period of time.
    pub fn simulate(&mut self, t_end: f64) {
        let (vt_start, vt_end) = (0., t_end);
        self.model.start(vt_start);

        let (mut vt_last, mut vt_next) = (self.model.get_t_last(), self.model.get_t_next());
        let mut rt_last = SystemTime::now();

        while vt_next < vt_end {
            let rt_next = rt_last + Duration::from_secs_f64((vt_next - vt_last) * self.scale);
            match rt_next.duration_since(SystemTime::now()) {
                Ok(duration) => sleep(duration),
                Err(err) => {
                    if let Some(max_jitter) = self.max_jitter {
                        if err.duration() > max_jitter {
                            panic!("too much jitter");
                        }
                    }
                }
            };
            self.model.collection(vt_next);
            self.model.transition(vt_next);

            rt_last = rt_next;
            (vt_last, vt_next) = (vt_next, self.model.get_t_next());
        }
        self.model.stop(vt_next);
    }
}
