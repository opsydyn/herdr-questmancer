//! Shared native output adapter for the existing RGB worlds (ADR-0002/0003/0005).
use std::fmt;

use ratatui::{Frame, layout::Rect, widgets::Clear};
use ratatui_image::{
    picker::{Picker, ProtocolType},
    protocol::kitty::Kitty,
    sliced::{SignedPosition, SlicedImage, SlicedProtocol},
};

use super::{scene_adapter::flush_rgb, scene_overlays};
use crate::{
    app::{ColorMode, Modal, Model},
    portrait::PortraitGallery,
    scene::{
        SceneFrame,
        fit::{CellPixels, FitScene, FitTransform},
        pixel::{PixelSize, Rgb, RgbBuffer},
        presentation::ScenePresentation,
        render_scene_for_world,
        snapshot::SceneSnapshot,
    },
};

struct CachedImage {
    hash: blake3::Hash,
    transform: FitTransform,
    protocol: SlicedProtocol,
    id: u32,
}

/// Ephemeral image leases only; no domain, agent-command or persistence state.
pub struct WorldFit {
    geometry: Option<CellPixels>,
    ids: [u32; 2],
    slot: usize,
    cache: Option<CachedImage>,
    cleanup: Vec<u32>,
    painted: bool,
    uploads: usize,
    profile_mask: Option<Rect>,
}

impl fmt::Debug for WorldFit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorldFit")
            .field("geometry", &self.geometry)
            .field("uploads", &self.uploads)
            .finish_non_exhaustive()
    }
}

impl WorldFit {
    /// Runtime must call this with the shared, negotiated startup Picker.
    pub fn from_picker(picker: Option<&Picker>, tmux: bool) -> Self {
        let geometry = picker
            .filter(|p| !tmux && p.protocol_type() == ProtocolType::Kitty)
            .and_then(|p| CellPixels::new(p.font_size().width, p.font_size().height));
        Self::new(geometry)
    }

    /// Construct from confirmed cell geometry; also supports terminal-free fixtures.
    pub fn new(geometry: Option<CellPixels>) -> Self {
        let mut bytes = [0; 8];
        let geometry = geometry.filter(|c| CellPixels::new(c.width, c.height).is_some());
        let geometry = if getrandom::fill(&mut bytes).is_ok() {
            geometry
        } else {
            None
        };
        let a = u32::from_ne_bytes(bytes[..4].try_into().expect("four bytes")).max(1);
        let mut b = u32::from_ne_bytes(bytes[4..].try_into().expect("four bytes")).max(1);
        if a == b {
            b = a.wrapping_add(1).max(1);
        }
        Self {
            geometry,
            ids: [a, b],
            slot: 0,
            cache: None,
            cleanup: Vec::new(),
            painted: false,
            uploads: 0,
            profile_mask: None,
        }
    }

    /// Kernel geometry is safe to read while `EventStream` owns stdin. Missing
    /// pixel axes retain startup negotiation; font zoom then requires reopening.
    pub fn refresh_geometry(&mut self) {
        if self.geometry.is_some()
            && let Ok(size) = rustix::termios::tcgetwinsize(std::io::stdout())
            && let Some(cell) =
                cell_pixels(size.ws_col, size.ws_row, size.ws_xpixel, size.ws_ypixel)
        {
            self.geometry = Some(cell);
        }
    }

    pub const fn uploads(&self) -> usize {
        self.uploads
    }

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

    /// Use the last drawn profile footprint, so covered clicks cannot reach a
    /// hidden adventurer or dismiss the profile while reading it.
    pub fn action_for_event(
        &self,
        event: &crossterm::event::Event,
        model: &Model,
        scene: &SceneFrame,
    ) -> super::input::Action {
        if let crossterm::event::Event::Mouse(mouse) = event
            && self.profile_mask.is_some_and(|area| {
                area.contains(ratatui::layout::Position::new(mouse.column, mouse.row))
            })
        {
            return super::input::Action::None;
        }
        super::input::action_for_scene_event_in(event, model.modal(), scene)
    }

    fn paint(&mut self, frame: &mut Frame<'_>, model: &Model) -> Option<SceneFrame> {
        self.painted = false;
        if model.modal() != &Modal::None || model.preferences().color_mode == ColorMode::Ansi16 {
            return None;
        }
        let area = frame.area();
        let picture = Rect::new(
            area.x,
            area.y.saturating_add(1),
            area.width,
            area.height.saturating_sub(3),
        );
        let scene = FitScene::project(model, picture, self.geometry?).ok()?;
        let hash = blake3::hash(
            &scene
                .rgb
                .pixels()
                .iter()
                .flat_map(|p| [p.r, p.g, p.b])
                .collect::<Vec<_>>(),
        );
        if self
            .cache
            .as_ref()
            .is_none_or(|c| c.hash != hash || c.transform != scene.transform)
        {
            if let Some(old) = self.cache.take() {
                self.cleanup.push(old.id);
            }
            self.slot = (self.slot + 1) % 2;
            let id = self.ids[self.slot];
            let protocol = Kitty::new(
                scene.transform.image(&scene.rgb),
                scene.transform.area.as_size(),
                id,
                false,
            )
            .ok()?;
            self.cache = Some(CachedImage {
                hash,
                transform: scene.transform,
                protocol: SlicedProtocol::Kitty(protocol),
                id,
            });
            self.uploads += 1;
        }
        frame.render_widget(Clear, area);
        let visible = paint_image_bands(
            frame,
            &self.cache.as_ref()?.protocol,
            scene.transform.area,
            self.profile_mask,
        );
        scene_overlays::render_party_status(frame, model);
        self.painted = true;
        let mut rendered = scene.terminal_frame();
        if !visible {
            rendered.next_frame_in = None;
        }
        Some(rendered)
    }
}

/// Derive axes only from a complete, exact kernel geometry report.
pub fn cell_pixels(cols: u16, rows: u16, width: u16, height: u16) -> Option<CellPixels> {
    if cols == 0 || rows == 0 || !width.is_multiple_of(cols) || !height.is_multiple_of(rows) {
        return None;
    }
    CellPixels::new(width / cols, height / rows)
}

/// Narrow deletion only; never delete another component's images or placements.
pub fn delete_image(id: u32) -> String {
    format!("\x1b_Ga=d,d=I,i={id},q=2;\x1b\\")
}

/// Both native and fallback output share the production overlay and input paths.
pub fn render_application(
    frame: &mut Frame<'_>,
    model: &Model,
    buffer: &mut RgbBuffer,
    portraits: Option<&PortraitGallery>,
    graphics: &mut WorldFit,
) -> SceneFrame {
    let presentation = ScenePresentation::from_model(model);
    graphics.profile_mask = profile_mask(frame.area(), model);
    let mut scene = if let Some(scene) = graphics.paint(frame, model) {
        scene
    } else {
        let mut area = frame.area();
        let status = model.modal() == &Modal::None && area.height >= 16 && area.width >= 24;
        if status {
            area.height -= 2;
        }
        let scene = render_scene_for_world(
            &SceneSnapshot::from_model(model),
            &presentation,
            PixelSize::new(area.width, area.height.saturating_mul(2)),
            buffer,
        );
        flush_rgb(
            frame.buffer_mut(),
            area,
            buffer,
            Rgb::BLACK,
            model.preferences().color_mode,
        );
        scene_overlays::render_scene_identity_labels(frame, model, &scene);
        if status {
            scene_overlays::render_party_status(frame, model);
        }
        scene
    };
    scene_overlays::render_scene_overlays(frame, model, &presentation, portraits);
    mask_scene_regions(&mut scene, graphics.profile_mask);
    scene
}

/// Include the existing one-column right gutter so all exposed bands start at
/// source column zero. The profile rectangle itself remains unchanged.
pub(crate) fn profile_mask(area: Rect, model: &Model) -> Option<Rect> {
    scene_overlays::adventurer_card_area(area, model).map(|card| {
        Rect::new(
            card.x,
            card.y,
            area.right().saturating_sub(card.x),
            card.height,
        )
    })
}

/// Render the same virtual image through visible row bands. A band's first cell
/// always survives the profile overlay, including the initial upload prefix.
pub(crate) fn paint_image_bands(
    frame: &mut Frame<'_>,
    protocol: &SlicedProtocol,
    world: Rect,
    mask: Option<Rect>,
) -> bool {
    let covered = mask
        .map(|mask| world.intersection(mask))
        .filter(|r| !r.is_empty());
    let bands = covered.map_or_else(
        || vec![world],
        |covered| {
            vec![
                Rect::new(world.x, world.y, world.width, covered.y - world.y),
                Rect::new(world.x, covered.y, covered.x - world.x, covered.height),
                Rect::new(
                    world.x,
                    covered.bottom(),
                    world.width,
                    world.bottom() - covered.bottom(),
                ),
            ]
        },
    );
    let mut visible = false;
    for band in bands.into_iter().filter(|band| !band.is_empty()) {
        let skip = i16::try_from(band.y - world.y).expect("image rows bounded to 297");
        frame.render_widget(
            SlicedImage::new(protocol, SignedPosition::from((0, -skip))),
            band,
        );
        visible = true;
    }
    visible
}

/// Click targets belong only to complete, visible actors. Keyboard selection
/// still traverses the whole domain party while a profile covers its neighbours.
pub(crate) fn mask_scene_regions(scene: &mut SceneFrame, mask: Option<Rect>) {
    let Some(mask) = mask else {
        return;
    };
    let intersects = |bounds: crate::scene::pixel::PixelRect| {
        bounds.x < i32::from(mask.right())
            && bounds.x + i32::from(bounds.width) > i32::from(mask.x)
            && bounds.y < i32::from(mask.bottom()) * 2
            && bounds.y + i32::from(bounds.height) > i32::from(mask.y) * 2
    };
    scene.actors.retain(|actor| !intersects(actor.bounds));
    scene.interactables.retain(|item| !intersects(item.bounds));
}
