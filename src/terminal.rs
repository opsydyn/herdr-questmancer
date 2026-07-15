use std::{
    io::{self, Stdout},
    panic,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use crossterm::{
    cursor::{Hide, Show},
    event::{DisableMouseCapture, EnableMouseCapture, EventStream},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use futures_util::StreamExt;
use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::signal::unix::{Signal, SignalKind, signal};

use crate::{
    app::{Model, View},
    config::PersistencePaths,
    domain::Timestamp,
    herdr::environment::HerdrEnvironment,
    interaction::reduce_action,
    persistence::{
        PersistenceClient, PersistenceDiagnostic, PersistenceError, PersistenceWorker, load_startup,
    },
    runtime::RuntimeRegistration,
    runtime_loop::{
        RuntimeConnection, RuntimeEvent, RuntimeExit, apply_command_result,
        apply_connection_update, bootstrap_model, dispatch_action_effects,
        dispatch_persistence_effects,
    },
    ui::{self, theatre::next_visible_frame_in},
};

type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Maps one startup wall-clock sample onto Tokio's monotonic clock.
///
/// Domain timestamps remain epoch-shaped for protocol and persistence
/// boundaries, while runtime ordering and animation cannot jump with later wall
/// clock adjustments.
#[derive(Clone, Debug)]
pub struct RuntimeClock {
    epoch: Timestamp,
    origin: tokio::time::Instant,
}

impl RuntimeClock {
    pub fn new(epoch: Timestamp) -> Self {
        Self {
            epoch,
            origin: tokio::time::Instant::now(),
        }
    }

    pub fn now(&self) -> Timestamp {
        timestamp_after(self.epoch, self.origin.elapsed())
    }

    fn deadline_after(&self, sampled_at: Timestamp, delay: Duration) -> tokio::time::Instant {
        let target = timestamp_after(sampled_at, delay);
        let offset = target.as_millis().saturating_sub(self.epoch.as_millis());
        self.origin + Duration::from_millis(offset.max(0).cast_unsigned())
    }
}

fn timestamp_after(timestamp: Timestamp, duration: Duration) -> Timestamp {
    let milliseconds = i64::try_from(duration.as_millis()).unwrap_or(i64::MAX);
    Timestamp::from_millis(timestamp.as_millis().saturating_add(milliseconds))
}

/// Owns the single resettable timer used to invalidate animated cafe frames.
///
/// Event-driven rendering deliberately stores no timer, so an unchanged desk
/// or no-motion cafe cannot wake because time passed. `wait` is cancellation
/// safe: cancelling it from `tokio::select!` does not consume an armed sleep.
#[derive(Debug, Default)]
pub struct AnimationScheduler {
    sleep: Option<std::pin::Pin<Box<tokio::time::Sleep>>>,
}

impl AnimationScheduler {
    pub const fn new() -> Self {
        Self { sleep: None }
    }

    pub fn reset_for(&mut self, model: &Model, clock: &RuntimeClock) {
        let Some(period) = next_visible_frame_in(model) else {
            self.sleep = None;
            return;
        };
        let deadline = clock.deadline_after(model.now(), period);
        if let Some(sleep) = &mut self.sleep {
            sleep.as_mut().reset(deadline);
        } else {
            self.sleep = Some(Box::pin(tokio::time::sleep_until(deadline)));
        }
    }

    pub async fn wait(&mut self) {
        match &mut self.sleep {
            Some(sleep) => sleep.as_mut().await,
            None => std::future::pending().await,
        }
    }
}

#[derive(Debug)]
struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> Result<(Self, Tui)> {
        enable_raw_mode()?;
        let guard = Self;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture, Hide)?;

        let terminal = Terminal::new(CrosstermBackend::new(stdout))?;
        Ok((guard, terminal))
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        restore();
    }
}

pub fn install_panic_hook() {
    let previous = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        restore();
        previous(info);
    }));
}

pub async fn run(initial_view: Option<View>) -> Result<()> {
    let startup = load_startup(PersistencePaths::from_env(), initial_view).await;
    let effective_view = startup.model.view();
    let _runtime = RuntimeRegistration::from_env(effective_view)?;
    let mut shutdown = Shutdown::install()?;
    let (mut persistence, mut persistence_diagnostics, persistence_worker) =
        PersistenceWorker::start(startup.paths);
    let mut collected_diagnostics = startup.diagnostics;
    let (guard, mut terminal) = match TerminalGuard::enter() {
        Ok(terminal) => terminal,
        Err(error) => {
            let persistence_result = shutdown_persistence(&persistence, persistence_worker).await;
            drain_diagnostics(&mut persistence_diagnostics, &mut collected_diagnostics);
            emit_diagnostics(&collected_diagnostics);
            let terminal_result: Result<()> = Err(error);
            return terminal_result.and(persistence_result);
        }
    };
    let environment = HerdrEnvironment::from_env().ok();
    let mut model = bootstrap_model(startup.model, environment.as_ref());
    if let Some(diagnostic) = collected_diagnostics.last() {
        model.set_status_message(Some(diagnostic.to_string()));
    }
    let clock = RuntimeClock::new(sample_wall_time());
    model.set_now(clock.now());

    let (loop_result, runtime_shutdown_result) = if let Some(environment) = environment.as_ref() {
        let mut connection = RuntimeConnection::start(environment);
        let loop_result = run_live_loop(
            &mut terminal,
            &mut model,
            &mut connection,
            &mut persistence,
            &mut persistence_diagnostics,
            &mut collected_diagnostics,
            &mut shutdown,
            &clock,
        )
        .await;
        let runtime_shutdown_result = connection
            .shutdown()
            .await
            .context("shut down terminal runtime tasks");
        (loop_result, runtime_shutdown_result)
    } else {
        let loop_result = run_offline_loop(
            &mut terminal,
            &mut model,
            &mut persistence,
            &mut persistence_diagnostics,
            &mut collected_diagnostics,
            &mut shutdown,
            &clock,
        )
        .await;
        (loop_result, Ok(()))
    };

    let persistence_shutdown_result = if let Ok(exit) = &loop_result {
        complete_runtime_exit(*exit, &persistence, persistence_worker)
            .await
            .map(drop)
    } else {
        shutdown_persistence(&persistence, persistence_worker).await
    };
    drain_diagnostics(&mut persistence_diagnostics, &mut collected_diagnostics);
    drop(guard);
    emit_diagnostics(&collected_diagnostics);

    loop_result
        .map(drop)
        .and(runtime_shutdown_result)
        .and(persistence_shutdown_result)
}

async fn run_live_loop(
    terminal: &mut Tui,
    model: &mut Model,
    connection: &mut RuntimeConnection,
    persistence: &mut PersistenceClient,
    persistence_diagnostics: &mut tokio::sync::mpsc::Receiver<PersistenceDiagnostic>,
    collected_diagnostics: &mut Vec<PersistenceDiagnostic>,
    shutdown: &mut Shutdown,
    clock: &RuntimeClock,
) -> Result<RuntimeExit> {
    let mut input = EventStream::new();
    let mut render_invalidation = AnimationScheduler::new();

    loop {
        terminal.draw(|frame| ui::render(frame, model))?;
        render_invalidation.reset_for(model, clock);

        tokio::select! {
            event = input.next() => {
                let Some(event) = event else {
                    return Ok(RuntimeExit::InputClosed);
                };
                let event = event.context("read terminal input")?;
                model.set_now(clock.now());
                let reduction = reduce_action(
                    model,
                    ui::input::action_for_event_in(&event, model.modal()),
                );
                let effects = dispatch_action_effects(persistence, model, reduction).await;
                connection.schedule(effects.desk);
                record_dispatch_errors(
                    model,
                    collected_diagnostics,
                    effects.persistence_errors,
                );
                if let Some(exit) = effects.exit {
                    return Ok(exit);
                }
            }
            runtime_event = connection.next_event() => {
                let observed_at = clock.now();
                model.set_now(observed_at);
                match runtime_event {
                    RuntimeEvent::Connection(update) => {
                        let effects = apply_connection_update(model, update, observed_at);
                        connection.schedule(effects.desk);
                        record_dispatch_errors(
                            model,
                            collected_diagnostics,
                            dispatch_persistence_effects(persistence, model, effects.persistence).await,
                        );
                    }
                    RuntimeEvent::Command(result) => {
                        let effects = apply_command_result(model, result, observed_at);
                        connection.schedule(effects.desk);
                        record_dispatch_errors(
                            model,
                            collected_diagnostics,
                            dispatch_persistence_effects(persistence, model, effects.persistence).await,
                        );
                    }
                    RuntimeEvent::CommandTaskFailed(message) => {
                        anyhow::bail!("terminal command task failed: {message}");
                    }
                }
            }
            exit = shutdown.requested() => return Ok(exit),
            Some(diagnostic) = persistence_diagnostics.recv() => {
                record_diagnostic(model, collected_diagnostics, diagnostic);
            }
            () = render_invalidation.wait() => {
                model.set_now(clock.now());
            }
        }
    }
}

async fn run_offline_loop(
    terminal: &mut Tui,
    model: &mut Model,
    persistence: &mut PersistenceClient,
    persistence_diagnostics: &mut tokio::sync::mpsc::Receiver<PersistenceDiagnostic>,
    collected_diagnostics: &mut Vec<PersistenceDiagnostic>,
    shutdown: &mut Shutdown,
    clock: &RuntimeClock,
) -> Result<RuntimeExit> {
    let mut input = EventStream::new();
    let mut render_invalidation = AnimationScheduler::new();

    loop {
        terminal.draw(|frame| ui::render(frame, model))?;
        render_invalidation.reset_for(model, clock);

        tokio::select! {
            event = input.next() => {
                let Some(event) = event else {
                    return Ok(RuntimeExit::InputClosed);
                };
                let event = event.context("read terminal input")?;
                model.set_now(clock.now());
                let reduction = reduce_action(
                    model,
                    ui::input::action_for_event_in(&event, model.modal()),
                );
                let effects = dispatch_action_effects(persistence, model, reduction).await;
                if !effects.desk.is_empty() {
                    model.set_status_message(Some(
                        "offline: action unavailable until connected to Herdr".to_owned(),
                    ));
                }
                record_dispatch_errors(
                    model,
                    collected_diagnostics,
                    effects.persistence_errors,
                );
                if let Some(exit) = effects.exit {
                    return Ok(exit);
                }
            }
            exit = shutdown.requested() => return Ok(exit),
            Some(diagnostic) = persistence_diagnostics.recv() => {
                record_diagnostic(model, collected_diagnostics, diagnostic);
            }
            () = render_invalidation.wait() => {
                model.set_now(clock.now());
            }
        }
    }
}

pub async fn complete_runtime_exit(
    exit: RuntimeExit,
    persistence: &PersistenceClient,
    worker: tokio::task::JoinHandle<()>,
) -> Result<RuntimeExit> {
    shutdown_persistence(persistence, worker).await?;
    Ok(exit)
}

pub async fn shutdown_persistence(
    persistence: &PersistenceClient,
    mut worker: tokio::task::JoinHandle<()>,
) -> Result<()> {
    let shutdown = tokio::time::timeout(Duration::from_secs(1), async {
        let persistence_result = persistence.shutdown().await;
        let worker_result = (&mut worker).await;
        persistence_result.context("shut down persistence writer")?;
        worker_result.context("join persistence writer")?;
        Ok(())
    })
    .await;

    if let Ok(result) = shutdown {
        result
    } else {
        worker.abort();
        let _ = worker.await;
        anyhow::bail!("persistence writer did not stop within one second")
    }
}

fn record_dispatch_errors(
    model: &mut Model,
    diagnostics: &mut Vec<PersistenceDiagnostic>,
    errors: Vec<PersistenceError>,
) {
    for error in errors {
        record_diagnostic(
            model,
            diagnostics,
            PersistenceDiagnostic {
                operation: error.operation,
                path: error.path,
                line: error.line,
                source_message: error.source_message,
            },
        );
    }
}

fn record_diagnostic(
    model: &mut Model,
    diagnostics: &mut Vec<PersistenceDiagnostic>,
    diagnostic: PersistenceDiagnostic,
) {
    model.set_status_message(Some(diagnostic.to_string()));
    if !diagnostics.contains(&diagnostic) {
        diagnostics.push(diagnostic);
    }
}

fn drain_diagnostics(
    receiver: &mut tokio::sync::mpsc::Receiver<PersistenceDiagnostic>,
    diagnostics: &mut Vec<PersistenceDiagnostic>,
) {
    while let Ok(diagnostic) = receiver.try_recv() {
        if !diagnostics.contains(&diagnostic) {
            diagnostics.push(diagnostic);
        }
    }
}

fn emit_diagnostics(diagnostics: &[PersistenceDiagnostic]) {
    for diagnostic in diagnostics {
        eprintln!("webmaster persistence: {diagnostic}");
    }
}

fn sample_wall_time() -> Timestamp {
    let milliseconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0_u128, |duration| duration.as_millis());
    Timestamp::from_millis(i64::try_from(milliseconds).unwrap_or(i64::MAX))
}

#[derive(Debug)]
struct Shutdown {
    interrupt: Signal,
    terminate: Signal,
    hangup: Signal,
}

impl Shutdown {
    fn install() -> Result<Self> {
        Ok(Self {
            interrupt: signal(SignalKind::interrupt())?,
            terminate: signal(SignalKind::terminate())?,
            hangup: signal(SignalKind::hangup())?,
        })
    }

    async fn requested(&mut self) -> RuntimeExit {
        tokio::select! {
            _ = self.interrupt.recv() => {}
            _ = self.terminate.recv() => {}
            _ = self.hangup.recv() => {}
        }
        RuntimeExit::Signal
    }
}

fn restore() {
    let _ = disable_raw_mode();
    let _ = execute!(
        io::stdout(),
        Show,
        DisableMouseCapture,
        LeaveAlternateScreen
    );
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{
        app::View,
        domain::{EventId, GuestbookEntry, GuestbookEvent},
        persistence::{
            PersistedStateV1, PersistenceWorker, WorkerPaths, load_guestbook, load_state,
        },
    };

    use super::*;

    #[tokio::test]
    async fn shutdown_observes_a_requested_signal() {
        let mut shutdown = Shutdown::install().unwrap();
        signal_hook::low_level::raise(signal_hook::consts::signal::SIGHUP).unwrap();

        let exit = tokio::time::timeout(Duration::from_secs(1), shutdown.requested())
            .await
            .expect("shutdown signal was not observed");
        assert_eq!(exit, RuntimeExit::Signal);
    }

    #[tokio::test(start_paused = true)]
    async fn signal_shutdown_flushes_state_after_an_acknowledged_append() {
        let directory = tempfile::tempdir().unwrap();
        let state_path = directory.path().join("state.json");
        let guestbook_path = directory.path().join("guestbook.jsonl");
        let (mut persistence, _diagnostics, worker) = PersistenceWorker::start(WorkerPaths::new(
            Some(state_path.clone()),
            Some(guestbook_path.clone()),
        ));
        let model = Model::new(View::Cafe);
        let state = PersistedStateV1::capture(&model);
        let entry = GuestbookEntry {
            id: EventId::new("signal-shutdown"),
            occurred_at: Timestamp::from_millis(1_000),
            agent: None,
            workspace: None,
            pane: None,
            pane_revision: 1,
            kind: GuestbookEvent::WorkCompleted,
            summary: "completed before signal".to_owned(),
        };
        persistence.append_guestbook(entry.clone()).await.unwrap();
        persistence.stage_state(state.clone()).unwrap();

        let mut shutdown = Shutdown::install().unwrap();
        signal_hook::low_level::raise(signal_hook::consts::signal::SIGHUP).unwrap();
        let exit = shutdown.requested().await;
        assert_eq!(exit, RuntimeExit::Signal);
        assert_eq!(
            complete_runtime_exit(exit, &persistence, worker)
                .await
                .unwrap(),
            RuntimeExit::Signal
        );

        assert_eq!(load_state(&state_path).await.unwrap(), Some(state));
        let replay = load_guestbook(&guestbook_path, 10).await;
        assert_eq!(
            replay.guestbook.entries().iter().collect::<Vec<_>>(),
            vec![&entry]
        );
    }
}
