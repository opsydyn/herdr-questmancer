//! Two authored poses in the sleeping cat's existing 10x5 reservation.
use std::{sync::OnceLock, time::Duration};

use super::{
    IndexedPaletteEntry, indexed_sprite,
    palette::{OAK_DARK, OAK_LIGHT},
};
use crate::scene::sprite::SpriteFrame;

pub const FRAME_DURATION: Duration = Duration::from_millis(400);
pub const REACTION_DURATION: Duration = Duration::from_millis(800);

pub fn reaction(elapsed: Duration) -> Option<(&'static SpriteFrame, Duration)> {
    static FRAMES: OnceLock<[SpriteFrame; 2]> = OnceLock::new();
    if elapsed >= REACTION_DURATION {
        return None;
    }
    let frames = FRAMES.get_or_init(|| {
        [
            [
                ".L.L......",
                "LLoLL.....",
                ".LLLL.LLL.",
                "..LLLLLLLL",
                "..oooooo.o",
            ],
            [
                "..........",
                ".L..L.LLL.",
                ".LLLLLLLLL",
                ".LLoLLLLLL",
                "LLLoooo..o",
            ],
        ]
        .map(|rows| {
            indexed_sprite(
                &rows,
                &[
                    IndexedPaletteEntry {
                        key: 'L',
                        colour: Some(OAK_LIGHT),
                    },
                    IndexedPaletteEntry {
                        key: 'o',
                        colour: Some(OAK_DARK),
                    },
                ],
            )
            .expect("authored cat poses are valid")
        })
    });
    if elapsed < FRAME_DURATION {
        Some((&frames[0], FRAME_DURATION - elapsed))
    } else {
        Some((&frames[1], REACTION_DURATION - elapsed))
    }
}
