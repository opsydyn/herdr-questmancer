//! Integer-pixel world projection shared by production and Storybook (ADR-0002/0003).
use image::{DynamicImage, RgbImage};
use ratatui::layout::Rect;

use crate::{
    app::Model,
    domain::{AgentKey, Presence},
    scene::{
        SceneFrame,
        pixel::{PixelPoint, PixelRect, Rgb, RgbBuffer},
        presentation::ScenePresentation,
        render_fitted_scene_for_world,
        snapshot::SceneSnapshot,
    },
};

/// Confirmed physical pixels per terminal cell; zero/implausible axes are rejected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CellPixels {
    pub width: u16,
    pub height: u16,
}

impl CellPixels {
    pub fn new(width: u16, height: u16) -> Option<Self> {
        ((1..=256).contains(&width) && (1..=256).contains(&height))
            .then_some(Self { width, height })
    }
}

/// A single transform owns both display pixels and mouse-cell interpretation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FitTransform {
    pub area: Rect,
    pub cell: CellPixels,
    pub scale: u16,
    pub padding: (u32, u32),
}

impl FitTransform {
    pub fn new(available: Rect, cell: CellPixels) -> Option<Self> {
        CellPixels::new(cell.width, cell.height)?;
        // A 16x ceiling bounds the uploaded RGBA canvas below 4 Mi pixels.
        let scale = (u32::from(available.width) * u32::from(cell.width) / 160)
            .min(u32::from(available.height) * u32::from(cell.height) / 90)
            .min(16);
        if scale == 0 {
            return None;
        }
        let width = u16::try_from((160 * scale).div_ceil(u32::from(cell.width))).ok()?;
        let height = u16::try_from((90 * scale).div_ceil(u32::from(cell.height))).ok()?;
        let pixel_width = u32::from(width) * u32::from(cell.width);
        let pixel_height = u32::from(height) * u32::from(cell.height);
        if pixel_width * pixel_height > 4_000_000 || height > 297 {
            return None;
        }
        Some(Self {
            area: Rect::new(
                available.x + (available.width - width) / 2,
                available.y + (available.height - height) / 2,
                width,
                height,
            ),
            cell,
            scale: u16::try_from(scale).ok()?,
            padding: (
                (pixel_width - 160 * scale) / 2,
                (pixel_height - 90 * scale) / 2,
            ),
        })
    }

    /// Mouse events supply cells, so sampling uses the cell's physical centre.
    pub fn source_at_cell(self, column: u16, row: u16) -> Option<PixelPoint> {
        if column < self.area.x
            || column >= self.area.right()
            || row < self.area.y
            || row >= self.area.bottom()
        {
            return None;
        }
        let x = u32::from(column - self.area.x) * u32::from(self.cell.width)
            + u32::from(self.cell.width) / 2;
        let y = u32::from(row - self.area.y) * u32::from(self.cell.height)
            + u32::from(self.cell.height) / 2;
        let x = x.checked_sub(self.padding.0)? / u32::from(self.scale);
        let y = y.checked_sub(self.padding.1)? / u32::from(self.scale);
        (x < 160 && y < 90).then(|| {
            PixelPoint::new(
                i32::try_from(x).expect("bounded world x"),
                i32::try_from(y).expect("bounded world y"),
            )
        })
    }

    /// Exact nearest-neighbour expansion; no source pixel is discarded.
    pub fn image(self, rgb: &RgbBuffer) -> DynamicImage {
        let width = u32::from(self.area.width) * u32::from(self.cell.width);
        let height = u32::from(self.area.height) * u32::from(self.cell.height);
        let image = RgbImage::from_fn(width, height, |x, y| {
            let colour = x
                .checked_sub(self.padding.0)
                .zip(y.checked_sub(self.padding.1))
                .and_then(|(x, y)| {
                    let x = i32::try_from(x / u32::from(self.scale)).ok()?;
                    let y = i32::try_from(y / u32::from(self.scale)).ok()?;
                    rgb.get(x, y)
                })
                .unwrap_or(Rgb::new(12, 16, 22));
            image::Rgb([colour.r, colour.g, colour.b])
        });
        DynamicImage::ImageRgb8(image)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum FitFailure {
    #[error("pane too small or image budget exceeded")]
    Geometry,
    #[error("canonical composition shows {visible}/{total}; using responsive fallback")]
    Incomplete { visible: usize, total: usize },
    #[error("an actor has no unambiguous mouse-cell centre at this scale")]
    HitGeometry,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FitTarget {
    Adventurer(AgentKey),
    Librarian,
}

#[derive(Clone, Debug)]
pub struct FitScene {
    pub rgb: RgbBuffer,
    pub frame: SceneFrame,
    pub transform: FitTransform,
}

fn contains(bounds: PixelRect, point: PixelPoint) -> bool {
    point.x >= bounds.x
        && point.y >= bounds.y
        && point.x < bounds.x + i32::from(bounds.width)
        && point.y < bounds.y + i32::from(bounds.height)
}

impl FitScene {
    pub fn project(model: &Model, area: Rect, cell: CellPixels) -> Result<Self, FitFailure> {
        Self::project_with(model, area, cell, render_fitted_scene_for_world)
    }

    #[cfg(feature = "storybook")]
    pub(crate) fn project_study(
        model: &Model,
        area: Rect,
        cell: CellPixels,
        study: super::RoomStudy,
    ) -> Result<Self, FitFailure> {
        Self::project_with(model, area, cell, |snapshot, presentation, rgb| {
            super::render_study_frame(
                snapshot,
                presentation,
                study,
                super::pixel::PixelSize::new(160, 90),
                rgb,
            )
        })
    }

    fn project_with(
        model: &Model,
        area: Rect,
        cell: CellPixels,
        render: impl FnOnce(&SceneSnapshot, &ScenePresentation, &mut RgbBuffer) -> SceneFrame,
    ) -> Result<Self, FitFailure> {
        let transform = FitTransform::new(area, cell).ok_or(FitFailure::Geometry)?;
        let snapshot = SceneSnapshot::from_model(model);
        let total = snapshot
            .agents
            .iter()
            .filter(|a| a.presence != Presence::Exited)
            .count();
        let presentation = ScenePresentation::from_model(model);
        let mut rgb = RgbBuffer::filled(160, 90, Rgb::BLACK);
        let frame = render(&snapshot, &presentation, &mut rgb);
        if frame.actors.len() != total {
            return Err(FitFailure::Incomplete {
                visible: frame.actors.len(),
                total,
            });
        }
        let scene = Self {
            rgb,
            frame,
            transform,
        };
        // Every returned actor must remain addressable with cell-based input.
        for actor in &scene.frame.actors {
            if !scene.cells().any(|(x, y)| {
                matches!(scene.target_at(x,y),
                Some(FitTarget::Adventurer(key)) if key == actor.agent)
            }) {
                return Err(FitFailure::HitGeometry);
            }
        }
        if !scene.frame.interactables.is_empty()
            && !scene
                .cells()
                .any(|(x, y)| scene.target_at(x, y) == Some(FitTarget::Librarian))
        {
            return Err(FitFailure::HitGeometry);
        }
        Ok(scene)
    }

    /// Convert hit regions to the existing half-block input coordinate system.
    /// Region edges use the very same cell-centre samples as `target_at`.
    pub fn terminal_frame(&self) -> SceneFrame {
        let mut frame = self.frame.clone();
        frame.actors = frame
            .actors
            .into_iter()
            .filter_map(|mut actor| {
                actor.bounds = self.terminal_bounds(actor.bounds)?;
                Some(actor)
            })
            .collect();
        frame.interactables = frame
            .interactables
            .into_iter()
            .filter_map(|mut item| {
                item.bounds = self.terminal_bounds(item.bounds)?;
                Some(item)
            })
            .collect();
        frame
    }

    fn terminal_bounds(&self, bounds: PixelRect) -> Option<PixelRect> {
        let mut points = self.cells().filter(|(x, y)| {
            self.transform
                .source_at_cell(*x, *y)
                .is_some_and(|p| contains(bounds, p))
        });
        let (x, y) = points.next()?;
        let (mut left, mut right, mut top, mut bottom) = (x, x, y, y);
        for (x, y) in points {
            left = left.min(x);
            right = right.max(x);
            top = top.min(y);
            bottom = bottom.max(y);
        }
        Some(PixelRect::new(
            i32::from(left),
            i32::from(top) * 2,
            right - left + 1,
            (bottom - top + 1) * 2,
        ))
    }

    pub fn cells(&self) -> impl Iterator<Item = (u16, u16)> {
        let area = self.transform.area;
        (area.y..area.bottom()).flat_map(move |y| (area.x..area.right()).map(move |x| (x, y)))
    }

    pub fn target_at(&self, column: u16, row: u16) -> Option<FitTarget> {
        let point = self.transform.source_at_cell(column, row)?;
        self.frame
            .actors
            .iter()
            .rev()
            .find(|actor| contains(actor.bounds, point))
            .map(|actor| FitTarget::Adventurer(actor.agent.clone()))
            .or_else(|| {
                self.frame
                    .interactables
                    .iter()
                    .any(|item| contains(item.bounds, point))
                    .then_some(FitTarget::Librarian)
            })
    }
}
