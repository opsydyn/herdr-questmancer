use std::{
    pin::Pin,
    task::{Context as TaskContext, Poll},
};

use anyhow::{Context, Result};
use crossterm::event::{Event, EventStream};
use futures_util::{Stream, StreamExt};
use tokio::signal::unix::{Signal, SignalKind, signal};

use super::{
    app::{Exit, StorybookApp, reduce},
    catalogue::{CoverageError, CoverageReport, Story, catalogue, validate_catalogue},
    fixtures::StoryContext,
    input, ui,
};
use crate::terminal::TerminalGuard;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RuntimeControl {
    Continue,
    Quit,
}

#[derive(Debug)]
struct InterruptSignal {
    receiver: Signal,
}

impl InterruptSignal {
    fn install() -> Result<Self> {
        let receiver =
            signal(SignalKind::interrupt()).context("install Storybook Ctrl-C signal listener")?;
        Ok(Self { receiver })
    }
}

impl Stream for InterruptSignal {
    type Item = ();

    fn poll_next(
        mut self: Pin<&mut Self>,
        context: &mut TaskContext<'_>,
    ) -> Poll<Option<Self::Item>> {
        self.receiver.poll_recv(context)
    }
}

fn preflight_with_validation(
    validation: std::result::Result<CoverageReport, CoverageError>,
) -> Result<(&'static [Story], StoryContext)> {
    validation.map_err(|error| anyhow::anyhow!(error.to_string()))?;
    Ok((catalogue(), StoryContext::fixed()))
}

fn preflight() -> Result<(&'static [Story], StoryContext)> {
    preflight_with_validation(validate_catalogue())
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

async fn next_control<Events, Interrupts>(
    events: &mut Events,
    mut interrupts: Pin<&mut Interrupts>,
    app: &mut StorybookApp,
    stories: &[Story],
) -> Result<RuntimeControl>
where
    Events: Stream<Item = std::io::Result<Event>> + Unpin,
    Interrupts: Stream<Item = ()>,
{
    tokio::select! {
        event = events.next() => control_for_event(app, event, stories),
        _ = interrupts.next() => Ok(RuntimeControl::Quit),
    }
}

pub async fn run() -> Result<()> {
    let (stories, context) = preflight()?;
    let interrupts = InterruptSignal::install()?;
    tokio::pin!(interrupts);
    let mut app = StorybookApp::new(stories);
    let (_guard, mut terminal) = TerminalGuard::enter()?;
    let mut events = EventStream::new();

    loop {
        terminal.draw(|frame| ui::render(frame, &app, stories, &context))?;
        let control = next_control(&mut events, interrupts.as_mut(), &mut app, stories).await?;
        if control == RuntimeControl::Quit {
            break;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        pin::Pin,
        sync::{
            Arc,
            atomic::{AtomicBool, AtomicUsize, Ordering},
        },
        task::{Context as TaskContext, Poll},
    };

    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use futures_util::{Stream, stream};

    use super::{
        RuntimeControl, control_for_event, next_control, preflight, preflight_with_validation,
    };
    use crate::storybook::{
        app::StorybookApp,
        catalogue::{catalogue, validate_coverage},
        fixtures::StoryContext,
    };

    #[derive(Debug)]
    struct TrackedInterrupts {
        ready: Arc<AtomicBool>,
        drops: Arc<AtomicUsize>,
    }

    impl Stream for TrackedInterrupts {
        type Item = ();

        fn poll_next(
            self: Pin<&mut Self>,
            _context: &mut TaskContext<'_>,
        ) -> Poll<Option<Self::Item>> {
            if self.ready.load(Ordering::SeqCst) {
                Poll::Ready(Some(()))
            } else {
                Poll::Pending
            }
        }
    }

    impl Drop for TrackedInterrupts {
        fn drop(&mut self) {
            self.drops.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn preflight_rejects_invalid_catalogue_validation() {
        let invalid = validate_coverage(&[], catalogue());
        let error = preflight_with_validation(invalid).unwrap_err();

        assert!(error.to_string().contains("Storybook coverage failed"));
    }

    #[test]
    fn preflight_returns_the_catalogue_and_fixed_context() {
        let (stories, context) = preflight().unwrap();

        assert!(std::ptr::eq(stories, catalogue()));
        assert_eq!(context, StoryContext::fixed());
    }

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

    #[tokio::test]
    async fn event_wins_without_replacing_the_interrupt_listener_then_interrupt_exits() {
        let stories = catalogue();
        let mut app = StorybookApp::new(stories);
        let ready = Arc::new(AtomicBool::new(false));
        let drops = Arc::new(AtomicUsize::new(0));
        {
            let interrupts = TrackedInterrupts {
                ready: Arc::clone(&ready),
                drops: Arc::clone(&drops),
            };
            tokio::pin!(interrupts);
            let mut events = stream::iter([Ok(Event::Resize(120, 40))]);

            assert_eq!(
                next_control(&mut events, interrupts.as_mut(), &mut app, stories)
                    .await
                    .unwrap(),
                RuntimeControl::Continue
            );
            assert_eq!(drops.load(Ordering::SeqCst), 0);

            ready.store(true, Ordering::SeqCst);
            let mut pending_events = stream::pending();
            assert_eq!(
                next_control(&mut pending_events, interrupts.as_mut(), &mut app, stories,)
                    .await
                    .unwrap(),
                RuntimeControl::Quit
            );
            assert_eq!(drops.load(Ordering::SeqCst), 0);
        }
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }
}
