use questmancer::portrait::librarian_asset;
use questmancer::scene::assets::librarian::{ledger_portrait, world};

fn approved_keeper() -> serde_json::Value {
    serde_json::from_str(include_str!("fixtures/art/librarian-keeper-master.json"))
        .expect("approved Keeper of the Stacks fixture")
}

fn approved_colour(fixture: &serde_json::Value, key: &str) -> questmancer::scene::pixel::Rgb {
    let hex = fixture["palette"][key]
        .as_str()
        .expect("approved hex colour");
    questmancer::scene::pixel::Rgb::new(
        u8::from_str_radix(&hex[..2], 16).expect("red hex"),
        u8::from_str_radix(&hex[2..4], 16).expect("green hex"),
        u8::from_str_radix(&hex[4..6], 16).expect("blue hex"),
    )
}

#[test]
fn librarian_world_uses_the_user_selected_keeper_pixels() {
    let approved = approved_keeper();
    for (y, row) in approved["rows"]
        .as_array()
        .expect("approved rows")
        .iter()
        .enumerate()
    {
        for (x, glyph) in row.as_str().expect("pixel row").chars().enumerate() {
            let expected = (glyph != '.').then(|| approved_colour(&approved, &glyph.to_string()));
            assert_eq!(
                world().pixels()[y * 16 + x],
                expected,
                "world Librarian must retain approved B at ({x},{y})"
            );
        }
    }
}

#[test]
fn librarian_ledger_fallback_retains_the_keepers_fur_books_and_robe() {
    let approved = approved_keeper();
    let portrait = ledger_portrait();
    for key in ["F", "g", "P", "r", "b", "u", "w"] {
        assert!(
            portrait
                .pixels()
                .contains(&Some(approved_colour(&approved, key))),
            "Ledger fallback loses Keeper material {key}"
        );
    }
}

#[test]
fn librarian_world_master_is_native_scale_and_non_empty() {
    let sprite = world();

    assert_eq!((sprite.size().width, sprite.size().height), (16, 24));
    assert!(sprite.pixels().iter().any(Option::is_some));
    assert_eq!(sprite.pixels()[0], None);
}

#[test]
fn librarian_art_is_grounded_and_the_ledger_uses_its_larger_canvas() {
    let world = world();
    let foot_row = world.pixels().iter().rposition(Option::is_some).unwrap() / 16;
    assert_eq!(foot_row, 21, "the Librarian shares the party's ground line");

    let portrait = ledger_portrait();
    let occupied_width = portrait
        .pixels()
        .iter()
        .enumerate()
        .filter_map(|(i, pixel)| pixel.is_some().then_some(i % 24))
        .fold((24, 0), |(left, right), x| (left.min(x), right.max(x)));
    assert!(
        occupied_width.1 - occupied_width.0 + 1 > 16,
        "the Ledger has authored portrait detail instead of a padded world sprite"
    );
}

#[test]
fn embedded_librarian_art_is_a_decodable_png() {
    let image = image::load_from_memory_with_format(librarian_asset(), image::ImageFormat::Png)
        .expect("embedded Librarian PNG decodes");

    assert!(image.width() >= 256);
    assert!(image.height() >= 256);
}

#[test]
fn librarian_ledger_fallback_fills_a_readable_portrait_canvas() {
    let sprite = ledger_portrait();

    assert_eq!((sprite.size().width, sprite.size().height), (24, 32));
    let occupied = sprite
        .pixels()
        .iter()
        .filter(|pixel| pixel.is_some())
        .count();
    assert!(
        occupied >= 120,
        "fallback silhouette is too sparse: {occupied}"
    );
    assert_eq!(sprite.pixels()[0], None);
}
