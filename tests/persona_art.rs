use std::collections::HashSet;

use questmancer::{
    domain::{
        AccentTone, AdventurerClass, AdventurerPersona, Ancestry, Footwear, Garb, HairTone,
        Keepsake, Legwear, PersonaKey, SkinTone,
    },
    ui::{
        persona::{
            appearance_roles_for_palette, compose_chamber_adventurer,
            compose_chamber_adventurer_for_palette, compose_profile_adventurer,
            compose_profile_adventurer_for_palette,
        },
        pixel::{Canvas, ColorRole, Palette},
        theatre::{TheatreFrame, TheatrePose},
    },
};

fn fixed_persona(key: &str) -> AdventurerPersona {
    AdventurerPersona::for_key(PersonaKey::new(key))
}

fn frame(pose: TheatrePose, animation_frame: u8) -> TheatreFrame {
    TheatreFrame {
        pose,
        animation_frame,
        focused: false,
        label: "test",
    }
}

fn silhouette(canvas: &Canvas) -> String {
    canvas
        .pixels()
        .chunks(usize::from(canvas.width()))
        .map(|row| {
            row.iter()
                .map(|pixel| if pixel.is_some() { '#' } else { '.' })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn assert_composed_adjacency_contrast(canvas: &Canvas, palette: Palette) {
    let width = usize::from(canvas.width());
    let height = usize::from(canvas.height());
    for y in 0..height {
        for x in 0..width {
            let Some(role) = canvas.pixels()[y * width + x] else {
                continue;
            };
            for (next_x, next_y) in [(x + 1, y), (x, y + 1)] {
                if next_x >= width || next_y >= height {
                    continue;
                }
                let Some(neighbour) = canvas.pixels()[next_y * width + next_x] else {
                    continue;
                };
                assert!(
                    role == neighbour || palette.roles_contrast(role, neighbour),
                    "{palette:?} collapses adjacent {role:?} and {neighbour:?} at ({x}, {y})"
                );
            }
        }
    }
}

fn assert_adjacency_contrast(roles: questmancer::ui::persona::AppearanceRoles, palette: Palette) {
    for (first, second) in [
        (roles.hair, roles.skin),
        (roles.hair, ColorRole::DarkStone),
        (roles.garb, roles.skin),
        (roles.garb, roles.hair),
        (roles.legwear, roles.garb),
        (roles.footwear, roles.legwear),
        (roles.footwear, ColorRole::DarkStone),
        (roles.keepsake, roles.garb),
        (roles.keepsake, roles.skin),
        (roles.keepsake, roles.hair),
        (roles.accent, roles.garb),
        (roles.accent, roles.skin),
        (roles.accent, roles.hair),
        (roles.accent, roles.keepsake),
    ] {
        assert!(palette.roles_contrast(first, second));
    }
}

#[test]
fn fantasy_composers_keep_fixed_chamber_and_profile_dimensions() {
    let persona = fixed_persona("art-fixture");
    let chamber = compose_chamber_adventurer(&persona, frame(TheatrePose::Delving, 0));
    let profile = compose_profile_adventurer(&persona);

    assert_eq!((chamber.width(), chamber.height()), (10, 12));
    assert_eq!((profile.width(), profile.height()), (16, 32));
}

#[test]
fn every_class_has_a_distinct_profile_and_chamber_gear_silhouette() {
    let classes = [
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
    let base = fixed_persona("class-fixture");

    let profiles = classes
        .map(|class| {
            let mut persona = base.clone();
            persona.class = class;
            silhouette(&compose_profile_adventurer(&persona))
        })
        .into_iter()
        .collect::<HashSet<_>>();
    let chambers = classes
        .map(|class| {
            let mut persona = base.clone();
            persona.class = class;
            silhouette(&compose_chamber_adventurer(
                &persona,
                frame(TheatrePose::Delving, 0),
            ))
        })
        .into_iter()
        .collect::<HashSet<_>>();

    assert_eq!(profiles.len(), classes.len());
    assert_eq!(chambers.len(), classes.len());
}

#[test]
fn wizard_and_ranger_gear_use_distinct_logical_pixels() {
    let mut wizard = fixed_persona("class-fixture");
    wizard.class = AdventurerClass::Wizard;
    let mut ranger = wizard.clone();
    ranger.class = AdventurerClass::Ranger;

    assert_ne!(
        silhouette(&compose_profile_adventurer(&wizard)),
        silhouette(&compose_profile_adventurer(&ranger)),
        "wizard spellbook/staff and ranger bow/quiver must use distinct logical pixels"
    );
}

#[test]
fn ancestry_anchors_change_the_profile_and_dwarf_is_compact_and_bearded() {
    let mut dwarf = fixed_persona("ancestry-fixture");
    dwarf.ancestry = Ancestry::Dwarf;
    let mut human = dwarf.clone();
    human.ancestry = Ancestry::Human;

    let dwarf = compose_profile_adventurer(&dwarf);
    let human = compose_profile_adventurer(&human);
    assert_ne!(
        silhouette(&dwarf),
        silhouette(&human),
        "dwarf must retain a compact, bearded silhouette"
    );

    let occupied_rows = |canvas: &Canvas| {
        canvas
            .pixels()
            .chunks(usize::from(canvas.width()))
            .filter(|row| row.iter().any(Option::is_some))
            .count()
    };
    assert!(occupied_rows(&dwarf) <= occupied_rows(&human));
    assert!(
        dwarf
            .pixels()
            .iter()
            .filter(|pixel| **pixel == Some(ColorRole::HairDark))
            .count()
            >= 8,
        "dwarf beard must remain a material recognition anchor"
    );
}

#[test]
fn every_ancestry_has_a_distinct_profile_anchor() {
    let ancestries = [
        Ancestry::Human,
        Ancestry::Dwarf,
        Ancestry::Elf,
        Ancestry::Halfling,
        Ancestry::Orc,
        Ancestry::Gnome,
        Ancestry::Goblin,
    ];
    let base = fixed_persona("ancestry-fixture");
    let silhouettes = ancestries
        .map(|ancestry| {
            let mut persona = base.clone();
            persona.ancestry = ancestry;
            silhouette(&compose_profile_adventurer(&persona))
        })
        .into_iter()
        .collect::<HashSet<_>>();

    assert_eq!(silhouettes.len(), ancestries.len());
}

#[test]
fn every_keepsake_has_distinct_profile_and_chamber_geometry() {
    let keepsakes = [
        Keepsake::Feather,
        Keepsake::LuckyCoin,
        Keepsake::Mug,
        Keepsake::PressedLeaf,
        Keepsake::Ribbon,
        Keepsake::TinyFamiliar,
    ];
    let base = fixed_persona("keepsake-fixture");
    let profiles = keepsakes
        .map(|keepsake| {
            let mut persona = base.clone();
            persona.appearance.keepsake = keepsake;
            silhouette(&compose_profile_adventurer(&persona))
        })
        .into_iter()
        .collect::<HashSet<_>>();
    let chambers = keepsakes
        .map(|keepsake| {
            let mut persona = base.clone();
            persona.appearance.keepsake = keepsake;
            silhouette(&compose_chamber_adventurer(
                &persona,
                frame(TheatrePose::Delving, 0),
            ))
        })
        .into_iter()
        .collect::<HashSet<_>>();

    assert_eq!(profiles.len(), keepsakes.len());
    assert_eq!(chambers.len(), keepsakes.len());
}

#[test]
fn chamber_states_have_explicit_non_colour_props() {
    let persona = fixed_persona("art-fixture");
    let states = [
        TheatrePose::Delving,
        TheatrePose::SeekingCounsel,
        TheatrePose::SpoilsUnopened,
        TheatrePose::VictoryRecorded,
        TheatrePose::Resting,
        TheatrePose::Departed,
        TheatrePose::Unknown,
    ];
    let silhouettes = states
        .map(|pose| silhouette(&compose_chamber_adventurer(&persona, frame(pose, 0))))
        .into_iter()
        .collect::<HashSet<_>>();

    assert_eq!(silhouettes.len(), states.len());
}

#[test]
fn motion_is_deterministic_and_only_delving_animates_the_figure() {
    let persona = fixed_persona("motion-fixture");
    let render =
        |pose, animation_frame| compose_chamber_adventurer(&persona, frame(pose, animation_frame));

    assert_eq!(
        render(TheatrePose::Delving, 0),
        render(TheatrePose::Delving, 2)
    );
    assert_ne!(
        render(TheatrePose::Delving, 0),
        render(TheatrePose::Delving, 1)
    );
    for pose in [
        TheatrePose::SeekingCounsel,
        TheatrePose::VictoryRecorded,
        TheatrePose::Resting,
        TheatrePose::Departed,
        TheatrePose::Unknown,
    ] {
        assert_eq!(render(pose, 0), render(pose, 7));
    }
}

#[test]
fn spoils_sparkle_is_deterministic_and_bounded() {
    let persona = fixed_persona("spoils-fixture");
    let render = |animation_frame| {
        compose_chamber_adventurer(
            &persona,
            frame(TheatrePose::SpoilsUnopened, animation_frame),
        )
    };

    assert_eq!(render(0), render(8));
    assert_eq!(render(1), render(9));
    assert_ne!(render(0), render(1));
}

#[test]
fn profile_uses_both_top_and_bottom_halves_and_keeps_recognition_roles() {
    let persona = fixed_persona("coverage-fixture");
    let roles = appearance_roles_for_palette(&persona.appearance, Palette::Xterm256);
    let profile = compose_profile_adventurer(&persona);
    let rows = profile
        .pixels()
        .chunks(usize::from(profile.width()))
        .collect::<Vec<_>>();

    assert!(rows[..16].iter().any(|row| row.iter().any(Option::is_some)));
    assert!(rows[16..].iter().any(|row| row.iter().any(Option::is_some)));
    for anchor in [roles.skin, roles.hair, roles.garb, roles.keepsake] {
        assert!(profile.pixels().contains(&Some(anchor)));
    }
}

#[test]
fn palette_aware_composers_preserve_silhouettes_and_avoid_adjacent_collisions() {
    let mut persona = fixed_persona("palette-fixture");
    persona.appearance.skin_tone = SkinTone::Ebony;

    for palette in [Palette::Xterm256, Palette::Ansi16] {
        let roles = appearance_roles_for_palette(&persona.appearance, palette);
        assert_adjacency_contrast(roles, palette);

        assert_eq!(
            silhouette(&compose_profile_adventurer(&persona)),
            silhouette(&compose_profile_adventurer_for_palette(&persona, palette))
        );
        assert_eq!(
            silhouette(&compose_chamber_adventurer(
                &persona,
                frame(TheatrePose::SeekingCounsel, 0),
            )),
            silhouette(&compose_chamber_adventurer_for_palette(
                &persona,
                frame(TheatrePose::SeekingCounsel, 0),
                palette,
            ))
        );
    }
}

#[test]
fn palette_collision_safety_covers_every_appearance_trait_combination() {
    let skin_tones = [
        SkinTone::Porcelain,
        SkinTone::Rose,
        SkinTone::Sand,
        SkinTone::Umber,
        SkinTone::Sienna,
        SkinTone::Ebony,
    ];
    let hair_tones = [
        HairTone::Black,
        HairTone::Espresso,
        HairTone::Chestnut,
        HairTone::Copper,
        HairTone::Gold,
        HairTone::Silver,
    ];
    let garbs = [
        Garb::Armour,
        Garb::Cloak,
        Garb::Doublet,
        Garb::Leathers,
        Garb::Robes,
        Garb::Vestments,
        Garb::WorkApron,
    ];
    let legwear = [
        Legwear::BootsAndBreeches,
        Legwear::Greaves,
        Legwear::RobeHem,
        Legwear::TravelingSkirt,
    ];
    let footwear = [
        Footwear::Boots,
        Footwear::Sabatons,
        Footwear::Sandals,
        Footwear::SoftShoes,
    ];
    let keepsakes = [
        Keepsake::Feather,
        Keepsake::LuckyCoin,
        Keepsake::Mug,
        Keepsake::PressedLeaf,
        Keepsake::Ribbon,
        Keepsake::TinyFamiliar,
    ];
    let accents = [
        AccentTone::Amber,
        AccentTone::Cyan,
        AccentTone::Lime,
        AccentTone::Magenta,
        AccentTone::Red,
        AccentTone::Blue,
        AccentTone::Violet,
        AccentTone::Teal,
    ];

    for palette in [Palette::Xterm256, Palette::Ansi16] {
        for skin_tone in skin_tones {
            for hair_tone in hair_tones {
                for garb in garbs {
                    for legwear in legwear {
                        for footwear in footwear {
                            for keepsake in keepsakes {
                                for accent in accents {
                                    let mut persona = fixed_persona("palette-exhaustive");
                                    persona.appearance.skin_tone = skin_tone;
                                    persona.appearance.hair_tone = hair_tone;
                                    persona.appearance.garb = garb;
                                    persona.appearance.legwear = legwear;
                                    persona.appearance.footwear = footwear;
                                    persona.appearance.keepsake = keepsake;
                                    persona.appearance.accent = accent;
                                    assert_adjacency_contrast(
                                        appearance_roles_for_palette(&persona.appearance, palette),
                                        palette,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn composed_class_ancestry_and_state_layers_never_collapse_adjacent_roles() {
    let classes = [
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
    let ancestries = [
        Ancestry::Human,
        Ancestry::Dwarf,
        Ancestry::Elf,
        Ancestry::Halfling,
        Ancestry::Orc,
        Ancestry::Gnome,
        Ancestry::Goblin,
    ];
    let poses = [
        TheatrePose::Delving,
        TheatrePose::SeekingCounsel,
        TheatrePose::SpoilsUnopened,
        TheatrePose::VictoryRecorded,
        TheatrePose::Resting,
        TheatrePose::Departed,
        TheatrePose::Unknown,
    ];

    for palette in [Palette::Xterm256, Palette::Ansi16] {
        for skin_tone in [SkinTone::Porcelain, SkinTone::Umber, SkinTone::Ebony] {
            for class in classes {
                for ancestry in ancestries {
                    for keepsake in [
                        Keepsake::Feather,
                        Keepsake::LuckyCoin,
                        Keepsake::Mug,
                        Keepsake::PressedLeaf,
                        Keepsake::Ribbon,
                        Keepsake::TinyFamiliar,
                    ] {
                        let mut persona = fixed_persona("composed-palette-fixture");
                        persona.class = class;
                        persona.ancestry = ancestry;
                        persona.appearance.skin_tone = skin_tone;
                        persona.appearance.keepsake = keepsake;
                        assert_composed_adjacency_contrast(
                            &compose_profile_adventurer_for_palette(&persona, palette),
                            palette,
                        );
                        for pose in poses {
                            assert_composed_adjacency_contrast(
                                &compose_chamber_adventurer_for_palette(
                                    &persona,
                                    frame(pose, 0),
                                    palette,
                                ),
                                palette,
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn all_fantasy_semantic_roles_project_in_both_palettes() {
    let roles = [
        ColorRole::Stone,
        ColorRole::DarkStone,
        ColorRole::Timber,
        ColorRole::Parchment,
        ColorRole::Ink,
        ColorRole::Hearth,
        ColorRole::Moss,
        ColorRole::RuneGlow,
        ColorRole::Counsel,
        ColorRole::Spoils,
        ColorRole::Selection,
        ColorRole::Fog,
        ColorRole::Goblin,
        ColorRole::SkinLight,
        ColorRole::SkinMedium,
        ColorRole::SkinDark,
        ColorRole::HairDark,
        ColorRole::HairLight,
        ColorRole::Leather,
        ColorRole::Steel,
        ColorRole::ClothWarm,
        ColorRole::ClothCool,
    ];

    assert_eq!(roles.len(), 22);
    for palette in [Palette::Xterm256, Palette::Ansi16] {
        for (first, second) in [
            (ColorRole::Stone, ColorRole::DarkStone),
            (ColorRole::Stone, ColorRole::Moss),
            (ColorRole::DarkStone, ColorRole::Parchment),
            (ColorRole::DarkStone, ColorRole::RuneGlow),
            (ColorRole::Timber, ColorRole::Hearth),
            (ColorRole::Parchment, ColorRole::Ink),
            (ColorRole::RuneGlow, ColorRole::Counsel),
            (ColorRole::Counsel, ColorRole::Spoils),
            (ColorRole::Spoils, ColorRole::Selection),
            (ColorRole::SkinLight, ColorRole::SkinMedium),
            (ColorRole::SkinMedium, ColorRole::SkinDark),
            (ColorRole::HairDark, ColorRole::HairLight),
            (ColorRole::Leather, ColorRole::Steel),
            (ColorRole::ClothWarm, ColorRole::ClothCool),
        ] {
            assert!(palette.roles_contrast(first, second));
        }
    }
}
