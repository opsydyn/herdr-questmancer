use crate::{
    domain::{AdventurerPersona, PersonaKey},
    ui::{
        persona::{compose_chamber_adventurer_for_palette, compose_profile_adventurer},
        pixel::{Canvas, ColorRole, Palette},
        theatre::{TheatreFrame, TheatrePose},
    },
};

use super::{
    AssetId,
    assets::{
        ACCENT_TONES, ANCESTRIES, BODY_PROPORTIONS, CLASSES, COLOR_ROLES, FACE_DETAILS, FOOTWEAR,
        GARBS, HAIR_SHAPES, HAIR_TONES, HEAD_SHAPES, KEEPSAKES, LEGWEAR, POSES, SKIN_TONES,
    },
    fixtures::{AssetAtlas, AtlasContent, AtlasTile, StoryContext, StoryFixture},
};

pub fn profile_tile(label: &'static str, mutate: impl FnOnce(&mut AdventurerPersona)) -> AtlasTile {
    let mut persona = AdventurerPersona::for_key(PersonaKey::new("storybook-atlas"));
    mutate(&mut persona);
    AtlasTile {
        label,
        preferred_width: 18,
        preferred_height: 18,
        content: AtlasContent::pixel(
            compose_profile_adventurer(&persona),
            Palette::Xterm256,
            ColorRole::DarkStone,
        ),
    }
}

pub fn chamber_tile(label: &'static str, pose: TheatrePose, animation_frame: u8) -> AtlasTile {
    let persona = AdventurerPersona::for_key(PersonaKey::new("storybook-pose-atlas"));
    AtlasTile {
        label,
        preferred_width: 14,
        preferred_height: 8,
        content: AtlasContent::pixel(
            compose_chamber_adventurer_for_palette(
                &persona,
                TheatreFrame {
                    pose,
                    animation_frame,
                    focused: false,
                    label,
                },
                Palette::Xterm256,
            ),
            Palette::Xterm256,
            ColorRole::DarkStone,
        ),
    }
}

fn atlas(tiles: Vec<AtlasTile>) -> StoryFixture {
    StoryFixture::AssetAtlas(AssetAtlas { tiles })
}

macro_rules! profile_atlas {
    ($builder:ident, $assets:ident, $variant:ident, $field:ident) => {
        pub fn $builder(_: &StoryContext) -> StoryFixture {
            atlas(
                $assets
                    .iter()
                    .map(|asset| match *asset {
                        AssetId::$variant(value) => profile_tile(asset.label(), |persona| {
                            persona.appearance.$field = value;
                        }),
                        _ => unreachable!("asset family contains the wrong AssetId variant"),
                    })
                    .collect(),
            )
        }
    };
}

pub fn classes(_: &StoryContext) -> StoryFixture {
    atlas(
        CLASSES
            .iter()
            .map(|asset| match *asset {
                AssetId::Class(value) => profile_tile(asset.label(), |persona| {
                    persona.class = value;
                }),
                _ => unreachable!("class assets contain a non-class AssetId"),
            })
            .collect(),
    )
}

pub fn ancestries(_: &StoryContext) -> StoryFixture {
    atlas(
        ANCESTRIES
            .iter()
            .map(|asset| match *asset {
                AssetId::Ancestry(value) => profile_tile(asset.label(), |persona| {
                    persona.ancestry = value;
                }),
                _ => unreachable!("ancestry assets contain a non-ancestry AssetId"),
            })
            .collect(),
    )
}

profile_atlas!(
    body_proportions,
    BODY_PROPORTIONS,
    BodyProportions,
    proportions
);
profile_atlas!(head_shapes, HEAD_SHAPES, HeadShape, head_shape);
profile_atlas!(skin_tones, SKIN_TONES, SkinTone, skin_tone);
profile_atlas!(hair_shapes, HAIR_SHAPES, HairShape, hair);
profile_atlas!(hair_tones, HAIR_TONES, HairTone, hair_tone);
profile_atlas!(face_details, FACE_DETAILS, FaceDetail, face_detail);
profile_atlas!(garb, GARBS, Garb, garb);
profile_atlas!(legwear, LEGWEAR, Legwear, legwear);
profile_atlas!(footwear, FOOTWEAR, Footwear, footwear);
profile_atlas!(keepsakes, KEEPSAKES, Keepsake, keepsake);
profile_atlas!(accent_tones, ACCENT_TONES, AccentTone, accent);

pub fn palette_roles(_: &StoryContext) -> StoryFixture {
    atlas(
        COLOR_ROLES
            .iter()
            .map(|asset| match *asset {
                AssetId::ColorRole(role) => {
                    let mut canvas = Canvas::new(8, 8);
                    canvas.fill_rect(0, 0, 8, 8, role);
                    AtlasTile {
                        label: asset.label(),
                        preferred_width: 18,
                        preferred_height: 8,
                        content: AtlasContent::pixel(
                            canvas,
                            Palette::Xterm256,
                            ColorRole::DarkStone,
                        ),
                    }
                }
                _ => unreachable!("colour-role assets contain a non-colour-role AssetId"),
            })
            .collect(),
    )
}

pub fn poses(_: &StoryContext) -> StoryFixture {
    atlas(
        POSES
            .iter()
            .map(|asset| match *asset {
                AssetId::Pose(pose) => chamber_tile(
                    asset.label(),
                    pose,
                    u8::from(pose == TheatrePose::SpoilsUnopened) * 4,
                ),
                _ => unreachable!("pose assets contain a non-pose AssetId"),
            })
            .collect(),
    )
}
