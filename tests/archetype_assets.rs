use questmancer::{
    domain::{
        AccentTone, AdventurerClass, AdventurerPersona, Garb, HairTone, PersonaKey, SkinTone,
    },
    scene::{
        assets::{
            adventurer::{
                adventurer_animation_frame, adventurer_portrait_frame, adventurer_roster_frame,
            },
            roster::{RosterFamily, master as roster_master},
        },
        pixel::PixelSize,
        stage::ScenePose,
    },
};

const CORE_ARCHETYPES: [AdventurerClass; 8] = [
    AdventurerClass::Barbarian,
    AdventurerClass::Bard,
    AdventurerClass::Cleric,
    AdventurerClass::Druid,
    AdventurerClass::Paladin,
    AdventurerClass::Ranger,
    AdventurerClass::Rogue,
    AdventurerClass::Wizard,
];

#[test]
fn cleric_sunward_world_and_fallback_use_the_approved_open_face_and_equipment() {
    let approved: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/art/cleric-sunward-master.json"))
            .expect("approved Sunward Chaplain study");
    let mut persona = AdventurerPersona::for_key(PersonaKey::new("sunward-approval"));
    persona.class = AdventurerClass::Cleric;
    for skin in SkinTone::ALL {
        persona.appearance.skin_tone = *skin;
        let world = adventurer_animation_frame(&persona, ScenePose::Working, 0);
        let card = adventurer_portrait_frame(&persona).expect("Cleric fallback");
        for (y, row) in approved["rows"]
            .as_array()
            .expect("approved rows")
            .iter()
            .enumerate()
        {
            for (x, glyph) in row.as_str().expect("pixel row").chars().enumerate() {
                assert_eq!(
                    world.pixels()[y * 16 + x].is_some(),
                    glyph != '.',
                    "Cleric {skin:?} approved A silhouette at ({x},{y})"
                );
                assert_eq!(
                    card.pixels()[(y + 4) * 24 + x + 4],
                    world.pixels()[y * 16 + x],
                    "fallback must retain the same unscaled personalised Cleric"
                );
            }
        }
    }
}

#[test]
fn cleric_sunward_rituals_keep_equipment_and_anchors_with_distinct_gestures() {
    let mut persona = AdventurerPersona::for_key(PersonaKey::new("sunward-rituals"));
    persona.class = AdventurerClass::Cleric;
    let work_a = adventurer_animation_frame(&persona, ScenePose::Working, 0);
    let work_b = adventurer_animation_frame(&persona, ScenePose::Working, 1);
    let mace = work_a.pixels()[5 * 16 + 2].expect("approved sun mace core");
    assert_ne!(work_a, work_b, "working hands and book must change");
    assert_eq!(
        &work_a.pixels()[..11 * 16],
        &work_b.pixels()[..11 * 16],
        "head and mace must stay still"
    );
    assert_eq!(
        &work_a.pixels()[19 * 16..],
        &work_b.pixels()[19 * 16..],
        "feet must stay planted"
    );
    let signal = adventurer_animation_frame(&persona, ScenePose::SeekingCounsel, 0);
    let wait = adventurer_animation_frame(&persona, ScenePose::SeekingCounsel, 1);
    assert_ne!(signal, wait, "raised counsel hand must settle");
    let settled = adventurer_animation_frame(&persona, ScenePose::Settled, 0);
    for (pose, index) in [
        (ScenePose::Working, 0),
        (ScenePose::Working, 1),
        (ScenePose::SeekingCounsel, 0),
        (ScenePose::SeekingCounsel, 1),
        (ScenePose::ReturningWithSpoils, 0),
        (ScenePose::Resting, 0),
        (ScenePose::Unknown, 0),
        (ScenePose::Settled, 0),
    ] {
        let frame = adventurer_animation_frame(&persona, pose, index);
        assert_eq!(frame.size(), PixelSize::new(16, 24));
        let feet = frame.pixels()[21 * 16..22 * 16]
            .iter()
            .enumerate()
            .filter_map(|(x, p)| p.is_some().then_some(x))
            .collect::<Vec<_>>();
        assert_eq!(
            feet,
            vec![5, 6, 7, 10, 11, 12],
            "{pose:?} {index} foot anchors"
        );
        assert!(frame.pixels()[22 * 16..].iter().all(Option::is_none));
        assert_eq!(
            frame.pixels()[5 * 16 + 2],
            Some(mace),
            "sun mace in {pose:?}"
        );
        assert!(
            frame.pixels()[16 * 16 + 13].is_some(),
            "prayerbook stays visible in {pose:?}"
        );
        if pose != ScenePose::Settled {
            assert_ne!(
                frame, settled,
                "{pose:?} must not assert settled completion"
            );
        }
    }
}

#[test]
fn sorcerer_storm_native_route_embeds_the_selected_card() {
    use sha2::{Digest, Sha256};
    let approved: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/art/sorcerer-storm-masters.json"))
            .expect("approved Storm Channeler study");
    let bytes = questmancer::portrait::portrait_asset(AdventurerClass::Sorcerer)
        .expect("native Sorcerer card");
    assert_eq!(
        format!("{:x}", Sha256::digest(bytes)),
        approved["native_sha256"]
            .as_str()
            .expect("selected card digest"),
        "native card must match the selected Storm Channeler asset"
    );
}

#[test]
fn sorcerer_storm_world_and_card_retain_the_approved_quiet_orb_silhouettes() {
    let approved: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/art/sorcerer-storm-masters.json"))
            .expect("approved Storm Channeler study");
    let mut persona = AdventurerPersona::for_key(PersonaKey::new("storm-approval"));
    persona.class = AdventurerClass::Sorcerer;
    for skin in SkinTone::ALL {
        persona.appearance.skin_tone = *skin;
        for (index, pose) in [ScenePose::Working, ScenePose::Resting, ScenePose::Unknown]
            .into_iter()
            .enumerate()
        {
            let frame = adventurer_animation_frame(&persona, pose, 0);
            for (y, row) in approved["options"][index]["rows"]
                .as_array()
                .expect("approved pixel rows")
                .iter()
                .enumerate()
            {
                for (x, glyph) in row.as_str().expect("pixel row").chars().enumerate() {
                    assert_eq!(
                        frame.pixels()[y * 16 + x].is_some(),
                        glyph != '.',
                        "Sorcerer {skin:?} {pose:?} approved silhouette at ({x},{y})"
                    );
                }
            }
            assert!(frame.pixels()[..16].iter().all(Option::is_none), "no halo");
            let orb = frame.pixels()[13 * 16 + 13].expect("quiet orb retains its core");
            assert!(orb.g > 200 && orb.b > 200, "class-owned mint orb core");
        }
        let world = adventurer_animation_frame(&persona, ScenePose::Working, 0);
        let card = adventurer_portrait_frame(&persona).expect("Sorcerer fallback");
        for y in 0..24 {
            for x in 0..16 {
                assert_eq!(
                    card.pixels()[(y + 4) * 24 + x + 4],
                    world.pixels()[y * 16 + x]
                );
            }
        }
    }
}

#[test]
fn sorcerer_storm_rituals_keep_head_feet_and_orb_with_distinct_state_gestures() {
    let mut persona = AdventurerPersona::for_key(PersonaKey::new("storm-rituals"));
    persona.class = AdventurerClass::Sorcerer;
    let work_a = adventurer_animation_frame(&persona, ScenePose::Working, 0);
    let work_b = adventurer_animation_frame(&persona, ScenePose::Working, 1);
    assert_ne!(work_a, work_b, "working focus must move");
    assert_eq!(
        &work_a.pixels()[..9 * 16],
        &work_b.pixels()[..9 * 16],
        "no head bob"
    );
    assert_eq!(
        &work_a.pixels()[19 * 16..],
        &work_b.pixels()[19 * 16..],
        "feet stay planted"
    );
    let signal = adventurer_animation_frame(&persona, ScenePose::SeekingCounsel, 0);
    let wait = adventurer_animation_frame(&persona, ScenePose::SeekingCounsel, 1);
    assert_ne!(signal, wait, "counsel gesture must settle");
    let settled = adventurer_animation_frame(&persona, ScenePose::Settled, 0);
    for (pose, index) in [
        (ScenePose::Working, 0),
        (ScenePose::Working, 1),
        (ScenePose::SeekingCounsel, 0),
        (ScenePose::SeekingCounsel, 1),
        (ScenePose::ReturningWithSpoils, 0),
        (ScenePose::Resting, 0),
        (ScenePose::Unknown, 0),
        (ScenePose::Settled, 0),
    ] {
        let frame = adventurer_animation_frame(&persona, pose, index);
        assert_eq!(frame.size(), PixelSize::new(16, 24));
        let feet = frame.pixels()[21 * 16..22 * 16]
            .iter()
            .enumerate()
            .filter_map(|(x, p)| p.is_some().then_some(x))
            .collect::<Vec<_>>();
        assert_eq!(feet, vec![4, 5, 6, 9, 10, 11], "{pose:?} {index} anchors");
        assert!(frame.pixels()[22 * 16..].iter().all(Option::is_none));
        assert!(
            frame.pixels()[13 * 16 + 13].is_some(),
            "orb in {pose:?} {index}"
        );
        if pose != ScenePose::Settled {
            assert_ne!(
                frame, settled,
                "{pose:?} must not assert settled completion"
            );
        }
    }
}

#[test]
fn druid_world_and_card_use_the_approved_bramble_silhouette() {
    let approved: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/art/druid-bramble-master.json"))
            .expect("approved Druid study fixture");
    let mut persona = AdventurerPersona::for_key(PersonaKey::new("bramble-approval"));
    persona.class = AdventurerClass::Druid;
    for skin in [SkinTone::Porcelain, SkinTone::Ebony] {
        persona.appearance.skin_tone = skin;
        let world = adventurer_animation_frame(&persona, ScenePose::Working, 0);
        let card = adventurer_portrait_frame(&persona).expect("Druid authored card fallback");
        for (y, row) in approved["rows"]
            .as_array()
            .expect("approved rows")
            .iter()
            .enumerate()
        {
            for (x, glyph) in row.as_str().expect("pixel row").chars().enumerate() {
                let expected = glyph != '.';
                assert_eq!(
                    world.pixels()[y * 16 + x].is_some(),
                    expected,
                    "Druid {skin:?} must use approved B at ({x},{y})"
                );
                assert_eq!(
                    card.pixels()[(y + 4) * 24 + x + 4],
                    world.pixels()[y * 16 + x],
                    "card fallback must use the same unscaled Druid at ({x},{y})"
                );
            }
        }
    }
}

#[test]
fn druid_rituals_preserve_bramble_head_and_feet_with_distinct_semantic_poses() {
    let mut persona = AdventurerPersona::for_key(PersonaKey::new("bramble-rituals"));
    persona.class = AdventurerClass::Druid;
    let work_a = adventurer_animation_frame(&persona, ScenePose::Working, 0);
    let work_b = adventurer_animation_frame(&persona, ScenePose::Working, 1);
    assert_ne!(work_a, work_b, "working hands and foliage must change");
    assert_eq!(
        &work_a.pixels()[..12 * 16],
        &work_b.pixels()[..12 * 16],
        "working must not bob the head"
    );
    assert_eq!(
        &work_a.pixels()[20 * 16..],
        &work_b.pixels()[20 * 16..],
        "working feet stay planted"
    );
    let signal = adventurer_animation_frame(&persona, ScenePose::SeekingCounsel, 0);
    let wait = adventurer_animation_frame(&persona, ScenePose::SeekingCounsel, 1);
    assert_ne!(signal, wait, "the raised counsel gesture must settle");
    let settled = adventurer_animation_frame(&persona, ScenePose::Settled, 0);
    for pose in [
        ScenePose::Working,
        ScenePose::SeekingCounsel,
        ScenePose::ReturningWithSpoils,
        ScenePose::Resting,
        ScenePose::Unknown,
        ScenePose::Settled,
    ] {
        let frame = adventurer_animation_frame(&persona, pose, 0);
        assert_eq!(frame.size(), PixelSize::new(16, 24), "{pose:?}");
        let feet = frame.pixels()[21 * 16..22 * 16]
            .iter()
            .enumerate()
            .filter_map(|(x, p)| p.is_some().then_some(x))
            .collect::<Vec<_>>();
        assert_eq!(
            feet,
            vec![4, 5, 6, 9, 10, 11],
            "{pose:?} must retain the approved foot anchors"
        );
        assert!(
            frame.pixels()[22 * 16..].iter().all(Option::is_none),
            "{pose:?} must not drift down"
        );
        if pose != ScenePose::Settled {
            assert_ne!(frame, settled, "{pose:?} must not claim settled completion");
        }
    }
}

#[test]
fn every_core_archetype_uses_a_world_master() {
    for class in CORE_ARCHETYPES {
        let mut persona =
            AdventurerPersona::for_key(PersonaKey::new(format!("core-archetype-{class:?}")));
        persona.class = class;

        let frame = adventurer_animation_frame(&persona, ScenePose::Working, 1);

        assert_eq!(
            frame.size(),
            PixelSize::new(16, 24),
            "{class:?} does not use a world master"
        );
    }
}

#[test]
fn every_core_archetype_has_a_portrait_canvas() {
    for class in CORE_ARCHETYPES {
        let mut persona =
            AdventurerPersona::for_key(PersonaKey::new(format!("core-portrait-{class:?}")));
        persona.class = class;

        let frame = adventurer_portrait_frame(&persona)
            .unwrap_or_else(|| panic!("{class:?} has no portrait master"));

        assert_eq!(frame.size(), PixelSize::new(24, 32), "{class:?}");
    }
}

#[test]
fn ritual_card_fallbacks_reuse_the_current_personalised_sprite_without_stretching() {
    for &class in AdventurerClass::ALL {
        for (skin, hair) in [
            (SkinTone::Porcelain, HairTone::Black),
            (SkinTone::Ebony, HairTone::Gold),
        ] {
            let mut persona = AdventurerPersona::for_key(PersonaKey::new("card-continuity"));
            persona.class = class;
            persona.appearance.skin_tone = skin;
            persona.appearance.hair_tone = hair;
            let world = adventurer_animation_frame(&persona, ScenePose::Working, 0);
            let card = adventurer_portrait_frame(&persona).expect("authored card fallback");

            assert_eq!(card.size(), PixelSize::new(24, 32));
            for y in 0..32 {
                for x in 0..24 {
                    let expected = if (4..20).contains(&x) && (4..28).contains(&y) {
                        world.pixels()[(y - 4) * 16 + x - 4]
                    } else {
                        None
                    };
                    assert_eq!(
                        card.pixels()[y * 24 + x],
                        expected,
                        "{class:?} {skin:?} card pixel ({x}, {y}) must retain the current sprite"
                    );
                }
            }
        }
    }
}

#[test]
fn core_archetype_masters_are_pairwise_distinct_at_both_scales() {
    let frames = CORE_ARCHETYPES.map(|class| {
        let mut persona =
            AdventurerPersona::for_key(PersonaKey::new(format!("distinct-master-{class:?}")));
        persona.class = class;
        (
            class,
            adventurer_animation_frame(&persona, ScenePose::Working, 0),
            adventurer_portrait_frame(&persona).expect("core portrait exists"),
        )
    });

    for (index, (class, world, portrait)) in frames.iter().enumerate() {
        for (other_class, other_world, other_portrait) in frames.iter().skip(index + 1) {
            assert_ne!(
                world, other_world,
                "{class:?} aliases {other_class:?} at world scale"
            );
            assert_ne!(
                portrait, other_portrait,
                "{class:?} aliases {other_class:?} at portrait scale"
            );
        }
    }
}

#[test]
fn every_domain_class_routes_to_authored_world_and_portrait_assets() {
    for class in AdventurerClass::ALL {
        let mut persona =
            AdventurerPersona::for_key(PersonaKey::new(format!("production-route-{class:?}")));
        persona.class = *class;

        let world = adventurer_animation_frame(&persona, ScenePose::Working, 0);
        let portrait = adventurer_portrait_frame(&persona)
            .unwrap_or_else(|| panic!("{class:?} has no portrait route"));

        assert_eq!(world.size(), PixelSize::new(16, 24), "{class:?}");
        assert_eq!(portrait.size(), PixelSize::new(24, 32), "{class:?}");
    }
}

#[test]
fn same_class_personas_with_different_appearance_render_distinct_world_masters() {
    for class in AdventurerClass::ALL {
        let mut left =
            AdventurerPersona::for_key(PersonaKey::new(format!("persona-left-{class:?}")));
        left.class = *class;
        left.appearance.skin_tone = SkinTone::Porcelain;
        left.appearance.hair_tone = HairTone::Black;
        left.appearance.accent = AccentTone::Cyan;
        let mut right = left.clone();
        right.appearance.skin_tone = SkinTone::Ebony;
        right.appearance.hair_tone = HairTone::Gold;
        right.appearance.accent = AccentTone::Red;

        let left_frame = adventurer_animation_frame(&left, ScenePose::Working, 0);
        let right_frame = adventurer_animation_frame(&right, ScenePose::Working, 0);

        assert_ne!(
            left_frame, right_frame,
            "{class:?}: personas with different appearance alias the same master"
        );
    }
}

#[test]
fn persona_substitution_preserves_the_authored_silhouette() {
    for class in AdventurerClass::ALL {
        let mut left =
            AdventurerPersona::for_key(PersonaKey::new(format!("silhouette-left-{class:?}")));
        left.class = *class;
        left.appearance.skin_tone = SkinTone::Porcelain;
        left.appearance.hair_tone = HairTone::Black;
        left.appearance.accent = AccentTone::Cyan;
        let mut right = left.clone();
        right.appearance.skin_tone = SkinTone::Ebony;
        right.appearance.hair_tone = HairTone::Gold;
        right.appearance.accent = AccentTone::Red;

        let left_frame = adventurer_animation_frame(&left, ScenePose::Working, 0);
        let right_frame = adventurer_animation_frame(&right, ScenePose::Working, 0);

        assert_eq!(left_frame.size(), right_frame.size(), "{class:?}");
        let masks_match = left_frame
            .pixels()
            .iter()
            .zip(right_frame.pixels())
            .all(|(left, right)| left.is_some() == right.is_some());
        assert!(
            masks_match,
            "{class:?}: persona substitution altered the transparency mask"
        );
    }
}

#[test]
fn persona_substitution_is_deterministic_for_a_fixed_persona() {
    for class in AdventurerClass::ALL {
        let mut persona =
            AdventurerPersona::for_key(PersonaKey::new(format!("deterministic-{class:?}")));
        persona.class = *class;

        assert_eq!(
            adventurer_animation_frame(&persona, ScenePose::Working, 0),
            adventurer_animation_frame(&persona, ScenePose::Working, 0),
            "{class:?}"
        );
    }
}

/// Class is the primary visual identity, so no two classes may wear the same
/// body. Six classes used to: Artificer and Runewright borrowed the Wizard,
/// Testmender the Cleric, Pathseeker the Ranger. A borrowed master makes two
/// different adventurers indistinguishable in the world.
#[test]
fn no_two_classes_share_a_world_or_portrait_master() {
    let masters = AdventurerClass::ALL
        .iter()
        .map(|class| {
            let mut persona =
                AdventurerPersona::for_key(PersonaKey::new(format!("alias-{class:?}")));
            persona.class = *class;
            // A fixed appearance so palette variation cannot mask two classes
            // sharing one authored master.
            persona.appearance.skin_tone = SkinTone::Sand;
            persona.appearance.hair_tone = HairTone::Chestnut;
            persona.appearance.accent = AccentTone::Amber;
            (
                class,
                adventurer_animation_frame(&persona, ScenePose::Working, 0),
                adventurer_portrait_frame(&persona).expect("every class has a portrait"),
            )
        })
        .collect::<Vec<_>>();

    for (index, (class, world, portrait)) in masters.iter().enumerate() {
        for (other, other_world, other_portrait) in masters.iter().skip(index + 1) {
            assert_ne!(
                world, other_world,
                "{class:?} wears {other:?}'s body at world scale"
            );
            assert_ne!(
                portrait, other_portrait,
                "{class:?} wears {other:?}'s face at portrait scale"
            );
        }
    }
}

/// The pose contract: a semantic state must change the sprite, not only the
/// label beside it. Before this, eleven of twelve classes rendered identically
/// across every pose, so the world never showed what an agent was doing.
#[test]
fn returning_and_resting_change_every_class_sprite() {
    for class in AdventurerClass::ALL {
        let mut persona = AdventurerPersona::for_key(PersonaKey::new(format!("pose-{class:?}")));
        persona.class = *class;

        let working = adventurer_animation_frame(&persona, ScenePose::Working, 0);
        let spoils = adventurer_animation_frame(&persona, ScenePose::ReturningWithSpoils, 0);
        let resting = adventurer_animation_frame(&persona, ScenePose::Resting, 0);

        assert_ne!(
            working, spoils,
            "{class:?}: returning with spoils looks identical to working"
        );
        assert_ne!(
            working, resting,
            "{class:?}: resting looks identical to working"
        );
        assert_ne!(
            spoils, resting,
            "{class:?}: returning with spoils looks identical to resting"
        );
        assert_eq!(working.size(), spoils.size(), "{class:?}");
        assert_eq!(working.size(), resting.size(), "{class:?}");
    }
}

/// A pose decoration must not repaint the whole adventurer: the class has to
/// stay recognisable while its state changes.
///
/// The Barbarian is excluded because it does not use decorations at all — it
/// has a fully authored frame per pose, the reference the contract is modelled
/// on, and authored art is free to redraw as much as it likes.
#[test]
fn pose_decorations_leave_most_of_the_class_master_intact() {
    for class in AdventurerClass::ALL {
        if *class == AdventurerClass::Barbarian {
            continue;
        }
        let mut persona = AdventurerPersona::for_key(PersonaKey::new(format!("intact-{class:?}")));
        persona.class = *class;
        let working = adventurer_animation_frame(&persona, ScenePose::Working, 0);

        for pose in [ScenePose::ReturningWithSpoils, ScenePose::Resting] {
            let posed = adventurer_animation_frame(&persona, pose, 0);
            let shared = working
                .pixels()
                .iter()
                .zip(posed.pixels())
                .filter(|(left, right)| left == right)
                .count();
            let total = working.pixels().len();
            assert!(
                shared * 100 >= total * 70,
                "{class:?} {pose:?}: only {shared}/{total} pixels survived the pose"
            );
        }
    }
}

/// Garb is part of the persisted persona, so it has to be visible somewhere.
/// It reads in the trim band rather than the body mass, but two adventurers
/// alike in every other respect must still not render identically.
#[test]
fn garb_changes_the_sprite_without_changing_the_class() {
    for class in AdventurerClass::ALL {
        let mut robed = AdventurerPersona::for_key(PersonaKey::new(format!("garb-{class:?}")));
        robed.class = *class;
        robed.appearance.garb = Garb::Robes;
        let mut armoured = robed.clone();
        armoured.appearance.garb = Garb::Armour;

        let robed_frame = adventurer_animation_frame(&robed, ScenePose::Working, 0);
        let armoured_frame = adventurer_animation_frame(&armoured, ScenePose::Working, 0);

        // The Barbarian's masters carry no trim cluster, so garb has nowhere
        // to show on it; every other class must respond.
        if *class != AdventurerClass::Barbarian {
            assert_ne!(
                robed_frame, armoured_frame,
                "{class:?}: garb is persisted but invisible"
            );
        }

        let shared = robed_frame
            .pixels()
            .iter()
            .zip(armoured_frame.pixels())
            .filter(|(left, right)| left == right)
            .count();
        assert!(
            shared * 100 >= robed_frame.pixels().len() * 80,
            "{class:?}: garb repainted the class instead of trimming it"
        );
    }
}

#[test]
fn every_class_has_an_authored_roster_master_at_the_small_tier() {
    for class in AdventurerClass::ALL {
        let mut persona = AdventurerPersona::for_key(PersonaKey::new(format!("roster-{class:?}")));
        persona.class = *class;

        let frame = adventurer_roster_frame(&persona);

        assert_eq!(frame.size(), PixelSize::new(8, 12), "{class:?}");
        assert!(
            frame.pixels().iter().any(Option::is_some),
            "{class:?} roster master is empty"
        );
        let bottom_row_has_feet = frame.pixels()[8 * 11..].iter().any(Option::is_some)
            || frame.pixels()[8 * 10..8 * 11].iter().any(Option::is_some);
        assert!(
            bottom_row_has_feet,
            "{class:?} roster master has no foot row"
        );
    }
}

#[test]
fn roster_silhouette_families_are_pairwise_distinct() {
    let families = RosterFamily::ALL
        .iter()
        .map(|family| (family, roster_master(*family).0))
        .collect::<Vec<_>>();

    for (index, (family, frame)) in families.iter().enumerate() {
        for (other_family, other_frame) in families.iter().skip(index + 1) {
            assert_ne!(
                frame, other_frame,
                "roster {family:?} aliases roster {other_family:?}"
            );
        }
    }
}

#[test]
fn all_classes_have_distinct_small_silhouettes_even_without_colour() {
    let mut persona = AdventurerPersona::for_key(PersonaKey::new("roster-silhouette"));
    let mut silhouettes = Vec::new();
    for &class in AdventurerClass::ALL {
        persona.class = class;
        let sprite = adventurer_roster_frame(&persona);
        let mask = sprite
            .pixels()
            .iter()
            .map(Option::is_some)
            .collect::<Vec<_>>();
        for (other, other_mask) in &silhouettes {
            assert_ne!(&mask, other_mask, "{class:?} shares {other:?}'s silhouette");
        }
        silhouettes.push((class, mask));
    }
}

#[test]
fn same_class_personas_stay_distinguishable_at_roster_scale() {
    let mut left = AdventurerPersona::for_key(PersonaKey::new("roster-left"));
    left.class = AdventurerClass::Sorcerer;
    left.appearance.skin_tone = SkinTone::Porcelain;
    left.appearance.hair_tone = HairTone::Black;
    left.appearance.accent = AccentTone::Cyan;
    let mut right = left.clone();
    // A dedicated silhouette still needs to distinguish two of the same class.
    right.appearance.skin_tone = SkinTone::Ebony;
    right.appearance.hair_tone = HairTone::Gold;
    right.appearance.accent = AccentTone::Red;

    assert_ne!(
        adventurer_roster_frame(&left),
        adventurer_roster_frame(&right)
    );
}

#[test]
fn barbarian_v2_has_truthful_pose_specific_world_frames() {
    let mut persona = AdventurerPersona::for_key(PersonaKey::new("barbarian-v2-poses"));
    persona.class = AdventurerClass::Barbarian;

    let settled = adventurer_animation_frame(&persona, ScenePose::Settled, 0);
    let working_0 = adventurer_animation_frame(&persona, ScenePose::Working, 0);
    let working_1 = adventurer_animation_frame(&persona, ScenePose::Working, 1);
    let counsel = adventurer_animation_frame(&persona, ScenePose::SeekingCounsel, 0);
    let spoils = adventurer_animation_frame(&persona, ScenePose::ReturningWithSpoils, 0);
    let resting = adventurer_animation_frame(&persona, ScenePose::Resting, 0);
    let unknown = adventurer_animation_frame(&persona, ScenePose::Unknown, 0);

    for frame in [
        &settled, &working_0, &working_1, &counsel, &spoils, &resting, &unknown,
    ] {
        assert_eq!(frame.size(), PixelSize::new(16, 24));
    }
    assert_ne!(
        working_0, working_1,
        "working must animate by authored pixels"
    );
    for (label, frame) in [
        ("working", &working_0),
        ("counsel", &counsel),
        ("spoils", &spoils),
        ("resting", &resting),
        ("unknown", &unknown),
    ] {
        assert_ne!(settled, *frame, "{label} aliases the settled pose");
    }
}
