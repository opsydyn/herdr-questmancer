//! Owned, bounded local reads. Source delivery is not storage acknowledgement.
use super::{Checkpoint, MappingError, ReadError, Registration, Selection, Snapshot, read::Lease};
use crate::{
    app::Model,
    domain::Timestamp,
    persistence::PersistenceClient,
    work::{Projection, SourceId, source::SourceInbox},
};
use std::{
    path::{Component, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::{
    sync::{mpsc, oneshot, watch},
    task::JoinHandle,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeSelection {
    selection: Selection,
    registration: Registration,
}
impl RuntimeSelection {
    pub fn new(source: SourceId, campaign: &str, path: PathBuf) -> Result<Self, ReadError> {
        if path
            .components()
            .any(|c| !matches!(c, Component::RootDir | Component::Normal(_)))
        {
            return Err(ReadError::Selection);
        }
        let registration = Registration::retained_records(source.clone(), campaign)
            .map_err(|_| ReadError::Selection)?;
        Ok(Self {
            selection: Selection::new(source, campaign, path)?,
            registration,
        })
    }
    pub fn source(&self) -> &SourceId {
        self.registration.source()
    }
}

#[derive(Debug)]
pub struct Producer {
    stop: watch::Sender<bool>,
    cancelled: Arc<AtomicBool>,
    task: Option<JoinHandle<()>>,
}
impl Producer {
    pub fn start(
        selection: RuntimeSelection,
        projection: &Projection,
        checkpoint: Option<Checkpoint>,
    ) -> Result<(Self, SourceInbox), MappingError> {
        if checkpoint
            .as_ref()
            .is_some_and(|c| c.registration() != &selection.registration)
            || projection.sources.contains_key(selection.source()) != checkpoint.is_some()
        {
            return Err(MappingError::Identity);
        }
        if let Some(c) = &checkpoint {
            c.validate(projection)?;
        }
        let generation = selection.registration.next_generation(projection)?;
        let accepted_generation = projection
            .sources
            .get(selection.source())
            .map(|s| s.generation);
        let (sender, inbox) = crate::work::source::projects_channel(selection.source().clone());
        let (stop, receiver) = watch::channel(false);
        let cancelled = Arc::new(AtomicBool::new(false));
        let task = tokio::spawn(run(
            selection,
            generation,
            accepted_generation,
            checkpoint,
            sender,
            receiver,
            cancelled.clone(),
        ));
        Ok((
            Self {
                stop,
                cancelled,
                task: Some(task),
            },
            inbox,
        ))
    }
    pub async fn shutdown(mut self) -> Result<(), tokio::task::JoinError> {
        self.cancelled.store(true, Ordering::Relaxed);
        let _ = self.stop.send(true);
        self.task.take().expect("owned producer task").await
    }
}
impl Drop for Producer {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Relaxed);
        let _ = self.stop.send(true);
    }
}

#[derive(Debug)]
enum Input {
    Snapshot(Box<Snapshot>, Arc<Lease>),
    Failure { message: String, rebind: bool },
}
#[derive(Debug)]
enum Acknowledgement {
    Accepted(Checkpoint),
    Retry,
    Stop,
}

/// One trusted attachment observation, consumed against the latest model. It
/// cannot advance the producer checkpoint until atomic publication is acknowledged.
#[derive(Debug)]
pub struct Observation {
    selection: RuntimeSelection,
    generation: u64,
    checkpoint: Option<Checkpoint>,
    input: Input,
    acknowledgement: oneshot::Sender<Acknowledgement>,
}
impl Observation {
    pub fn source(&self) -> &SourceId {
        self.selection.source()
    }
    /// Returns whether domain facts, freshness or the visible diagnostic changed.
    pub async fn consume(self, client: &PersistenceClient, model: &mut Model) -> bool {
        let source = self.selection.source().clone();
        let progress = |model: &Model| {
            model
                .work()
                .sources
                .get(&source)
                .map(|s| (s.generation, s.cursor, s.freshness))
        };
        let before = progress(model);
        let notice = model.work_source_notice().cloned();
        // Fence every completion, including read/lease failures, before touching
        // newer health or facts. A superseded producer has no authority there.
        let stale = model
            .work()
            .sources
            .get(&source)
            .map_or(self.generation != 1, |s| {
                self.generation < s.generation
                    || self.generation > s.generation
                        && s.generation.checked_add(1) != Some(self.generation)
                    || self.generation == s.generation
                        && s.freshness == crate::work::Freshness::Cached
            });
        if stale {
            let _ = self.acknowledgement.send(Acknowledgement::Stop);
            return false;
        }
        let response = match self.input {
            Input::Failure { message, rebind } => {
                fail(model, &source, &message, rebind);
                if rebind {
                    Acknowledgement::Stop
                } else {
                    Acknowledgement::Retry
                }
            }
            Input::Snapshot(snapshot, lease) => {
                let selected = self.selection.selection.clone();
                let checked = tokio::task::spawn_blocking(move || lease.check(&selected)).await;
                if matches!(checked, Ok(Ok(()))) {
                    match crate::work::runtime::accept_snapshot(
                        client,
                        model,
                        &self.selection.registration,
                        self.generation,
                        &snapshot,
                        self.checkpoint.as_ref(),
                    )
                    .await
                    {
                        Ok(checkpoint) => {
                            model.set_work_source_notice(None);
                            Acknowledgement::Accepted(checkpoint)
                        }
                        Err(error) => {
                            let stop = error.requires_rebind()
                                || matches!(
                                    error,
                                    crate::work::runtime::ObservationError::Mapping(
                                        MappingError::Exhausted | MappingError::Stale
                                    )
                                );
                            model.set_work_source_notice(Some(crate::work::source::SourceNotice { source: source.clone(), message: format!("{error}{}", if stop { "; attachment stopped. Confirm/rebind the retained project and restart; use fresh tokens for a replacement." } else { "; retaining facts and retrying." }) }));
                            if stop {
                                Acknowledgement::Stop
                            } else {
                                Acknowledgement::Retry
                            }
                        }
                    }
                } else {
                    fail(model, &source, &ReadError::Lease.to_string(), true);
                    Acknowledgement::Stop
                }
            }
        };
        let _ = self.acknowledgement.send(response);
        before != progress(model) || notice.as_ref() != model.work_source_notice()
    }
}
pub(crate) fn fail(model: &mut Model, source: &SourceId, message: &str, rebind: bool) {
    let mut work = model.work().clone();
    if rebind {
        work.reconcile(source);
    } else {
        work = crate::work::reduce(work, crate::work::Message::Disconnected(source.clone()))
            .expect("disconnection cannot fail");
    }
    model.replace_work(work);
    model.set_work_source_notice(Some(crate::work::source::SourceNotice { source: source.clone(), message: format!("{message}{}", if rebind { "; attachment stopped. Confirm/rebind the retained project and restart; use fresh tokens for a replacement." } else { "; retaining last facts and retrying." }) }));
}

fn capture_one(
    existing: Option<Arc<Lease>>,
    selected: &Selection,
    control: &AtomicBool,
) -> Result<(Arc<Lease>, Result<Snapshot, ReadError>), ReadError> {
    let budget = Instant::now() + Duration::from_secs(2);
    let check = || capture_budget(control, budget);
    check()?;
    let lease = existing.map_or_else(|| Lease::open(selected).map(Arc::new), Ok)?;
    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let snapshot = lease.capture(
        selected,
        Timestamp::from_millis(i64::try_from(epoch).unwrap_or(i64::MAX)),
        &check,
    );
    Ok((lease, snapshot))
}
fn capture_budget(control: &AtomicBool, deadline: Instant) -> Result<(), ReadError> {
    if control.load(Ordering::Relaxed) {
        Err(ReadError::Cancelled)
    } else if Instant::now() >= deadline {
        Err(ReadError::TimedOut)
    } else {
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
async fn run(
    selection: RuntimeSelection,
    mut generation: u64,
    mut accepted_generation: Option<u64>,
    mut checkpoint: Option<Checkpoint>,
    sender: mpsc::Sender<Observation>,
    mut stop: watch::Receiver<bool>,
    cancelled: Arc<AtomicBool>,
) {
    let mut lease = None;
    let mut backoff = Duration::from_secs(5);
    loop {
        if *stop.borrow() {
            return;
        }
        let selected = selection.selection.clone();
        let existing = lease.clone();
        let control = cancelled.clone();
        let mut capture =
            tokio::task::spawn_blocking(move || capture_one(existing, &selected, &control));
        let result = tokio::select! {
            result = &mut capture => result,
            _ = stop.changed() => {
                cancelled.store(true, Ordering::Relaxed);
                let _ = capture.await;
                return;
            }
        };
        let input = match result {
            Ok(Ok((held, snapshot))) => {
                lease = Some(held.clone());
                match snapshot {
                    Ok(snapshot) => Input::Snapshot(Box::new(snapshot), held),
                    Err(ReadError::Cancelled) => return,
                    Err(error) => Input::Failure {
                        rebind: matches!(error, ReadError::Lease),
                        message: error.to_string(),
                    },
                }
            }
            Ok(Err(ReadError::Cancelled)) => return,
            Ok(Err(error)) => Input::Failure {
                rebind: matches!(error, ReadError::Lease),
                message: error.to_string(),
            },
            Err(error) => Input::Failure {
                rebind: false,
                message: format!("source capture task failed: {error}"),
            },
        };
        let (acknowledgement, response) = oneshot::channel();
        let observation = Observation {
            selection: selection.clone(),
            generation,
            checkpoint: checkpoint.clone(),
            input,
            acknowledgement,
        };
        tokio::select! {
            result = sender.send(observation) => if result.is_err() { return; },
            _ = stop.changed() => return,
        }
        let result = tokio::select! {
            result = response => result,
            _ = stop.changed() => return,
        };
        let delay = match result {
            Ok(Acknowledgement::Accepted(next)) => {
                checkpoint = Some(next);
                accepted_generation = Some(generation);
                backoff = Duration::from_secs(5);
                backoff
            }
            Ok(Acknowledgement::Retry) => {
                if let Some(accepted) = accepted_generation {
                    let Some(next) = accepted.checked_add(1) else {
                        fail_counter(&selection, &sender, &mut stop, generation, checkpoint).await;
                        return;
                    };
                    generation = next;
                }
                let delay = backoff;
                backoff = backoff.saturating_mul(2).min(Duration::from_secs(30));
                delay
            }
            Ok(Acknowledgement::Stop) => {
                let _ = stop.changed().await;
                return;
            }
            Err(_) => return,
        };
        tokio::select! { () = tokio::time::sleep(delay) => {}, _ = stop.changed() => return }
    }
}
async fn fail_counter(
    selection: &RuntimeSelection,
    sender: &mpsc::Sender<Observation>,
    stop: &mut watch::Receiver<bool>,
    generation: u64,
    checkpoint: Option<Checkpoint>,
) {
    let (acknowledgement, _) = oneshot::channel();
    let observation = Observation {
        selection: selection.clone(),
        generation,
        checkpoint,
        input: Input::Failure {
            message: MappingError::Exhausted.to_string(),
            rebind: true,
        },
        acknowledgement,
    };
    tokio::select! { _ = sender.send(observation) => {}, _ = stop.changed() => return }
    let _ = stop.changed().await;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cancellation_and_expired_budget_do_not_start_a_filesystem_read() {
        let selected = Selection::new(
            SourceId::new("projects:cancelled"),
            "test",
            PathBuf::from("/not/a/project"),
        )
        .unwrap();
        let cancelled = AtomicBool::new(true);
        assert!(matches!(
            capture_one(None, &selected, &cancelled),
            Err(ReadError::Cancelled)
        ));
        assert!(matches!(
            capture_budget(&AtomicBool::new(false), Instant::now()),
            Err(ReadError::TimedOut)
        ));
    }
}
