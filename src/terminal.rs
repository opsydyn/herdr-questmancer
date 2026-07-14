use std::{
    io::{self, Stdout},
    panic,
    time::Duration,
};

use anyhow::Result;
use crossterm::{
    cursor::{Hide, Show},
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

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
        let mut stdout = io::stdout();
        if let Err(error) = execute!(stdout, EnterAlternateScreen, EnableMouseCapture, Hide) {
            let _ = disable_raw_mode();
            return Err(error.into());
        }

        let terminal = Terminal::new(CrosstermBackend::new(stdout))?;
        Ok((Self, terminal))
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
    let (_guard, mut terminal) = TerminalGuard::enter()?;
    let mut model = Model::new(initial_view);
    let mut needs_render = true;

    loop {
        if needs_render {
            terminal.draw(|frame| ui::render(frame, &model))?;
            needs_render = false;
        }

        if !event::poll(Duration::from_millis(250))? {
            continue;
        }

        let Event::Key(key) = event::read()? else {
            continue;
        };
        if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
            continue;
        }

        match ui::input::action_for(key) {
            Action::Switch(view) => {
                model.switch_to(view);
                needs_render = true;
            }
            Action::Quit => break,
            Action::ShowHelp | Action::Dismiss | Action::None => {}
        }
    }

    Ok(())
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
