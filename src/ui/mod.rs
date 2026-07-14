mod theme;
mod views;

pub mod input;
pub mod pixel;

use ratatui::Frame;

use crate::app::{Modal, Model, View};

pub fn render(frame: &mut Frame<'_>, model: &Model) {
    match model.view() {
        View::Desk => views::desk::render(frame, model),
        View::Cafe => views::cafe::render(frame),
    }
    if matches!(model.modal(), Modal::Reply { .. } | Modal::Search { .. }) {
        views::reply::render(frame, model);
    }
}
