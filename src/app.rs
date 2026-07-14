use clap::ValueEnum;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum View {
    #[default]
    Desk,
    Cafe,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Model {
    view: View,
}

impl Model {
    pub const fn new(view: View) -> Self {
        Self { view }
    }

    pub const fn view(&self) -> View {
        self.view
    }

    pub const fn switch_to(&mut self, view: View) {
        self.view = view;
    }
}
