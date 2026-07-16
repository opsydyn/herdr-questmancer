use crate::{
    app::{DisplayPreferences, Model},
    domain::Agent,
    ui::{
        pixel::{Canvas, ColorRole, Palette},
        theatre::TheatreFrame,
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StoryContext;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AtlasTile {
    pub label: &'static str,
    pub preferred_width: u16,
    pub preferred_height: u16,
    pub content: AtlasContent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AtlasContent {
    Pixel {
        canvas: Canvas,
        palette: Palette,
        background: ColorRole,
    },
    AdventurerCard {
        agent: Agent,
        theatre: TheatreFrame,
        preferences: DisplayPreferences,
    },
    Chamber {
        agent: Agent,
        theatre: TheatreFrame,
        selected: bool,
        preferences: DisplayPreferences,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetAtlas {
    pub tiles: Vec<AtlasTile>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(
    clippy::large_enum_variant,
    reason = "the fixture boundary intentionally stores the exact Model payload"
)]
pub enum StoryFixture {
    Application(Model),
    AssetAtlas(AssetAtlas),
}
