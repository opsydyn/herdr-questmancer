//! Small, static card illustrations of the existing saved keepsake assignment.
use crate::domain::Keepsake;

use super::{IndexedPaletteEntry, indexed_sprite};
use crate::scene::{pixel::Rgb, sprite::SpriteFrame};

pub const fn name(keepsake: Keepsake) -> &'static str {
    match keepsake {
        Keepsake::Feather => "Feather",
        Keepsake::LuckyCoin => "Lucky Coin",
        Keepsake::Mug => "Mug",
        Keepsake::PressedLeaf => "Pressed Leaf",
        Keepsake::Ribbon => "Ribbon",
        Keepsake::TinyFamiliar => "Tiny Familiar",
    }
}

pub const fn description(keepsake: Keepsake) -> &'static str {
    match keepsake {
        Keepsake::Feather => "From a bird with excellent timing.",
        Keepsake::LuckyCoin => "Always lands on a promising side.",
        Keepsake::Mug => "The last sip is mostly optimism.",
        Keepsake::PressedLeaf => "A small autumn, carefully folded.",
        Keepsake::Ribbon => "Tied once. Retold many times.",
        Keepsake::TinyFamiliar => "Offers advice in very small squeaks.",
    }
}

pub fn illustration(keepsake: Keepsake) -> SpriteFrame {
    let rows = match keepsake {
        Keepsake::Feather => [
            ".....oo.", "....owo.", "...owwo.", "..owowo.", ".owowo..", ".oowo...", "..oo....",
            ".o......",
        ],
        Keepsake::LuckyCoin => [
            "..oooo..", ".oyhyyo.", "oyhhyyso", "oyhyysso", "oyyyssoo", "oyyssooo", ".ossooo.",
            "..oooo..",
        ],
        Keepsake::Mug => [
            ".ooooo..", ".owhwooo", ".owwwo.o", ".owwwo.o", ".owwwooo", ".oswso..", ".ossso..",
            "..ooo...",
        ],
        Keepsake::PressedLeaf => [
            "...o....", "..ogo...", ".ogggo..", "ogogogo.", "ogggggo.", ".ogogo..", "..ogo...",
            "...oo...",
        ],
        Keepsake::Ribbon => [
            "oo....oo", "orooooro", "orrrorro", ".oorroo.", "..orro..", ".orro...", ".oro....",
            "..o.....",
        ],
        Keepsake::TinyFamiliar => [
            "oo..oo..", "orooro..", "owwwoo..", "owowoo..", ".owwwoo.", ".ossso.o", ".ossssoo",
            "..ooo...",
        ],
    };
    let palette = [
        ('o', Rgb::new(55, 36, 35)),
        ('w', Rgb::new(244, 239, 213)),
        ('h', Rgb::new(255, 245, 152)),
        ('y', Rgb::new(215, 153, 40)),
        ('s', Rgb::new(138, 100, 67)),
        ('g', Rgb::new(81, 119, 48)),
        ('r', Rgb::new(167, 63, 74)),
    ]
    .map(|(key, colour)| IndexedPaletteEntry {
        key,
        colour: Some(colour),
    });
    indexed_sprite(&rows, &palette).expect("authored keepsake palette and rows are valid")
}
