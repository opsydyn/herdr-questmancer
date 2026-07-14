use std::collections::HashSet;

use herdr_webmaster::{
    domain::{
        AccentTone, Accessory, BodyProportions, DeskProp, FaceDetail, HairShape, HairTone,
        HeadShape, OutfitBottom, OutfitTop, PersonaAppearance, Shoes, SkinTone,
    },
    ui::{
        persona::{
            AppearanceRoles, appearance_roles, appearance_roles_for_palette, compose_profile,
            compose_profile_for_palette, compose_seated, compose_seated_for_palette,
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
        top: OutfitTop::StripeJumper,
        bottom: OutfitBottom::Shorts,
        shoes: Shoes::Platforms,
        accessory: Accessory::Headphones,
        desk_prop: DeskProp::NoveltyMug,
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
        top: OutfitTop::HighCollar,
        bottom: OutfitBottom::Slacks,
        shoes: Shoes::Boots,
        accessory: Accessory::Pager,
        desk_prop: DeskProp::Phone,
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
        top: OutfitTop::WorkShirt,
        bottom: OutfitBottom::Jeans,
        shoes: Shoes::Loafers,
        accessory: Accessory::ShoulderBag,
        desk_prop: DeskProp::TinyCactus,
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
    } else if role == roles.top {
        't'
    } else if role == roles.bottom {
        'b'
    } else if role == roles.shoes {
        'f'
    } else if role == roles.accessory {
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
        (roles.top, roles.skin),
        (roles.top, roles.hair),
        (roles.bottom, roles.top),
        (roles.shoes, roles.bottom),
        (roles.accessory, roles.top),
        (roles.accessory, roles.skin),
        (roles.accessory, roles.hair),
        (roles.accent, roles.top),
        (roles.accent, roles.skin),
        (roles.accent, roles.hair),
        (roles.accent, roles.accessory),
    ] {
        assert!(palette.roles_contrast(upper, lower));
    }
}

#[test]
fn fixed_personas_have_exact_dimensions_and_distinct_silhouettes() {
    let appearances = [compact(), tall(), broad()];
    let seated =
        appearances.map(|appearance| compose_seated(&appearance, frame(TheatrePose::Idle, 0)));
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
fn canonical_roles_preserve_requested_typed_traits_without_palette_fallback() {
    let roles = appearance_roles(&broad());

    assert_eq!(roles.skin, ColorRole::SkinTone(SkinShade::Rose));
    assert_eq!(roles.hair, ColorRole::HairTone(HairShade::Black));
    assert_eq!(roles.top, ColorRole::Fabric(FabricShade::Green));
    assert_eq!(roles.bottom, ColorRole::Fabric(FabricShade::Navy));
    assert_eq!(roles.shoes, ColorRole::Footwear(FootwearShade::Black));
    assert_eq!(roles.accessory, ColorRole::AccentTone(AccentShade::Teal));
    assert_eq!(roles.accent, ColorRole::AccentTone(AccentShade::Magenta));
}

#[test]
fn ansi_safe_roles_cover_the_full_persona_adjacency_graph() {
    let mut appearance = compact();
    appearance.skin_tone = SkinTone::Rose;
    appearance.hair_tone = HairTone::Gold;
    appearance.face_detail = FaceDetail::Visor;
    appearance.top = OutfitTop::BandTee;
    appearance.bottom = OutfitBottom::Jeans;
    appearance.shoes = Shoes::Loafers;
    appearance.accessory = Accessory::Headphones;
    appearance.accent = AccentTone::Red;

    let canonical = appearance_roles(&appearance);
    assert_eq!(canonical.skin, ColorRole::SkinTone(SkinShade::Rose));
    assert_eq!(canonical.hair, ColorRole::HairTone(HairShade::Gold));
    assert_eq!(canonical.top, ColorRole::Fabric(FabricShade::Navy));
    assert_eq!(
        canonical.accessory,
        ColorRole::AccentTone(AccentShade::Amber)
    );
    assert_eq!(canonical.accent, ColorRole::AccentTone(AccentShade::Red));

    let safe = appearance_roles_for_palette(&appearance, Palette::Ansi16);
    assert_adjacency_contrast(safe, Palette::Ansi16);
    assert_ne!(safe.accessory, canonical.accessory);
    assert_ne!(safe.accent, canonical.accent);
}

#[test]
fn xterm_safe_roles_preserve_non_colliding_black_hair_and_loafers() {
    let appearance = broad();
    let canonical = appearance_roles(&appearance);
    let safe = appearance_roles_for_palette(&appearance, Palette::Xterm256);

    assert_eq!(safe.hair, ColorRole::HairTone(HairShade::Black));
    assert_eq!(safe.shoes, ColorRole::Footwear(FootwearShade::Black));
    assert_eq!(safe.hair, canonical.hair);
    assert_eq!(safe.shoes, canonical.shoes);
    assert_adjacency_contrast(safe, Palette::Xterm256);
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
        frame(TheatrePose::Working, 0),
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
    let tops = [
        OutfitTop::BandTee,
        OutfitTop::StripeJumper,
        OutfitTop::HighCollar,
        OutfitTop::WorkShirt,
        OutfitTop::Hoodie,
        OutfitTop::Cardigan,
        OutfitTop::Waistcoat,
        OutfitTop::TrackTop,
    ];
    let bottoms = [
        OutfitBottom::Jeans,
        OutfitBottom::Slacks,
        OutfitBottom::Cargos,
        OutfitBottom::Skirt,
        OutfitBottom::Shorts,
    ];
    let shoes = [
        Shoes::Trainers,
        Shoes::Boots,
        Shoes::Loafers,
        Shoes::HighTops,
        Shoes::Platforms,
    ];
    let accessories = [
        Accessory::Headphones,
        Accessory::Pager,
        Accessory::Lanyard,
        Accessory::Wristband,
        Accessory::Scarf,
        Accessory::Badge,
        Accessory::PocketPen,
        Accessory::ShoulderBag,
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
                for top in tops {
                    for bottom in bottoms {
                        for shoes in shoes {
                            for accessory in accessories {
                                for accent in accents {
                                    let mut appearance = compact();
                                    appearance.skin_tone = skin_tone;
                                    appearance.hair_tone = hair_tone;
                                    appearance.top = top;
                                    appearance.bottom = bottom;
                                    appearance.shoes = shoes;
                                    appearance.accessory = accessory;
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
        let seated = compose_seated(&appearance, frame(TheatrePose::Idle, 0));
        let profile = compose_profile(&appearance);

        for canvas in [&seated, &profile] {
            for anchor in [roles.hair, roles.skin, roles.top, roles.accessory] {
                assert!(canvas.pixels().contains(&Some(anchor)));
            }
        }
    }
}

#[test]
fn seated_state_is_explicit_in_the_non_colour_silhouette() {
    let appearance = tall();
    let working = compose_seated(&appearance, frame(TheatrePose::Working, 0));
    let working_next = compose_seated(&appearance, frame(TheatrePose::Working, 1));
    let blocked = compose_seated(&appearance, frame(TheatrePose::Blocked, 0));
    let blocked_next = compose_seated(&appearance, frame(TheatrePose::Blocked, 1));
    let done = compose_seated(&appearance, frame(TheatrePose::DoneSeen, 0));
    let idle = compose_seated(&appearance, frame(TheatrePose::Idle, 0));
    let exited = compose_seated(&appearance, frame(TheatrePose::Exited, 0));

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
    let blocked = compose_seated(&appearance, frame(TheatrePose::Blocked, 0));

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
            "....ssssssss....\n",
            "....ssssssss....\n",
            "....ssssssss....\n",
            "....ssshhhss....\n",
            "....ssssssss....\n",
            "................\n",
            "..tattttcttttt..\n",
            "ssttatttctttttss\n",
            "sstttattctttttss\n",
            "ssttttatctttttss\n",
            "sstttttactttttss\n",
            "ssttttttatttttss\n",
            "ssttttttcattaaas\n",
            "ssttttttctttaaas\n",
            "..ttttttctttaaa.\n",
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
        TheatrePose::Working,
        TheatrePose::Blocked,
        TheatrePose::DoneUnseen,
        TheatrePose::DoneSeen,
        TheatrePose::Idle,
        TheatrePose::Exited,
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
