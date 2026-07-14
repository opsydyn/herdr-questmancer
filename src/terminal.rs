use std::{
    io::{self, Stdout},
    panic,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use anyhow::Result;
use crossterm::{
    cursor::{Hide, Show},
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use signal_hook::{
    consts::signal::{SIGHUP, SIGINT, SIGTERM},
    flag,
};

use crate::{
    app::{Model, View},
    ui::{self, input::Action},
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

pub fn run(initial_view: View) -> Result<()> {
    let shutdown = Shutdown::install()?;
    let (_guard, mut terminal) = TerminalGuard::enter()?;
    let mut model = Model::new(initial_view);
    let mut needs_render = true;

    loop {
        if shutdown.is_requested() {
            break;
        }

        if needs_render {
            terminal.draw(|frame| ui::render(frame, &model))?;
            needs_render = false;
        }

        if !event::poll(Duration::from_millis(250))? {
            continue;
        }

        match ui::input::action_for_event(&event::read()?) {
            Action::Switch(view) => {
                model.switch_to(view);
                needs_render = true;
            }
            Action::Redraw => needs_render = true,
            Action::Quit => break,
            Action::ShowHelp | Action::Dismiss | Action::None => {}
        }
    }

    Ok(())
}

#[derive(Debug)]
struct Shutdown {
    requested: Arc<AtomicBool>,
}

impl Shutdown {
    fn install() -> Result<Self> {
        let requested = Arc::new(AtomicBool::new(false));
        for signal in [SIGINT, SIGTERM, SIGHUP] {
            flag::register(signal, Arc::clone(&requested))?;
        }
        Ok(Self { requested })
    }

    fn is_requested(&self) -> bool {
        self.requested.load(Ordering::Relaxed)
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

    #[test]
    fn shutdown_observes_a_requested_signal() {
        let requested = Arc::new(AtomicBool::new(false));
        let shutdown = Shutdown {
            requested: Arc::clone(&requested),
        };

        assert!(!shutdown.is_requested());
        requested.store(true, Ordering::Relaxed);
        assert!(shutdown.is_requested());
    }
}
