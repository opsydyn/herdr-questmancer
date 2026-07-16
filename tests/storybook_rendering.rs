#![cfg(feature = "storybook")]

use questmancer::{
    domain::{AdventurerPersona, PersonaKey},
    storybook::{
        AssetId, asset_inventory,
        catalogue::catalogue,
        fixtures::{AtlasContent, StoryContext, StoryFixture},
    },
    ui::{
        persona::compose_chamber_adventurer_for_palette,
        pixel::pack,
        theatre::{TheatreFrame, TheatrePose},
    },
};

#[test]
fn class_atlas_uses_production_profile_canvases() {
    let story = catalogue()
        .iter()
        .find(|story| story.id.as_str() == "atlas.classes")
        .unwrap();
    let StoryFixture::AssetAtlas(atlas) = (story.build)(&StoryContext::fixed()) else {
        panic!("class atlas must be an asset atlas");
    };
    assert_eq!(atlas.tiles.len(), 11);
    for tile in &atlas.tiles {
        let AtlasContent::Pixel { canvas, .. } = &tile.content else {
            panic!("class tiles must contain production pixel canvases");
        };
        assert_eq!((canvas.width(), canvas.height()), (16, 32));
        assert!(canvas.pixels().iter().any(Option::is_some));
    }
}

#[test]
fn pose_atlas_uses_all_seven_production_theatre_poses() {
    let story = catalogue()
        .iter()
        .find(|story| story.id.as_str() == "atlas.poses")
        .unwrap();
    let StoryFixture::AssetAtlas(atlas) = (story.build)(&StoryContext::fixed()) else {
        panic!("pose atlas must be an asset atlas");
    };
    assert_eq!(atlas.tiles.len(), 7);
}

#[test]
fn pixel_atlases_are_packed_through_the_production_packer() {
    for story in catalogue() {
        let StoryFixture::AssetAtlas(atlas) = (story.build)(&StoryContext::fixed()) else {
            panic!("atlas catalogue entries must build asset atlases");
        };
        for tile in atlas.tiles {
            let AtlasContent::Pixel {
                canvas,
                palette,
                background,
                packed,
            } = tile.content
            else {
                panic!("Task 4 atlas tiles must contain production pixel content");
            };
            assert_eq!(packed, pack(&canvas, &palette, background));
        }
    }
}

#[test]
fn every_atlas_builder_matches_its_canonical_asset_family() {
    let inventory = asset_inventory();
    for story in catalogue() {
        let StoryFixture::AssetAtlas(atlas) = (story.build)(&StoryContext::fixed()) else {
            panic!("atlas catalogue entries must build asset atlases");
        };
        let expected = inventory
            .iter()
            .filter(|asset| asset_belongs_to_story(**asset, story.id.as_str()))
            .map(|asset| asset.label())
            .collect::<Vec<_>>();
        let actual = atlas
            .tiles
            .iter()
            .map(|tile| tile.label)
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "{}", story.id.as_str());
    }
}

#[test]
fn pose_atlas_uses_the_exact_production_pose_and_frame_mapping() {
    let story = catalogue()
        .iter()
        .find(|story| story.id.as_str() == "atlas.poses")
        .unwrap();
    let StoryFixture::AssetAtlas(atlas) = (story.build)(&StoryContext::fixed()) else {
        panic!("pose atlas must be an asset atlas");
    };
    let persona = AdventurerPersona::for_key(PersonaKey::new("storybook-pose-atlas"));
    let poses = asset_inventory()
        .into_iter()
        .filter_map(|asset| match asset {
            AssetId::Pose(pose) => Some((asset.label(), pose)),
            _ => None,
        });

    for (tile, (label, pose)) in atlas.tiles.iter().zip(poses) {
        let AtlasContent::Pixel {
            canvas, palette, ..
        } = &tile.content
        else {
            panic!("pose tiles must contain production pixel canvases");
        };
        let animation_frame = if pose == TheatrePose::SpoilsUnopened {
            4
        } else {
            0
        };
        let expected = compose_chamber_adventurer_for_palette(
            &persona,
            TheatreFrame {
                pose,
                animation_frame,
                focused: false,
                label,
            },
            *palette,
        );
        assert_eq!(canvas, &expected, "{label}");
    }
}

fn asset_belongs_to_story(asset: AssetId, story_id: &str) -> bool {
    matches!(
        (story_id, asset),
        ("atlas.classes", AssetId::Class(_))
            | ("atlas.ancestries", AssetId::Ancestry(_))
            | ("atlas.body-proportions", AssetId::BodyProportions(_))
            | ("atlas.head-shapes", AssetId::HeadShape(_))
            | ("atlas.skin-tones", AssetId::SkinTone(_))
            | ("atlas.hair-shapes", AssetId::HairShape(_))
            | ("atlas.hair-tones", AssetId::HairTone(_))
            | ("atlas.face-details", AssetId::FaceDetail(_))
            | ("atlas.garb", AssetId::Garb(_))
            | ("atlas.legwear", AssetId::Legwear(_))
            | ("atlas.footwear", AssetId::Footwear(_))
            | ("atlas.keepsakes", AssetId::Keepsake(_))
            | ("atlas.accent-tones", AssetId::AccentTone(_))
            | ("atlas.palette-roles", AssetId::ColorRole(_))
            | ("atlas.poses", AssetId::Pose(_))
    )
}
