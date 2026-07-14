use serde::{Deserialize, Serialize};

use crate::herdr::protocol::AgentInfo;

use super::PersonaKey;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentPersona {
    pub key: PersonaKey,
    pub handle: String,
    pub appearance: PersonaAppearance,
}

impl AgentPersona {
    #[must_use]
    pub fn for_agent(agent: &AgentInfo, workspace_root: Option<&str>) -> Self {
        let key = PersonaKey::for_agent(agent, workspace_root);
        let name = agent_name(agent).unwrap_or("webmaster");
        Self {
            handle: handle_for_key(&key, name),
            appearance: Self::appearance_for_key(&key),
            key,
        }
    }

    #[must_use]
    pub fn appearance_for_key(key: &PersonaKey) -> PersonaAppearance {
        let digest = labelled_hash(key.as_str(), "appearance");
        PersonaAppearance {
            proportions: pick(&BODY_PROPORTIONS, digest[0]),
            head_shape: pick(&HEAD_SHAPES, digest[1]),
            skin_tone: pick(&SKIN_TONES, digest[2]),
            hair: pick(&HAIR_SHAPES, digest[3]),
            hair_tone: pick(&HAIR_TONES, digest[4]),
            face_detail: pick(&FACE_DETAILS, digest[5]),
            top: pick(&OUTFIT_TOPS, digest[6]),
            bottom: pick(&OUTFIT_BOTTOMS, digest[7]),
            shoes: pick(&SHOES, digest[8]),
            accessory: pick(&ACCESSORIES, digest[9]),
            desk_prop: pick(&DESK_PROPS, digest[10]),
            accent: pick(&ACCENT_TONES, digest[11]),
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

fn handle_for_key(key: &PersonaKey, agent_name: &str) -> String {
    const JOINERS: &[&str] = &["web_ring", "master_2000", "online", "dot_com", "56k"];
    let digest = labelled_hash(key.as_str(), "handle");
    let base = slug(agent_name);
    let joiner = JOINERS[usize::from(digest[0]) % JOINERS.len()];
    let number = u16::from_le_bytes([digest[1], digest[2]]) % 100;
    match digest[3] % 4 {
        0 => format!("xX_{base}_{joiner}_Xx"),
        1 => format!("{base}_{joiner}_{number:02}"),
        2 => format!("~{base}_{joiner}~"),
        _ => format!("{base}@mums_house"),
    }
}

fn slug(value: &str) -> String {
    let slug = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>();
    let trimmed = slug.trim_matches('_');
    if trimmed.is_empty() {
        "agent".to_owned()
    } else {
        trimmed.to_owned()
    }
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
    pub top: OutfitTop,
    pub bottom: OutfitBottom,
    pub shoes: Shoes,
    pub accessory: Accessory,
    pub desk_prop: DeskProp,
    pub accent: AccentTone,
}

macro_rules! trait_enum {
    ($name:ident { $($variant:ident),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $name { $($variant),+ }
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
trait_enum!(OutfitTop {
    BandTee,
    StripeJumper,
    HighCollar,
    WorkShirt,
    Hoodie,
    Cardigan,
    Waistcoat,
    TrackTop
});
trait_enum!(OutfitBottom {
    Jeans,
    Slacks,
    Cargos,
    Skirt,
    Shorts
});
trait_enum!(Shoes {
    Trainers,
    Boots,
    Loafers,
    HighTops,
    Platforms
});
trait_enum!(Accessory {
    Headphones,
    Pager,
    Lanyard,
    Wristband,
    Scarf,
    Badge,
    PocketPen,
    ShoulderBag
});
trait_enum!(DeskProp {
    NoveltyMug,
    FloppyStack,
    DeskFan,
    PizzaBox,
    Joystick,
    Phone,
    Manual,
    TinyCactus
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

const BODY_PROPORTIONS: [BodyProportions; 4] = [
    BodyProportions::Compact,
    BodyProportions::Average,
    BodyProportions::Tall,
    BodyProportions::Broad,
];
const HEAD_SHAPES: [HeadShape; 4] = [
    HeadShape::Round,
    HeadShape::Square,
    HeadShape::Long,
    HeadShape::Angular,
];
const SKIN_TONES: [SkinTone; 6] = [
    SkinTone::Porcelain,
    SkinTone::Rose,
    SkinTone::Sand,
    SkinTone::Umber,
    SkinTone::Sienna,
    SkinTone::Ebony,
];
const HAIR_SHAPES: [HairShape; 8] = [
    HairShape::Crop,
    HairShape::Fringe,
    HairShape::Curls,
    HairShape::Quiff,
    HairShape::Bob,
    HairShape::Spikes,
    HairShape::Ponytail,
    HairShape::Shaved,
];
const HAIR_TONES: [HairTone; 6] = [
    HairTone::Black,
    HairTone::Espresso,
    HairTone::Chestnut,
    HairTone::Copper,
    HairTone::Gold,
    HairTone::Silver,
];
const FACE_DETAILS: [FaceDetail; 6] = [
    FaceDetail::None,
    FaceDetail::RoundGlasses,
    FaceDetail::SquareGlasses,
    FaceDetail::Visor,
    FaceDetail::Freckles,
    FaceDetail::Moustache,
];
const OUTFIT_TOPS: [OutfitTop; 8] = [
    OutfitTop::BandTee,
    OutfitTop::StripeJumper,
    OutfitTop::HighCollar,
    OutfitTop::WorkShirt,
    OutfitTop::Hoodie,
    OutfitTop::Cardigan,
    OutfitTop::Waistcoat,
    OutfitTop::TrackTop,
];
const OUTFIT_BOTTOMS: [OutfitBottom; 5] = [
    OutfitBottom::Jeans,
    OutfitBottom::Slacks,
    OutfitBottom::Cargos,
    OutfitBottom::Skirt,
    OutfitBottom::Shorts,
];
const SHOES: [Shoes; 5] = [
    Shoes::Trainers,
    Shoes::Boots,
    Shoes::Loafers,
    Shoes::HighTops,
    Shoes::Platforms,
];
const ACCESSORIES: [Accessory; 8] = [
    Accessory::Headphones,
    Accessory::Pager,
    Accessory::Lanyard,
    Accessory::Wristband,
    Accessory::Scarf,
    Accessory::Badge,
    Accessory::PocketPen,
    Accessory::ShoulderBag,
];
const DESK_PROPS: [DeskProp; 8] = [
    DeskProp::NoveltyMug,
    DeskProp::FloppyStack,
    DeskProp::DeskFan,
    DeskProp::PizzaBox,
    DeskProp::Joystick,
    DeskProp::Phone,
    DeskProp::Manual,
    DeskProp::TinyCactus,
];
const ACCENT_TONES: [AccentTone; 8] = [
    AccentTone::Amber,
    AccentTone::Cyan,
    AccentTone::Lime,
    AccentTone::Magenta,
    AccentTone::Red,
    AccentTone::Blue,
    AccentTone::Violet,
    AccentTone::Teal,
];
