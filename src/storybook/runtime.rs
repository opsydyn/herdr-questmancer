use std::{
    io::{self, Write},
    pin::Pin,
    task::{Context as TaskContext, Poll},
};

use anyhow::{Context, Result};
use crossterm::event::{Event, EventStream};
use futures_util::{Stream, StreamExt};
use tokio::signal::unix::{Signal, SignalKind, signal};

use super::{
    app::{Exit, Mode, StorybookApp, reduce},
    catalogue::{CoverageError, CoverageReport, Story, catalogue, validate_catalogue},
    fit::CellPixels,
    fit_preview::{FitPreview, delete_image},
    fixtures::StoryContext,
    input, ui,
};
use crate::terminal::TerminalGuard;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RuntimeControl {
    Continue,
    Quit,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum GraphicsMode {
    #[default]
    Detect,
    Off,
}

#[derive(Clone, Debug, Default)]
pub struct RunOptions {
    pub initial_story: Option<String>,
    pub inspect: bool,
    pub graphics: GraphicsMode,
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
    preview: &mut FitPreview,
) -> Result<RuntimeControl>
where
    Events: Stream<Item = std::io::Result<Event>> + Unpin,
    Interrupts: Stream<Item = ()>,
{
    let walk_frame = if app.mode() == Mode::Inspect && !app.help_visible() {
        preview.next_walk_frame()
    } else {
        None
    };
    tokio::select! {
        event = events.next() => {
            if let Some(Ok(event)) = &event
                && app.mode() == Mode::Inspect && !app.help_visible()
                && preview.is_for_story(app.selected_story(stories).id.as_str())
                && preview.handle_event(event)
            { return Ok(RuntimeControl::Continue); }
            let was_inspecting = app.mode() == Mode::Inspect;
            let next = control_for_event(app, event, stories);
            if !was_inspecting && app.mode() == Mode::Inspect {
                preview.restart_walk();
            }
            next
        },
        _ = interrupts.next() => Ok(RuntimeControl::Quit),
        () = async {
            if let Some(deadline) = walk_frame {
                tokio::time::sleep_until(deadline).await;
            }
        }, if walk_frame.is_some() => Ok(RuntimeControl::Continue),
    }
}

pub async fn run() -> Result<()> {
    run_with_options(RunOptions::default()).await
}

fn cleanup_images(ids: &[u32]) -> Result<()> {
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

pub async fn run_with_options(options: RunOptions) -> Result<()> {
    let (stories, context) = preflight()?;
    let interrupts = InterruptSignal::install()?;
    tokio::pin!(interrupts);
    let mut app = StorybookApp::new(stories);
    if let Some(id) = &options.initial_story {
        let index = stories
            .iter()
            .position(|s| s.id.as_str() == id)
            .with_context(|| format!("unknown Storybook story: {id}"))?;
        app.select(index, stories);
    }
    if options.inspect {
        let _ = reduce(&mut app, super::app::Action::Inspect, stories);
    }
    let (_guard, mut terminal) = TerminalGuard::enter()?;
    let picker = (options.graphics == GraphicsMode::Detect)
        .then(ratatui_image::picker::Picker::from_query_stdio)
        .and_then(Result::ok);
    let portraits = picker.as_ref().map_or_else(
        || crate::portrait::PortraitGallery::fallback("graphics disabled"),
        crate::portrait::PortraitGallery::from_picker,
    );
    let mut preview = picker
        .as_ref()
        .filter(|p| p.protocol_type() == ratatui_image::picker::ProtocolType::Kitty)
        .filter(|_| std::env::var_os("TMUX").is_none())
        .and_then(|p| CellPixels::new(p.font_size().width, p.font_size().height))
        .map_or_else(
            || {
                FitPreview::unavailable(
                    "Kitty with confirmed geometry required; tmux unsupported in this spike",
                )
            },
            FitPreview::new,
        );
    let mut events = EventStream::new();

    let outcome = async {
        loop {
            preview.begin_frame();
            terminal.draw(|frame| {
                ui::render_with_fit(
                    frame,
                    &app,
                    stories,
                    &context,
                    Some(&portraits),
                    Some(&mut preview),
                );
            })?;
            cleanup_images(&preview.finish_frame())?;
            let control = next_control(
                &mut events,
                interrupts.as_mut(),
                &mut app,
                stories,
                &mut preview,
            )
            .await?;
            if control == RuntimeControl::Quit {
                break;
            }
        }
        Ok(())
    }
    .await;
    let cleanup = cleanup_images(&preview.shutdown());
    outcome.and(cleanup)
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
                next_control(
                    &mut events,
                    interrupts.as_mut(),
                    &mut app,
                    stories,
                    &mut crate::storybook::fit_preview::FitPreview::unavailable("test")
                )
                .await
                .unwrap(),
                RuntimeControl::Continue
            );
            assert_eq!(drops.load(Ordering::SeqCst), 0);

            ready.store(true, Ordering::SeqCst);
            let mut pending_events = stream::pending();
            assert_eq!(
                next_control(
                    &mut pending_events,
                    interrupts.as_mut(),
                    &mut app,
                    stories,
                    &mut crate::storybook::fit_preview::FitPreview::unavailable("test")
                )
                .await
                .unwrap(),
                RuntimeControl::Quit
            );
            assert_eq!(drops.load(Ordering::SeqCst), 0);
        }
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }
}
