mod theme;
mod views;

pub mod cafe_scene;
pub mod input;
pub mod persona;
pub mod pixel;
pub mod theatre;
pub mod widgets;

use ratatui::Frame;

use crate::app::{Modal, Model, View};

pub fn render(frame: &mut Frame<'_>, model: &Model) {
    match model.view() {
        View::Desk => views::desk::render(frame, model),
        View::Cafe => views::cafe::render(frame, model),
    }
    if matches!(model.modal(), Modal::Reply { .. } | Modal::Search { .. }) {
        views::reply::render(frame, model);
    }
}
