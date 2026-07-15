use std::{path::PathBuf, pin::Pin, time::Duration};

use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
    time::{Instant, Sleep},
};

use crate::domain::GuestbookEntry;

use super::{
    PersistedStateV1, PersistenceDiagnostic, PersistenceError, append_guestbook, publish_state,
};

const DEBOUNCE: Duration = Duration::from_millis(250);
const DIAGNOSTIC_CAPACITY: usize = 16;

enum PersistenceMessage {
    StageState(PersistedStateV1),
    AppendGuestbook {
        entry: GuestbookEntry,
        acknowledgement: oneshot::Sender<Result<(), PersistenceError>>,
    },
    Flush(oneshot::Sender<Result<(), PersistenceError>>),
    Shutdown(oneshot::Sender<Result<(), PersistenceError>>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkerPaths {
    pub state: Option<PathBuf>,
    pub guestbook: Option<PathBuf>,
}

impl WorkerPaths {
    pub fn new(state: Option<PathBuf>, guestbook: Option<PathBuf>) -> Self {
        Self { state, guestbook }
    }
}

#[derive(Debug)]
pub struct PersistenceClient {
    sender: mpsc::UnboundedSender<PersistenceMessage>,
    last_staged: Option<PersistedStateV1>,
}

impl PersistenceClient {
    pub fn stage_state(&mut self, state: PersistedStateV1) -> Result<bool, PersistenceError> {
        if self.last_staged.as_ref() == Some(&state) {
            return Ok(false);
        }
        self.sender
            .send(PersistenceMessage::StageState(state.clone()))
            .map_err(|_| channel_error("stage state"))?;
        self.last_staged = Some(state);
        Ok(true)
    }

    pub async fn append_guestbook(&self, entry: GuestbookEntry) -> Result<(), PersistenceError> {
        let (acknowledgement, response) = oneshot::channel();
        self.sender
            .send(PersistenceMessage::AppendGuestbook {
                entry,
                acknowledgement,
            })
            .map_err(|_| channel_error("append guestbook"))?;
        receive_acknowledgement(response, "append guestbook").await
    }

    pub async fn flush(&self) -> Result<(), PersistenceError> {
        let (acknowledgement, response) = oneshot::channel();
        self.sender
            .send(PersistenceMessage::Flush(acknowledgement))
            .map_err(|_| channel_error("flush persistence"))?;
        receive_acknowledgement(response, "flush persistence").await
    }

    pub async fn shutdown(&self) -> Result<(), PersistenceError> {
        let (acknowledgement, response) = oneshot::channel();
        self.sender
            .send(PersistenceMessage::Shutdown(acknowledgement))
            .map_err(|_| channel_error("request persistence shutdown"))?;
        receive_acknowledgement(response, "persistence shutdown").await
    }
}

#[derive(Debug)]
pub struct PersistenceWorker;

impl PersistenceWorker {
    pub fn start(
        paths: WorkerPaths,
    ) -> (
        PersistenceClient,
        mpsc::Receiver<PersistenceDiagnostic>,
        JoinHandle<()>,
    ) {
        let (sender, messages) = mpsc::unbounded_channel();
        let (diagnostic_sender, diagnostics) = mpsc::channel(DIAGNOSTIC_CAPACITY);
        let worker = tokio::spawn(run(paths, messages, diagnostic_sender));
        (
            PersistenceClient {
                sender,
                last_staged: None,
            },
            diagnostics,
            worker,
        )
    }
}

async fn run(
    paths: WorkerPaths,
    mut messages: mpsc::UnboundedReceiver<PersistenceMessage>,
    diagnostics: mpsc::Sender<PersistenceDiagnostic>,
) {
    let mut dirty_state = None;
    let mut debounce: Option<Pin<Box<Sleep>>> = None;

    loop {
        tokio::select! {
            biased;
            message = messages.recv() => match message {
                Some(PersistenceMessage::StageState(state)) => {
                    dirty_state = Some(state);
                    let deadline = Instant::now() + DEBOUNCE;
                    if let Some(timer) = debounce.as_mut() {
                        timer.as_mut().reset(deadline);
                    } else {
                        debounce = Some(Box::pin(tokio::time::sleep_until(deadline)));
                    }
                }
                Some(PersistenceMessage::AppendGuestbook { entry, acknowledgement }) => {
                    let result = append_entry(&paths, &entry, &diagnostics).await;
                    let _ = acknowledgement.send(result);
                }
                Some(PersistenceMessage::Flush(acknowledgement)) => {
                    let result = publish_dirty_state(&paths, &mut dirty_state, &diagnostics).await;
                    debounce = None;
                    let _ = acknowledgement.send(result);
                }
                Some(PersistenceMessage::Shutdown(acknowledgement)) => {
                    let result = publish_dirty_state(&paths, &mut dirty_state, &diagnostics).await;
                    let _ = acknowledgement.send(result);
                    return;
                }
                None => return,
            },
            () = wait_for_debounce(&mut debounce), if debounce.is_some() => {
                let _ = publish_dirty_state(&paths, &mut dirty_state, &diagnostics).await;
                debounce = None;
            }
        }
    }
}

async fn append_entry(
    paths: &WorkerPaths,
    entry: &GuestbookEntry,
    diagnostics: &mpsc::Sender<PersistenceDiagnostic>,
) -> Result<(), PersistenceError> {
    let Some(path) = &paths.guestbook else {
        return Ok(());
    };
    append_guestbook(path, entry).await.inspect_err(|error| {
        let _ = diagnostics.try_send(diagnostic_from(error));
    })
}

async fn wait_for_debounce(debounce: &mut Option<Pin<Box<Sleep>>>) {
    debounce.as_mut().expect("debounce branch is guarded").await;
}

async fn publish_dirty_state(
    paths: &WorkerPaths,
    dirty_state: &mut Option<PersistedStateV1>,
    diagnostics: &mpsc::Sender<PersistenceDiagnostic>,
) -> Result<(), PersistenceError> {
    let Some(state) = dirty_state.take() else {
        return Ok(());
    };
    let Some(path) = &paths.state else {
        return Ok(());
    };
    publish_state(path, &state).await.inspect_err(|error| {
        let _ = diagnostics.try_send(diagnostic_from(error));
    })
}

fn diagnostic_from(error: &PersistenceError) -> PersistenceDiagnostic {
    PersistenceDiagnostic {
        operation: error.operation,
        path: error.path.clone(),
        line: error.line,
        source_message: error.source_message.clone(),
    }
}

fn channel_error(operation: &'static str) -> PersistenceError {
    PersistenceError {
        operation,
        path: PathBuf::new(),
        line: None,
        source_message: "persistence worker is unavailable".to_owned(),
    }
}

async fn receive_acknowledgement(
    response: oneshot::Receiver<Result<(), PersistenceError>>,
    operation: &'static str,
) -> Result<(), PersistenceError> {
    response.await.map_err(|_| channel_error(operation))?
}
