use serde::{Deserialize, Serialize};

use crate::herdr::protocol::AgentInfo;

use super::PersonaKey;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdventurerPersona {
    pub key: PersonaKey,
    pub name: String,
    pub ancestry: Ancestry,
    pub class: AdventurerClass,
    pub epithet: Epithet,
    pub appearance: PersonaAppearance,
}

impl AdventurerPersona {
    #[must_use]
    pub fn for_key(key: PersonaKey) -> Self {
        const COMMON_ANCESTRIES: [Ancestry; 6] = [
            Ancestry::Human,
            Ancestry::Dwarf,
            Ancestry::Elf,
            Ancestry::Halfling,
            Ancestry::Orc,
            Ancestry::Gnome,
        ];
        const CLASSES: [AdventurerClass; 11] = [
            AdventurerClass::Barbarian,
            AdventurerClass::Bard,
            AdventurerClass::Cleric,
            AdventurerClass::Paladin,
            AdventurerClass::Ranger,
            AdventurerClass::Rogue,
            AdventurerClass::Wizard,
            AdventurerClass::Artificer,
            AdventurerClass::Runewright,
            AdventurerClass::Testmender,
            AdventurerClass::Pathseeker,
        ];
        const FIRST_NAMES: [&str; 12] = [
            "Elowen", "Merrin", "Arnoldus", "Pius", "Rowan", "Tamsin", "Brindle", "Nessa", "Orin",
            "Sabine", "Alder", "Lyra",
        ];
        const BYNAMES: [&str; 12] = [
            "Typeweaver",
            "Ironjaw",
            "Manytools",
            "Blackquill",
            "Brightward",
            "Mossfoot",
            "Runehand",
            "Mapkeeper",
            "Copperkettle",
            "Longpath",
            "Softstep",
            "Embercloak",
        ];
        const EPITHETS: [&str; 8] = [
            "Keeper of Schemas",
            "Mender of Tests",
            "Walker of Worktrees",
            "Delver of Forgotten Modules",
            "Breaker of Builds",
            "Reader of Runes",
            "Warden of Boundaries",
            "Cartographer of Call Stacks",
        ];

        let digest = labelled_hash(key.as_str(), "adventurer");
        let ancestry = if digest[0] == 0 {
            Ancestry::Goblin
        } else {
            COMMON_ANCESTRIES[usize::from(digest[0] - 1) % COMMON_ANCESTRIES.len()]
        };

        Self {
            name: format!(
                "{} {}",
                FIRST_NAMES[usize::from(digest[1]) % FIRST_NAMES.len()],
                BYNAMES[usize::from(digest[2]) % BYNAMES.len()],
            ),
            ancestry,
            class: CLASSES[usize::from(digest[3]) % CLASSES.len()],
            epithet: Epithet(EPITHETS[usize::from(digest[4]) % EPITHETS.len()].to_owned()),
            appearance: Self::appearance_for_key(&key),
            key,
        }
    }

    #[must_use]
    pub fn for_agent(agent: &AgentInfo, workspace_root: Option<&str>) -> Self {
        Self::for_key(PersonaKey::for_agent(agent, workspace_root))
    }

    #[must_use]
    pub fn appearance_for_key(key: &PersonaKey) -> PersonaAppearance {
        let digest = labelled_hash(key.as_str(), "appearance");
        PersonaAppearance {
            proportions: pick(BodyProportions::ALL, digest[0]),
            head_shape: pick(HeadShape::ALL, digest[1]),
            skin_tone: pick(SkinTone::ALL, digest[2]),
            hair: pick(HairShape::ALL, digest[3]),
            hair_tone: pick(HairTone::ALL, digest[4]),
            face_detail: pick(FaceDetail::ALL, digest[5]),
            garb: pick(Garb::ALL, digest[6]),
            legwear: pick(Legwear::ALL, digest[7]),
            footwear: pick(Footwear::ALL, digest[8]),
            keepsake: pick(Keepsake::ALL, digest[9]),
            accent: pick(AccentTone::ALL, digest[10]),
        }
    }
}

impl PersonaKey {
    #[must_use]
    pub fn for_agent(agent: &AgentInfo, workspace_root: Option<&str>) -> Self {
        let identity = if let Some(session) = &agent.agent_session {
            format!(
                "session\0{}\0{}\0{}\0{}",
                session.source, session.agent, session.kind, session.value
            )
        } else if let (Some(root), Some(name)) = (workspace_root, agent_name(agent)) {
            format!("workspace-agent\0{root}\0{name}")
        } else {
            format!("pane\0{}\0{}", agent.workspace_id, agent.pane_id)
        };
        let hash = blake3::hash(identity.as_bytes()).to_hex();
        Self::new(format!("persona-{}", &hash[..24]))
    }
}

fn agent_name(agent: &AgentInfo) -> Option<&str> {
    agent
        .name
        .as_deref()
        .or(agent.agent.as_deref())
        .or(agent.display_agent.as_deref())
}

fn labelled_hash(key: &str, label: &str) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(label.as_bytes());
    hasher.update(&[0]);
    hasher.update(key.as_bytes());
    *hasher.finalize().as_bytes()
}

fn pick<T: Copy>(choices: &[T], byte: u8) -> T {
    choices[usize::from(byte) % choices.len()]
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct PersonaAppearance {
    pub proportions: BodyProportions,
    pub head_shape: HeadShape,
    pub skin_tone: SkinTone,
    pub hair: HairShape,
    pub hair_tone: HairTone,
    pub face_detail: FaceDetail,
    pub garb: Garb,
    pub legwear: Legwear,
    pub footwear: Footwear,
    pub keepsake: Keepsake,
    pub accent: AccentTone,
}

macro_rules! exhaustive_enum {
    ($(#[$meta:meta])* pub enum $name:ident { $($variant:ident),+ $(,)? }) => {
        $(#[$meta])*
        pub enum $name { $($variant),+ }

        impl $name {
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];
        }
    };
}

exhaustive_enum! {
    #[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Ancestry { Human, Dwarf, Elf, Halfling, Orc, Gnome, Goblin }
}

exhaustive_enum! {
    #[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
    #[serde(rename_all = "snake_case")]
    pub enum AdventurerClass {
        Barbarian, Bard, Cleric, Paladin, Ranger, Rogue, Wizard, Artificer, Runewright,
        Testmender, Pathseeker
    }
}

impl AdventurerClass {
    #[must_use]
    pub const fn gear(self) -> AdventuringGear {
        match self {
            Self::Barbarian => AdventuringGear::Axe,
            Self::Bard => AdventuringGear::Lute,
            Self::Cleric => AdventuringGear::HolySymbol,
            Self::Paladin => AdventuringGear::Shield,
            Self::Ranger => AdventuringGear::BowAndQuiver,
            Self::Rogue => AdventuringGear::ThievesTools,
            Self::Wizard => AdventuringGear::SpellbookAndStaff,
            Self::Artificer => AdventuringGear::Toolkit,
            Self::Runewright => AdventuringGear::RuneChisel,
            Self::Testmender => AdventuringGear::TestKit,
            Self::Pathseeker => AdventuringGear::MapAndCompass,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Epithet(String);

impl Epithet {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

macro_rules! trait_enum {
    ($name:ident { $($variant:ident),+ $(,)? }) => {
        exhaustive_enum! {
            #[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
            #[serde(rename_all = "snake_case")]
            pub enum $name { $($variant),+ }
        }
    };
}

trait_enum!(BodyProportions {
    Compact,
    Average,
    Tall,
    Broad
});
trait_enum!(HeadShape {
    Round,
    Square,
    Long,
    Angular
});
trait_enum!(SkinTone {
    Porcelain,
    Rose,
    Sand,
    Umber,
    Sienna,
    Ebony
});
trait_enum!(HairShape {
    Crop,
    Fringe,
    Curls,
    Quiff,
    Bob,
    Spikes,
    Ponytail,
    Shaved
});
trait_enum!(HairTone {
    Black,
    Espresso,
    Chestnut,
    Copper,
    Gold,
    Silver
});
trait_enum!(FaceDetail {
    None,
    RoundGlasses,
    SquareGlasses,
    Visor,
    Freckles,
    Moustache
});
trait_enum!(Garb {
    Armour,
    Cloak,
    Doublet,
    Leathers,
    Robes,
    Vestments,
    WorkApron
});
trait_enum!(Legwear {
    BootsAndBreeches,
    Greaves,
    RobeHem,
    TravelingSkirt
});
trait_enum!(Footwear {
    Boots,
    Sabatons,
    Sandals,
    SoftShoes
});
trait_enum!(Keepsake {
    Feather,
    LuckyCoin,
    Mug,
    PressedLeaf,
    Ribbon,
    TinyFamiliar
});
trait_enum!(AdventuringGear {
    Axe,
    BowAndQuiver,
    HolySymbol,
    Lute,
    MapAndCompass,
    RuneChisel,
    Shield,
    SpellbookAndStaff,
    TestKit,
    ThievesTools,
    Toolkit
});
trait_enum!(AccentTone {
    Amber,
    Cyan,
    Lime,
    Magenta,
    Red,
    Blue,
    Violet,
    Teal
});
