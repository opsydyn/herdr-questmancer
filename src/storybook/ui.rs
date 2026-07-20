use std::sync::{Mutex, OnceLock};

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::{
    domain::{AdventurerPersona, PersonaKey},
    scene::{
        assets::{
            adventurer::{adventurer_animation_frame, adventurer_portrait_frame},
            archetypes::{goblin_portrait_frame, goblin_world_frame},
        },
        pixel::{PixelSize, Rgb, RgbBuffer},
        presentation::ScenePresentation,
        render_scene_for_world,
        snapshot::SceneSnapshot,
        sprite::{SpriteFrame, blit, blit_scaled},
        stage::ScenePose,
    },
    ui::{scene_adapter::flush_rgb, scene_overlays::render_scene_overlays},
};

use super::{
    app::{Mode, StorybookApp},
    catalogue::{Category, Story},
    fixtures::{ArchetypeGallery, CORE_ARCHETYPES, StoryContext, StoryFixture},
};

static BUFFER: OnceLock<Mutex<RgbBuffer>> = OnceLock::new();

pub fn render(
    frame: &mut Frame<'_>,
    app: &StorybookApp,
    stories: &[Story],
    context: &StoryContext,
    portraits: Option<&crate::portrait::PortraitGallery>,
) {
    let story = app.selected_story(stories);
    let fixture = (story.build)(*context);
    if app.mode() == Mode::Inspect {
        render_fixture(frame, frame.area(), story, &fixture, portraits);
    } else {
        render_catalogue(frame, app, stories, story, &fixture, portraits);
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
    portraits: Option<&crate::portrait::PortraitGallery>,
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
            "QUESTMANCER STORYBOOK  |  {}x{}  |  production RGB  |  portrait: {}",
            story.viewport.reference_width,
            story.viewport.reference_height,
            portraits.map_or("not detected", |gallery| gallery.capability().label())
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
        render_fixture(frame, columns[1], story, fixture, portraits);
        frame.render_widget(
            Paragraph::new(
                [
                    vec![
                        Line::from(story.description),
                        Line::from(""),
                        Line::from(format!("owns: {}", story.owns[0].label())),
                        Line::from(format!(
                            "portrait: {}",
                            portraits
                                .map_or("not detected", |gallery| { gallery.capability().label() })
                        )),
                    ],
                    portraits
                        .and_then(crate::portrait::PortraitGallery::diagnostic)
                        .map(|diagnostic| vec![Line::from(""), Line::from(diagnostic)])
                        .unwrap_or_default(),
                ]
                .concat(),
            )
            .block(Block::default().borders(Borders::ALL).title(" EVIDENCE ")),
            columns[2],
        );
    } else {
        render_fixture(frame, rows[1], story, fixture, portraits);
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
            Category::Assets => "Assets",
            Category::Interactions => "Interaction",
        };
        ListItem::new(format!("{marker} {category} / {}", story.title))
    });
    frame.render_widget(
        List::new(items).block(Block::default().borders(Borders::ALL).title(" STORIES ")),
        area,
    );
}

fn render_fixture(
    frame: &mut Frame<'_>,
    area: Rect,
    story: &Story,
    fixture: &StoryFixture,
    portraits: Option<&crate::portrait::PortraitGallery>,
) {
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
    if let StoryFixture::ArchetypeGallery(gallery) = fixture {
        render_archetype_gallery(frame, area, *gallery);
        return;
    }
    let StoryFixture::SceneApplication(model) = fixture else {
        unreachable!("all non-gallery Storybook fixtures are scene applications")
    };
    let mut buffer = BUFFER
        .get_or_init(|| Mutex::new(RgbBuffer::filled(0, 0, Rgb::BLACK)))
        .lock()
        .expect("Storybook RGB buffer lock is not poisoned");
    let snapshot = SceneSnapshot::from_model(model);
    let presentation = ScenePresentation::from_model(model);
    let scene_frame = render_scene_for_world(
        &snapshot,
        &presentation,
        PixelSize::new(area.width, area.height.saturating_mul(2)),
        &mut buffer,
    );
    flush_rgb(frame.buffer_mut(), area, &buffer, Rgb::BLACK);
    crate::ui::scene_overlays::render_scene_identity_labels(frame, model, &scene_frame);
    render_scene_overlays(frame, model, &presentation, portraits);
}

fn render_archetype_gallery(frame: &mut Frame<'_>, area: Rect, gallery: ArchetypeGallery) {
    const BACKGROUND: Rgb = Rgb::new(17, 20, 29);
    let mut buffer = BUFFER
        .get_or_init(|| Mutex::new(RgbBuffer::filled(0, 0, Rgb::BLACK)))
        .lock()
        .expect("Storybook RGB buffer lock is not poisoned");
    buffer.ensure_size(area.width, area.height.saturating_mul(2), BACKGROUND);

    let entries = gallery_entries(gallery);
    let columns = if entries.len() <= 2 { entries.len() } else { 4 };
    let rows = entries.len().div_ceil(columns);
    let cell_width = area.width / u16::try_from(columns).expect("gallery columns fit u16");
    let cell_height =
        area.height.saturating_mul(2) / u16::try_from(rows).expect("gallery rows fit u16");

    let mut labels = Vec::with_capacity(entries.len());
    for (index, (label, sprite)) in entries.iter().enumerate() {
        let column = u16::try_from(index % columns).expect("gallery column fits u16");
        let row = u16::try_from(index / columns).expect("gallery row fits u16");
        let scale = if entries.len() <= 2 { 2 } else { 1 };
        let painted_width = sprite.size().width.saturating_mul(scale);
        let painted_height = sprite.size().height.saturating_mul(scale);
        let x = column
            .saturating_mul(cell_width)
            .saturating_add(cell_width.saturating_sub(painted_width) / 2);
        let y = row
            .saturating_mul(cell_height)
            .saturating_add(cell_height.saturating_sub(painted_height) / 2)
            .saturating_sub(1);
        if scale == 1 {
            blit(
                sprite,
                crate::scene::pixel::PixelPoint::new(i32::from(x), i32::from(y)),
                &mut buffer,
            );
        } else {
            blit_scaled(
                sprite,
                crate::scene::pixel::PixelPoint::new(i32::from(x), i32::from(y)),
                scale,
                &mut buffer,
            );
        }

        let label_y = area.y
            + row
                .saturating_mul(cell_height)
                .saturating_add(cell_height.saturating_sub(2))
                / 2;
        let label_area = Rect::new(
            area.x + column.saturating_mul(cell_width),
            label_y.min(area.bottom().saturating_sub(1)),
            cell_width,
            1,
        );
        labels.push((*label, label_area));
    }
    flush_rgb(frame.buffer_mut(), area, &buffer, BACKGROUND);
    for (label, label_area) in labels {
        frame.render_widget(
            Paragraph::new(label)
                .alignment(ratatui::layout::Alignment::Center)
                .style(Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            label_area,
        );
    }
}

fn gallery_entries(gallery: ArchetypeGallery) -> Vec<(&'static str, SpriteFrame)> {
    if gallery == ArchetypeGallery::GoblinEasterEgg {
        return vec![
            ("Goblin · world", goblin_world_frame()),
            ("Goblin · portrait", goblin_portrait_frame()),
        ];
    }

    CORE_ARCHETYPES
        .iter()
        .map(|class| {
            let mut persona =
                AdventurerPersona::for_key(PersonaKey::new(format!("storybook-master-{class:?}")));
            persona.class = *class;
            let sprite = match gallery {
                ArchetypeGallery::WorldMasters => {
                    adventurer_animation_frame(&persona, ScenePose::Working, 0)
                }
                ArchetypeGallery::PortraitMasters => adventurer_portrait_frame(&persona)
                    .expect("every core archetype has a portrait master"),
                ArchetypeGallery::GoblinEasterEgg => unreachable!(),
            };
            (class_label(*class), sprite)
        })
        .collect()
}

const fn class_label(class: crate::domain::AdventurerClass) -> &'static str {
    use crate::domain::AdventurerClass;
    match class {
        AdventurerClass::Barbarian => "Barbarian",
        AdventurerClass::Bard => "Bard",
        AdventurerClass::Cleric => "Cleric",
        AdventurerClass::Druid => "Druid",
        AdventurerClass::Paladin => "Paladin",
        AdventurerClass::Ranger => "Ranger",
        AdventurerClass::Rogue => "Rogue",
        AdventurerClass::Wizard => "Wizard",
        AdventurerClass::Artificer => "Artificer",
        AdventurerClass::Runewright => "Runewright",
        AdventurerClass::Testmender => "Testmender",
        AdventurerClass::Pathseeker => "Pathseeker",
    }
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
