use questmancer::{
    domain::{AdventurerClass, AdventurerPersona, PersonaKey},
    scene::{
        assets::adventurer::{adventurer_animation_frame, adventurer_portrait_frame},
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
