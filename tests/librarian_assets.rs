use questmancer::portrait::librarian_asset;
use questmancer::scene::assets::librarian::{ledger_portrait, world};

#[test]
fn librarian_world_master_is_native_scale_and_non_empty() {
    let sprite = world();

    assert_eq!((sprite.size().width, sprite.size().height), (16, 24));
    assert!(sprite.pixels().iter().any(Option::is_some));
    assert_eq!(sprite.pixels()[0], None);
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
