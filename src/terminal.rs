use std::{
    io::{self, Stdout},
    panic,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
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
use signal_hook::{
    consts::signal::{SIGHUP, SIGINT, SIGTERM},
    flag,
};

use crate::{
    app::{Model, View},
    domain::Timestamp,
    herdr::environment::HerdrEnvironment,
    interaction::reduce_action,
    runtime_loop::{
        RuntimeConnection, RuntimeEvent, apply_command_result, apply_connection_update,
        bootstrap_model,
    },
    ui,
};

type Tui = Terminal<CrosstermBackend<Stdout>>;

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
    let mut render_invalidation = tokio::time::interval(Duration::from_secs(1));
    render_invalidation.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        terminal.draw(|frame| ui::render(frame, model))?;

        tokio::select! {
            event = input.next() => {
                let Some(event) = event else {
                    break;
                };
                let event = event.context("read terminal input")?;
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
                match runtime_event {
                    RuntimeEvent::Connection(update) => {
                        let commands = apply_connection_update(model, update, now());
                        connection.schedule(commands);
                    }
                    RuntimeEvent::Command(result) => {
                        apply_command_result(model, result, now());
                    }
                    RuntimeEvent::CommandTaskFailed(message) => {
                        anyhow::bail!("terminal command task failed: {message}");
                    }
                }
            }
            () = shutdown.requested() => break,
            _ = render_invalidation.tick() => {
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
    let mut render_invalidation = tokio::time::interval(Duration::from_secs(1));
    render_invalidation.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        terminal.draw(|frame| ui::render(frame, model))?;

        tokio::select! {
            event = input.next() => {
                let Some(event) = event else {
                    break;
                };
                let event = event.context("read terminal input")?;
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
            _ = render_invalidation.tick() => {
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
    requested: Arc<AtomicBool>,
    poll: tokio::time::Interval,
}

impl Shutdown {
    fn install() -> Result<Self> {
        let requested = Arc::new(AtomicBool::new(false));
        for signal in [SIGINT, SIGTERM, SIGHUP] {
            flag::register(signal, Arc::clone(&requested))?;
        }
        Ok(Self {
            requested,
            poll: tokio::time::interval(Duration::from_millis(50)),
        })
    }

    fn is_requested(&self) -> bool {
        self.requested.load(Ordering::Relaxed)
    }

    async fn requested(&mut self) {
        while !self.is_requested() {
            self.poll.tick().await;
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
    use super::*;

    #[tokio::test]
    async fn shutdown_observes_a_requested_signal() {
        let requested = Arc::new(AtomicBool::new(false));
        let shutdown = Shutdown {
            requested: Arc::clone(&requested),
            poll: tokio::time::interval(Duration::from_millis(50)),
        };

        assert!(!shutdown.is_requested());
        requested.store(true, Ordering::Relaxed);
        assert!(shutdown.is_requested());
    }
}
