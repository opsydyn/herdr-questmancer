#![cfg(feature = "storybook")]

use questmancer::storybook::catalogue::catalogue;

#[test]
fn catalogue_contains_every_production_scene_interaction_once() {
    let titles = catalogue()
        .iter()
        .map(|story| story.title)
        .collect::<Vec<_>>();

    for title in [
        "Interaction / Selected Adventurer",
        "Interaction / Counsel Parchment",
        "Interaction / Search Parchment",
        "Interaction / Scrying Parchment",
        "Interaction / Help Parchment",
        "Interaction / Narrow Parchment",
    ] {
        assert_eq!(
            titles
                .iter()
                .filter(|candidate| **candidate == title)
                .count(),
            1,
            "missing or duplicated Storybook interaction: {title}"
        );
    }
}
