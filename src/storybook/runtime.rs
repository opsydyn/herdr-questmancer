use anyhow::{Context, Result};
use crossterm::event::{Event, EventStream};
use futures_util::StreamExt;

use super::{
    app::{Exit, StorybookApp, reduce},
    catalogue::{Story, catalogue, validate_catalogue},
    fixtures::StoryContext,
    input, ui,
};
use crate::terminal::TerminalGuard;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RuntimeControl {
    Continue,
    Quit,
}

fn control_for_event(
    app: &mut StorybookApp,
    event: Option<std::io::Result<Event>>,
    stories: &[Story],
) -> Result<RuntimeControl> {
    let Some(event) = event else {
        return Ok(RuntimeControl::Quit);
    };
    let action = input::action_for_event(&event.context("read Storybook input")?);
    Ok(if reduce(app, action, stories) == Exit::Quit {
        RuntimeControl::Quit
    } else {
        RuntimeControl::Continue
    })
}

pub async fn run() -> Result<()> {
    validate_catalogue().map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let stories = catalogue();
    let context = StoryContext::fixed();
    let mut app = StorybookApp::new(stories);
    let (_guard, mut terminal) = TerminalGuard::enter()?;
    let mut events = EventStream::new();

    loop {
        terminal.draw(|frame| ui::render(frame, &app, stories, &context))?;
        let control = tokio::select! {
            event = events.next() => control_for_event(&mut app, event, stories),
            result = tokio::signal::ctrl_c() => {
                result.context("install Storybook Ctrl-C handler")?;
                Ok(RuntimeControl::Quit)
            }
        }?;
        if control == RuntimeControl::Quit {
            break;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

    use super::{RuntimeControl, control_for_event};
    use crate::storybook::{app::StorybookApp, catalogue::catalogue};

    #[test]
    fn event_stream_end_requests_a_clean_exit() {
        let stories = catalogue();
        let mut app = StorybookApp::new(stories);

        assert_eq!(
            control_for_event(&mut app, None, stories).unwrap(),
            RuntimeControl::Quit
        );
    }

    #[test]
    fn quit_event_requests_a_clean_exit() {
        let stories = catalogue();
        let mut app = StorybookApp::new(stories);
        let quit = Event::Key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));

        assert_eq!(
            control_for_event(&mut app, Some(Ok(quit)), stories).unwrap(),
            RuntimeControl::Quit
        );
    }
}
