mod theme;
mod views;

use ratatui::Frame;

use crate::app::{Model, View};

pub fn render(frame: &mut Frame<'_>, model: &Model) {
    match model.view() {
        View::Desk => views::desk::render(frame),
        View::Cafe => views::cafe::render(frame),
    }
}
