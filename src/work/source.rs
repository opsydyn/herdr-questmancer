//! Bounded, independent source delivery. An adapter owns its producer;
//! Legacy channel closure reports unavailability without deleting work.
//! Projects delivery uses its own acknowledged attachment lifecycle.
use super::{Message, SourceId};
use tokio::sync::mpsc;
#[derive(Clone, Debug, PartialEq)]
pub struct Envelope {
    pub version: u32,
    pub message: Message,
}
#[derive(Debug)]
pub struct SourceInbox {
    source: SourceId,
    receiver: Receiver,
    closed: bool,
}
#[derive(Debug)]
enum Receiver {
    Work(mpsc::Receiver<Envelope>),
    Projects(mpsc::Receiver<crate::sources::herdr_projects::Observation>),
}
pub fn channel(source: SourceId, capacity: usize) -> (mpsc::Sender<Envelope>, SourceInbox) {
    let (sender, receiver) = mpsc::channel(capacity.clamp(1, 128));
    (
        sender,
        SourceInbox {
            source,
            receiver: Receiver::Work(receiver),
            closed: false,
        },
    )
}
impl SourceInbox {
    pub async fn next_projects(&mut self) -> crate::sources::herdr_projects::Observation {
        let Receiver::Projects(receiver) = &mut self.receiver else {
            return std::future::pending().await;
        };
        match receiver.recv().await {
            Some(observation) if observation.source() == &self.source => observation,
            _ => std::future::pending().await,
        }
    }
    pub async fn next(&mut self) -> Envelope {
        if self.closed {
            return std::future::pending().await;
        }
        let Receiver::Work(receiver) = &mut self.receiver else {
            return std::future::pending().await;
        };
        if let Some(envelope) = receiver.recv().await {
            let source = match &envelope.message {
                Message::Baseline { source, .. }
                | Message::Update { source, .. }
                | Message::BindingRemoved { source, .. }
                | Message::Disconnected(source) => source,
            };
            if source != &self.source {
                return Envelope {
                    version: 0,
                    message: Message::Disconnected(self.source.clone()),
                };
            }
            return envelope;
        }
        self.closed = true;
        Envelope {
            version: 1,
            message: Message::Disconnected(self.source.clone()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceNotice {
    pub source: SourceId,
    pub message: String,
}
pub(crate) fn projects_channel(
    source: SourceId,
) -> (
    mpsc::Sender<crate::sources::herdr_projects::Observation>,
    SourceInbox,
) {
    let (sender, projects) = mpsc::channel(1);
    (
        sender,
        SourceInbox {
            source,
            receiver: Receiver::Projects(projects),
            closed: false,
        },
    )
}
