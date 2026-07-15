mod theme;
mod views;

pub mod cafe_scene;
pub mod copy;
pub mod input;
pub mod persona;
pub mod pixel;
pub mod theatre;
pub mod widgets;

use ratatui::Frame;

use crate::app::{Modal, Model, View};

pub fn render(frame: &mut Frame<'_>, model: &Model) {
    match model.view() {
        View::Guild => views::guild_hall::render(frame, model),
        View::Delve => views::cafe::render(frame, model),
    }
    if matches!(model.modal(), Modal::Counsel { .. } | Modal::Search { .. }) {
        views::reply::render(frame, model);
    }
}
