use std::sync::OnceLock;

use super::{IndexedPaletteEntry, indexed_sprite};
use crate::scene::{pixel::Rgb, sprite::SpriteFrame};

pub(crate) fn illustration() -> &'static SpriteFrame {
    static SCROLL: OnceLock<SpriteFrame> = OnceLock::new();
    SCROLL.get_or_init(|| {
        let rows = [
            "..oooo..t.",
            ".owhhho...",
            "ohooooso..",
            ".oowwsoo..",
            "..owtwso..",
            "..otbtso.t",
            "..owtwso..",
            ".ohwwssoo.",
            "owoooosho.",
            ".ooooooo..",
        ];
        let palette = [
            ('o', Rgb::new(55, 36, 35)),
            ('h', Rgb::new(254, 240, 168)),
            ('w', Rgb::new(242, 218, 149)),
            ('s', Rgb::new(171, 103, 49)),
            ('t', Rgb::new(35, 133, 133)),
            ('b', Rgb::new(156, 237, 201)),
        ]
        .map(|(key, colour)| IndexedPaletteEntry {
            key,
            colour: Some(colour),
        });
        indexed_sprite(&rows, &palette).expect("authored Inbox scroll rows and palette are valid")
    })
}
