use std::{
    io::{self, Stdout},
    panic,
    time::{SystemTime, UNIX_EPOCH},
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
    domain::Timestamp,
    herdr::environment::HerdrEnvironment,
    interaction::reduce_action,
    runtime_loop::{
        RuntimeConnection, RuntimeEvent, apply_command_result, apply_connection_update,
        bootstrap_model,
    },
    ui::{
        self,
        theatre::{RenderCadence, cadence_for},
    },
};

type Tui = Terminal<CrosstermBackend<Stdout>>;

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

    pub fn reset(&mut self, cadence: RenderCadence) {
        let Some(period) = cadence.frame_period() else {
            self.sleep = None;
            return;
        };
        let deadline = tokio::time::Instant::now() + period;
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

pub async fn run(initial_view: View) -> Result<()> {
    let mut shutdown = Shutdown::install()?;
    let (_guard, mut terminal) = TerminalGuard::enter()?;
    let environment = HerdrEnvironment::from_env().ok();
    let mut model = bootstrap_model(initial_view, environment.as_ref());
    model.set_now(now());

    if let Some(environment) = environment.as_ref() {
        let mut connection = RuntimeConnection::start(environment);
        let result = run_live_loop(&mut terminal, &mut model, &mut connection, &mut shutdown).await;
        let shutdown_result = connection
            .shutdown()
            .await
            .context("shut down terminal runtime tasks");
        result.and(shutdown_result)
    } else {
        run_offline_loop(&mut terminal, &mut model, &mut shutdown).await
    }
}

async fn run_live_loop(
    terminal: &mut Tui,
    model: &mut Model,
    connection: &mut RuntimeConnection,
    shutdown: &mut Shutdown,
) -> Result<()> {
    let mut input = EventStream::new();
    let mut render_invalidation = AnimationScheduler::new();

    loop {
        terminal.draw(|frame| ui::render(frame, model))?;
        render_invalidation.reset(cadence_for(model));

        tokio::select! {
            event = input.next() => {
                let Some(event) = event else {
                    break;
                };
                let event = event.context("read terminal input")?;
                model.set_now(now());
                let reduction = reduce_action(
                    model,
                    ui::input::action_for_event_in(&event, model.modal()),
                );
                connection.schedule(reduction.commands);
                if reduction.control.is_break() {
                    break;
                }
            }
            runtime_event = connection.next_event() => {
                let observed_at = now();
                model.set_now(observed_at);
                match runtime_event {
                    RuntimeEvent::Connection(update) => {
                        let commands = apply_connection_update(model, update, observed_at);
                        connection.schedule(commands);
                    }
                    RuntimeEvent::Command(result) => {
                        apply_command_result(model, result, observed_at);
                    }
                    RuntimeEvent::CommandTaskFailed(message) => {
                        anyhow::bail!("terminal command task failed: {message}");
                    }
                }
            }
            () = shutdown.requested() => break,
            () = render_invalidation.wait() => {
                model.set_now(now());
            }
        }
    }

    Ok(())
}

async fn run_offline_loop(
    terminal: &mut Tui,
    model: &mut Model,
    shutdown: &mut Shutdown,
) -> Result<()> {
    let mut input = EventStream::new();
    let mut render_invalidation = AnimationScheduler::new();

    loop {
        terminal.draw(|frame| ui::render(frame, model))?;
        render_invalidation.reset(cadence_for(model));

        tokio::select! {
            event = input.next() => {
                let Some(event) = event else {
                    break;
                };
                let event = event.context("read terminal input")?;
                model.set_now(now());
                let reduction = reduce_action(
                    model,
                    ui::input::action_for_event_in(&event, model.modal()),
                );
                if !reduction.commands.is_empty() {
                    model.set_status_message(Some(
                        "offline: action unavailable until connected to Herdr".to_owned(),
                    ));
                }
                if reduction.control.is_break() {
                    break;
                }
            }
            () = shutdown.requested() => break,
            () = render_invalidation.wait() => {
                model.set_now(now());
            }
        }
    }

    Ok(())
}

fn now() -> Timestamp {
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

    async fn requested(&mut self) {
        tokio::select! {
            _ = self.interrupt.recv() => {}
            _ = self.terminate.recv() => {}
            _ = self.hangup.recv() => {}
        }
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

    use super::*;

    #[tokio::test]
    async fn shutdown_observes_a_requested_signal() {
        let mut shutdown = Shutdown::install().unwrap();
        signal_hook::low_level::raise(signal_hook::consts::signal::SIGHUP).unwrap();

        tokio::time::timeout(Duration::from_secs(1), shutdown.requested())
            .await
            .expect("shutdown signal was not observed");
    }
}
