use std::{collections::BTreeMap, fmt};

use image::DynamicImage;
use ratatui::layout::Size;
use ratatui_image::{Resize, picker::Picker, picker::ProtocolType, protocol::Protocol};

use crate::domain::{AdventurerClass, AdventurerPersona};

const CARD_PORTRAIT_SIZE: Size = Size::new(24, 16);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum PortraitKey {
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
        self.portraits.get(&PortraitKey::Class(persona.class))
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
        AdventurerClass::Artificer => Some(include_bytes!("assets/portraits/artificer-card.png")),
        AdventurerClass::Barbarian => Some(include_bytes!("assets/portraits/barbarian-card.png")),
        AdventurerClass::Bard => Some(include_bytes!("assets/portraits/bard-card.png")),
        AdventurerClass::Cleric => Some(include_bytes!("assets/portraits/cleric-card.png")),
        AdventurerClass::Druid => Some(include_bytes!("assets/portraits/druid-card.png")),
        AdventurerClass::Paladin => Some(include_bytes!("assets/portraits/paladin-card.png")),
        AdventurerClass::Ranger => Some(include_bytes!("assets/portraits/ranger-card.png")),
        AdventurerClass::Rogue => Some(include_bytes!("assets/portraits/rogue-card.png")),
        AdventurerClass::Testmender => Some(include_bytes!("assets/portraits/testmender-card.png")),
        AdventurerClass::Wizard => Some(include_bytes!("assets/portraits/wizard-card.png")),
        AdventurerClass::Runewright | AdventurerClass::Pathseeker => None,
    }
}

fn native_portrait_assets() -> [(PortraitKey, &'static [u8]); 10] {
    [
        (
            PortraitKey::Class(AdventurerClass::Artificer),
            portrait_asset(AdventurerClass::Artificer).expect("Artificer portrait is embedded"),
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
            PortraitKey::Class(AdventurerClass::Cleric),
            portrait_asset(AdventurerClass::Cleric).expect("Cleric portrait is embedded"),
        ),
        (
            PortraitKey::Class(AdventurerClass::Druid),
            portrait_asset(AdventurerClass::Druid).expect("Druid portrait is embedded"),
        ),
        (
            PortraitKey::Class(AdventurerClass::Paladin),
            portrait_asset(AdventurerClass::Paladin).expect("Paladin portrait is embedded"),
        ),
        (
            PortraitKey::Class(AdventurerClass::Ranger),
            portrait_asset(AdventurerClass::Ranger).expect("Ranger portrait is embedded"),
        ),
        (
            PortraitKey::Class(AdventurerClass::Rogue),
            portrait_asset(AdventurerClass::Rogue).expect("Rogue portrait is embedded"),
        ),
        (
            PortraitKey::Class(AdventurerClass::Testmender),
            portrait_asset(AdventurerClass::Testmender).expect("Testmender portrait is embedded"),
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
    use crate::domain::Ancestry;

    #[test]
    fn approved_classes_have_embedded_native_portraits() {
        for class in AdventurerClass::ALL {
            assert_eq!(
                portrait_asset(*class).is_some(),
                matches!(
                    class,
                    AdventurerClass::Artificer
                        | AdventurerClass::Barbarian
                        | AdventurerClass::Bard
                        | AdventurerClass::Cleric
                        | AdventurerClass::Druid
                        | AdventurerClass::Paladin
                        | AdventurerClass::Ranger
                        | AdventurerClass::Rogue
                        | AdventurerClass::Testmender
                        | AdventurerClass::Wizard
                ),
                "{class:?}"
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
        let mut artificer = barbarian.clone();
        artificer.class = AdventurerClass::Artificer;
        let mut cleric = barbarian.clone();
        cleric.class = AdventurerClass::Cleric;
        let mut ranger = barbarian.clone();
        ranger.class = AdventurerClass::Ranger;
        let mut testmender = barbarian.clone();
        testmender.class = AdventurerClass::Testmender;
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
        let mut pathseeker = barbarian.clone();
        pathseeker.class = AdventurerClass::Pathseeker;
        let mut goblin_wizard = wizard.clone();
        goblin_wizard.ancestry = Ancestry::Goblin;
        let mut orc_ranger = ranger.clone();
        orc_ranger.ancestry = Ancestry::Orc;
        let mut orc_runewright = pathseeker.clone();
        orc_runewright.ancestry = Ancestry::Orc;
        orc_runewright.class = AdventurerClass::Runewright;

        assert_eq!(gallery.capability(), PortraitCapability::Kitty);
        assert!(gallery.portrait_for(&artificer).is_some());
        assert!(gallery.portrait_for(&barbarian).is_some());
        assert!(gallery.portrait_for(&cleric).is_some());
        assert!(gallery.portrait_for(&ranger).is_some());
        assert!(gallery.portrait_for(&testmender).is_some());
        assert!(gallery.portrait_for(&bard).is_some());
        assert!(gallery.portrait_for(&paladin).is_some());
        assert!(gallery.portrait_for(&rogue).is_some());
        assert!(gallery.portrait_for(&wizard).is_some());
        assert!(gallery.portrait_for(&druid).is_some());
        assert!(gallery.portrait_for(&goblin_wizard).is_some());
        assert!(gallery.portrait_for(&orc_ranger).is_some());
        assert!(gallery.portrait_for(&orc_runewright).is_none());
        assert!(gallery.portrait_for(&pathseeker).is_none());
        assert!(gallery.librarian().is_some());
        assert!(gallery.diagnostic().is_none());
    }

    #[test]
    fn native_class_portraits_take_priority_over_ancestry_portraits() {
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

            let selected = gallery.portrait_for(&persona).expect("class portrait");
            let portrait = gallery
                .portraits
                .get(&PortraitKey::Class(class))
                .expect("class protocol");

            assert!(std::ptr::eq(selected, portrait), "{ancestry:?} {class:?}");
        }
    }

    #[test]
    fn ancestry_portraits_are_reserved_and_never_selected_for_regular_adventurers() {
        let mut picker = Picker::halfblocks();
        picker.set_protocol_type(ProtocolType::Kitty);
        let gallery = PortraitGallery::from_picker(&picker);
        let mut persona =
            AdventurerPersona::for_key(crate::domain::PersonaKey::new("ancestry-fallback"));
        persona.ancestry = Ancestry::Orc;
        persona.class = AdventurerClass::Runewright;

        assert!(gallery.portrait_for(&persona).is_none());
    }

    #[test]
    fn invalid_png_data_cannot_displace_the_sprite_fallback() {
        let mut picker = Picker::halfblocks();
        picker.set_protocol_type(ProtocolType::Kitty);

        assert!(prepare_portrait(&picker, b"not a png").is_err());
    }
}
