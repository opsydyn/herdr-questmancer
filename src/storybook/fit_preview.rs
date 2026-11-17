//! Isolated native-fit preview state; production never imports this module.
use std::fmt;

use crossterm::event::{Event, KeyCode, MouseButton, MouseEventKind};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Clear, Paragraph},
};
use ratatui_image::{protocol::kitty::Kitty, sliced::SlicedProtocol};

use super::fit::{CellPixels, FitScene, FitTarget, FitTransform};
use crate::{
    app::{Modal, Model},
    portrait::PortraitGallery,
    scene::{
        SceneFrame, SceneTarget,
        pixel::{PixelSize, Rgb, RgbBuffer},
        presentation::ScenePresentation,
        render_scene_for_world,
        snapshot::SceneSnapshot,
    },
    ui::{scene_adapter::flush_rgb, scene_overlays::render_scene_overlays},
};

struct CachedImage {
    hash: blake3::Hash,
    transform: FitTransform,
    protocol: SlicedProtocol,
    id: u32,
}

/// The preview owns two image IDs and no live agent or persistent state.
pub struct FitPreview {
    geometry: Option<CellPixels>,
    diagnostic: String,
    ids: [u32; 2],
    slot: usize,
    cache: Option<CachedImage>,
    cleanup: Vec<u32>,
    painted: bool,
    story: String,
    model: Option<Model>,
    scene: Option<FitScene>,
    fallback: Option<(Rect, SceneFrame)>,
    uploads: usize,
    profile_mask: Option<Rect>,
}

impl fmt::Debug for FitPreview {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FitPreview")
            .field("geometry", &self.geometry)
            .field("story", &self.story)
            .field("uploads", &self.uploads)
            .finish_non_exhaustive()
    }
}

impl FitPreview {
    pub fn new(geometry: CellPixels) -> Self {
        let mut bytes = [0; 8];
        if getrandom::fill(&mut bytes).is_err() {
            return Self::unavailable("image identity allocation failed");
        }
        let a = u32::from_ne_bytes(bytes[..4].try_into().expect("four bytes")).max(1);
        let mut b = u32::from_ne_bytes(bytes[4..].try_into().expect("four bytes")).max(1);
        if a == b {
            b = a.wrapping_add(1).max(1);
        }
        Self::with_ids(Some(geometry), String::new(), [a, b])
    }

    pub fn unavailable(reason: impl Into<String>) -> Self {
        Self::with_ids(None, reason.into(), [1, 2])
    }

    fn with_ids(geometry: Option<CellPixels>, diagnostic: String, ids: [u32; 2]) -> Self {
        Self {
            geometry,
            diagnostic,
            ids,
            slot: 0,
            cache: None,
            cleanup: Vec::new(),
            painted: false,
            story: String::new(),
            model: None,
            scene: None,
            fallback: None,
            uploads: 0,
            profile_mask: None,
        }
    }

    pub fn begin_frame(&mut self) {
        self.painted = false;
    }

    /// Runtime writes these narrow deletion commands after the frame is flushed.
    pub fn finish_frame(&mut self) -> Vec<u32> {
        if !self.painted
            && let Some(old) = self.cache.take()
        {
            self.cleanup.push(old.id);
        }
        std::mem::take(&mut self.cleanup)
    }

    pub fn shutdown(&mut self) -> Vec<u32> {
        self.painted = false;
        self.finish_frame()
    }

    pub const fn uploads(&self) -> usize {
        self.uploads
    }

    /// Input belongs only to the currently installed local preview fixture.
    pub(crate) fn is_for_story(&self, story: &str) -> bool {
        self.model.is_some() && self.story == story
    }

    pub fn render(
        &mut self,
        frame: &mut Frame<'_>,
        area: Rect,
        story: &str,
        base: &Model,
        portraits: Option<&PortraitGallery>,
    ) {
        if self.story != story {
            story.clone_into(&mut self.story);
            self.model = Some(base.clone());
        }
        let model = self.model.as_ref().expect("preview fixture is installed");
        let picture = Rect::new(
            area.x,
            area.y.saturating_add(1),
            area.width,
            area.height.saturating_sub(3),
        );
        let overlay = model.modal() != &Modal::None || model.adventurer_card_visible();
        self.profile_mask = crate::ui::world_fit::profile_mask(frame.area(), model);
        let projection = self
            .geometry
            .filter(|_| model.modal() == &Modal::None)
            .map(|cell| {
                super::density::scale_for_story(story).map_or_else(
                    || FitScene::project(model, picture, cell),
                    |scale| super::density::project(model, picture, cell, scale),
                )
            });
        let mut status = if overlay {
            "Parchment / responsive backdrop".to_owned()
        } else {
            format!("Fit fallback: {}", self.diagnostic)
        };
        self.scene = None;
        self.fallback = None;
        frame.render_widget(Clear, area);
        if let Some(Ok(scene)) = projection.as_ref() {
            let hash = rgb_hash(&scene.rgb);
            let changed = self
                .cache
                .as_ref()
                .is_none_or(|c| c.hash != hash || c.transform != scene.transform);
            if changed {
                if let Some(old) = self.cache.take() {
                    self.cleanup.push(old.id);
                }
                self.slot = (self.slot + 1) % 2;
                let id = self.ids[self.slot];
                let image = scene.transform.image(&scene.rgb);
                match Kitty::new(image, scene.transform.area.as_size(), id, false) {
                    Ok(protocol) => {
                        self.cache = Some(CachedImage {
                            hash,
                            transform: scene.transform,
                            protocol: SlicedProtocol::Kitty(protocol),
                            id,
                        });
                        self.uploads += 1;
                    }
                    Err(error) => {
                        status = format!("Fit encoding failed: {error}");
                    }
                }
            }
            if let Some(cache) = &self.cache {
                crate::ui::world_fit::paint_image_bands(
                    frame,
                    &cache.protocol,
                    scene.transform.area,
                    self.profile_mask,
                );
                self.painted = true;
                status = format!(
                    "Kitty fit {}x / {}/{} adventurers",
                    scene.transform.scale,
                    scene.frame.actors.len(),
                    model.domain().agents.len()
                );
                if let Some(scale) = super::density::scale_for_story(story) {
                    status = format!("EXPERIMENT {scale:?} / {status}");
                }
                self.scene = Some(scene.clone());
            }
        } else if let Some(Err(error)) = projection {
            status = format!("Fit fallback: {error}");
        }
        if !self.painted {
            self.fallback = Some((picture, paint_fallback(frame, picture, model)));
        }
        frame.render_widget(
            Paragraph::new(status).style(Style::new().fg(Color::Yellow)),
            Rect::new(area.x, area.y, area.width, area.height.min(1)),
        );
        paint_footer(frame, area, model);
        if overlay {
            render_scene_overlays(
                frame,
                model,
                &ScenePresentation::from_model(model),
                portraits,
            );
        }
    }

    /// Local fixture actions only: this path never builds an `AgentCommand`.
    pub fn handle_event(&mut self, event: &Event) -> bool {
        if let Event::Mouse(mouse) = event
            && self.profile_mask.is_some_and(|area| {
                area.contains(ratatui::layout::Position::new(mouse.column, mouse.row))
            })
        {
            return true;
        }
        let target = match event {
            Event::Mouse(mouse) if mouse.kind == MouseEventKind::Down(MouseButton::Left) => self
                .scene
                .as_ref()
                .and_then(|s| {
                    preview_target(
                        s.terminal_frame(),
                        Rect::default(),
                        self.profile_mask,
                        mouse.column,
                        mouse.row,
                    )
                })
                .or_else(|| {
                    self.fallback.as_ref().and_then(|(area, scene)| {
                        preview_target(
                            scene.clone(),
                            *area,
                            self.profile_mask,
                            mouse.column,
                            mouse.row,
                        )
                    })
                }),
            _ => None,
        };
        let Some(model) = self.model.as_mut() else {
            return false;
        };
        let overlay = model.modal() != &Modal::None || model.adventurer_card_visible();
        if let Event::Key(key) = event {
            if model.ledger_page().is_some() && ledger_key(model, key.code) {
                return true;
            }
            match key.code {
                KeyCode::Tab => model.select_next_agent(),
                KeyCode::BackTab => model.select_previous_agent(),
                KeyCode::Char('c') | KeyCode::Enter if model.modal() == &Modal::None => {
                    if model.adventurer_card_visible() {
                        model.dismiss_adventurer_card();
                    } else {
                        model.show_adventurer_card();
                    }
                }
                KeyCode::Char('L') => {
                    model.dismiss_adventurer_card();
                    model.toggle_ledger();
                }
                KeyCode::Esc if overlay => {
                    model.dismiss_modal();
                    model.dismiss_adventurer_card();
                }
                _ => return false,
            }
            return true;
        }
        if model.modal() == &Modal::None
            && let Some(target) = target
        {
            match target {
                FitTarget::Adventurer(key) => {
                    model.select_agent(&key);
                    model.show_adventurer_card();
                }
                FitTarget::Librarian => model.toggle_ledger(),
            }
            return true;
        }
        false
    }
}

fn paint_fallback(frame: &mut Frame<'_>, picture: Rect, model: &Model) -> SceneFrame {
    let mut buffer = RgbBuffer::filled(0, 0, Rgb::BLACK);
    let scene = render_scene_for_world(
        &SceneSnapshot::from_model(model),
        &ScenePresentation::from_model(model),
        PixelSize::new(picture.width, picture.height.saturating_mul(2)),
        &mut buffer,
    );
    flush_rgb(
        frame.buffer_mut(),
        picture,
        &buffer,
        Rgb::BLACK,
        model.preferences().color_mode,
    );
    scene
}

fn preview_target(
    mut scene: SceneFrame,
    offset: Rect,
    mask: Option<Rect>,
    column: u16,
    row: u16,
) -> Option<FitTarget> {
    for bounds in scene
        .actors
        .iter_mut()
        .map(|a| &mut a.bounds)
        .chain(scene.interactables.iter_mut().map(|a| &mut a.bounds))
    {
        bounds.x += i32::from(offset.x);
        bounds.y += i32::from(offset.y) * 2;
    }
    crate::ui::world_fit::mask_scene_regions(&mut scene, mask);
    scene.target_at(column, row).map(|target| match target {
        SceneTarget::Agent(key) => FitTarget::Adventurer(key.clone()),
        SceneTarget::Interactable(_) => FitTarget::Librarian,
    })
}

/// Delete only a preview-owned identity; never use the terminal-wide delete action.
pub fn delete_image(id: u32) -> String {
    format!("\x1b_Ga=d,d=I,i={id},q=2;\x1b\\")
}

fn rgb_hash(rgb: &RgbBuffer) -> blake3::Hash {
    let bytes = rgb
        .pixels()
        .iter()
        .flat_map(|p| [p.r, p.g, p.b])
        .collect::<Vec<_>>();
    blake3::hash(&bytes)
}

fn ledger_key(model: &mut Model, code: KeyCode) -> bool {
    match code {
        KeyCode::Char('j') | KeyCode::Right | KeyCode::Down => model.next_ledger_page(),
        KeyCode::Char('k') | KeyCode::Left | KeyCode::Up => model.previous_ledger_page(),
        KeyCode::Char('g') => model.first_ledger_page(),
        KeyCode::Char('G') => model.last_ledger_page(),
        KeyCode::Char('?') => model.toggle_ledger(),
        _ => return false,
    }
    true
}

fn paint_footer(frame: &mut Frame<'_>, area: Rect, model: &Model) {
    if area.height >= 3 {
        let label = model.selected_agent().map_or_else(
            || "No selection".to_owned(),
            |a| format!("{} / {:?} / {:?}", a.name, a.persona.class, a.presence),
        );
        let urgent = model
            .domain()
            .agents
            .values()
            .filter(|a| {
                a.attention.summons() == Some(crate::domain::GuildSummons::CounselRequested)
            })
            .count();
        frame.render_widget(
            Paragraph::new(format!(
                "{label} / {urgent} need counsel\nTab adventurer / c card / L Ledger / Esc back"
            )),
            Rect::new(area.x, area.bottom() - 2, area.width, 2),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storybook::fixtures::{FitParty, StoryContext, hall_fit_fixture};
    use crossterm::event::{KeyEvent, KeyModifiers};
    use ratatui::{Terminal, backend::TestBackend};

    fn draw(preview: &mut FitPreview, model: &Model, size: (u16, u16)) -> (String, Vec<u32>) {
        draw_story(preview, model, size, "test-fit")
    }
    fn draw_story(
        preview: &mut FitPreview,
        model: &Model,
        size: (u16, u16),
        story: &str,
    ) -> (String, Vec<u32>) {
        let mut terminal = Terminal::new(TestBackend::new(size.0, size.1)).unwrap();
        preview.begin_frame();
        terminal
            .draw(|f| preview.render(f, f.area(), story, model, None))
            .unwrap();
        let text = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect();
        (text, preview.finish_frame())
    }
    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    #[test]
    fn capacity_studies_reach_all_thirty_two_identities_and_keep_fit_beneath_profiles() {
        use crate::{
            app::View,
            storybook::density::{self, PartySize},
        };
        for (view, story) in [
            (View::Guild, "world.capacity-hall-32-small"),
            (View::Delve, "world.capacity-delve-32-small"),
        ] {
            let model = density::fixture(StoryContext::fixed(), PartySize::ThirtyTwo, view);
            let mut preview = FitPreview::with_ids(CellPixels::new(8, 16), String::new(), [81, 82]);
            let (text, removed) = draw_story(&mut preview, &model, (80, 24), story);
            assert!(text.contains("EXPERIMENT Small") && text.contains("32/32 adventurers"));
            assert!(removed.is_empty());
            let transform = preview.scene.as_ref().unwrap().transform;
            assert!(preview.handle_event(&key(KeyCode::Char('c'))));
            let (text, removed) = draw_story(&mut preview, &model, (80, 24), story);
            assert!(text.contains("ADVENTURER"));
            assert!(removed.is_empty());
            assert_eq!(preview.scene.as_ref().unwrap().transform, transform);
            assert_eq!(preview.uploads(), 1);
            assert!(preview.handle_event(&key(KeyCode::Esc)));
            assert!(
                draw_story(&mut preview, &model, (80, 24), story)
                    .1
                    .is_empty()
            );
            let mut visited = std::collections::BTreeSet::new();
            for _ in 0..32 {
                visited.insert(
                    preview
                        .model
                        .as_ref()
                        .unwrap()
                        .selected_agent_key()
                        .cloned()
                        .unwrap(),
                );
                assert!(preview.handle_event(&key(KeyCode::Tab)));
            }
            assert_eq!(visited.len(), 32);
            for _ in 0..32 {
                assert!(preview.handle_event(&key(KeyCode::BackTab)));
            }
            assert_eq!(
                preview.model.as_ref().unwrap().selected_agent_key(),
                model.selected_agent_key()
            );
            assert!(!model.adventurer_card_visible());
        }
    }

    #[test]
    fn cache_reuses_static_frames_and_deletes_only_its_two_owned_ids_after_resize() {
        let model = hall_fit_fixture(StoryContext::fixed(), FitParty::Five);
        let mut p = FitPreview::with_ids(CellPixels::new(8, 16), String::new(), [41, 42]);
        let (first, removed) = draw(&mut p, &model, (80, 24));
        assert!(first.contains("a=T,U=1"));
        assert!(removed.is_empty());
        assert_eq!(p.uploads(), 1);
        for _ in 0..4 {
            assert!(draw(&mut p, &model, (80, 24)).1.is_empty());
        }
        assert_eq!(
            p.uploads(),
            1,
            "unchanged rendering must not reencode or upload"
        );
        let (_, removed) = draw(&mut p, &model, (96, 28));
        assert_eq!(removed, vec![42]);
        assert_eq!(p.uploads(), 2);
        assert_eq!(p.shutdown(), vec![41]);
        assert!(p.shutdown().is_empty());
        assert_eq!(delete_image(41), "\x1b_Ga=d,d=I,i=41,q=2;\x1b\\");
    }

    #[test]
    fn profiles_reuse_world_images_while_ledger_and_help_release_them() {
        let model = hall_fit_fixture(StoryContext::fixed(), FitParty::Five);
        let original = model.selected_agent_key().cloned();
        let mut p = FitPreview::with_ids(CellPixels::new(8, 16), String::new(), [51, 52]);
        draw(&mut p, &model, (80, 24));
        assert!(p.handle_event(&key(KeyCode::Tab)));
        assert_ne!(
            p.model.as_ref().unwrap().selected_agent_key(),
            original.as_ref()
        );
        assert_eq!(
            model.selected_agent_key(),
            original.as_ref(),
            "base fixture is immutable"
        );
        assert_eq!(draw(&mut p, &model, (80, 24)).1, vec![52]); // Selection changed the rune.
        assert!(p.handle_event(&key(KeyCode::Char('c'))));
        let (card, removed) = draw(&mut p, &model, (80, 24));
        assert!(card.contains("ADVENTURER"));
        assert!(removed.is_empty());
        assert!(p.scene.is_some(), "profile must retain the fitted world");
        assert!(p.handle_event(&key(KeyCode::Esc)));
        draw(&mut p, &model, (80, 24));
        assert_eq!(p.uploads(), 2);
        assert!(p.handle_event(&key(KeyCode::Char('L'))));
        let first_page = p.model.as_ref().unwrap().ledger_page();
        assert!(p.handle_event(&key(KeyCode::Char('j'))));
        assert_ne!(p.model.as_ref().unwrap().ledger_page(), first_page);
        let (ledger, removed) = draw(&mut p, &model, (80, 24));
        assert!(ledger.contains("LIBRARIAN'S LEDGER"));
        assert_eq!(removed, vec![51]);
        assert!(p.handle_event(&key(KeyCode::Esc)));
        draw(&mut p, &model, (80, 24));
        p.begin_frame(); // A frame that renders another story or help instead.
        assert_eq!(p.finish_frame(), vec![52]);
        draw(&mut p, &model, (80, 24));
        assert_eq!(p.uploads(), 4);
    }

    #[test]
    fn profile_keeps_fit_and_lease_for_both_fixture_rooms() {
        for view in [crate::app::View::Guild, crate::app::View::Delve] {
            let mut model = hall_fit_fixture(StoryContext::fixed(), FitParty::Five);
            model.switch_to(view);
            let mut preview = FitPreview::with_ids(CellPixels::new(8, 16), String::new(), [91, 92]);
            draw(&mut preview, &model, (120, 34));
            let transform = preview.scene.as_ref().unwrap().transform;
            assert!(preview.handle_event(&key(KeyCode::Char('c'))));
            let (text, removed) = draw(&mut preview, &model, (120, 34));
            assert!(text.contains("ADVENTURER"));
            assert!(removed.is_empty());
            assert_eq!(preview.scene.as_ref().unwrap().transform, transform);
            assert_eq!(preview.uploads(), 1);
            assert!(preview.handle_event(&key(KeyCode::Esc)));
            assert!(draw(&mut preview, &model, (120, 34)).1.is_empty());
            assert_eq!(preview.uploads(), 1);
            assert!(
                !model.adventurer_card_visible(),
                "fixture source must stay immutable"
            );
        }
    }

    #[test]
    fn mouse_hit_opens_only_the_local_fixture_card_and_ignores_letterboxing() {
        let model = hall_fit_fixture(StoryContext::fixed(), FitParty::Five);
        let mut p = FitPreview::with_ids(CellPixels::new(8, 16), String::new(), [71, 72]);
        draw(&mut p, &model, (96, 28));
        let key = model.domain().agents.keys().nth(1).unwrap().clone();
        let scene = p.scene.as_ref().unwrap();
        let (column, row) = scene
            .cells()
            .find(|(x, y)| scene.target_at(*x, *y) == Some(FitTarget::Adventurer(key.clone())))
            .unwrap();
        let click = |column, row| {
            Event::Mouse(crossterm::event::MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column,
                row,
                modifiers: KeyModifiers::NONE,
            })
        };
        assert!(
            !p.handle_event(&click(0, 0)),
            "letterboxing must not select an actor"
        );
        assert!(p.handle_event(&click(column, row)));
        assert_eq!(p.model.as_ref().unwrap().selected_agent_key(), Some(&key));
        assert!(p.model.as_ref().unwrap().adventurer_card_visible());
        assert_ne!(
            model.selected_agent_key(),
            Some(&key),
            "base fixture remains unchanged"
        );
    }

    #[test]
    fn unsupported_geometry_falls_back_while_crowded_stations_keep_the_fitted_party() {
        let model = hall_fit_fixture(StoryContext::fixed(), FitParty::Five);
        let mut p = FitPreview::unavailable("graphics unavailable");
        let (text, removed) = draw(&mut p, &model, (80, 24));
        assert!(text.contains("Fit fallback: graphics unavailable"));
        assert!(!text.contains("a=T,U=1"));
        assert_eq!(p.uploads(), 0);
        assert!(removed.is_empty());
        let crowded = hall_fit_fixture(StoryContext::fixed(), FitParty::Crowded);
        let mut p = FitPreview::with_ids(CellPixels::new(8, 16), String::new(), [61, 62]);
        let (text, removed) = draw(&mut p, &crowded, (96, 28));
        assert!(text.contains("11/11 adventurers"));
        assert!(text.contains("a=T,U=1"));
        assert_eq!(p.uploads(), 1);
        assert!(removed.is_empty());
    }
}
