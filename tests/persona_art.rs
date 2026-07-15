use std::collections::HashSet;

use questmancer::{
    domain::{
        AccentTone, AdventurerClass, BodyProportions, FaceDetail, Footwear, Garb, HairShape,
        HairTone, HeadShape, Keepsake, Legwear, PersonaAppearance, SkinTone,
    },
    ui::{
        persona::{
            AppearanceRoles, appearance_roles, appearance_roles_for_palette, compose_profile,
            compose_profile_for_palette, compose_profile_with_gear, compose_seated,
            compose_seated_for_palette, compose_seated_with_gear,
        },
        pixel::{
            AccentShade, Canvas, ColorRole, FabricShade, FootwearShade, HairShade, Palette,
            SkinShade,
        },
        theatre::{TheatreFrame, TheatrePose},
    },
};

fn compact() -> PersonaAppearance {
    PersonaAppearance {
        proportions: BodyProportions::Compact,
        head_shape: HeadShape::Round,
        skin_tone: SkinTone::Sand,
        hair: HairShape::Curls,
        hair_tone: HairTone::Chestnut,
        face_detail: FaceDetail::RoundGlasses,
        garb: Garb::Cloak,
        legwear: Legwear::TravelingSkirt,
        footwear: Footwear::Sabatons,
        keepsake: Keepsake::TinyFamiliar,
        accent: AccentTone::Blue,
    }
}

fn tall() -> PersonaAppearance {
    PersonaAppearance {
        proportions: BodyProportions::Tall,
        head_shape: HeadShape::Long,
        skin_tone: SkinTone::Ebony,
        hair: HairShape::Quiff,
        hair_tone: HairTone::Gold,
        face_detail: FaceDetail::Visor,
        garb: Garb::Armour,
        legwear: Legwear::Greaves,
        footwear: Footwear::Boots,
        keepsake: Keepsake::LuckyCoin,
        accent: AccentTone::Cyan,
    }
}

fn broad() -> PersonaAppearance {
    PersonaAppearance {
        proportions: BodyProportions::Broad,
        head_shape: HeadShape::Square,
        skin_tone: SkinTone::Rose,
        hair: HairShape::Shaved,
        hair_tone: HairTone::Black,
        face_detail: FaceDetail::Moustache,
        garb: Garb::Leathers,
        legwear: Legwear::BootsAndBreeches,
        footwear: Footwear::SoftShoes,
        keepsake: Keepsake::TinyFamiliar,
        accent: AccentTone::Magenta,
    }
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

fn role_symbol(role: ColorRole, roles: AppearanceRoles) -> char {
    if role == roles.hair {
        'h'
    } else if role == roles.skin {
        's'
    } else if role == roles.garb {
        't'
    } else if role == roles.legwear {
        'b'
    } else if role == roles.footwear {
        'f'
    } else if role == roles.keepsake {
        'a'
    } else if role == roles.accent {
        'c'
    } else if role == roles.highlight {
        '+'
    } else if role == roles.shadow {
        '-'
    } else {
        '?'
    }
}

fn logical_role_map(canvas: &Canvas, roles: AppearanceRoles) -> String {
    canvas
        .pixels()
        .chunks(usize::from(canvas.width()))
        .map(|row| {
            row.iter()
                .map(|pixel| pixel.map_or('.', |role| role_symbol(role, roles)))
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn assert_adjacency_contrast(roles: AppearanceRoles, palette: Palette) {
    for (upper, lower) in [
        (roles.hair, roles.skin),
        (roles.hair, ColorRole::PanelBackground),
        (roles.garb, roles.skin),
        (roles.garb, roles.hair),
        (roles.legwear, roles.garb),
        (roles.footwear, roles.legwear),
        (roles.footwear, ColorRole::PanelBackground),
        (roles.keepsake, roles.garb),
        (roles.keepsake, roles.skin),
        (roles.keepsake, roles.hair),
        (roles.accent, roles.garb),
        (roles.accent, roles.skin),
        (roles.accent, roles.hair),
        (roles.accent, roles.keepsake),
    ] {
        assert!(palette.roles_contrast(upper, lower));
    }
}

#[test]
fn fixed_personas_have_exact_dimensions_and_distinct_silhouettes() {
    let appearances = [compact(), tall(), broad()];
    let seated =
        appearances.map(|appearance| compose_seated(&appearance, frame(TheatrePose::Resting, 0)));
    let profiles = appearances.map(|appearance| compose_profile(&appearance));

    for canvas in &seated {
        assert_eq!((canvas.width(), canvas.height()), (10, 12));
    }
    for canvas in &profiles {
        assert_eq!((canvas.width(), canvas.height()), (16, 32));
    }

    assert_eq!(
        seated.iter().map(silhouette).collect::<HashSet<_>>().len(),
        3
    );
    assert_eq!(
        profiles
            .iter()
            .map(silhouette)
            .collect::<HashSet<_>>()
            .len(),
        3
    );
}

#[test]
fn every_class_derived_gear_has_a_transitional_sprite_silhouette() {
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
    let appearance = tall();
    let seated = classes
        .map(|class| {
            silhouette(&compose_seated_with_gear(
                &appearance,
                class.gear(),
                frame(TheatrePose::Resting, 0),
            ))
        })
        .into_iter()
        .collect::<HashSet<_>>();
    let profiles = classes
        .map(|class| silhouette(&compose_profile_with_gear(&appearance, class.gear())))
        .into_iter()
        .collect::<HashSet<_>>();

    assert_eq!(seated.len(), classes.len());
    assert_eq!(profiles.len(), classes.len());
}

#[test]
fn canonical_roles_preserve_requested_typed_traits_without_palette_fallback() {
    let roles = appearance_roles(&broad());

    assert_eq!(roles.skin, ColorRole::SkinTone(SkinShade::Rose));
    assert_eq!(roles.hair, ColorRole::HairTone(HairShade::Black));
    assert_eq!(roles.garb, ColorRole::Fabric(FabricShade::Green));
    assert_eq!(roles.legwear, ColorRole::Fabric(FabricShade::Navy));
    assert_eq!(roles.footwear, ColorRole::Footwear(FootwearShade::Black));
    assert_eq!(roles.keepsake, ColorRole::AccentTone(AccentShade::Teal));
    assert_eq!(roles.accent, ColorRole::AccentTone(AccentShade::Magenta));
}

#[test]
fn ansi_safe_roles_cover_the_full_persona_adjacency_graph() {
    let mut appearance = compact();
    appearance.skin_tone = SkinTone::Rose;
    appearance.hair_tone = HairTone::Gold;
    appearance.face_detail = FaceDetail::Visor;
    appearance.garb = Garb::Armour;
    appearance.legwear = Legwear::BootsAndBreeches;
    appearance.footwear = Footwear::SoftShoes;
    appearance.keepsake = Keepsake::Feather;
    appearance.accent = AccentTone::Red;

    let canonical = appearance_roles(&appearance);
    assert_eq!(canonical.skin, ColorRole::SkinTone(SkinShade::Rose));
    assert_eq!(canonical.hair, ColorRole::HairTone(HairShade::Gold));
    assert_eq!(canonical.garb, ColorRole::Fabric(FabricShade::Navy));
    assert_eq!(
        canonical.keepsake,
        ColorRole::AccentTone(AccentShade::Amber)
    );
    assert_eq!(canonical.accent, ColorRole::AccentTone(AccentShade::Red));

    let safe = appearance_roles_for_palette(&appearance, Palette::Ansi16);
    assert_adjacency_contrast(safe, Palette::Ansi16);
    assert_ne!(safe.keepsake, canonical.keepsake);
    assert_ne!(safe.accent, canonical.accent);
}

#[test]
fn xterm_safe_roles_preserve_non_colliding_black_hair_and_soft_shoes() {
    let appearance = broad();
    let canonical = appearance_roles(&appearance);
    let safe = appearance_roles_for_palette(&appearance, Palette::Xterm256);

    assert_eq!(safe.hair, ColorRole::HairTone(HairShade::Black));
    assert_eq!(safe.footwear, ColorRole::Footwear(FootwearShade::Black));
    assert_eq!(safe.hair, canonical.hair);
    assert_eq!(safe.footwear, canonical.footwear);
    assert_adjacency_contrast(safe, Palette::Xterm256);
}

#[test]
fn ansi_safe_roles_keep_black_hair_and_soft_shoes_visible_against_transparency() {
    let appearance = broad();
    let canonical = appearance_roles(&appearance);
    assert_eq!(canonical.hair, ColorRole::HairTone(HairShade::Black));
    assert_eq!(
        canonical.footwear,
        ColorRole::Footwear(FootwearShade::Black)
    );

    let safe = appearance_roles_for_palette(&appearance, Palette::Ansi16);
    assert_ne!(safe.hair, canonical.hair);
    assert_ne!(safe.footwear, canonical.footwear);
    assert_adjacency_contrast(safe, Palette::Ansi16);
}

#[test]
fn palette_aware_composers_apply_xterm_collision_fallbacks() {
    let mut appearance = tall();
    appearance.skin_tone = SkinTone::Ebony;
    appearance.hair_tone = HairTone::Chestnut;
    let canonical = appearance_roles(&appearance);
    assert_eq!(canonical.skin, ColorRole::SkinTone(SkinShade::Ebony));
    assert_eq!(canonical.hair, ColorRole::HairTone(HairShade::Chestnut));

    let safe = appearance_roles_for_palette(&appearance, Palette::Xterm256);
    assert_eq!(safe.skin, canonical.skin);
    assert_ne!(safe.hair, canonical.hair);
    assert_adjacency_contrast(safe, Palette::Xterm256);

    let canonical_profile = compose_profile(&appearance);
    let safe_profile = compose_profile_for_palette(&appearance, Palette::Xterm256);
    let safe_seated = compose_seated_for_palette(
        &appearance,
        frame(TheatrePose::Delving, 0),
        Palette::Xterm256,
    );
    assert!(canonical_profile.pixels().contains(&Some(canonical.hair)));
    assert!(safe_profile.pixels().contains(&Some(safe.hair)));
    assert!(!safe_profile.pixels().contains(&Some(canonical.hair)));
    assert!(safe_seated.pixels().contains(&Some(safe.hair)));
    assert!(!safe_seated.pixels().contains(&Some(canonical.hair)));
}

#[test]
fn safe_roles_exhaust_every_colour_trait_combination_for_both_palettes() {
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
                                    let mut appearance = compact();
                                    appearance.skin_tone = skin_tone;
                                    appearance.hair_tone = hair_tone;
                                    appearance.garb = garb;
                                    appearance.legwear = legwear;
                                    appearance.footwear = footwear;
                                    appearance.keepsake = keepsake;
                                    appearance.accent = accent;

                                    assert_adjacency_contrast(
                                        appearance_roles_for_palette(&appearance, palette),
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
fn both_representations_preserve_each_personas_recognition_anchors() {
    for appearance in [compact(), tall(), broad()] {
        let roles = appearance_roles(&appearance);
        let seated = compose_seated(&appearance, frame(TheatrePose::Resting, 0));
        let profile = compose_profile(&appearance);

        for canvas in [&seated, &profile] {
            for anchor in [roles.hair, roles.skin, roles.garb, roles.keepsake] {
                assert!(canvas.pixels().contains(&Some(anchor)));
            }
        }
    }
}

#[test]
fn seated_state_is_explicit_in_the_non_colour_silhouette() {
    let appearance = tall();
    let working = compose_seated(&appearance, frame(TheatrePose::Delving, 0));
    let working_next = compose_seated(&appearance, frame(TheatrePose::Delving, 1));
    let blocked = compose_seated(&appearance, frame(TheatrePose::SeekingCounsel, 0));
    let blocked_next = compose_seated(&appearance, frame(TheatrePose::SeekingCounsel, 1));
    let done = compose_seated(&appearance, frame(TheatrePose::VictoryRecorded, 0));
    let idle = compose_seated(&appearance, frame(TheatrePose::Resting, 0));
    let exited = compose_seated(&appearance, frame(TheatrePose::Departed, 0));

    assert_ne!(silhouette(&working), silhouette(&working_next));
    assert_ne!(silhouette(&working), silhouette(&blocked));
    assert_ne!(silhouette(&blocked), silhouette(&done));
    assert_eq!(silhouette(&blocked), silhouette(&blocked_next));
    assert_eq!(silhouette(&done), silhouette(&idle));
    assert!(exited.pixels().iter().all(Option::is_none));
}

#[test]
fn compact_blocked_role_map_is_a_stable_semantic_golden() {
    let appearance = compact();
    let roles = appearance_roles(&appearance);
    let blocked = compose_seated(&appearance, frame(TheatrePose::SeekingCounsel, 0));

    assert_eq!(
        logical_role_map(&blocked, roles),
        concat!(
            "...hhh....\n",
            "..sssss...\n",
            ".ahcccha..\n",
            "..sssss.s.\n",
            "...tttt.ss\n",
            "...cccc...\n",
            "..stttt...\n",
            "...cccc...\n",
            "...bbbb...\n",
            "..bb..bb..\n",
            "..bb..bb..\n",
            ".fff..fff."
        )
    );
}

#[test]
fn profile_is_a_separately_authored_neutral_composition() {
    let appearance = broad();
    let profile = compose_profile(&appearance);
    let profile_golden = logical_role_map(&profile, appearance_roles(&appearance));

    assert_eq!(
        profile_golden,
        concat!(
            "................\n",
            "................\n",
            "....hhhhhhhh....\n",
            "....hhhhhhhh....\n",
            "...assssssssa...\n",
            "...assssssssa...\n",
            "...assssssssa...\n",
            "...assshhhssa...\n",
            "....ssssssss....\n",
            "................\n",
            "..tttttttttttt..\n",
            "ssttttttttttttss\n",
            "ssttttttttttttss\n",
            "sstttttcctttttss\n",
            "sstttttcctttttss\n",
            "ssttttttttttttss\n",
            "ssttttttttttttss\n",
            "ssttttttttttttss\n",
            "..tttttttttttt..\n",
            "...bbbbbbbbbb...\n",
            "...bbbbbbbbbb...\n",
            "...bbbbbbbbbb...\n",
            "....bbb..bbb....\n",
            "....bbb..bbb....\n",
            "....bbb..bbb....\n",
            "....bbb..bbb....\n",
            "....bbb..bbb....\n",
            "....bbb..bbb....\n",
            "....bbb..bbb....\n",
            "...fffff.fffff..\n",
            "...fffff.fffff..\n",
            "................"
        )
    );

    for pose in [
        TheatrePose::Delving,
        TheatrePose::SeekingCounsel,
        TheatrePose::SpoilsUnopened,
        TheatrePose::VictoryRecorded,
        TheatrePose::Resting,
        TheatrePose::Departed,
        TheatrePose::Unknown,
    ] {
        let _ = compose_seated(&appearance, frame(pose, 7));
        assert_eq!(
            logical_role_map(&compose_profile(&appearance), appearance_roles(&appearance)),
            profile_golden
        );
    }

    assert_ne!(profile_golden.lines().count(), 12);
    assert!(profile_golden.contains('h'));
    assert!(profile_golden.contains('s'));
    assert!(profile_golden.contains('t'));
    assert!(profile_golden.contains('a'));
    assert!(profile_golden.contains('b'));
    assert!(profile_golden.contains('f'));
}
