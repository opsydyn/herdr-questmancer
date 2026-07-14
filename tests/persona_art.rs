use std::collections::HashSet;

use herdr_webmaster::{
    domain::{
        AccentTone, Accessory, BodyProportions, DeskProp, FaceDetail, HairShape, HairTone,
        HeadShape, OutfitBottom, OutfitTop, PersonaAppearance, Shoes, SkinTone,
    },
    ui::{
        persona::{AppearanceRoles, appearance_roles, compose_profile, compose_seated},
        pixel::{Canvas, ColorRole, Palette, pack},
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
        accent: AccentTone::Amber,
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
        bottom: OutfitBottom::Cargos,
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
fn appearance_mapping_preserves_typed_colour_traits_and_ansi_contrast() {
    let compact_roles = appearance_roles(&compact());
    let tall_roles = appearance_roles(&tall());

    assert_ne!(compact_roles.skin, tall_roles.skin);
    assert_ne!(compact_roles.hair, tall_roles.hair);
    assert_ne!(compact_roles.top, tall_roles.top);
    assert_ne!(compact_roles.bottom, tall_roles.bottom);
    assert_ne!(compact_roles.accent, tall_roles.accent);

    for roles in [compact_roles, tall_roles, appearance_roles(&broad())] {
        for (upper, lower) in [
            (roles.hair, roles.skin),
            (roles.skin, roles.top),
            (roles.top, roles.bottom),
            (roles.bottom, roles.shoes),
            (roles.top, roles.accessory),
            (roles.accessory, roles.accent),
            (roles.hair, ColorRole::PanelBackground),
            (roles.shoes, ColorRole::PanelBackground),
        ] {
            let mut pair = Canvas::new(1, 2);
            pair.set(0, 0, upper);
            pair.set(0, 1, lower);
            let text = pack(&pair, &Palette::Ansi16, ColorRole::PanelBackground);
            assert_eq!(text.lines[0].spans[0].content, "▀");
            assert_ne!(
                text.lines[0].spans[0].style.fg,
                text.lines[0].spans[0].style.bg
            );
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
