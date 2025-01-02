use crate::simulation::Simulator;
use crate::Event;
use std::sync::Arc;
use tokio::sync::broadcast::{channel, Receiver, Sender};

pub type OutputReceiver = Receiver<Arc<Event>>;

#[derive(Debug)]
pub struct OutputQueue(Sender<Arc<Event>>);

impl OutputQueue {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = channel(capacity);
        Self(tx)
    }

    pub fn subscribe(&self) -> OutputReceiver {
        self.0.subscribe()
    }

    pub fn propagate_output(&self, component: &impl Simulator) {
        // Safety: ejecting events from output handler
        for event in unsafe { component.get_component().eject() } {
            tracing::info!("propagating output event {event}");
            if self.0.send(Arc::new(event)).is_err() {
                tracing::warn!("all receivers have been dropped");
                break;
            }
        }
    }
}
