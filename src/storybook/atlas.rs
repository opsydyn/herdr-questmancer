use crate::{
    app::DisplayPreferences,
    domain::{AdventurerPersona, PersonaKey},
    ui::{
        persona::{compose_chamber_adventurer_for_palette, compose_profile_adventurer},
        pixel::{Canvas, ColorRole, Palette},
        theatre::{TheatreFrame, TheatrePose, frame_for},
    },
};

use super::{
    AssetId,
    assets::{
        ACCENT_TONES, ANCESTRIES, BODY_PROPORTIONS, CLASSES, COLOR_ROLES, FACE_DETAILS, FOOTWEAR,
        GARBS, HAIR_SHAPES, HAIR_TONES, HEAD_SHAPES, KEEPSAKES, LEGWEAR, POSES, SKIN_TONES,
    },
    fixtures::{
        AssetAtlas, AtlasContent, AtlasTile, StoryContext, StoryFixture, guild_fixture,
        guild_populated_fixture,
    },
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

pub fn adventurer_cards(context: &StoryContext) -> StoryFixture {
    let (agent, theatre, preferences) = widget_inputs(*context);
    atlas(vec![
        AtlasTile {
            label: "Full adventurer card",
            preferred_width: 36,
            preferred_height: 21,
            content: AtlasContent::AdventurerCard {
                agent: agent.clone(),
                theatre,
                preferences,
            },
        },
        AtlasTile {
            label: "Compact adventurer card",
            preferred_width: 30,
            preferred_height: 12,
            content: AtlasContent::AdventurerCard {
                agent,
                theatre,
                preferences,
            },
        },
    ])
}

pub fn chambers(context: &StoryContext) -> StoryFixture {
    let (agent, _theatre, preferences) = widget_inputs(*context);
    atlas(
        POSES
            .iter()
            .flat_map(|asset| {
                let AssetId::Pose(pose) = *asset else {
                    unreachable!("pose assets contain a non-pose AssetId");
                };
                [(true, 30, 12), (false, 26, 9)].map(|(full, width, height)| AtlasTile {
                    label: chamber_matrix_label(pose, full),
                    preferred_width: width,
                    preferred_height: height,
                    content: AtlasContent::Chamber {
                        agent: agent.clone(),
                        theatre: TheatreFrame {
                            pose,
                            animation_frame: u8::from(pose == TheatrePose::SpoilsUnopened) * 4,
                            focused: false,
                            label: theatre_label(pose),
                        },
                        selected: full,
                        preferences,
                    },
                })
            })
            .collect(),
    )
}

const fn theatre_label(pose: TheatrePose) -> &'static str {
    match pose {
        TheatrePose::Delving => "DELVING",
        TheatrePose::SeekingCounsel => "COUNSEL REQUESTED",
        TheatrePose::SpoilsUnopened => "SPOILS RETURNED",
        TheatrePose::VictoryRecorded => "VICTORY RECORDED",
        TheatrePose::Resting => "RESTING",
        TheatrePose::Departed => "DEPARTED",
        TheatrePose::Unknown => "UNKNOWN",
    }
}

const fn chamber_matrix_label(pose: TheatrePose, full: bool) -> &'static str {
    match (full, pose) {
        (true, TheatrePose::Delving) => "Full chamber - delving",
        (false, TheatrePose::Delving) => "Compact chamber - delving",
        (true, TheatrePose::SeekingCounsel) => "Full chamber - seeking counsel",
        (false, TheatrePose::SeekingCounsel) => "Compact chamber - seeking counsel",
        (true, TheatrePose::SpoilsUnopened) => "Full chamber - spoils unopened",
        (false, TheatrePose::SpoilsUnopened) => "Compact chamber - spoils unopened",
        (true, TheatrePose::VictoryRecorded) => "Full chamber - victory recorded",
        (false, TheatrePose::VictoryRecorded) => "Compact chamber - victory recorded",
        (true, TheatrePose::Resting) => "Full chamber - resting",
        (false, TheatrePose::Resting) => "Compact chamber - resting",
        (true, TheatrePose::Departed) => "Full chamber - departed",
        (false, TheatrePose::Departed) => "Compact chamber - departed",
        (true, TheatrePose::Unknown) => "Full chamber - unknown",
        (false, TheatrePose::Unknown) => "Compact chamber - unknown",
    }
}

pub fn guild_regions(context: &StoryContext) -> StoryFixture {
    atlas(vec![AtlasTile {
        label: "All Guild regions",
        preferred_width: 122,
        preferred_height: 36,
        content: AtlasContent::Application {
            model: guild_populated_fixture(context),
        },
    }])
}

fn widget_inputs(
    context: StoryContext,
) -> (crate::domain::Agent, TheatreFrame, DisplayPreferences) {
    let model = guild_fixture(&context);
    let preferences = *model.preferences();
    let agent = model
        .selected_agent()
        .expect("the fixed guild fixture has a selected adventurer")
        .clone();
    let theatre = frame_for(&agent, model.now(), &preferences);
    (agent, theatre, preferences)
}
