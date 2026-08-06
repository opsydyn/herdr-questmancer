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
fn every_core_archetype_has_an_independent_portrait_master() {
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
fn same_family_personas_stay_distinguishable_at_roster_scale() {
    let mut left = AdventurerPersona::for_key(PersonaKey::new("roster-left"));
    left.class = AdventurerClass::Wizard;
    left.appearance.skin_tone = SkinTone::Porcelain;
    left.appearance.hair_tone = HairTone::Black;
    left.appearance.accent = AccentTone::Cyan;
    let mut right = left.clone();
    // Artificer shares the Caster family, so only the persona palette can
    // separate these two adventurers in a narrow pane.
    right.class = AdventurerClass::Artificer;
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
