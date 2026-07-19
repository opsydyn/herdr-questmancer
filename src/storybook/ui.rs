use std::sync::{Mutex, OnceLock};

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::{
    scene::{
        pixel::{PixelSize, Rgb, RgbBuffer},
        presentation::ScenePresentation,
        render_scene_for_world,
        snapshot::SceneSnapshot,
    },
    ui::{scene_adapter::flush_rgb, scene_overlays::render_scene_overlays},
};

use super::{
    app::{Mode, StorybookApp},
    catalogue::{Category, Story},
    fixtures::{StoryContext, StoryFixture},
};

pub fn render(
    frame: &mut Frame<'_>,
    app: &StorybookApp,
    stories: &[Story],
    context: &StoryContext,
) {
    let story = app.selected_story(stories);
    let fixture = (story.build)(context);
    if app.mode() == Mode::Inspect {
        render_fixture(frame, frame.area(), story, &fixture);
    } else {
        render_catalogue(frame, app, stories, story, &fixture);
    }
    if app.help_visible() {
        let area = centered(frame.area(), 58, 9);
        frame.render_widget(
            Paragraph::new("j/k story   h/l category\nEnter inspect   Esc back\n? help   q quit")
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" STORYBOOK HELP "),
                )
                .style(Style::new().fg(Color::Black).bg(Color::Yellow)),
            area,
        );
    }
}

fn render_catalogue(
    frame: &mut Frame<'_>,
    app: &StorybookApp,
    stories: &[Story],
    story: &Story,
    fixture: &StoryFixture,
) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.area());
    frame.render_widget(
        Paragraph::new(format!(
            "QUESTMANCER STORYBOOK  |  {}x{}  |  production RGB",
            story.viewport.reference_width, story.viewport.reference_height
        ))
        .style(Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        rows[0],
    );
    if rows[1].width >= 120 {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(22),
                Constraint::Percentage(60),
                Constraint::Percentage(18),
            ])
            .split(rows[1]);
        render_story_list(frame, columns[0], app, stories);
        render_fixture(frame, columns[1], story, fixture);
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(story.description),
                Line::from(""),
                Line::from(format!("owns: {}", story.owns[0].label())),
            ])
            .block(Block::default().borders(Borders::ALL).title(" EVIDENCE ")),
            columns[2],
        );
    } else {
        render_fixture(frame, rows[1], story, fixture);
    }
    frame.render_widget(
        Paragraph::new("[j/k] story  [h/l] category  [enter] inspect  [?] help  [q] quit"),
        rows[2],
    );
}

fn render_story_list(frame: &mut Frame<'_>, area: Rect, app: &StorybookApp, stories: &[Story]) {
    let items = stories.iter().enumerate().map(|(index, story)| {
        let marker = if index == app.selected_index() {
            ">"
        } else {
            " "
        };
        let category = match story.category {
            Category::Worlds => "World",
            Category::Interactions => "Interaction",
        };
        ListItem::new(format!("{marker} {category} / {}", story.title))
    });
    frame.render_widget(
        List::new(items).block(Block::default().borders(Borders::ALL).title(" STORIES ")),
        area,
    );
}

fn render_fixture(frame: &mut Frame<'_>, area: Rect, story: &Story, fixture: &StoryFixture) {
    if area.width < story.viewport.minimum_width || area.height < story.viewport.minimum_height {
        frame.render_widget(
            Paragraph::new(format!(
                "Needs {}x{}; canvas is {}x{}.",
                story.viewport.minimum_width,
                story.viewport.minimum_height,
                area.width,
                area.height
            )),
            area,
        );
        return;
    }
    let StoryFixture::SceneApplication(model) = fixture;
    static BUFFER: OnceLock<Mutex<RgbBuffer>> = OnceLock::new();
    let mut buffer = BUFFER
        .get_or_init(|| Mutex::new(RgbBuffer::filled(0, 0, Rgb::BLACK)))
        .lock()
        .expect("Storybook RGB buffer lock is not poisoned");
    let snapshot = SceneSnapshot::from_model(model);
    let presentation = ScenePresentation::from_model(model);
    render_scene_for_world(
        &snapshot,
        &presentation,
        PixelSize::new(area.width, area.height.saturating_mul(2)),
        &mut buffer,
    );
    flush_rgb(frame.buffer_mut(), area, &buffer, Rgb::BLACK);
    render_scene_overlays(frame, model, &presentation);
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    )
}
