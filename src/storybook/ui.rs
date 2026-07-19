use std::sync::{Mutex, OnceLock};

use ratatui::{
    Frame, Terminal,
    backend::TestBackend,
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

use crate::{
    app::{CharacterSet, ColorMode, DisplayPreferences, Model, Motion},
    scene::{
        self,
        pixel::{PixelPoint, PixelSize, Rgb, RgbBuffer},
        sprite,
    },
    ui::{
        pixel,
        scene_adapter::flush_rgb,
        widgets::{render_adventurer_card, render_chamber},
    },
};

use super::{
    app::{Mode, StorybookApp},
    asset_inventory,
    catalogue::{Category, Story, validate_coverage},
    fixtures::{AtlasContent, AtlasTile, StoryContext, StoryFixture},
};

const WIDE_MINIMUM: u16 = 120;
const MEDIUM_MINIMUM: u16 = 80;
const LONGEST_COMPACT_HEADER: &str =
    "Storybook offline fixture realm ref 120x36 Unicode Xterm-256 motion reduced";
const COMPACT_FOOTER: &str = "[j/k] story [h/l] category [enter] inspect [?] help [esc/q/^c] quit";
const SINGLE_ROW_CHROME_MINIMUM: usize = if LONGEST_COMPACT_HEADER.len() > COMPACT_FOOTER.len() {
    LONGEST_COMPACT_HEADER.len()
} else {
    COMPACT_FOOTER.len()
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShellLayout {
    pub header: Rect,
    pub catalogue: Option<Rect>,
    pub canvas: Rect,
    pub evidence: Option<Rect>,
    pub selector: Option<Rect>,
    pub footer: Rect,
}

pub fn shell_layout(area: Rect) -> ShellLayout {
    let chrome_height = u16::from(usize::from(area.width) < SINGLE_ROW_CHROME_MINIMUM) + 1;
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(chrome_height),
            Constraint::Min(0),
            Constraint::Length(chrome_height),
        ])
        .split(area);
    if area.width >= WIDE_MINIMUM {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(22),
                Constraint::Percentage(56),
                Constraint::Percentage(22),
            ])
            .split(outer[1]);
        ShellLayout {
            header: outer[0],
            catalogue: Some(columns[0]),
            canvas: columns[1],
            evidence: Some(columns[2]),
            selector: None,
            footer: outer[2],
        }
    } else if area.width >= MEDIUM_MINIMUM {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(outer[1]);
        let evidence = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(columns[0]);
        ShellLayout {
            header: outer[0],
            catalogue: Some(evidence[0]),
            canvas: columns[1],
            evidence: Some(evidence[1]),
            selector: None,
            footer: outer[2],
        }
    } else {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(outer[1]);
        ShellLayout {
            header: outer[0],
            catalogue: None,
            canvas: rows[1],
            evidence: None,
            selector: Some(rows[0]),
            footer: outer[2],
        }
    }
}

pub fn render(
    frame: &mut Frame<'_>,
    app: &StorybookApp,
    stories: &[Story],
    context: &StoryContext,
) {
    let story = app.selected_story(stories);
    let fixture = (story.build)(context);
    if app.mode() == Mode::Inspect {
        render_inspection(frame, story, &fixture);
    } else {
        render_catalogue(frame, app, stories, story, &fixture);
    }
    if app.help_visible() {
        render_help(frame);
    }
}

fn render_catalogue(
    frame: &mut Frame<'_>,
    app: &StorybookApp,
    stories: &[Story],
    story: &Story,
    fixture: &StoryFixture,
) {
    let layout = shell_layout(frame.area());
    render_header(frame, layout.header, story, fixture_preferences(fixture));
    render_catalogue_body(frame, layout, app, stories, story, fixture);
    render_catalogue_footer(frame, layout.footer);
}

fn render_header(
    frame: &mut Frame<'_>,
    area: Rect,
    story: &Story,
    preferences: DisplayPreferences,
) {
    let reference = format!(
        "ref {}x{}",
        story.viewport.reference_width, story.viewport.reference_height
    );
    let modes = format!(
        "{} | {} | motion {}",
        character_set_label(preferences.character_set),
        color_mode_label(preferences.color_mode),
        motion_label(preferences.motion)
    );
    let header = if area.height > 1 {
        Text::from(vec![
            Line::from(format!("Storybook | offline fixture realm | {reference}")),
            Line::from(modes),
        ])
    } else if area.width < WIDE_MINIMUM {
        Text::from(format!(
            "Storybook offline fixture realm {reference} {} {} motion {}",
            character_set_label(preferences.character_set),
            color_mode_label(preferences.color_mode),
            motion_label(preferences.motion)
        ))
    } else {
        Text::from(format!(
            "QUESTMANCER STORYBOOK | offline fixture realm | {reference} | {modes}"
        ))
    };
    frame.render_widget(
        Paragraph::new(header).style(Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        area,
    );
}

fn render_catalogue_body(
    frame: &mut Frame<'_>,
    layout: ShellLayout,
    app: &StorybookApp,
    stories: &[Story],
    story: &Story,
    fixture: &StoryFixture,
) {
    if let Some(catalogue) = layout.catalogue {
        render_story_list(frame, catalogue, app, stories);
    }
    if let Some(evidence) = layout.evidence {
        render_evidence(frame, evidence, story, stories);
    }
    if let Some(selector) = layout.selector {
        frame.render_widget(
            Paragraph::new(format!(
                "{}/{} {}",
                app.selected_index() + 1,
                stories.len(),
                story.title
            )),
            selector,
        );
    }
    render_canvas(frame, layout.canvas, story, fixture);
}

fn render_story_list(frame: &mut Frame<'_>, area: Rect, app: &StorybookApp, stories: &[Story]) {
    let items = stories.iter().enumerate().map(|(index, story)| {
        let marker = if index == app.selected_index() {
            ">"
        } else {
            " "
        };
        ListItem::new(format!(
            "{marker} {} / {}",
            category_label(story.category),
            story.title
        ))
        .style(if index == app.selected_index() {
            Style::new().fg(Color::Yellow)
        } else {
            Style::new()
        })
    });
    frame.render_widget(
        List::new(items).block(Block::default().borders(Borders::ALL).title(" STORIES ")),
        area,
    );
}

fn render_evidence(frame: &mut Frame<'_>, area: Rect, story: &Story, stories: &[Story]) {
    let inventory = asset_inventory();
    let owned = known_unique_owned_count(&inventory, stories);
    let (validation, issues) = match validate_coverage(&inventory, stories) {
        Ok(_) => ("PASS", None),
        Err(error) => (
            "FAIL",
            Some(format!(
                "issues: {} missing / {} duplicate / {} unknown",
                error.missing().len(),
                error.duplicates().len(),
                error.unknown().len()
            )),
        ),
    };
    let mut lines = vec![
        Line::from(story.description),
        Line::from(format!("owns: {}", story.owns.len())),
        Line::from(format!("shows: {}", story.shows.len())),
        Line::from(format!("total owned: {owned}")),
        Line::from(format!("validation: {validation}")),
        Line::from("owned assets:"),
    ];
    lines.extend(
        story
            .owns
            .iter()
            .map(|asset| Line::from(format!("  {}", asset.label()))),
    );
    lines.push(Line::from("shown assets:"));
    lines.extend(
        story
            .shows
            .iter()
            .map(|asset| Line::from(format!("  {}", asset.label()))),
    );
    if let Some(issues) = issues {
        lines.push(Line::from(issues));
    }
    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .block(Block::default().borders(Borders::ALL).title(" COVERAGE "))
            .wrap(Wrap { trim: true }),
        area,
    );
}

pub fn known_unique_owned_count(inventory: &[super::AssetId], stories: &[Story]) -> usize {
    inventory
        .iter()
        .filter(|asset| {
            stories
                .iter()
                .flat_map(|story| story.owns)
                .filter(|owned| *owned == *asset)
                .count()
                == 1
        })
        .count()
}

fn render_canvas(frame: &mut Frame<'_>, area: Rect, story: &Story, fixture: &StoryFixture) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" PRODUCTION CANVAS ");
    let canvas = block.inner(area);
    frame.render_widget(block, area);
    render_story_or_minimum(frame, canvas, story, fixture);
}

fn render_story_or_minimum(
    frame: &mut Frame<'_>,
    area: Rect,
    story: &Story,
    fixture: &StoryFixture,
) {
    if area.width < story.viewport.minimum_width || area.height < story.viewport.minimum_height {
        frame.render_widget(
            Paragraph::new(format!(
                "This story needs at least {}x{}.\nCanvas available: {}x{}.",
                story.viewport.minimum_width,
                story.viewport.minimum_height,
                area.width,
                area.height,
            ))
            .style(Style::new().fg(Color::Yellow))
            .wrap(Wrap { trim: true }),
            area,
        );
        return;
    }
    render_fixture(frame, area, fixture);
}

fn render_inspection(frame: &mut Frame<'_>, story: &Story, fixture: &StoryFixture) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(frame.area());
    render_story_or_minimum(frame, rows[0], story, fixture);
    frame.render_widget(
        Paragraph::new("[esc] catalogue  [?] help  [q] quit").alignment(Alignment::Center),
        rows[1],
    );
}

fn render_fixture(frame: &mut Frame<'_>, area: Rect, fixture: &StoryFixture) {
    if area.is_empty() {
        return;
    }
    match fixture {
        StoryFixture::Application(model) => {
            let source = render_application_buffer(model, area.width, area.height);
            blit(&source, frame.buffer_mut(), area);
        }
        StoryFixture::AssetAtlas(atlas) => render_atlas(frame, area, &atlas.tiles),
        StoryFixture::PixelScene(fixture) => {
            static BUFFER: OnceLock<Mutex<RgbBuffer>> = OnceLock::new();
            let mut buffer = BUFFER
                .get_or_init(|| Mutex::new(RgbBuffer::filled(0, 0, Rgb::BLACK)))
                .lock()
                .expect("scene Storybook buffer lock is not poisoned");
            scene::render_scene_for_story(
                &fixture.snapshot,
                fixture.world_override,
                PixelSize::new(area.width, area.height.saturating_mul(2)),
                &mut buffer,
            );
            flush_rgb(frame.buffer_mut(), area, &buffer, Rgb::BLACK);
        }
    }
}

pub fn render_application_buffer(model: &Model, width: u16, height: u16) -> Buffer {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("in-memory terminal is valid");
    terminal
        .draw(|frame| crate::ui::render(frame, model))
        .expect("in-memory render is infallible");
    terminal.backend().buffer().clone()
}

pub fn blit(source: &Buffer, target: &mut Buffer, area: Rect) {
    for y in 0..area.height.min(source.area.height) {
        for x in 0..area.width.min(source.area.width) {
            let source_position = (
                source.area.x.saturating_add(x),
                source.area.y.saturating_add(y),
            );
            let target_position = (area.x.saturating_add(x), area.y.saturating_add(y));
            let Some(cell) = source.cell(source_position).cloned() else {
                continue;
            };
            if let Some(target_cell) = target.cell_mut(target_position) {
                *target_cell = cell;
            }
        }
    }
}

fn render_atlas(frame: &mut Frame<'_>, area: Rect, tiles: &[AtlasTile]) {
    let maximum_preferred_width = tiles
        .iter()
        .map(|tile| tile.preferred_width)
        .max()
        .unwrap_or(1)
        .max(1);
    let maximum_preferred_height = tiles
        .iter()
        .map(|tile| tile.preferred_height)
        .max()
        .unwrap_or(1)
        .max(1);
    let columns = (area.width / maximum_preferred_width).max(1);
    let rows = u16::try_from(tiles.len().div_ceil(usize::from(columns))).unwrap_or(u16::MAX);

    for row in 0..rows {
        let y = area
            .y
            .saturating_add(row.saturating_mul(maximum_preferred_height));
        if y >= area.bottom() {
            break;
        }
        for column in 0..columns {
            let index = usize::from(row)
                .saturating_mul(usize::from(columns))
                .saturating_add(usize::from(column));
            let Some(tile) = tiles.get(index) else {
                break;
            };
            let x = area
                .x
                .saturating_add(column.saturating_mul(maximum_preferred_width));
            if x >= area.right() {
                break;
            }
            let tile_area = Rect::new(
                x,
                y,
                tile.preferred_width.min(area.right().saturating_sub(x)),
                tile.preferred_height.min(area.bottom().saturating_sub(y)),
            );
            render_atlas_tile(frame, tile_area, tile);
        }
    }
}

fn render_atlas_tile(frame: &mut Frame<'_>, area: Rect, tile: &AtlasTile) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", tile.label));
    let content_area = block.inner(area);
    frame.render_widget(block, area);
    if content_area.is_empty() {
        return;
    }
    match &tile.content {
        AtlasContent::Pixel {
            canvas,
            palette,
            background,
            ..
        } => {
            let pixels = pixel::pack(canvas, palette, *background);
            frame.render_widget(Paragraph::new(pixels), content_area);
        }
        AtlasContent::RgbSprite {
            frame: sprite_frame,
            background,
        } => {
            render_rgb_sprite(frame, content_area, sprite_frame, *background, 1);
        }
        AtlasContent::RgbSpriteScaled {
            frame: sprite_frame,
            background,
            scale,
        } => {
            render_rgb_sprite(
                frame,
                content_area,
                sprite_frame,
                *background,
                (*scale).max(1),
            );
        }
        AtlasContent::AdventurerCard {
            agent,
            theatre,
            preferences,
        } => render_adventurer_card(frame, content_area, agent, *theatre, preferences),
        AtlasContent::Chamber {
            agent,
            theatre,
            selected,
            preferences,
        } => render_chamber(frame, content_area, agent, *theatre, *selected, preferences),
        AtlasContent::Application { model } => {
            let source = render_application_buffer(model, content_area.width, content_area.height);
            blit(&source, frame.buffer_mut(), content_area);
        }
    }
}

fn render_rgb_sprite(
    frame: &mut Frame<'_>,
    content_area: Rect,
    sprite_frame: &sprite::SpriteFrame,
    background: Rgb,
    scale: u16,
) {
    let logical_size = PixelSize::new(content_area.width, content_area.height.saturating_mul(2));
    let mut pixels = RgbBuffer::filled(logical_size.width, logical_size.height, background);
    let sprite_size = sprite_frame.size();
    let scaled_width = sprite_size.width.saturating_mul(scale);
    let scaled_height = sprite_size.height.saturating_mul(scale);
    let origin = PixelPoint::new(
        i32::from(logical_size.width.saturating_sub(scaled_width) / 2),
        i32::from(logical_size.height.saturating_sub(scaled_height) / 2),
    );
    sprite::blit_scaled(sprite_frame, origin, scale, &mut pixels);
    flush_rgb(frame.buffer_mut(), content_area, &pixels, background);
}

fn render_catalogue_footer(frame: &mut Frame<'_>, area: Rect) {
    let footer = if area.height > 1 {
        Text::from(vec![
            Line::from("[j/k] story [h/l] cat [enter] inspect"),
            Line::from("[?] help [esc/q/^c] quit"),
        ])
    } else {
        Text::from(COMPACT_FOOTER)
    };
    frame.render_widget(Paragraph::new(footer).alignment(Alignment::Center), area);
}

fn render_help(frame: &mut Frame<'_>) {
    let area = centered_rect(frame.area(), 62, 14);
    frame.render_widget(Clear, area);
    let text = Text::from(vec![
        Line::from("j/down   next story"),
        Line::from("k/up     previous story"),
        Line::from("l/right  next category"),
        Line::from("h/left   previous category"),
        Line::from("enter    inspect selected story"),
        Line::from("?        toggle this help"),
        Line::from("esc      close help / leave inspection / quit"),
        Line::from("q        quit"),
        Line::from("ctrl-c   quit"),
    ]);
    frame.render_widget(
        Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" STORYBOOK KEYS "),
            )
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width.saturating_sub(2)).max(1);
    let height = height.min(area.height.saturating_sub(2)).max(1);
    Rect::new(
        area.x.saturating_add(area.width.saturating_sub(width) / 2),
        area.y
            .saturating_add(area.height.saturating_sub(height) / 2),
        width,
        height,
    )
}

fn fixture_preferences(fixture: &StoryFixture) -> DisplayPreferences {
    match fixture {
        StoryFixture::Application(model) => *model.preferences(),
        StoryFixture::AssetAtlas(_) | StoryFixture::PixelScene(_) => DisplayPreferences::default(),
    }
}

const fn category_label(category: Category) -> &'static str {
    match category {
        Category::AssetAtlas => "Atlas",
        Category::Widgets => "Widgets",
        Category::FullScenes => "Scenes",
        Category::Compatibility => "Compatibility",
    }
}

const fn character_set_label(character_set: CharacterSet) -> &'static str {
    match character_set {
        CharacterSet::Unicode => "Unicode",
        CharacterSet::Ascii => "ASCII",
    }
}

const fn color_mode_label(color_mode: ColorMode) -> &'static str {
    match color_mode {
        ColorMode::Xterm256 => "Xterm-256",
        ColorMode::Ansi16 => "ANSI-16",
    }
}

const fn motion_label(motion: Motion) -> &'static str {
    match motion {
        Motion::Full => "full",
        Motion::Reduced => "reduced",
        Motion::None => "none",
    }
}
