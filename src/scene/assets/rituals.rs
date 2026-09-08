//! Authored party rituals from the approved 2026-09-05 and 2026-09-08 storyboards.
//! Frame timing belongs here; both rooms consume the same sample.

mod arcane_art;
mod support_art;
mod tool_art;
mod trail_art;
use std::{sync::OnceLock, time::Duration};

use super::{IndexedPaletteEntry, indexed_sprite};
use crate::{
    app::Motion,
    domain::AdventurerClass,
    scene::{pixel::Rgb, sprite::SpriteFrame, stage::ScenePose},
};

#[derive(Clone, Copy)]
enum Playback {
    Loop,
    Once,
}

struct Sequence {
    frames: &'static [usize],
    frame_ms: u64,
    playback: Playback,
}

const WORKING: Sequence = Sequence {
    frames: &[0, 1],
    frame_ms: 500,
    playback: Playback::Loop,
};
const COUNSEL: Sequence = Sequence {
    frames: &[2, 3],
    frame_ms: 600,
    playback: Playback::Once,
};
const SPOILS: Sequence = Sequence {
    frames: &[4, 7],
    frame_ms: 1_000,
    playback: Playback::Once,
};
const RESTING: Sequence = Sequence {
    frames: &[5],
    frame_ms: 1,
    playback: Playback::Once,
};
const UNKNOWN: Sequence = Sequence {
    frames: &[6],
    frame_ms: 1,
    playback: Playback::Once,
};
const SETTLED: Sequence = Sequence {
    frames: &[7],
    frame_ms: 1,
    playback: Playback::Once,
};

fn sequence(pose: ScenePose) -> &'static Sequence {
    match pose {
        ScenePose::Working => &WORKING,
        ScenePose::SeekingCounsel => &COUNSEL,
        ScenePose::ReturningWithSpoils => &SPOILS,
        ScenePose::Resting => &RESTING,
        ScenePose::Unknown => &UNKNOWN,
        ScenePose::Settled => &SETTLED,
    }
}

const fn class_index(class: AdventurerClass) -> usize {
    match class {
        AdventurerClass::Wizard => 0,
        AdventurerClass::Ranger => 1,
        AdventurerClass::Barbarian => 2,
        AdventurerClass::Bard => 3,
        AdventurerClass::Artificer => 4,
        AdventurerClass::Testmender => 5,
        AdventurerClass::Cleric => 6,
        AdventurerClass::Paladin => 7,
        AdventurerClass::Druid => 8,
        AdventurerClass::Rogue => 9,
        AdventurerClass::Pathseeker => 10,
        AdventurerClass::Runewright => 11,
        AdventurerClass::Mage => 12,
        AdventurerClass::Sorcerer => 13,
    }
}

/// An absent age settles a one-shot gesture. The scene uses this for stale
/// snapshots and socket boundaries, without inventing a new domain event.
pub(crate) fn sample(
    pose: ScenePose,
    motion: Motion,
    elapsed: Option<Duration>,
) -> (u8, Option<Duration>) {
    let sequence = sequence(pose);
    let count = sequence.frames.len();
    let static_frame = match sequence.playback {
        Playback::Loop => 0,
        Playback::Once => count - 1,
    };
    let Some(elapsed) = elapsed.filter(|_| motion == Motion::Full && count > 1) else {
        return (
            u8::try_from(static_frame).expect("authored frame index fits u8"),
            None,
        );
    };
    let elapsed_ms = elapsed.as_millis();
    let step = elapsed_ms / u128::from(sequence.frame_ms);
    let frame = match sequence.playback {
        Playback::Loop => usize::try_from(step % count as u128).expect("bounded frame index"),
        Playback::Once if step >= (count - 1) as u128 => {
            return (
                u8::try_from(count - 1).expect("authored frame index fits u8"),
                None,
            );
        }
        Playback::Once => usize::try_from(step).expect("bounded one-shot frame index"),
    };
    let remaining = u128::from(sequence.frame_ms) - elapsed_ms % u128::from(sequence.frame_ms);
    (
        u8::try_from(frame).expect("authored frame index fits u8"),
        Some(Duration::from_millis(
            u64::try_from(remaining).expect("frame duration fits u64"),
        )),
    )
}

struct ClassArt {
    palette: &'static [IndexedPaletteEntry],
    frames: [&'static [&'static str]; 8],
    roster: Option<&'static [&'static str]>,
}

pub(crate) fn world_master(
    class: AdventurerClass,
    pose: ScenePose,
    frame: u8,
) -> Option<(SpriteFrame, &'static [IndexedPaletteEntry])> {
    static FRAMES: OnceLock<[[SpriteFrame; 8]; ART.len()]> = OnceLock::new();
    let class = class_index(class);
    let frames = FRAMES.get_or_init(|| {
        std::array::from_fn(|class| {
            ART[class].frames.map(|rows| {
                indexed_sprite(rows, ART[class].palette).expect("authored party ritual is valid")
            })
        })
    });
    let sequence = sequence(pose);
    let index = sequence.frames[usize::from(frame) % sequence.frames.len()];
    Some((frames.get(class)?[index].clone(), ART[class].palette))
}

pub(crate) fn roster_master(
    class: AdventurerClass,
) -> Option<(SpriteFrame, &'static [IndexedPaletteEntry])> {
    static FRAMES: OnceLock<[Option<SpriteFrame>; ART.len()]> = OnceLock::new();
    let class = class_index(class);
    let frames = FRAMES.get_or_init(|| {
        std::array::from_fn(|class| {
            ART[class].roster.map(|rows| {
                indexed_sprite(rows, ART[class].palette).expect("authored pilot roster is valid")
            })
        })
    });
    Some((frames.get(class)?.as_ref()?.clone(), ART[class].palette))
}

const WIZARD_PALETTE: &[IndexedPaletteEntry] = &[
    IndexedPaletteEntry {
        key: 'k',
        colour: Some(Rgb::new(166, 125, 106)),
    },
    IndexedPaletteEntry {
        key: 'K',
        colour: Some(Rgb::new(224, 169, 146)),
    },
    IndexedPaletteEntry {
        key: 'h',
        colour: Some(Rgb::new(233, 194, 178)),
    },
    IndexedPaletteEntry {
        key: 'd',
        colour: Some(Rgb::new(92, 57, 34)),
    },
    IndexedPaletteEntry {
        key: 'D',
        colour: Some(Rgb::new(169, 111, 55)),
    },
    IndexedPaletteEntry {
        key: 'l',
        colour: Some(Rgb::new(237, 181, 77)),
    },
    IndexedPaletteEntry {
        key: 'm',
        colour: Some(Rgb::new(112, 132, 134)),
    },
    IndexedPaletteEntry {
        key: 'M',
        colour: Some(Rgb::new(218, 226, 217)),
    },
    IndexedPaletteEntry {
        key: 'e',
        colour: Some(Rgb::new(112, 220, 255)),
    },
    IndexedPaletteEntry {
        key: 'p',
        colour: Some(Rgb::new(230, 207, 154)),
    },
    IndexedPaletteEntry {
        key: 'P',
        colour: Some(Rgb::new(248, 232, 184)),
    },
    IndexedPaletteEntry {
        key: 'a',
        colour: Some(Rgb::new(201, 69, 64)),
    },
    IndexedPaletteEntry {
        key: 'o',
        colour: Some(Rgb::new(33, 28, 49)),
    },
    IndexedPaletteEntry {
        key: 'r',
        colour: Some(Rgb::new(126, 128, 128)),
    },
    IndexedPaletteEntry {
        key: 'R',
        colour: Some(Rgb::new(180, 183, 184)),
    },
    IndexedPaletteEntry {
        key: 'c',
        colour: Some(Rgb::new(60, 49, 126)),
    },
    IndexedPaletteEntry {
        key: 'C',
        colour: Some(Rgb::new(105, 81, 180)),
    },
    IndexedPaletteEntry {
        key: 'v',
        colour: Some(Rgb::new(146, 118, 209)),
    },
];

const WIZARD_WORKING_A: &[&str] = &[
    "................",
    "..........oo....",
    ".........oCo....",
    ".......ooCco....",
    ".....ocCCCCco...",
    "...oollllllloo..",
    "....okKKKKKko...",
    "....oKhKhKhKo...",
    ".e..oKoKKoKKo...",
    "Me..okKKKKKko...",
    ".m.orRKKKRRro...",
    ".m.orRRRRRRro...",
    ".m..orRRRRro....",
    ".m...orRRro.....",
    ".mKocCCCCCCo....",
    ".m.ocKpPdPPKo...",
    ".m.ocoPPdPPoo...",
    ".m.oc.DDoDD.o...",
    ".m..ocCCCCco....",
    ".m...oddddo.....",
    ".m..oDd..dDo....",
    ".m..ooo..ooo....",
    "................",
    "................",
];

const WIZARD_WORKING_B: &[&str] = &[
    "................",
    "..........oo....",
    ".........oCo....",
    ".......ooCco....",
    ".....ocCCCCco...",
    "...oollllllloo..",
    "....okKKKKKko...",
    "....oKhKhKhKo...",
    ".e..oKoKKoKKo...",
    "Me..okKKKKKko...",
    ".m.orRKKKRRro...",
    ".m.orRRRRRRro...",
    ".m..orRRRRro....",
    ".m...orRRro.....",
    ".mKoc..P........",
    ".m.ocKpPPdPKo...",
    ".m.ocoPPPdPoo...",
    ".m.oc.DDoDD.o...",
    ".m..ocCCCCco....",
    ".m...oddddo.....",
    ".m..oDd..dDo....",
    ".m..ooo..ooo....",
    "................",
    "................",
];

const WIZARD_COUNSEL_RAISED: &[&str] = &[
    "................",
    "..........oo....",
    ".........oCo....",
    ".......ooCco....",
    ".....ocCCCCco...",
    "...oollllllloo..",
    "....okKKKKKko...",
    "....oKhKhKhKo...",
    ".e..oKoKKoKKo...",
    "Me..okKKKKKko...",
    ".m.orRKKKRRro...",
    ".m.orRRRRRRroK..",
    ".m..orRRRRro.K..",
    ".m...orRRro..Co.",
    ".mKocCCCCCCo.Co.",
    ".m.ocCvCCvoPD...",
    ".m.ocClllloPD...",
    ".m.ocCCCCCooo...",
    ".m..ocCCCCco....",
    ".m...oddddo.....",
    ".m..oDd..dDo....",
    ".m..ooo..ooo....",
    "................",
    "................",
];

const WIZARD_COUNSEL_WAIT: &[&str] = &[
    "................",
    "..........oo....",
    ".........oCo....",
    ".......ooCco....",
    ".....ocCCCCco...",
    "...oollllllloo..",
    "....okKKKKKko...",
    "....oKhKhKhKo...",
    ".e..oKoKKoKKo...",
    "Me..okKKKKKko...",
    ".m.orRKKKRRro...",
    ".m.orRRRRRRro...",
    ".m..orRRRRro....",
    ".m...orRRro.....",
    ".mKocCCCCCCo....",
    ".m.ocCvCCvoPD...",
    ".m.ocClllloPD...",
    ".m.ocCCCCCooo...",
    ".m..ocCCCCco....",
    ".m...oddddo.....",
    ".m..oDd..dDo....",
    ".m..ooo..ooo....",
    "................",
    "................",
];

const WIZARD_SPOILS: &[&str] = &[
    "................",
    "..........oo....",
    ".........oCo....",
    ".......ooCco....",
    ".....ocCCCCco...",
    "...oollllllloo..",
    "....okKKKKKko...",
    "....oKhKhKhKo...",
    ".e..oKoKKoKKo...",
    "Me..okKKKKKko...",
    ".m.orRKKKRRro...",
    ".m.orRRRRRRro...",
    ".m..orRRRRro....",
    ".m...orRRro.....",
    ".mKocCCCCCCo....",
    ".m.ocCKDDDCKo...",
    ".m.ocCoPalDoo...",
    ".m.ocCoPaPDoo...",
    ".m..oc.oooo.....",
    ".m...oddddo.....",
    ".m..oDd..dDo....",
    ".m..ooo..ooo....",
    "................",
    "................",
];

const WIZARD_RESTING: &[&str] = &[
    "................",
    "..........oo....",
    ".........oCo....",
    ".......ooCco....",
    ".....ocCCCCco...",
    "...oollllllloo..",
    "....okKKKKKko...",
    "....oKhKhKhKo...",
    ".e..oooKKooKo...",
    "Me..okKKKKKko...",
    ".m.orRKKKRRro...",
    ".m.orRRRRRRro...",
    ".m..orRRRRro....",
    ".m...orRRro.....",
    ".mKocCCCCCCo....",
    ".m.ococCCCoco...",
    ".m.ococClCoco...",
    ".m.ococCCcoco...",
    ".m..ocCCCCco....",
    ".m..oddddddo....",
    ".m..oDd..dDo....",
    ".m..ooo..ooo....",
    "................",
    "................",
];

const WIZARD_UNKNOWN: &[&str] = &[
    "................",
    "..........oo....",
    ".........oCo....",
    ".......ooCco....",
    ".....ocCCCCco...",
    "...oollllllloo..",
    "....okKKKKKko...",
    "....oKhKhKhKo...",
    ".e..ooKKKKKoo...",
    "Me..okKKKKKko...",
    ".m.orRKKKRRro...",
    ".m.orRRRRRRro...",
    ".m..orRRRRro....",
    ".m...orRRro.....",
    ".mKocCCCCCCo....",
    ".m.ocCvCCvCco...",
    ".m.ocCllllCco...",
    ".m.ocCCCCCCco...",
    ".m..ocCCCCco....",
    ".m...oddddo.....",
    ".m..oDd..dDo....",
    ".m..ooo..ooo....",
    "................",
    "................",
];

const WIZARD_SETTLED: &[&str] = &[
    "................",
    "..........oo....",
    ".........oCo....",
    ".......ooCco....",
    ".....ocCCCCco...",
    "...oollllllloo..",
    "....okKKKKKko...",
    "....oKhKhKhKo...",
    ".e..oKoKKoKKo...",
    "Me..okKKKKKko...",
    ".m.orRKKKRRro...",
    ".m.orRRRRRRro...",
    ".m..orRRRRro....",
    ".m...orRRro.....",
    ".mKocCCCCCCo....",
    ".m.ocCvCCvDPD...",
    ".m.ocCllllDaD...",
    ".m.ocCCCCCooo...",
    ".m..ocCCCCco....",
    ".m...oddddo.....",
    ".m..oDd..dDo....",
    ".m..ooo..ooo....",
    "................",
    "................",
];

const WIZARD_ROSTER: &[&str] = &[
    ".....oo.", "...oCCo.", "..ollllo", "e.oKKKKo", "M.orRRro", "m..oRRo.", "m.oCCCo.", "m.oClCo.",
    "m.oCCCo.", "m..oddo.", "m.oo.oo.", "........",
];

const RANGER_PALETTE: &[IndexedPaletteEntry] = &[
    IndexedPaletteEntry {
        key: 'k',
        colour: Some(Rgb::new(166, 125, 106)),
    },
    IndexedPaletteEntry {
        key: 'K',
        colour: Some(Rgb::new(224, 169, 146)),
    },
    IndexedPaletteEntry {
        key: 'h',
        colour: Some(Rgb::new(233, 194, 178)),
    },
    IndexedPaletteEntry {
        key: 'd',
        colour: Some(Rgb::new(92, 57, 34)),
    },
    IndexedPaletteEntry {
        key: 'D',
        colour: Some(Rgb::new(169, 111, 55)),
    },
    IndexedPaletteEntry {
        key: 'l',
        colour: Some(Rgb::new(237, 181, 77)),
    },
    IndexedPaletteEntry {
        key: 'm',
        colour: Some(Rgb::new(112, 132, 134)),
    },
    IndexedPaletteEntry {
        key: 'M',
        colour: Some(Rgb::new(218, 226, 217)),
    },
    IndexedPaletteEntry {
        key: 'e',
        colour: Some(Rgb::new(112, 220, 255)),
    },
    IndexedPaletteEntry {
        key: 'p',
        colour: Some(Rgb::new(230, 207, 154)),
    },
    IndexedPaletteEntry {
        key: 'P',
        colour: Some(Rgb::new(248, 232, 184)),
    },
    IndexedPaletteEntry {
        key: 'a',
        colour: Some(Rgb::new(201, 69, 64)),
    },
    IndexedPaletteEntry {
        key: 'o',
        colour: Some(Rgb::new(29, 37, 32)),
    },
    IndexedPaletteEntry {
        key: 'r',
        colour: Some(Rgb::new(73, 58, 25)),
    },
    IndexedPaletteEntry {
        key: 'R',
        colour: Some(Rgb::new(105, 67, 37)),
    },
    IndexedPaletteEntry {
        key: 'c',
        colour: Some(Rgb::new(26, 47, 26)),
    },
    IndexedPaletteEntry {
        key: 'C',
        colour: Some(Rgb::new(79, 125, 57)),
    },
    IndexedPaletteEntry {
        key: 'v',
        colour: Some(Rgb::new(137, 169, 74)),
    },
];

const RANGER_WORKING_A: &[&str] = &[
    "................",
    "................",
    "......oo........",
    "....ocCCco......",
    "...ocCvvCCo.....",
    "MmocCCCCCcco....",
    "DdocCKKKKKCo....",
    "DdocKhKhKhCo....",
    "DdocKoKKoKCo....",
    "DdocKKhKKKCo....",
    "Dd.ockKKkco...D.",
    "Dd.ooCCCCoo...D.",
    "ddocCCdCCCco...D",
    "..oKKPPPPPKKo..D",
    "..ocoPdPpPoo...D",
    "...ooPPdPPo....D",
    "...o.ooooo.....D",
    "....odlDdo....D.",
    "....oddddo....D.",
    "...oddo.oddo....",
    "...oDdo.odDo....",
    "...oooo.oooo....",
    "................",
    "................",
];

const RANGER_WORKING_B: &[&str] = &[
    "................",
    "................",
    "......oo........",
    "....ocCCco......",
    "...ocCvvCCo.....",
    "MmocCCCCCcco....",
    "DdocCKKKKKCo....",
    "DdocKhKhKhCo....",
    "DdocKoKKoKCo....",
    "DdocKKhKKKCo....",
    "Dd.ockKKkco...D.",
    "Dd.ooCCCCoo...D.",
    "ddocCCdCCCco...D",
    "..oKKPPKPPKKo..D",
    "..ocoPdPdPoo...D",
    "...ooPPdPPo....D",
    "...o.ooooo.....D",
    "....odlDdo....D.",
    "....oddddo....D.",
    "...oddo.oddo....",
    "...oDdo.odDo....",
    "...oooo.oooo....",
    "................",
    "................",
];

const RANGER_COUNSEL_RAISED: &[&str] = &[
    "................",
    "................",
    "......oo........",
    "....ocCCco......",
    "...ocCvvCCo.....",
    "MmocCCCCCcco....",
    "DdocCKKKKKCo....",
    "DdocKhKhKhCo....",
    "DdocKoKKoKCo....",
    "DdocKKhKKKCoK...",
    "Dd.ockKKkco.K.D.",
    "Dd.ooCCCCoo.CoD.",
    "ddocCCdCCCcoCo.D",
    "..oKCClCCCCKo..D",
    "..ocKPPDoCco...D",
    "...ooDaDoCo....D",
    "...ooooooco....D",
    "....odlDdo....D.",
    "....oddddo....D.",
    "...oddo.oddo....",
    "...oDdo.odDo....",
    "...oooo.oooo....",
    "................",
    "................",
];

const RANGER_COUNSEL_WAIT: &[&str] = &[
    "................",
    "................",
    "......oo........",
    "....ocCCco......",
    "...ocCvvCCo.....",
    "MmocCCCCCcco....",
    "DdocCKKKKKCo....",
    "DdocKhKhKhCo....",
    "DdocKoKKoKCo....",
    "DdocKKhKKKCo....",
    "Dd.ockKKkco...D.",
    "Dd.ooCCCCoo...D.",
    "ddocCCdCCCco...D",
    "..oKCClCCCCKo..D",
    "..ocKPPDoCco...D",
    "...ooDaDoCo....D",
    "...ooooooco....D",
    "....odlDdo....D.",
    "....oddddo....D.",
    "...oddo.oddo....",
    "...oDdo.odDo....",
    "...oooo.oooo....",
    "................",
    "................",
];

const RANGER_SPOILS: &[&str] = &[
    "................",
    "................",
    "......oo........",
    "....ocCCco......",
    "...ocCvvCCo.....",
    "MmocCCCCCcco....",
    "DdocCKKKKKCo....",
    "DdocKhKhKhCo....",
    "DdocKoKKoKCo....",
    "DdocKKhKKKCo....",
    "Dd.ockKKkco...D.",
    "Dd.ooCCCCoo...D.",
    "ddocCCdCCCco...D",
    "..oKCClCCCCKo..D",
    "..ocKPPPPPKo...D",
    "...ooPDalPo....D",
    "...o.ooooo.....D",
    "....odlDdo....D.",
    "....oddddo....D.",
    "...oddo.oddo....",
    "...oDdo.odDo....",
    "...oooo.oooo....",
    "................",
    "................",
];

const RANGER_RESTING: &[&str] = &[
    "................",
    "................",
    "......oo........",
    "....ocCCco......",
    "...ocCvvCCo.....",
    "MmocCCCCCcco....",
    "DdocCKKKKKCo....",
    "DdocKhKhKhCo....",
    "DdocooKKooCo....",
    "DdocKKhKKKCo....",
    "Dd.ockKKkco...D.",
    "Dd.ooCCCCoo...D.",
    "ddocCCdCCCco...D",
    "..oKCClCCCCKo..D",
    "..ococCCCCoo...D",
    "...oocClCco....D",
    "...oodDDDDdo...D",
    "....odlDdo....D.",
    "....oddddo....D.",
    "...oddddddDo....",
    "...oDd...dDo....",
    "...ooo...ooo....",
    "................",
    "................",
];

const RANGER_UNKNOWN: &[&str] = &[
    "................",
    "................",
    "......oo........",
    "....ocCCco......",
    "...ocCvvCCo.....",
    "MmocCCCCCcco....",
    "DdocCKKKKKCo....",
    "DdocKhKhKhCo....",
    "DdocoKKKKKoo....",
    "DdocKKhKKKCo....",
    "Dd.ockKKkco...D.",
    "Dd.ooCCCCoo...D.",
    "ddocCCdCCCco...D",
    "..oKCClCCCCKo..D",
    "..ocCCdCCCco...D",
    "...oCCldCCo....D",
    "...ocCCCCco....D",
    "....odlDdo....D.",
    "....oddddo....D.",
    "...oddo.oddo....",
    "...oDdo.odDo....",
    "...oooo.oooo....",
    "................",
    "................",
];

const RANGER_SETTLED: &[&str] = &[
    "................",
    "................",
    "......oo........",
    "....ocCCco......",
    "...ocCvvCCo.....",
    "MmocCCCCCcco....",
    "DdocCKKKKKCo....",
    "DdocKhKhKhCo....",
    "DdocKoKKoKCo....",
    "DdocKKhKKKCo....",
    "Dd.ockKKkco...D.",
    "Dd.ooCCCCoo...D.",
    "ddocCCdCCCco...D",
    "..oKCClCCCCKo..D",
    "..ocCCdCCCco...D",
    "...oCCldCCPPD..D",
    "...ocCCCCcPaD..D",
    "....odDDdoooo.D.",
    "....oddddo....D.",
    "...oddo.oddo....",
    "...oDdo.odDo....",
    "...oooo.oooo....",
    "................",
    "................",
];

const RANGER_ROSTER: &[&str] = &[
    "...oo...", "..oCCo..", ".oKvKCo.", ".oKoKCo.", "..oKKoD.", ".oCCCo.D", ".oClCo.D", "..oddo.D",
    "..oddoD.", ".od.odo.", ".oo.ooo.", "........",
];

const BARBARIAN_PALETTE: &[IndexedPaletteEntry] = &[
    IndexedPaletteEntry {
        key: 'k',
        colour: Some(Rgb::new(166, 125, 106)),
    },
    IndexedPaletteEntry {
        key: 'K',
        colour: Some(Rgb::new(224, 169, 146)),
    },
    IndexedPaletteEntry {
        key: 'h',
        colour: Some(Rgb::new(233, 194, 178)),
    },
    IndexedPaletteEntry {
        key: 'd',
        colour: Some(Rgb::new(92, 57, 34)),
    },
    IndexedPaletteEntry {
        key: 'D',
        colour: Some(Rgb::new(169, 111, 55)),
    },
    IndexedPaletteEntry {
        key: 'l',
        colour: Some(Rgb::new(237, 181, 77)),
    },
    IndexedPaletteEntry {
        key: 'm',
        colour: Some(Rgb::new(112, 132, 134)),
    },
    IndexedPaletteEntry {
        key: 'M',
        colour: Some(Rgb::new(218, 226, 217)),
    },
    IndexedPaletteEntry {
        key: 'e',
        colour: Some(Rgb::new(112, 220, 255)),
    },
    IndexedPaletteEntry {
        key: 'p',
        colour: Some(Rgb::new(230, 207, 154)),
    },
    IndexedPaletteEntry {
        key: 'P',
        colour: Some(Rgb::new(248, 232, 184)),
    },
    IndexedPaletteEntry {
        key: 'a',
        colour: Some(Rgb::new(201, 69, 64)),
    },
    IndexedPaletteEntry {
        key: 'o',
        colour: Some(Rgb::new(40, 29, 33)),
    },
    IndexedPaletteEntry {
        key: 'r',
        colour: Some(Rgb::new(116, 51, 27)),
    },
    IndexedPaletteEntry {
        key: 'R',
        colour: Some(Rgb::new(166, 73, 39)),
    },
    IndexedPaletteEntry {
        key: 'c',
        colour: Some(Rgb::new(92, 42, 44)),
    },
    IndexedPaletteEntry {
        key: 'C',
        colour: Some(Rgb::new(180, 91, 42)),
    },
    IndexedPaletteEntry {
        key: 'v',
        colour: Some(Rgb::new(226, 158, 96)),
    },
];

const BARBARIAN_WORKING_A: &[&str] = &[
    "................",
    "................",
    ".....r..r.......",
    "....orRrRro.....",
    "...orRRRRRro....",
    "...oRKKKKKRo....",
    "...oKhKhKhKo....",
    "...oKoKKoKKo....",
    "...okKKKKKko....",
    "...orRKKhRro....",
    "...orRRRRRro....",
    "mMM.orRRRro.....",
    "MMmkKKKKKKKko...",
    ".d.KhKKKKKhKo...",
    ".dKKkKdKKkKKo...",
    ".d.okKdKKKko....",
    ".d.odDlDDdo.....",
    ".d..oDDDDoo.....",
    ".d..oddddo......",
    ".d..oddo.ddo....",
    ".d.oDDdo.dDDo...",
    "...ooooo.oooo...",
    "................",
    "................",
];

const BARBARIAN_WORKING_B: &[&str] = &[
    "................",
    "................",
    ".....r..r.......",
    "....orRrRro.....",
    "...orRRRRRro....",
    "...oRKKKKKRo....",
    "...oKhKhKhKo....",
    "...oKoKKoKKo....",
    "...okKKKKKko....",
    "...orRKKhRro....",
    ".mMorRRRRRro....",
    "mMM.orRRRro.....",
    "MMmkKKKKKKKko...",
    ".d.KhKKKKKhKo...",
    ".dKKkKdKKkKKo...",
    ".d.okKdKKKko....",
    ".d.odDlDDdo.....",
    ".d..oDDDDoo.....",
    ".d..oddddo......",
    ".d..oddo.ddo....",
    ".d.oDDdo.dDDo...",
    "...ooooo.oooo...",
    "................",
    "................",
];

const BARBARIAN_COUNSEL_RAISED: &[&str] = &[
    "................",
    "................",
    ".....r..r.......",
    "....orRrRro.....",
    "...orRRRRRro....",
    "...oRKKKKKRo....",
    "...oKhKhKhKo....",
    "...oKoKKoKKo....",
    "...okKKKKKko....",
    "...orRKKhRro....",
    "...orRRRRRro.K..",
    "mMM.orRRRro..K..",
    "MMmkKKKKKKKkoKo.",
    ".d.KhKKKKKhKoCo.",
    ".d.KkKdKKkKKo...",
    ".d.okKdKKKko....",
    ".d.odDlDDdo.....",
    ".d..oDDDDoo.....",
    ".d..oddddo......",
    ".d..oddo.ddo....",
    ".d.oDDdo.dDDo...",
    "...ooooo.oooo...",
    "................",
    "................",
];

const BARBARIAN_COUNSEL_WAIT: &[&str] = &[
    "................",
    "................",
    ".....r..r.......",
    "....orRrRro.....",
    "...orRRRRRro....",
    "...oRKKKKKRo....",
    "...oKhKhKhKo....",
    "...oKoKKoKKo....",
    "...okKKKKKko....",
    "...orRKKhRro....",
    "...orRRRRRro....",
    "mMM.orRRRro.....",
    "MMmkKKKKKKKko...",
    ".d.KhKKKKKhKo...",
    ".d.KkKdKKkKKo...",
    ".d.okKdKKKko....",
    ".d.odDlDDdo.....",
    ".d..oDDDDoo.....",
    ".d..oddddo......",
    ".d..oddo.ddo....",
    ".d.oDDdo.dDDo...",
    "...ooooo.oooo...",
    "................",
    "................",
];

const BARBARIAN_SPOILS: &[&str] = &[
    "................",
    "................",
    ".....r..r.......",
    "....orRrRro.....",
    "...orRRRRRro....",
    "...oRKKKKKRo....",
    "...oKhKhKhKo....",
    "...oKoKKoKKo....",
    "...okKKKKKko....",
    "...orRKKhRro....",
    "...orRRRRRro....",
    "mMM.orRRRro.....",
    "MMmkKKKKKKKko...",
    ".d.KhKKKKKhKo...",
    ".d.KkKDDDkDKo...",
    ".d.okolPlDoo....",
    ".d.odolalDo.....",
    ".d..o.oooo......",
    ".d..oddddo......",
    ".d..oddo.ddo....",
    ".d.oDDdo.dDDo...",
    "...ooooo.oooo...",
    "................",
    "................",
];

const BARBARIAN_RESTING: &[&str] = &[
    "................",
    "................",
    ".....r..r.......",
    "....orRrRro.....",
    "...orRRRRRro....",
    "...oRKKKKKRo....",
    "...oKhKhKhKo....",
    "...oooKKooKo....",
    "...okKKKKKko....",
    "...orRKKhRro....",
    "...orRRRRRro....",
    "mMM.orRRRro.....",
    "MMmkKKKKKKKko...",
    ".d.KKKhKKKKKo...",
    ".d.KkKKKKKkKo...",
    ".d.okKdKKKko....",
    ".d.odDlDDdo.....",
    ".d..oDDDDoo.....",
    ".d..oddddo......",
    ".d.odddddddDo...",
    ".d.oDDd..dDDo...",
    "...oooo..oooo...",
    "................",
    "................",
];

const BARBARIAN_UNKNOWN: &[&str] = &[
    "................",
    "................",
    ".....r..r.......",
    "....orRrRro.....",
    "...orRRRRRro....",
    "...oRKKKKKRo....",
    "...oKhKhKhKo....",
    "...ooKKKKKoo....",
    "...okKKKKKko....",
    "...orRKKhRro....",
    "...orRRRRRro....",
    "mMM.orRRRro.....",
    "MMmkKKKKKKKko...",
    ".d.KhKKKKKhKo...",
    ".d.KkKdKKkKKo...",
    ".d.okKdKKKko....",
    ".d.odDlDDdo.....",
    ".d..oDDDDoo.....",
    ".d..oddddo......",
    ".d..oddo.ddo....",
    ".d.oDDdo.dDDo...",
    "...ooooo.oooo...",
    "................",
    "................",
];

const BARBARIAN_SETTLED: &[&str] = &[
    "................",
    "................",
    ".....r..r.......",
    "....orRrRro.....",
    "...orRRRRRro....",
    "...oRKKKKKRo....",
    "...oKhKhKhKo....",
    "...oKoKKoKKo....",
    "...okKKKKKko....",
    "...orRKKhRro....",
    "...orRRRRRro....",
    "mMM.orRRRro.....",
    "MMmkKKKKKKKko...",
    ".d.KhKKKKKhKo...",
    ".d.KkKdKKkKKo...",
    ".d.okKdKKKko....",
    ".d.odDlDDdoDDD..",
    ".d..oDDDDoolal..",
    ".d..oddddo.ooo..",
    ".d..oddo.ddo....",
    ".d.oDDdo.dDDo...",
    "...ooooo.oooo...",
    "................",
    "................",
];

const BARBARIAN_ROSTER: &[&str] = &[
    "..r.r...", ".oRRRo..", ".oKKKo..", ".oKoKo..", "..oRRo..", "MKKKKKo.", "mKldKKo.", "d.oDDo..",
    "d.oddo..", "d.od.do.", "..oo.oo.", "........",
];

const ART: [ClassArt; 14] = [
    ClassArt {
        palette: WIZARD_PALETTE,
        frames: [
            WIZARD_WORKING_A,
            WIZARD_WORKING_B,
            WIZARD_COUNSEL_RAISED,
            WIZARD_COUNSEL_WAIT,
            WIZARD_SPOILS,
            WIZARD_RESTING,
            WIZARD_UNKNOWN,
            WIZARD_SETTLED,
        ],
        roster: Some(WIZARD_ROSTER),
    },
    ClassArt {
        palette: RANGER_PALETTE,
        frames: [
            RANGER_WORKING_A,
            RANGER_WORKING_B,
            RANGER_COUNSEL_RAISED,
            RANGER_COUNSEL_WAIT,
            RANGER_SPOILS,
            RANGER_RESTING,
            RANGER_UNKNOWN,
            RANGER_SETTLED,
        ],
        roster: Some(RANGER_ROSTER),
    },
    ClassArt {
        palette: BARBARIAN_PALETTE,
        frames: [
            BARBARIAN_WORKING_A,
            BARBARIAN_WORKING_B,
            BARBARIAN_COUNSEL_RAISED,
            BARBARIAN_COUNSEL_WAIT,
            BARBARIAN_SPOILS,
            BARBARIAN_RESTING,
            BARBARIAN_UNKNOWN,
            BARBARIAN_SETTLED,
        ],
        roster: Some(BARBARIAN_ROSTER),
    },
    tool_art::BARD,
    tool_art::ARTIFICER,
    tool_art::TESTMENDER,
    support_art::CLERIC,
    support_art::PALADIN,
    support_art::DRUID,
    trail_art::ROGUE,
    trail_art::PATHSEEKER,
    trail_art::RUNEWRIGHT,
    arcane_art::MAGE,
    arcane_art::SORCERER,
];
