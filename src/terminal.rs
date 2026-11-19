use std::{
    io::{self, Stdout, Write},
    panic,
    sync::{Arc, Mutex, OnceLock, Weak, atomic::AtomicBool},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use crossterm::{
    cursor::{Hide, Show},
    event::{
        DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
        EventStream,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use futures_util::StreamExt;
use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::signal::unix::{Signal, SignalKind, signal};

use crate::{
    app::{Model, View},
    config::PersistencePaths,
    domain::{PaneId, Timestamp},
    herdr::environment::HerdrEnvironment,
    persistence::{
        DiagnosticReceiver, PersistenceClient, PersistenceDiagnostic, PersistenceError,
        PersistenceWorker, load_startup,
    },
    portrait::PortraitGallery,
    runtime::RuntimeRegistration,
    runtime_loop::{
        RuntimeConnection, RuntimeEvent, RuntimeExit, apply_command_result,
        apply_connection_update, bootstrap_model, dispatch_action_effects,
        dispatch_persistence_effects,
    },
    scene::pixel::{Rgb, RgbBuffer},
    ui::world_fit::{WorldFit, delete_image, render_application},
};

pub(crate) type Tui = Terminal<CrosstermBackend<Stdout>>;

#[derive(Debug)]
pub struct RuntimeLifecycle {
    connection: Option<RuntimeConnection>,
    persistence: PersistenceClient,
    persistence_worker: tokio::task::JoinHandle<()>,
    owned: Option<crate::owned_work::OwnedRuntime>,
    owned_paths: crate::persistence::WorkerPaths,
    projects: Option<crate::sources::herdr_projects::Producer>,
    projects_inbox: Option<crate::work::source::SourceInbox>,
}

impl RuntimeLifecycle {
    pub fn start(
        environment: Option<&HerdrEnvironment>,
        paths: crate::persistence::WorkerPaths,
    ) -> (Self, DiagnosticReceiver) {
        let (persistence, diagnostics, persistence_worker) =
            PersistenceWorker::start(paths.clone());
        (
            Self {
                connection: environment.map(RuntimeConnection::start),
                persistence,
                persistence_worker,
                owned: None,
                owned_paths: paths,
                projects: None,
                projects_inbox: None,
            },
            diagnostics,
        )
    }

    pub fn connection_mut(&mut self) -> Option<&mut RuntimeConnection> {
        self.connection.as_mut()
    }

    pub const fn persistence_mut(&mut self) -> &mut PersistenceClient {
        &mut self.persistence
    }

    pub fn start_owned(&mut self, model: &mut Model) -> Result<(), String> {
        if self.owned.is_none() {
            self.owned = Some(crate::owned_work::OwnedRuntime::start(
                &self.owned_paths,
                model,
            )?);
        }
        self.dispatch_owned(model);
        Ok(())
    }
    pub fn dispatch_owned(&mut self, model: &mut Model) {
        if let Some(owned) = &mut self.owned {
            owned.dispatch(model);
        }
    }
    pub async fn consume_owned(&mut self, model: &mut Model) {
        if let Some(owned) = &mut self.owned {
            owned.consume_next(&mut self.persistence, model).await;
        }
    }
    pub async fn finish_owned(&mut self, model: &mut Model) -> Result<(), String> {
        if let Some(mut owned) = self.owned.take() {
            owned.shutdown(&mut self.persistence, model).await?;
        }
        Ok(())
    }

    pub fn start_projects(
        &mut self,
        selection: Option<crate::sources::herdr_projects::RuntimeSelection>,
        cache: &crate::persistence::WorkCache,
        model: &mut Model,
    ) {
        let Some(selection) = selection else {
            return;
        };
        if self.projects.is_some() {
            return;
        }
        let source = selection.source().clone();
        match crate::sources::herdr_projects::Producer::start(
            selection,
            model.work(),
            cache.herdr_projects(&source).cloned(),
        ) {
            Ok((producer, inbox)) => {
                self.projects = Some(producer);
                self.projects_inbox = Some(inbox);
            }
            Err(error) => crate::sources::herdr_projects::source_failure(
                model,
                &source,
                &error.to_string(),
                true,
            ),
        }
    }
    pub fn projects_inbox_mut(&mut self) -> &mut Option<crate::work::source::SourceInbox> {
        &mut self.projects_inbox
    }

    fn live_parts_mut(
        &mut self,
    ) -> Option<(
        &mut RuntimeConnection,
        &mut PersistenceClient,
        &mut Option<crate::work::source::SourceInbox>,
        &mut Option<crate::owned_work::OwnedRuntime>,
    )> {
        self.connection.as_mut().map(|connection| {
            (
                connection,
                &mut self.persistence,
                &mut self.projects_inbox,
                &mut self.owned,
            )
        })
    }

    pub async fn complete(self, exit: Option<RuntimeExit>) -> Result<Option<RuntimeExit>> {
        let Self {
            connection,
            persistence,
            persistence_worker,
            projects,
            projects_inbox,
            owned,
            owned_paths: _,
        } = self;
        drop(owned);
        drop(projects_inbox);
        let projects_result = if let Some(producer) = projects {
            producer
                .shutdown()
                .await
                .context("join project source producer")
        } else {
            Ok(())
        };
        let runtime_result = if let Some(connection) = connection {
            connection
                .shutdown()
                .await
                .context("shut down terminal runtime tasks")
        } else {
            Ok(())
        };
        let persistence_result = shutdown_persistence(&persistence, persistence_worker).await;
        projects_result
            .and(runtime_result)
            .and(persistence_result)?;
        Ok(exit)
    }
}

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

/// Owns the single resettable timer used to invalidate visible scene effects.
///
/// Event-driven rendering deliberately stores no timer, so a static Guild Hall
/// or Delve cannot wake merely because time passed. `wait` is cancellation safe:
/// cancelling it from `tokio::select!` does not consume an armed sleep.
#[derive(Debug, Default)]
pub struct AnimationScheduler {
    sleep: Option<std::pin::Pin<Box<tokio::time::Sleep>>>,
}

impl AnimationScheduler {
    pub const fn new() -> Self {
        Self { sleep: None }
    }

    pub fn reset_after(
        &mut self,
        sampled_at: Timestamp,
        delay: Option<Duration>,
        clock: &RuntimeClock,
    ) {
        let Some(period) = delay else {
            self.sleep = None;
            return;
        };
        let deadline = clock.deadline_after(sampled_at, period);
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
enum RestoreAction {
    Crossterm,
    #[cfg(test)]
    Probe(std::sync::Arc<std::sync::atomic::AtomicUsize>),
}

#[derive(Debug)]
struct RestoreGate {
    restored: AtomicBool,
    action: RestoreAction,
}

impl RestoreGate {
    fn new(action: RestoreAction) -> Self {
        Self {
            restored: AtomicBool::new(false),
            action,
        }
    }

    fn restore_once(&self) {
        if self
            .restored
            .compare_exchange(
                false,
                true,
                std::sync::atomic::Ordering::SeqCst,
                std::sync::atomic::Ordering::SeqCst,
            )
            .is_err()
        {
            return;
        }
        match &self.action {
            RestoreAction::Crossterm => restore(),
            #[cfg(test)]
            RestoreAction::Probe(restore_count) => {
                restore_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
        }
    }
}

fn active_restore_gate() -> &'static Mutex<Option<Weak<RestoreGate>>> {
    static ACTIVE: OnceLock<Mutex<Option<Weak<RestoreGate>>>> = OnceLock::new();
    ACTIVE.get_or_init(|| Mutex::new(None))
}

fn register_restore_gate(gate: &Arc<RestoreGate>) {
    *active_restore_gate()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Arc::downgrade(gate));
}

fn clear_restore_gate(gate: &Arc<RestoreGate>) {
    let mut active = active_restore_gate()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if active
        .as_ref()
        .and_then(Weak::upgrade)
        .is_some_and(|candidate| Arc::ptr_eq(&candidate, gate))
    {
        *active = None;
    }
}

fn restore_active_terminal() {
    let gate = active_restore_gate()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
        .and_then(Weak::upgrade);
    if let Some(gate) = gate {
        gate.restore_once();
    }
}

#[derive(Debug)]
pub(crate) struct TerminalGuard {
    restore: Arc<RestoreGate>,
}

impl TerminalGuard {
    pub(crate) fn enter() -> Result<(Self, Tui)> {
        enable_raw_mode()?;
        let restore = Arc::new(RestoreGate::new(RestoreAction::Crossterm));
        register_restore_gate(&restore);
        let guard = Self { restore };
        let mut stdout = io::stdout();
        execute!(
            stdout,
            EnterAlternateScreen,
            EnableMouseCapture,
            EnableBracketedPaste,
            Hide
        )?;

        let terminal = Terminal::new(CrosstermBackend::new(stdout))?;
        Ok((guard, terminal))
    }

    #[cfg(test)]
    fn for_test(restore_count: std::sync::Arc<std::sync::atomic::AtomicUsize>) -> Self {
        Self {
            restore: Arc::new(RestoreGate::new(RestoreAction::Probe(restore_count))),
        }
    }

    #[cfg(test)]
    fn restore_for_panic(&self) {
        self.restore.restore_once();
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        self.restore.restore_once();
        clear_restore_gate(&self.restore);
    }
}

pub fn install_panic_hook() {
    let previous = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        restore_active_terminal();
        previous(info);
    }));
}

pub async fn run(initial_view: Option<View>) -> Result<()> {
    run_application(initial_view).await
}

/// A fresh managed tab can have a character grid before Herdr applies the
/// attached client's pixel geometry. The terminal Picker still decides whether
/// the pane supports native images.
async fn wait_for_managed_pane_pixels() {
    let _ = tokio::time::timeout(Duration::from_millis(500), async {
        while let Ok(size) = rustix::termios::tcgetwinsize(io::stdout()) {
            if size.ws_xpixel != 0 && size.ws_ypixel != 0 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await;
}

#[allow(
    clippy::too_many_lines,
    reason = "the application lifecycle keeps startup, runtime, persistence and terminal cleanup together"
)]
async fn run_application(initial_view: Option<View>) -> Result<()> {
    let startup = load_startup(PersistencePaths::from_env(), initial_view).await;
    let effective_view = startup.model.view();
    let profile_writer = startup.profile_writer;
    let _runtime = if profile_writer.is_some() {
        RuntimeRegistration::from_env(effective_view)?
    } else {
        None
    };
    let mut shutdown = Shutdown::install()?;
    let environment = HerdrEnvironment::from_env().ok();
    let (mut lifecycle, mut persistence_diagnostics) =
        RuntimeLifecycle::start(environment.as_ref(), startup.paths);
    let mut collected_diagnostics = startup.diagnostics;
    let (guard, mut terminal) = match TerminalGuard::enter() {
        Ok(terminal) => terminal,
        Err(error) => {
            let lifecycle_result = lifecycle.complete(None).await.map(drop);
            drain_diagnostics(&mut persistence_diagnostics, &mut collected_diagnostics);
            emit_diagnostics(&collected_diagnostics);
            let terminal_result: Result<()> = Err(error);
            return terminal_result.and(lifecycle_result);
        }
    };
    let managed_pane_id = std::env::var("HERDR_PANE_ID")
        .ok()
        .filter(|value| !value.is_empty())
        .map(PaneId::new);
    if managed_pane_id.is_some()
        && std::env::var("QUESTMANCER_WAIT_FOR_PANE_PIXELS").as_deref() == Ok("1")
    {
        wait_for_managed_pane_pixels().await;
    }
    let picker = ratatui_image::picker::Picker::from_query_stdio();
    let portraits = match &picker {
        Ok(picker) => PortraitGallery::from_picker(picker),
        Err(error) => {
            PortraitGallery::fallback(format!("portrait capability detection failed: {error}"))
        }
    };
    let mut world_fit =
        WorldFit::from_picker(picker.as_ref().ok(), std::env::var_os("TMUX").is_some());
    let mut model = bootstrap_model(startup.model, environment.as_ref());
    model.set_managed_pane_id(managed_pane_id);
    if let Some(diagnostic) = collected_diagnostics.last() {
        model.set_persistence_diagnostic(diagnostic.to_string());
    }
    let clock = RuntimeClock::new(sample_wall_time());
    model.set_now(clock.now());
    lifecycle.start_projects(startup.projects, &startup.work_cache, &mut model);
    lifecycle
        .start_owned(&mut model)
        .map_err(anyhow::Error::msg)?;

    let loop_result = if environment.is_some() {
        let (connection, persistence, projects_inbox, owned) = lifecycle
            .live_parts_mut()
            .expect("live lifecycle starts a runtime connection");
        run_live_loop(
            RuntimeLoopContext {
                terminal: &mut terminal,
                shutdown: &mut shutdown,
                clock: &clock,
                portraits: &portraits,
                world_fit: &mut world_fit,
                projects_inbox,
                owned,
            },
            &mut model,
            connection,
            persistence,
            &mut persistence_diagnostics,
            &mut collected_diagnostics,
        )
        .await
    } else {
        run_offline_loop(
            RuntimeLoopContext {
                terminal: &mut terminal,
                shutdown: &mut shutdown,
                clock: &clock,
                portraits: &portraits,
                world_fit: &mut world_fit,
                projects_inbox: &mut lifecycle.projects_inbox,
                owned: &mut lifecycle.owned,
            },
            &mut model,
            &mut lifecycle.persistence,
            &mut persistence_diagnostics,
            &mut collected_diagnostics,
        )
        .await
    };

    let graphics_result = cleanup_world_images(&world_fit.shutdown());
    let owned_result = lifecycle
        .finish_owned(&mut model)
        .await
        .map_err(anyhow::Error::msg);
    let lifecycle_result = lifecycle
        .complete(loop_result.as_ref().ok().copied())
        .await
        .map(drop);
    drain_diagnostics(&mut persistence_diagnostics, &mut collected_diagnostics);
    drop(guard);
    emit_diagnostics(&collected_diagnostics);

    loop_result
        .map(drop)
        .and(graphics_result)
        .and(owned_result)
        .and(lifecycle_result)
}

struct RuntimeLoopContext<'a> {
    terminal: &'a mut Tui,
    shutdown: &'a mut Shutdown,
    clock: &'a RuntimeClock,
    portraits: &'a PortraitGallery,
    world_fit: &'a mut WorldFit,
    projects_inbox: &'a mut Option<crate::work::source::SourceInbox>,
    owned: &'a mut Option<crate::owned_work::OwnedRuntime>,
}

fn cleanup_world_images(ids: &[u32]) -> Result<()> {
    if ids.is_empty() {
        return Ok(());
    }
    let mut stdout = io::stdout().lock();
    for id in ids {
        stdout.write_all(delete_image(*id).as_bytes())?;
    }
    stdout.flush()?;
    Ok(())
}

fn draw_scene_application(
    terminal: &mut Tui,
    model: &Model,
    buffer: &mut RgbBuffer,
    portraits: &PortraitGallery,
    world_fit: &mut WorldFit,
) -> Result<crate::scene::SceneFrame> {
    world_fit.refresh_geometry();
    let mut rendered = None;
    terminal.draw(|frame| {
        rendered = Some(render_application(
            frame,
            model,
            buffer,
            Some(portraits),
            world_fit,
        ));
    })?;
    cleanup_world_images(&world_fit.finish_frame())?;
    rendered.context("scene application draw did not produce a frame")
}

#[allow(clippy::too_many_lines)] // One select loop keeps all existing runtime effects ordered.
async fn run_live_loop(
    context: RuntimeLoopContext<'_>,
    model: &mut Model,
    connection: &mut RuntimeConnection,
    persistence: &mut PersistenceClient,
    persistence_diagnostics: &mut DiagnosticReceiver,
    collected_diagnostics: &mut Vec<PersistenceDiagnostic>,
) -> Result<RuntimeExit> {
    let mut input = EventStream::new();
    let mut render_invalidation = AnimationScheduler::new();
    let mut buffer = RgbBuffer::filled(0, 0, Rgb::BLACK);
    let mut rendered_frame = None;
    let mut redraw = true;

    loop {
        if redraw {
            let viewport = context.terminal.size()?;
            model.set_work_viewport_size(viewport.width, viewport.height);
            let frame = draw_scene_application(
                context.terminal,
                model,
                &mut buffer,
                context.portraits,
                context.world_fit,
            )?;
            let next_wake = [
                frame.next_frame_in,
                model.work().next_attention_in(model.now()),
            ]
            .into_iter()
            .flatten()
            .min();
            render_invalidation.reset_after(model.now(), next_wake, context.clock);
            rendered_frame = Some(frame);
        }
        let scene_frame = rendered_frame
            .as_ref()
            .expect("initial draw produces a scene frame");
        redraw = true;

        tokio::select! {
            event = input.next() => {
                let Some(event) = event else {
                    return Ok(RuntimeExit::InputClosed);
                };
                let event = event.context("read terminal input")?;
                model.set_now(context.clock.now());
                if crate::owned_work::authoring::handle_event(model,&event) {
                    if let Some(owned)=context.owned { owned.dispatch(model); }
                    continue;
                }
                let action = context.world_fit.action_for_event(&event, model, scene_frame);
                let reduction = crate::interaction::reduce_scene_action(
                    model,
                    action,
                    scene_frame,
                );
                let effects = dispatch_action_effects(persistence, model, reduction).await;
                if let Some(owned)=context.owned {owned.dispatch(model);}
                connection.schedule(effects.agent_commands);
                record_dispatch_errors(
                    model,
                    collected_diagnostics,
                    effects.persistence_errors,
                );
                if let Some(exit) = effects.exit {
                    return Ok(exit);
                }
            }
            () = next_owned(context.owned) => { if let Some(owned)=context.owned {owned.consume_next(persistence,model).await;owned.dispatch(model);} }
            observation = next_projects(context.projects_inbox) => {
                model.set_now(context.clock.now());
                redraw = observation.consume(persistence, model).await;
            }
            runtime_event = connection.next_event() => {
                let observed_at = context.clock.now();
                model.set_now(observed_at);
                match runtime_event {
                    RuntimeEvent::Work(envelope) => {
                        if let Err(message)=crate::work::runtime::accept(persistence,model,envelope).await {
                            model.set_action_feedback(message);
                        }
                    }
                    RuntimeEvent::Connection(update) => {
                        let effects = apply_connection_update(model, update, observed_at);
                        if effects.resubscribe { connection.resubscribe(); }
                        connection.schedule(effects.agent_commands);
                        record_dispatch_errors(
                            model,
                            collected_diagnostics,
                            dispatch_persistence_effects(persistence, model, effects.persistence).await,
                        );
                    }
                    RuntimeEvent::Command(result) => {
                        let effects = apply_command_result(model, result, observed_at);
                        if effects.resubscribe { connection.resubscribe(); }
                        connection.schedule(effects.agent_commands);
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
            exit = context.shutdown.requested() => return Ok(exit),
            Some(diagnostic) = persistence_diagnostics.recv() => {
                record_diagnostic(model, collected_diagnostics, diagnostic);
            }
            () = render_invalidation.wait() => {
                model.set_now(context.clock.now());
            }
        }
    }
}

async fn run_offline_loop(
    context: RuntimeLoopContext<'_>,
    model: &mut Model,
    persistence: &mut PersistenceClient,
    persistence_diagnostics: &mut DiagnosticReceiver,
    collected_diagnostics: &mut Vec<PersistenceDiagnostic>,
) -> Result<RuntimeExit> {
    let mut input = EventStream::new();
    let mut render_invalidation = AnimationScheduler::new();
    let mut buffer = RgbBuffer::filled(0, 0, Rgb::BLACK);
    let mut rendered_frame = None;
    let mut redraw = true;

    loop {
        if redraw {
            let viewport = context.terminal.size()?;
            model.set_work_viewport_size(viewport.width, viewport.height);
            let frame = draw_scene_application(
                context.terminal,
                model,
                &mut buffer,
                context.portraits,
                context.world_fit,
            )?;
            let next_wake = [
                frame.next_frame_in,
                model.work().next_attention_in(model.now()),
            ]
            .into_iter()
            .flatten()
            .min();
            render_invalidation.reset_after(model.now(), next_wake, context.clock);
            rendered_frame = Some(frame);
        }
        let scene_frame = rendered_frame
            .as_ref()
            .expect("initial draw produces a scene frame");
        redraw = true;

        tokio::select! {
            event = input.next() => {
                let Some(event) = event else {
                    return Ok(RuntimeExit::InputClosed);
                };
                let event = event.context("read terminal input")?;
                model.set_now(context.clock.now());
                if crate::owned_work::authoring::handle_event(model,&event) {
                    if let Some(owned)=context.owned { owned.dispatch(model); }
                    continue;
                }
                let action = context.world_fit.action_for_event(&event, model, scene_frame);
                let reduction = crate::interaction::reduce_scene_action(
                    model,
                    action,
                    scene_frame,
                );
                let effects = dispatch_action_effects(persistence, model, reduction).await;
                if let Some(owned)=context.owned {owned.dispatch(model);}
                if !effects.agent_commands.is_empty() {
                    model.set_persistence_diagnostic(
                        "offline: action unavailable until connected to Herdr".to_owned(),
                    );
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
            () = next_owned(context.owned) => { if let Some(owned)=context.owned {owned.consume_next(persistence,model).await;owned.dispatch(model);} }
            observation = next_projects(context.projects_inbox) => {
                model.set_now(context.clock.now());
                redraw = observation.consume(persistence, model).await;
            }
            exit = context.shutdown.requested() => return Ok(exit),
            Some(diagnostic) = persistence_diagnostics.recv() => {
                record_diagnostic(model, collected_diagnostics, diagnostic);
            }
            () = render_invalidation.wait() => {
                model.set_now(context.clock.now());
            }
        }
    }
}

async fn next_owned(owned: &mut Option<crate::owned_work::OwnedRuntime>) {
    if let Some(owned) = owned {
        owned.wait_result().await;
    } else {
        std::future::pending::<()>().await;
    }
}

async fn next_projects(
    inbox: &mut Option<crate::work::source::SourceInbox>,
) -> crate::sources::herdr_projects::Observation {
    match inbox {
        Some(inbox) => inbox.next_projects().await,
        None => std::future::pending().await,
    }
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
    model.set_persistence_diagnostic(diagnostic.to_string());
    if !diagnostics.contains(&diagnostic) {
        diagnostics.push(diagnostic);
    }
}

fn drain_diagnostics(
    receiver: &mut DiagnosticReceiver,
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
        eprintln!("questmancer persistence: {diagnostic}");
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
        DisableBracketedPaste,
        LeaveAlternateScreen
    );
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
        time::Duration,
    };

    use crate::{
        app::View,
        domain::{ChronicleEntry, ChronicleEvent, EventId},
        persistence::{PersistedStateV1, WorkerPaths, load_chronicle, load_state},
    };

    use super::*;

    #[test]
    fn terminal_guard_runs_its_restore_action_exactly_once_on_drop() {
        let restore_count = Arc::new(AtomicUsize::new(0));
        let guard = TerminalGuard::for_test(Arc::clone(&restore_count));
        drop(guard);
        assert_eq!(restore_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn moving_the_terminal_guard_does_not_duplicate_its_restore_action() {
        let restore_count = Arc::new(AtomicUsize::new(0));
        let guard = TerminalGuard::for_test(Arc::clone(&restore_count));
        let moved_guard = guard;
        drop(moved_guard);
        assert_eq!(restore_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn panic_restore_and_guard_drop_share_one_exactly_once_gate() {
        let restore_count = Arc::new(AtomicUsize::new(0));
        let guard = TerminalGuard::for_test(Arc::clone(&restore_count));
        guard.restore_for_panic();
        drop(guard);
        assert_eq!(restore_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn sequential_terminal_sessions_each_restore_once() {
        let restore_count = Arc::new(AtomicUsize::new(0));
        for expected in 1..=2 {
            let guard = TerminalGuard::for_test(Arc::clone(&restore_count));
            drop(guard);
            assert_eq!(restore_count.load(Ordering::SeqCst), expected);
        }
    }

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
        let chronicle_path = directory.path().join("chronicle.jsonl");
        let environment =
            HerdrEnvironment::new(directory.path().join("missing.sock"), "/usr/bin/herdr");
        let (mut lifecycle, _diagnostics) = RuntimeLifecycle::start(
            Some(&environment),
            WorkerPaths::new(Some(state_path.clone()), Some(chronicle_path.clone())),
        );
        lifecycle.connection_mut().unwrap().schedule([
            crate::command::AgentCommand::RefreshSnapshot(
                crate::snapshot_refresh::SnapshotRequest::default(),
            ),
        ]);
        let model = Model::new(View::Delve);
        let state = PersistedStateV1::capture(&model);
        let entry: ChronicleEntry = crate::domain::LegacyChronicleEntry {
            id: EventId::new("signal-shutdown"),
            occurred_at: Timestamp::from_millis(1_000),
            adventurer: None,
            campaign: None,
            pane: None,
            pane_revision: 1,
            event: ChronicleEvent::SpoilsReturned,
            summary: "completed before signal".to_owned(),
        }
        .into();
        lifecycle
            .persistence_mut()
            .append_chronicle(entry.clone())
            .await
            .unwrap();
        lifecycle
            .persistence_mut()
            .stage_state(state.clone())
            .unwrap();

        let mut shutdown = Shutdown::install().unwrap();
        signal_hook::low_level::raise(signal_hook::consts::signal::SIGHUP).unwrap();
        let exit = shutdown.requested().await;
        assert_eq!(exit, RuntimeExit::Signal);
        assert_eq!(
            lifecycle.complete(Some(exit)).await.unwrap(),
            Some(RuntimeExit::Signal)
        );

        assert_eq!(load_state(&state_path).await.unwrap(), Some(state));
        let replay = load_chronicle(&chronicle_path, 10).await;
        assert_eq!(
            replay.chronicle.entries().iter().collect::<Vec<_>>(),
            vec![&entry]
        );
    }
}
