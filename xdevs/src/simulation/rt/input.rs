use crate::{simulation::Simulator, Event};
use std::time::{Duration, SystemTime};
use tokio::{
    sync::mpsc::{channel, Receiver, Sender},
    time::timeout,
};

pub type InputSender = Sender<Event>;

#[derive(Debug)]
pub struct InputQueue {
    sender: Sender<Event>,
    receiver: Receiver<Event>,
    window: Option<Duration>,
}

impl InputQueue {
    pub fn new(buffer: usize, window: Option<Duration>) -> Self {
        let (sender, receiver) = channel(buffer);
        Self {
            sender,
            receiver,
            window,
        }
    }

    pub fn subscribe(&self) -> InputSender {
        self.sender.clone()
    }

    pub async fn wait_event(&mut self, t_next: Option<SystemTime>, component: &impl Simulator) {
        tracing::debug!("waiting for external events");
        let duration = match t_next {
            Some(t_next) => t_next.duration_since(SystemTime::now()).unwrap_or_default(),
            None => Duration::MAX,
        };
        self.inject_timeout(duration, component).await;
        // If there is a window, keep receiving events until the window expires
        if let Some(window) = self.window {
            tracing::debug!("waiting for more external events within the window");
            let t_max = match t_next {
                Some(t_next) => std::cmp::min(t_next, SystemTime::now() + window),
                None => SystemTime::now() + window,
            };
            while let Ok(duration) = t_max.duration_since(SystemTime::now()) {
                self.inject_timeout(duration, component).await;
            }
        }
    }

    async fn inject_timeout(&mut self, duration: Duration, component: &impl Simulator) {
        match timeout(duration, self.receiver.recv()).await {
            Err(_) => {
                tracing::debug!("timeout expired without any external events");
            }
            Ok(None) => {
                tracing::error!("all senders have been dropped");
                unreachable!();
            }
            Ok(Some(event)) => {
                tracing::info!("injecting input event {event}");
                // Safety: injecting event from input handler
                match unsafe { component.get_component().inject(event) } {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("failed to inject event: {:?}. Skipping.", e);
                    }
                }
            }
        }
    }
}
