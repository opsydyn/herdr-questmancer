use std::{collections::BTreeMap, fmt};

use image::DynamicImage;
use ratatui::layout::Size;
use ratatui_image::{Resize, picker::Picker, picker::ProtocolType, protocol::Protocol};

use crate::domain::{AdventurerClass, AdventurerPersona, Ancestry};

const CARD_PORTRAIT_SIZE: Size = Size::new(24, 16);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum PortraitKey {
    Ancestry(Ancestry),
    Class(AdventurerClass),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PortraitCapability {
    Unsupported,
    Kitty,
    Sixel,
    Iterm2,
}

impl PortraitCapability {
    #[must_use]
    pub const fn from_protocol(protocol: ProtocolType) -> Self {
        match protocol {
            ProtocolType::Halfblocks => Self::Unsupported,
            ProtocolType::Kitty => Self::Kitty,
            ProtocolType::Sixel => Self::Sixel,
            ProtocolType::Iterm2 => Self::Iterm2,
        }
    }

    #[must_use]
    pub const fn is_native(self) -> bool {
        !matches!(self, Self::Unsupported)
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Unsupported => "authored sprite fallback",
            Self::Kitty => "native Kitty",
            Self::Sixel => "native Sixel",
            Self::Iterm2 => "native iTerm2",
        }
    }
}

pub struct PortraitGallery {
    capability: PortraitCapability,
    portraits: BTreeMap<PortraitKey, Protocol>,
    librarian: Option<Protocol>,
    diagnostic: Option<String>,
}

impl PortraitGallery {
    /// Detect native terminal graphics and prepare all embedded card portraits.
    ///
    /// Detection or decoding failures deliberately produce an empty gallery so
    /// callers can render the canonical authored sprite instead.
    #[must_use]
    pub fn detect() -> Self {
        match Picker::from_query_stdio() {
            Ok(picker) => Self::from_picker(&picker),
            Err(error) => Self::fallback(format!("portrait capability detection failed: {error}")),
        }
    }

    #[must_use]
    pub fn fallback(diagnostic: impl Into<String>) -> Self {
        Self {
            capability: PortraitCapability::Unsupported,
            portraits: BTreeMap::new(),
            librarian: None,
            diagnostic: Some(diagnostic.into()),
        }
    }

    #[must_use]
    pub fn portrait_for(&self, persona: &AdventurerPersona) -> Option<&Protocol> {
        self.portraits
            .get(&PortraitKey::Ancestry(persona.ancestry))
            .or_else(|| self.portraits.get(&PortraitKey::Class(persona.class)))
    }

    #[must_use]
    pub const fn librarian(&self) -> Option<&Protocol> {
        self.librarian.as_ref()
    }

    #[must_use]
    pub const fn capability(&self) -> PortraitCapability {
        self.capability
    }

    #[must_use]
    pub fn diagnostic(&self) -> Option<&str> {
        self.diagnostic.as_deref()
    }

    pub(crate) fn from_picker(picker: &Picker) -> Self {
        let capability = PortraitCapability::from_protocol(picker.protocol_type());
        if !capability.is_native() {
            return Self {
                capability,
                portraits: BTreeMap::new(),
                librarian: None,
                diagnostic: None,
            };
        }

        let mut portraits = BTreeMap::new();
        let mut failures = Vec::new();
        for (key, bytes) in native_portrait_assets() {
            match prepare_portrait(picker, bytes) {
                Ok(protocol) => {
                    portraits.insert(key, protocol);
                }
                Err(error) => failures.push(format!("{key:?}: {error}")),
            }
        }
        let librarian = match prepare_portrait(picker, librarian_asset()) {
            Ok(protocol) => Some(protocol),
            Err(error) => {
                failures.push(format!("Librarian: {error}"));
                None
            }
        };

        Self {
            capability,
            portraits,
            librarian,
            diagnostic: (!failures.is_empty())
                .then(|| format!("portrait assets unavailable: {}", failures.join(", "))),
        }
    }
}

impl fmt::Debug for PortraitGallery {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PortraitGallery")
            .field("capability", &self.capability)
            .field("classes", &self.portraits.keys().collect::<Vec<_>>())
            .field("librarian", &self.librarian.is_some())
            .field("diagnostic", &self.diagnostic)
            .finish()
    }
}

#[must_use]
pub const fn librarian_asset() -> &'static [u8] {
    include_bytes!("assets/librarian.png")
}

#[must_use]
pub const fn portrait_asset(class: AdventurerClass) -> Option<&'static [u8]> {
    match class {
        AdventurerClass::Barbarian => Some(include_bytes!("assets/portraits/barbarian-card.png")),
        AdventurerClass::Bard => Some(include_bytes!("assets/portraits/bard-card.png")),
        AdventurerClass::Paladin => Some(include_bytes!("assets/portraits/paladin-card.png")),
        AdventurerClass::Rogue => Some(include_bytes!("assets/portraits/rogue-card.png")),
        AdventurerClass::Wizard => Some(include_bytes!("assets/portraits/wizard-card.png")),
        AdventurerClass::Cleric
        | AdventurerClass::Druid
        | AdventurerClass::Ranger
        | AdventurerClass::Artificer
        | AdventurerClass::Runewright
        | AdventurerClass::Testmender
        | AdventurerClass::Pathseeker => None,
    }
}

#[must_use]
pub const fn ancestry_portrait_asset(ancestry: Ancestry) -> Option<&'static [u8]> {
    match ancestry {
        Ancestry::Goblin => Some(include_bytes!("assets/portraits/goblin-card.png")),
        Ancestry::Orc => Some(include_bytes!("assets/portraits/orc-card.png")),
        Ancestry::Human
        | Ancestry::Dwarf
        | Ancestry::Elf
        | Ancestry::Halfling
        | Ancestry::Gnome => None,
    }
}

fn native_portrait_assets() -> [(PortraitKey, &'static [u8]); 7] {
    [
        (
            PortraitKey::Ancestry(Ancestry::Goblin),
            ancestry_portrait_asset(Ancestry::Goblin).expect("Goblin portrait is embedded"),
        ),
        (
            PortraitKey::Ancestry(Ancestry::Orc),
            ancestry_portrait_asset(Ancestry::Orc).expect("Orc portrait is embedded"),
        ),
        (
            PortraitKey::Class(AdventurerClass::Barbarian),
            portrait_asset(AdventurerClass::Barbarian).expect("Barbarian portrait is embedded"),
        ),
        (
            PortraitKey::Class(AdventurerClass::Bard),
            portrait_asset(AdventurerClass::Bard).expect("Bard portrait is embedded"),
        ),
        (
            PortraitKey::Class(AdventurerClass::Paladin),
            portrait_asset(AdventurerClass::Paladin).expect("Paladin portrait is embedded"),
        ),
        (
            PortraitKey::Class(AdventurerClass::Rogue),
            portrait_asset(AdventurerClass::Rogue).expect("Rogue portrait is embedded"),
        ),
        (
            PortraitKey::Class(AdventurerClass::Wizard),
            portrait_asset(AdventurerClass::Wizard).expect("Wizard portrait is embedded"),
        ),
    ]
}

fn prepare_portrait(picker: &Picker, bytes: &[u8]) -> Result<Protocol, String> {
    let image = decode_portrait(bytes)?;
    picker
        .new_protocol(image, CARD_PORTRAIT_SIZE, Resize::Fit(None))
        .map_err(|error| error.to_string())
}

fn decode_portrait(bytes: &[u8]) -> Result<DynamicImage, String> {
    image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approved_classes_and_goblin_have_embedded_native_portraits() {
        for class in AdventurerClass::ALL {
            assert_eq!(
                portrait_asset(*class).is_some(),
                matches!(
                    class,
                    AdventurerClass::Barbarian
                        | AdventurerClass::Bard
                        | AdventurerClass::Paladin
                        | AdventurerClass::Rogue
                        | AdventurerClass::Wizard
                ),
                "{class:?}"
            );
        }
        for ancestry in Ancestry::ALL {
            assert_eq!(
                ancestry_portrait_asset(*ancestry).is_some(),
                matches!(ancestry, Ancestry::Goblin | Ancestry::Orc),
                "{ancestry:?}"
            );
        }
    }

    #[test]
    fn every_native_portrait_is_a_decodable_transparent_three_by_four_png() {
        for (key, bytes) in native_portrait_assets() {
            let image = decode_portrait(bytes).expect("embedded portrait decodes");
            assert_eq!((image.width(), image.height()), (384, 512), "{key:?}");
            assert!(image.color().has_alpha(), "{key:?}");
            assert_eq!(image.to_rgba8().get_pixel(0, 0)[3], 0, "{key:?}");
        }
    }

    #[test]
    fn intermediary_capability_result_preserves_the_authored_sprite_fallback() {
        let gallery = PortraitGallery::from_picker(&Picker::halfblocks());
        let persona = AdventurerPersona::for_key(crate::domain::PersonaKey::new("barbarian"));

        assert_eq!(gallery.capability(), PortraitCapability::Unsupported);
        assert!(gallery.portrait_for(&persona).is_none());
        assert!(gallery.librarian().is_none());
    }

    #[test]
    fn native_picker_prepares_every_approved_asset_and_leaves_other_classes_unmapped() {
        let mut picker = Picker::halfblocks();
        picker.set_protocol_type(ProtocolType::Kitty);

        let gallery = PortraitGallery::from_picker(&picker);
        let mut barbarian = AdventurerPersona::for_key(crate::domain::PersonaKey::new("barbarian"));
        barbarian.class = AdventurerClass::Barbarian;
        barbarian.ancestry = Ancestry::Human;
        let mut rogue = barbarian.clone();
        rogue.class = AdventurerClass::Rogue;
        let mut bard = barbarian.clone();
        bard.class = AdventurerClass::Bard;
        let mut paladin = barbarian.clone();
        paladin.class = AdventurerClass::Paladin;
        let mut wizard = barbarian.clone();
        wizard.class = AdventurerClass::Wizard;
        let mut druid = barbarian.clone();
        druid.class = AdventurerClass::Druid;
        let mut goblin = druid.clone();
        goblin.ancestry = Ancestry::Goblin;
        let mut orc = druid.clone();
        orc.ancestry = Ancestry::Orc;

        assert_eq!(gallery.capability(), PortraitCapability::Kitty);
        assert!(gallery.portrait_for(&barbarian).is_some());
        assert!(gallery.portrait_for(&bard).is_some());
        assert!(gallery.portrait_for(&paladin).is_some());
        assert!(gallery.portrait_for(&rogue).is_some());
        assert!(gallery.portrait_for(&wizard).is_some());
        assert!(gallery.portrait_for(&goblin).is_some());
        assert!(gallery.portrait_for(&orc).is_some());
        assert!(gallery.portrait_for(&druid).is_none());
        assert!(gallery.librarian().is_some());
        assert!(gallery.diagnostic().is_none());
    }

    #[test]
    fn native_ancestry_portraits_take_priority_over_class_portraits() {
        let mut picker = Picker::halfblocks();
        picker.set_protocol_type(ProtocolType::Kitty);
        let gallery = PortraitGallery::from_picker(&picker);
        for (ancestry, class) in [
            (Ancestry::Goblin, AdventurerClass::Wizard),
            (Ancestry::Orc, AdventurerClass::Paladin),
        ] {
            let mut persona =
                AdventurerPersona::for_key(crate::domain::PersonaKey::new("ancestry"));
            persona.ancestry = ancestry;
            persona.class = class;

            let selected = gallery.portrait_for(&persona).expect("ancestry portrait");
            let portrait = gallery
                .portraits
                .get(&PortraitKey::Ancestry(ancestry))
                .expect("ancestry protocol");

            assert!(std::ptr::eq(selected, portrait), "{ancestry:?}");
        }
    }

    #[test]
    fn invalid_png_data_cannot_displace_the_sprite_fallback() {
        let mut picker = Picker::halfblocks();
        picker.set_protocol_type(ProtocolType::Kitty);

        assert!(prepare_portrait(&picker, b"not a png").is_err());
    }
}
