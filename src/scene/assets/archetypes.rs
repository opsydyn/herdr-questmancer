use std::sync::OnceLock;

use crate::{domain::AdventurerClass, scene::pixel::Rgb, scene::sprite::SpriteFrame};

use super::{IndexedPaletteEntry, indexed_sprite};

const GOBLIN_MATERIAL_PALETTE: &[IndexedPaletteEntry] = &[
    IndexedPaletteEntry {
        key: 'o',
        colour: Some(Rgb::new(24, 19, 28)),
    },
    IndexedPaletteEntry {
        key: 's',
        colour: Some(Rgb::new(54, 104, 58)),
    },
    IndexedPaletteEntry {
        key: 'g',
        colour: Some(Rgb::new(107, 183, 82)),
    },
    IndexedPaletteEntry {
        key: 'h',
        colour: Some(Rgb::new(180, 220, 104)),
    },
    IndexedPaletteEntry {
        key: 'd',
        colour: Some(Rgb::new(94, 49, 37)),
    },
    IndexedPaletteEntry {
        key: 'D',
        colour: Some(Rgb::new(170, 92, 56)),
    },
    IndexedPaletteEntry {
        key: 'a',
        colour: Some(Rgb::new(102, 65, 143)),
    },
    IndexedPaletteEntry {
        key: 'm',
        colour: Some(Rgb::new(191, 199, 205)),
    },
    IndexedPaletteEntry {
        key: 'M',
        colour: Some(Rgb::new(244, 239, 209)),
    },
    IndexedPaletteEntry {
        key: 'e',
        colour: Some(Rgb::new(255, 224, 102)),
    },
];

const WIZARD_MATERIAL_PALETTE: &[IndexedPaletteEntry] = &[
    IndexedPaletteEntry {
        key: 'o',
        colour: Some(Rgb::new(24, 19, 28)),
    },
    IndexedPaletteEntry {
        key: 'k',
        colour: Some(Rgb::new(156, 91, 59)),
    },
    IndexedPaletteEntry {
        key: 'K',
        colour: Some(Rgb::new(239, 173, 117)),
    },
    IndexedPaletteEntry {
        key: 'h',
        colour: Some(Rgb::new(255, 214, 157)),
    },
    IndexedPaletteEntry {
        key: 'r',
        colour: Some(Rgb::new(161, 154, 163)),
    },
    IndexedPaletteEntry {
        key: 'R',
        colour: Some(Rgb::new(238, 230, 216)),
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
        key: 'l',
        colour: Some(Rgb::new(237, 181, 77)),
    },
    IndexedPaletteEntry {
        key: 'm',
        colour: Some(Rgb::new(122, 81, 46)),
    },
    IndexedPaletteEntry {
        key: 'M',
        colour: Some(Rgb::new(181, 124, 66)),
    },
    IndexedPaletteEntry {
        key: 'e',
        colour: Some(Rgb::new(112, 220, 255)),
    },
];

const BARBARIAN_MATERIAL_PALETTE: &[IndexedPaletteEntry] = &[
    IndexedPaletteEntry {
        key: 'o',
        colour: Some(Rgb::new(24, 19, 28)),
    },
    IndexedPaletteEntry {
        key: 'k',
        colour: Some(Rgb::new(143, 74, 48)),
    },
    IndexedPaletteEntry {
        key: 'K',
        colour: Some(Rgb::new(222, 137, 84)),
    },
    IndexedPaletteEntry {
        key: 'h',
        colour: Some(Rgb::new(255, 198, 132)),
    },
    IndexedPaletteEntry {
        key: 'r',
        colour: Some(Rgb::new(83, 43, 31)),
    },
    IndexedPaletteEntry {
        key: 'R',
        colour: Some(Rgb::new(149, 75, 42)),
    },
    IndexedPaletteEntry {
        key: 'd',
        colour: Some(Rgb::new(93, 48, 34)),
    },
    IndexedPaletteEntry {
        key: 'D',
        colour: Some(Rgb::new(159, 83, 45)),
    },
    IndexedPaletteEntry {
        key: 'm',
        colour: Some(Rgb::new(128, 140, 145)),
    },
    IndexedPaletteEntry {
        key: 'M',
        colour: Some(Rgb::new(224, 230, 224)),
    },
    IndexedPaletteEntry {
        key: 'a',
        colour: Some(Rgb::new(190, 52, 48)),
    },
    IndexedPaletteEntry {
        key: 'e',
        colour: Some(Rgb::new(255, 226, 126)),
    },
];

const GOBLIN_MATERIAL: &[&str] = &[
    "................",
    "................",
    ".oo..........oo.",
    "osso........osso",
    "osggsooooosggso.",
    ".osggggggggggso.",
    ".osghgggggghgso.",
    ".osgeoggoeggso..",
    "..osggddgggso...",
    ".M.osgggggso....",
    "mM.osodddddoso..",
    ".mgosodDddddDoso",
    ".dgosodDaaaaDoso",
    ".dgosodDdMMdDoso",
    "..osodDDDDdoso..",
    "...odddddddo....",
    "...oddaadddo....",
    "...oddo.oddo....",
    "...odo...odo....",
    "..ooo.....ooo...",
    "................",
    "................",
    "................",
    "................",
];

const WIZARD_MATERIAL: &[&str] = &[
    "................",
    "..........oo....",
    ".......occCCo...",
    "......ocCCCCCo..",
    ".e...ocCCCCCCo..",
    "Me..ocCCCCCCo...",
    ".M..ocCCCCCCco..",
    ".m.oolllllloo...",
    ".m..okKKKKko....",
    ".m..okooooko....",
    ".m..okKKKKko....",
    ".m.orRRRRRRro...",
    ".m..orRRRRro....",
    ".m...orRRro.....",
    ".mKokocCCCCoko..",
    ".m..ocClCCco....",
    ".m..ocCCCCCco...",
    ".m..ocCllCCco...",
    ".m..ocCC.CCco...",
    ".m..ocC...CCo...",
    ".m...oo...oo....",
    ".m..oo.....oo...",
    "................",
    "................",
];

const BARBARIAN_MATERIAL: &[&str] = &[
    "................",
    ".....o.o.o......",
    "....orRrRRo.....",
    "...orRRRRRro....",
    "...orRKKKKRro...",
    "...rokhoohkor...",
    "...rokKhKKkor...",
    "....orRooRro....",
    ".mMMookKKKKoo...",
    "mMMMMKddddKKo...",
    ".mMmoKddddKKo...",
    "...mokDaaDoKKo..",
    "...mokDddDoKKo..",
    "...mokDddddKKo..",
    "...m.oodMMdoo...",
    "...m..oddddo....",
    "...m..oddddo....",
    "...m..od.oddo...",
    "...m..oo..oo....",
    "...m.oo....oo...",
    ".....ooo..ooo...",
    "....ooo....ooo..",
    "................",
    "................",
];

const GOBLIN_PORTRAIT_PALETTE: &[IndexedPaletteEntry] = &[
    IndexedPaletteEntry {
        key: 'o',
        colour: Some(Rgb::new(24, 19, 28)),
    },
    IndexedPaletteEntry {
        key: 's',
        colour: Some(Rgb::new(43, 84, 52)),
    },
    IndexedPaletteEntry {
        key: 'g',
        colour: Some(Rgb::new(76, 151, 63)),
    },
    IndexedPaletteEntry {
        key: 'h',
        colour: Some(Rgb::new(132, 194, 75)),
    },
    IndexedPaletteEntry {
        key: 'd',
        colour: Some(Rgb::new(71, 38, 32)),
    },
    IndexedPaletteEntry {
        key: 'D',
        colour: Some(Rgb::new(128, 67, 43)),
    },
    IndexedPaletteEntry {
        key: 'b',
        colour: Some(Rgb::new(188, 111, 58)),
    },
    IndexedPaletteEntry {
        key: 'a',
        colour: Some(Rgb::new(57, 42, 82)),
    },
    IndexedPaletteEntry {
        key: 'A',
        colour: Some(Rgb::new(100, 67, 145)),
    },
    IndexedPaletteEntry {
        key: 'p',
        colour: Some(Rgb::new(153, 103, 190)),
    },
    IndexedPaletteEntry {
        key: 'm',
        colour: Some(Rgb::new(103, 118, 124)),
    },
    IndexedPaletteEntry {
        key: 'M',
        colour: Some(Rgb::new(185, 199, 200)),
    },
    IndexedPaletteEntry {
        key: 'w',
        colour: Some(Rgb::new(244, 239, 209)),
    },
    IndexedPaletteEntry {
        key: 'e',
        colour: Some(Rgb::new(255, 213, 75)),
    },
];

const WIZARD_PORTRAIT_PALETTE: &[IndexedPaletteEntry] = &[
    IndexedPaletteEntry {
        key: 'o',
        colour: Some(Rgb::new(23, 19, 31)),
    },
    IndexedPaletteEntry {
        key: 'c',
        colour: Some(Rgb::new(48, 39, 91)),
    },
    IndexedPaletteEntry {
        key: 'C',
        colour: Some(Rgb::new(82, 64, 151)),
    },
    IndexedPaletteEntry {
        key: 'v',
        colour: Some(Rgb::new(123, 98, 210)),
    },
    IndexedPaletteEntry {
        key: 'k',
        colour: Some(Rgb::new(139, 79, 54)),
    },
    IndexedPaletteEntry {
        key: 'K',
        colour: Some(Rgb::new(226, 145, 91)),
    },
    IndexedPaletteEntry {
        key: 'h',
        colour: Some(Rgb::new(255, 207, 139)),
    },
    IndexedPaletteEntry {
        key: 'r',
        colour: Some(Rgb::new(151, 143, 151)),
    },
    IndexedPaletteEntry {
        key: 'R',
        colour: Some(Rgb::new(215, 207, 201)),
    },
    IndexedPaletteEntry {
        key: 'w',
        colour: Some(Rgb::new(246, 238, 221)),
    },
    IndexedPaletteEntry {
        key: 'l',
        colour: Some(Rgb::new(177, 114, 39)),
    },
    IndexedPaletteEntry {
        key: 'L',
        colour: Some(Rgb::new(247, 193, 74)),
    },
    IndexedPaletteEntry {
        key: 'm',
        colour: Some(Rgb::new(101, 63, 38)),
    },
    IndexedPaletteEntry {
        key: 'M',
        colour: Some(Rgb::new(177, 122, 63)),
    },
    IndexedPaletteEntry {
        key: 'e',
        colour: Some(Rgb::new(64, 193, 231)),
    },
    IndexedPaletteEntry {
        key: 'E',
        colour: Some(Rgb::new(174, 240, 255)),
    },
];

const BARBARIAN_PORTRAIT_PALETTE: &[IndexedPaletteEntry] = &[
    IndexedPaletteEntry {
        key: 'o',
        colour: Some(Rgb::new(26, 19, 26)),
    },
    IndexedPaletteEntry {
        key: 'k',
        colour: Some(Rgb::new(139, 67, 42)),
    },
    IndexedPaletteEntry {
        key: 'K',
        colour: Some(Rgb::new(215, 116, 65)),
    },
    IndexedPaletteEntry {
        key: 'h',
        colour: Some(Rgb::new(255, 187, 111)),
    },
    IndexedPaletteEntry {
        key: 'r',
        colour: Some(Rgb::new(65, 35, 29)),
    },
    IndexedPaletteEntry {
        key: 'R',
        colour: Some(Rgb::new(126, 61, 35)),
    },
    IndexedPaletteEntry {
        key: 'd',
        colour: Some(Rgb::new(79, 43, 35)),
    },
    IndexedPaletteEntry {
        key: 'D',
        colour: Some(Rgb::new(139, 73, 43)),
    },
    IndexedPaletteEntry {
        key: 'b',
        colour: Some(Rgb::new(192, 107, 55)),
    },
    IndexedPaletteEntry {
        key: 'F',
        colour: Some(Rgb::new(221, 211, 180)),
    },
    IndexedPaletteEntry {
        key: 'm',
        colour: Some(Rgb::new(103, 118, 124)),
    },
    IndexedPaletteEntry {
        key: 'M',
        colour: Some(Rgb::new(180, 194, 195)),
    },
    IndexedPaletteEntry {
        key: 'w',
        colour: Some(Rgb::new(231, 235, 221)),
    },
    IndexedPaletteEntry {
        key: 'a',
        colour: Some(Rgb::new(139, 34, 37)),
    },
    IndexedPaletteEntry {
        key: 'A',
        colour: Some(Rgb::new(210, 55, 49)),
    },
    IndexedPaletteEntry {
        key: 'e',
        colour: Some(Rgb::new(238, 225, 185)),
    },
    IndexedPaletteEntry {
        key: 'E',
        colour: Some(Rgb::new(255, 244, 211)),
    },
];

const GOBLIN_PORTRAIT: &[&str] = &[
    "........................",
    "...oo..............oo...",
    ".osso....oooooo....osso.",
    "osggso.osgggggggsoosggso",
    "osgggsosgghhhggsoosgggso",
    ".osggsosgggggggggsosggso",
    "...osgghhgggggghhggso...",
    "...osgheooggggooehggso..",
    "....osgggoohhoohggggso..",
    ".....osgggddddggggso....",
    ".w....osggddddddggso....",
    ".M.....osggggggggso.....",
    ".mM.ooddddddddddoo......",
    ".mM..odDddddddddDdo.....",
    "odo..odDddaaaaaddDdo....",
    ".od.ogDdaAAAAAadDgo.....",
    "..d.ogDdaApppAadDgo.....",
    "....ogDddaaaaaddDgo.....",
    "..odoodDDbbbbDDdo.......",
    "..odoodddddddddo........",
    "..ooooodddDdddoo........",
    ".......odddodddo........",
    ".......oddo.oddo........",
    ".......oddo.oddo........",
    ".......oddo.oddo........",
    "......oodo.oddoo........",
    ".....ooodo.odooo........",
    ".....oooo...oooo........",
    "......oo.....oo.........",
    "........................",
    "........................",
    "........................",
];

const WIZARD_PORTRAIT: &[&str] = &[
    "........................",
    ".............oo.........",
    "...........ocCo.........",
    "..oo......ocCCo.........",
    ".oEeEo...ocCCvCo........",
    ".oEeeo..ocCCvvCo........",
    "..oeo..ocCCvvvCo........",
    "..omo.ocCCvvvvCo........",
    "..omoocCCCvvvvCo........",
    "..omoocCCCCvvvvCo.......",
    "..omooocCCCCCCCCoo......",
    "..omooolLLLLLloo........",
    "..omo.okKKKKko..........",
    "..omo.okohhoko..........",
    "..omo.okKhhKko..........",
    "..omo.orRRRRro..........",
    "..omoorRRwRRro..........",
    "..omo.orRRRRro..........",
    "..oMoorRRRRRro..........",
    "..omkocCCCCCco..........",
    "..omocCCCLCCCco.........",
    "..omocCvCCCCCCo.........",
    "..omocCvCCvCCCo.........",
    "..omocCvCCvCCCo.........",
    "..omocCvLLvCCCo.........",
    "..omocCCllCCCCo.........",
    "..omocCCC.CCCCo.........",
    "..omocCC...CCCo.........",
    "..omocCo...ocCo.........",
    "...ooCo.....oCoo........",
    ".....oo........oo.......",
    "........................",
];

const BARBARIAN_PORTRAIT: &[&str] = &[
    "........................",
    "...........o..o..o......",
    ".........orRrRRRo.......",
    "........orRRRRRRRo......",
    "........orRKKKKRRo......",
    "........orKhhhKRRo......",
    "........orKohhoRRo......",
    "........orKKhKRrRo......",
    "...oMo.oorRRRRRRroo.....",
    ".oMMMo.ookKKKKKKkoo.....",
    "oMwwMo.okKKddddKKKo.....",
    "oMwwMMo.okKKDbbbbDKKo...",
    ".oMMMMookKDbbDDbbDKo....",
    ".....oDokKDbdDDdbDKo....",
    ".....oDokKDddAAddDKo....",
    ".....oDokKDddaaDdDKo....",
    ".....oDokKDddddddDKo....",
    ".....oDokDDdFFdDDko.....",
    ".....oDoodDFeEFdDoo.....",
    ".....oD.odDFFFFDdo......",
    ".....oD.oddddddddo......",
    ".....oD..oddddddo.......",
    ".....oD..oddo.oddo......",
    ".....oD..oddo.oddo......",
    ".....oD..oddo.oddo......",
    ".....oD..oddo.oddo......",
    ".....oD.oodo.oddoo......",
    ".....oDoooro.ooroo......",
    ".....oDoooo...oooo......",
    ".....ooooo.....ooo......",
    "........................",
    "........................",
];

// The second class batch keeps the same two-tier contract as the first three
// fixtures. These are authored frames, rather than palette swaps or resizes:
// gear must remain legible in the 16x24 world and gain material detail in the
// 24x32 portrait.
const BARD_PALETTE: &[IndexedPaletteEntry] = &[
    IndexedPaletteEntry {
        key: 'o',
        colour: Some(Rgb::new(29, 20, 31)),
    },
    IndexedPaletteEntry {
        key: 'k',
        colour: Some(Rgb::new(133, 73, 48)),
    },
    IndexedPaletteEntry {
        key: 'K',
        colour: Some(Rgb::new(235, 157, 101)),
    },
    IndexedPaletteEntry {
        key: 'h',
        colour: Some(Rgb::new(255, 218, 155)),
    },
    IndexedPaletteEntry {
        key: 'r',
        colour: Some(Rgb::new(71, 38, 36)),
    },
    IndexedPaletteEntry {
        key: 'R',
        colour: Some(Rgb::new(119, 59, 46)),
    },
    IndexedPaletteEntry {
        key: 'c',
        colour: Some(Rgb::new(80, 39, 77)),
    },
    IndexedPaletteEntry {
        key: 'C',
        colour: Some(Rgb::new(147, 66, 126)),
    },
    IndexedPaletteEntry {
        key: 'v',
        colour: Some(Rgb::new(211, 96, 164)),
    },
    IndexedPaletteEntry {
        key: 'd',
        colour: Some(Rgb::new(94, 56, 34)),
    },
    IndexedPaletteEntry {
        key: 'D',
        colour: Some(Rgb::new(171, 111, 55)),
    },
    IndexedPaletteEntry {
        key: 'l',
        colour: Some(Rgb::new(237, 181, 77)),
    },
    IndexedPaletteEntry {
        key: 'L',
        colour: Some(Rgb::new(255, 221, 117)),
    },
    IndexedPaletteEntry {
        key: 'e',
        colour: Some(Rgb::new(117, 221, 233)),
    },
];

const RANGER_PALETTE: &[IndexedPaletteEntry] = &[
    IndexedPaletteEntry {
        key: 'o',
        colour: Some(Rgb::new(21, 25, 26)),
    },
    IndexedPaletteEntry {
        key: 'k',
        colour: Some(Rgb::new(120, 72, 46)),
    },
    IndexedPaletteEntry {
        key: 'K',
        colour: Some(Rgb::new(225, 154, 97)),
    },
    IndexedPaletteEntry {
        key: 'h',
        colour: Some(Rgb::new(255, 220, 151)),
    },
    IndexedPaletteEntry {
        key: 'r',
        colour: Some(Rgb::new(54, 43, 25)),
    },
    IndexedPaletteEntry {
        key: 'R',
        colour: Some(Rgb::new(99, 79, 37)),
    },
    // Deep forest shadow rather than mid green: the old value sat within 26
    // of the Delve's moss and the Ranger vanished into it.
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
        colour: Some(Rgb::new(216, 181, 83)),
    },
    IndexedPaletteEntry {
        key: 'a',
        colour: Some(Rgb::new(94, 143, 69)),
    },
    IndexedPaletteEntry {
        key: 'm',
        colour: Some(Rgb::new(130, 143, 125)),
    },
    IndexedPaletteEntry {
        key: 'M',
        colour: Some(Rgb::new(198, 211, 184)),
    },
    IndexedPaletteEntry {
        key: 'e',
        colour: Some(Rgb::new(238, 199, 95)),
    },
];

const ROGUE_PALETTE: &[IndexedPaletteEntry] = &[
    IndexedPaletteEntry {
        key: 'o',
        colour: Some(Rgb::new(21, 18, 29)),
    },
    IndexedPaletteEntry {
        key: 'k',
        colour: Some(Rgb::new(125, 72, 47)),
    },
    IndexedPaletteEntry {
        key: 'K',
        colour: Some(Rgb::new(225, 151, 97)),
    },
    IndexedPaletteEntry {
        key: 'h',
        colour: Some(Rgb::new(255, 218, 151)),
    },
    IndexedPaletteEntry {
        key: 'r',
        colour: Some(Rgb::new(49, 30, 63)),
    },
    IndexedPaletteEntry {
        key: 'R',
        colour: Some(Rgb::new(82, 51, 110)),
    },
    IndexedPaletteEntry {
        key: 'c',
        colour: Some(Rgb::new(52, 38, 78)),
    },
    IndexedPaletteEntry {
        key: 'C',
        colour: Some(Rgb::new(98, 65, 137)),
    },
    IndexedPaletteEntry {
        key: 'v',
        colour: Some(Rgb::new(157, 111, 204)),
    },
    IndexedPaletteEntry {
        key: 'd',
        colour: Some(Rgb::new(86, 54, 35)),
    },
    IndexedPaletteEntry {
        key: 'D',
        colour: Some(Rgb::new(155, 100, 55)),
    },
    IndexedPaletteEntry {
        key: 'l',
        colour: Some(Rgb::new(219, 184, 87)),
    },
    IndexedPaletteEntry {
        key: 'a',
        colour: Some(Rgb::new(70, 190, 181)),
    },
    IndexedPaletteEntry {
        key: 'm',
        colour: Some(Rgb::new(130, 146, 151)),
    },
    IndexedPaletteEntry {
        key: 'M',
        colour: Some(Rgb::new(223, 232, 224)),
    },
    IndexedPaletteEntry {
        key: 'e',
        colour: Some(Rgb::new(91, 228, 223)),
    },
];

const BARD_WORLD: &[&str] = &[
    "................",
    "................",
    ".....oooo.......",
    "....orRRro......",
    "....orKKhro.....",
    "....orKohro.....",
    "....orKKKro.....",
    ".o..ocCCco..d...",
    ".oo.ocCvcodDd...",
    ".oo.ocCCco.dDd..",
    "....ocClcco..d..",
    "....ocCCCco.....",
    "....ocCcco......",
    "....odddddo.....",
    "....odDddDo.....",
    "....odDllDo.....",
    "....odddddo.....",
    "....oddo.oddo...",
    "....odo...odo...",
    "...ooo.....ooo..",
    "................",
    "................",
    "................",
    "................",
];

const RANGER_WORLD: &[&str] = &[
    "................",
    ".......oo.......",
    ".....ocCCo......",
    "....ocCCCco.....",
    "....ocKKhco..d..",
    "....ocKohco.dDd.",
    "....ocKKKco.dDd.",
    ".m..ocCCCco..d..",
    ".M..ocCvcoddd...",
    ".m..ocCCCco.....",
    "....ocClcco.....",
    "....ocCCCco.....",
    "....ocCcco......",
    "....odddddo.....",
    "....odDddDo.....",
    "....odDaaDo.....",
    "....odddddo.....",
    "....oddo.oddo...",
    "....odo...odo...",
    "...ooo.....ooo..",
    "................",
    "................",
    "................",
    "................",
];

const ROGUE_WORLD: &[&str] = &[
    "................",
    "......oooo......",
    ".....ocCCco.....",
    "....ocCCCCco....",
    "....ocKKhCco....",
    "....ocKooCco....",
    "....ocKKKCco....",
    ".m..ocCCCCco..m.",
    ".M..ocCvCCco..M.",
    ".m..ocCCCCco..m.",
    "....ocCllCco....",
    "....ocCCCCco....",
    "....ocCccCco....",
    "....odddddo.....",
    "....odDaaDo.....",
    "....odDddDo.....",
    "....odddddo.....",
    "....oddo.oddo...",
    "....odo...odo...",
    "...ooo.....ooo..",
    "................",
    "................",
    "................",
    "................",
];

const BARD_PORTRAIT: &[&str] = &[
    "........................",
    ".........oooo...........",
    ".......oorRRroo.........",
    "......orRRRRRRro........",
    "......orRKKhhRro........",
    "......orRKooKRRro.......",
    "......orRKKKKRRro.......",
    "......ooRRRRRRoo........",
    ".....occccccccco....d...",
    "....ocCCvvvCCcco...dDd..",
    "....ocCCvvvCCcco..dDDd..",
    "....ocCCcllCCcco...dDd..",
    "....ocCCCCCccccoo...d...",
    "....ocCCcllCCCCco.......",
    "....ocCCCCCCCCCco.......",
    "....ocCCCllllCCco.......",
    "....ocCCCCCCCCCco.......",
    "....ocCCCCC.CCCco.......",
    "....ocCCCC...CCCco......",
    "....odddddddddddo.......",
    "....odDdddddddDdo.......",
    "....odDddllllDdo........",
    "....odDdddddddDdo.......",
    "....oddddddddddo........",
    ".....odddo.odddo........",
    ".....oddo...oddo........",
    "....oooo.....oooo.......",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
];

const RANGER_PORTRAIT: &[&str] = &[
    "........................",
    "..........oooo..........",
    "........occcccco........",
    ".......ocCCCCCCco.......",
    "......ocCCvvvvCCco......",
    "......ocCCKhhKCCco......",
    "......ocCCKooKCCco......",
    "......ocCCKKKKCCco......",
    "......ocCCCCCCCCco......",
    ".m....ocCCCllCCCco...m..",
    ".M....ocCCCCCCCCco...M..",
    ".m...ocCCccccCCCco...m..",
    ".....ocCCcddddcCCco.....",
    ".....ocCCcDddDcCCco.....",
    ".....ocCCcddddcCCco.....",
    ".....ocCCCCCllCCCco.....",
    ".....ocCCCCCCCCCCco.....",
    ".....ocCCC.CCCC.Cco.....",
    ".....ocCCC..CCC..co.....",
    ".....odddddddddddo......",
    ".....odDdddddddDdo......",
    ".....odDdddeeddDdo......",
    ".....odDdddddddDdo......",
    ".....oddddddddddo.......",
    "......odddo.odddo.......",
    "......oddo...oddo.......",
    ".....oooo.....oooo......",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
];

const ROGUE_PORTRAIT: &[&str] = &[
    "........................",
    "........ooooooo.........",
    "......oocCCCCCcoo.......",
    ".....ocCCCCCCCCCco......",
    ".....ocCCvvvvvCCco......",
    ".....ocCCKhhKCCCco......",
    ".....ocCCKooKCCCco......",
    ".....ocCCKKKKCCCco......",
    ".....ocCCCCCCCCCco......",
    ".m...ocCCccccCCCco...m..",
    ".M...ocCCcddddCCCco...M.",
    ".m...ocCCcDddDCCCco...m.",
    ".....ocCCCccccCCCco.....",
    ".....ocCCCllCCCCCco.....",
    ".....ocCCCCCCCCCCco.....",
    ".....ocCCCCCCCCCcco.....",
    ".....ocCCC.CCCC.Ccco....",
    ".....ocCCC..CCC..cco....",
    ".....ocCC...CCC...co....",
    ".....oddddddddddddo.....",
    ".....odDdddddddddDo.....",
    ".....odDdddeeddddDo.....",
    ".....odDdddddddddDo.....",
    ".....oddddddddddddo.....",
    "......odddo..odddo......",
    "......oddo....oddo......",
    ".....oooo......oooo.....",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
];

const CLERIC_PALETTE: &[IndexedPaletteEntry] = &[
    IndexedPaletteEntry {
        key: 'o',
        colour: Some(Rgb::new(24, 25, 32)),
    },
    IndexedPaletteEntry {
        key: 'k',
        colour: Some(Rgb::new(139, 82, 52)),
    },
    IndexedPaletteEntry {
        key: 'K',
        colour: Some(Rgb::new(232, 164, 103)),
    },
    IndexedPaletteEntry {
        key: 'h',
        colour: Some(Rgb::new(255, 224, 164)),
    },
    IndexedPaletteEntry {
        key: 'c',
        colour: Some(Rgb::new(43, 80, 137)),
    },
    IndexedPaletteEntry {
        key: 'C',
        colour: Some(Rgb::new(72, 126, 195)),
    },
    IndexedPaletteEntry {
        key: 'v',
        colour: Some(Rgb::new(126, 181, 230)),
    },
    IndexedPaletteEntry {
        key: 'w',
        colour: Some(Rgb::new(177, 181, 188)),
    },
    IndexedPaletteEntry {
        key: 'W',
        colour: Some(Rgb::new(239, 234, 216)),
    },
    IndexedPaletteEntry {
        key: 'l',
        colour: Some(Rgb::new(232, 184, 69)),
    },
    IndexedPaletteEntry {
        key: 'L',
        colour: Some(Rgb::new(255, 224, 116)),
    },
    IndexedPaletteEntry {
        key: 'm',
        colour: Some(Rgb::new(112, 91, 58)),
    },
    IndexedPaletteEntry {
        key: 'M',
        colour: Some(Rgb::new(193, 159, 84)),
    },
    IndexedPaletteEntry {
        key: 'd',
        colour: Some(Rgb::new(101, 60, 36)),
    },
    IndexedPaletteEntry {
        key: 'D',
        colour: Some(Rgb::new(166, 105, 52)),
    },
    IndexedPaletteEntry {
        key: 'e',
        colour: Some(Rgb::new(83, 224, 238)),
    },
    IndexedPaletteEntry {
        key: 'E',
        colour: Some(Rgb::new(193, 249, 243)),
    },
];

const PALADIN_PALETTE: &[IndexedPaletteEntry] = &[
    IndexedPaletteEntry {
        key: 'o',
        colour: Some(Rgb::new(27, 23, 27)),
    },
    IndexedPaletteEntry {
        key: 'k',
        colour: Some(Rgb::new(143, 78, 47)),
    },
    IndexedPaletteEntry {
        key: 'K',
        colour: Some(Rgb::new(231, 153, 91)),
    },
    IndexedPaletteEntry {
        key: 'h',
        colour: Some(Rgb::new(255, 218, 147)),
    },
    IndexedPaletteEntry {
        key: 'r',
        colour: Some(Rgb::new(74, 42, 31)),
    },
    IndexedPaletteEntry {
        key: 'R',
        colour: Some(Rgb::new(126, 68, 43)),
    },
    IndexedPaletteEntry {
        key: 'c',
        colour: Some(Rgb::new(37, 69, 120)),
    },
    IndexedPaletteEntry {
        key: 'C',
        colour: Some(Rgb::new(64, 112, 179)),
    },
    IndexedPaletteEntry {
        key: 'w',
        colour: Some(Rgb::new(139, 150, 154)),
    },
    IndexedPaletteEntry {
        key: 'W',
        colour: Some(Rgb::new(218, 224, 215)),
    },
    IndexedPaletteEntry {
        key: 'l',
        colour: Some(Rgb::new(236, 180, 62)),
    },
    IndexedPaletteEntry {
        key: 'L',
        colour: Some(Rgb::new(255, 220, 105)),
    },
    IndexedPaletteEntry {
        key: 'm',
        colour: Some(Rgb::new(119, 126, 128)),
    },
    IndexedPaletteEntry {
        key: 'M',
        colour: Some(Rgb::new(206, 214, 207)),
    },
    IndexedPaletteEntry {
        key: 'd',
        colour: Some(Rgb::new(92, 55, 34)),
    },
    IndexedPaletteEntry {
        key: 'D',
        colour: Some(Rgb::new(157, 93, 45)),
    },
    IndexedPaletteEntry {
        key: 'a',
        colour: Some(Rgb::new(211, 52, 50)),
    },
    IndexedPaletteEntry {
        key: 'e',
        colour: Some(Rgb::new(255, 241, 172)),
    },
];

const CLERIC_WORLD: &[&str] = &[
    ".e..............",
    "oeo.............",
    "oMo....oooo.....",
    ".m....owWWWo....",
    ".m...owWWWWWo...",
    ".m...owwKhhwo...",
    ".m...owwKohwo...",
    ".m...owwKKKwo...",
    ".m....owWWWo....",
    ".m..oowWWWWWoo..",
    ".m...ocCCCCcollo",
    ".m...ocCvCCcolCo",
    ".m...ocClCCcolCo",
    ".m...ocCCCCcolCo",
    ".m...ocCCeCcolCo",
    ".m...ocClCCcolCo",
    ".m...ocCCCCcollo",
    ".m....odddddooo.",
    ".M....odDddDo...",
    ".m....oddo.oddo.",
    ".m....odo...odo.",
    ".....ooo...ooo..",
    "................",
    "................",
];

const PALADIN_WORLD: &[&str] = &[
    "..MMM...........",
    ".MwwM...........",
    "MMwwMM..rrrr....",
    ".oDDo..orRRRro..",
    "..oD..orRKKRro..",
    "..oD..orKhhKRro.",
    "..oD..orKooKRro.",
    "..oD..orKKhKRro.",
    "..oD..orRRRRro..",
    "..oD.oomMMMMmoo.",
    "..oD.omMwwMMollo",
    "..oD.omMwlMMolCo",
    "..oD.omMwwMMolCo",
    "..oD.omMwwMMolCo",
    "..oD.omMwwMMolCo",
    "..oD.omMwwMMolCo",
    "..oD..odddddollo",
    "..oD..odDaaDdoo.",
    "..oD..oddddddo..",
    "..oD..oddo.oddo.",
    "..oD..odo...odo.",
    ".....ooo...ooo..",
    "................",
    "................",
];

const CLERIC_PORTRAIT: &[&str] = &[
    "..e.....................",
    ".oeo....................",
    "oEeEo.....oooooo........",
    ".oMMMo..oowWWWoo........",
    "..m....owWWWWWWWo.......",
    "..m...owWWWwwWWWWo......",
    "..m...owWWwKKKwWWo......",
    "..m...owWWKhhhKWWo......",
    "..m...owWWKohhKWWo......",
    "..m...owWWKKKKKWWo......",
    "..m....owWWWWWWWo.......",
    "..m.....oowWWWwoo.......",
    "..m..oowWWWWWWWWoo......",
    "..m..owwCCCCCCCCwollLllo",
    "..m..owcCCvvvCCCwolCCClo",
    "..m..owcCClCCCCCwolCeClo",
    "..m..owcCCCCCCCCwolCCClo",
    "..m..owcCCCCeCCCwolClClo",
    "..m..owcCCClCCCCwolCCClo",
    "..m..owcCCCCCCCCwolCCClo",
    "..m..owcCCCCCCCCwolCCClo",
    "..m...odddddddddo.olllo.",
    "..m...odDddddddDo..ooo..",
    "..m...odDddllddDo.......",
    "..m...odddddddddo.......",
    "..m....oddo..oddo.......",
    "..M....oddo..oddo.......",
    "......oooo..oooo........",
    "........................",
    "........................",
    "........................",
    "........................",
];

const PALADIN_PORTRAIT: &[&str] = &[
    "...MMMM.................",
    ".MMwwMM.................",
    "MMwwwwMM..rrrrrr........",
    ".oDDDDDooorRRRroo.......",
    "...oD..orRRRRRRRo.......",
    "...oD..orRRKKKRRo.......",
    "...oD..orRKhhhKRRo......",
    "...oD..orRKohhKRRo......",
    "...oD..orRKKKhRRo.......",
    "...oD..orRRRRRRRo.......",
    "...oD...oorRRRroo.......",
    "...oD.oomMMMMMMmoo......",
    "...oDomMMwwwwMMmoollLllo",
    "...oDomMwWMMWwMmoolCCClo",
    "...oDomMwWllWwMmoolClClo",
    "...oDomMwWMMWwMmoolCeClo",
    "...oDomMwWMMWwMmoolCCClo",
    "...oDomMwWMMWwMmoolCCClo",
    "...oDomMwWMMWwMmoolCCClo",
    "...oDomMwWMMWwMmoolCCClo",
    "...oD.odddddddddo.olllo.",
    "...oD.odDddllddDo..ooo..",
    "...oD.odDddddddDo.......",
    "...oD.odddddddddo.......",
    "...oD..oddo..oddo.......",
    "...oD..oddo..oddo.......",
    "...oD.oooo..oooo........",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
];

#[must_use]
pub fn world_frame(class: AdventurerClass) -> Option<SpriteFrame> {
    world_master(class).map(|(frame, _)| frame)
}

/// Returns the class world master together with its authoring palette so
/// persona substitution can map role colours without duplicating the routing.
#[must_use]
pub fn world_master(
    class: AdventurerClass,
) -> Option<(SpriteFrame, &'static [IndexedPaletteEntry])> {
    let (cell, rows, palette) = world_route(class)?;
    Some((cached(cell, rows, palette), palette))
}

type WorldRoute = (
    &'static OnceLock<SpriteFrame>,
    &'static [&'static str],
    &'static [IndexedPaletteEntry],
);

fn world_route(class: AdventurerClass) -> Option<WorldRoute> {
    match class {
        AdventurerClass::Cleric | AdventurerClass::Testmender => {
            Some((&CLERIC_WORLD_FRAME, CLERIC_WORLD, CLERIC_PALETTE))
        }
        AdventurerClass::Paladin => Some((&PALADIN_WORLD_FRAME, PALADIN_WORLD, PALADIN_PALETTE)),
        AdventurerClass::Wizard | AdventurerClass::Artificer | AdventurerClass::Runewright => {
            Some((
                &WIZARD_WORLD_FRAME,
                WIZARD_MATERIAL,
                WIZARD_MATERIAL_PALETTE,
            ))
        }
        AdventurerClass::Barbarian => Some((
            &BARBARIAN_WORLD_FRAME,
            BARBARIAN_MATERIAL,
            BARBARIAN_MATERIAL_PALETTE,
        )),
        AdventurerClass::Bard => Some((&BARD_WORLD_FRAME, BARD_WORLD, BARD_PALETTE)),
        AdventurerClass::Ranger | AdventurerClass::Pathseeker => {
            Some((&RANGER_WORLD_FRAME, RANGER_WORLD, RANGER_PALETTE))
        }
        AdventurerClass::Rogue => Some((&ROGUE_WORLD_FRAME, ROGUE_WORLD, ROGUE_PALETTE)),
        AdventurerClass::Druid => None,
    }
}

#[must_use]
pub fn portrait_frame(class: AdventurerClass) -> Option<SpriteFrame> {
    match class {
        AdventurerClass::Cleric | AdventurerClass::Testmender => Some(cached(
            &CLERIC_PORTRAIT_FRAME,
            CLERIC_PORTRAIT,
            CLERIC_PALETTE,
        )),
        AdventurerClass::Paladin => Some(cached(
            &PALADIN_PORTRAIT_FRAME,
            PALADIN_PORTRAIT,
            PALADIN_PALETTE,
        )),
        AdventurerClass::Wizard | AdventurerClass::Artificer | AdventurerClass::Runewright => {
            Some(cached(
                &WIZARD_PORTRAIT_FRAME,
                WIZARD_PORTRAIT,
                WIZARD_PORTRAIT_PALETTE,
            ))
        }
        AdventurerClass::Barbarian => Some(cached(
            &BARBARIAN_PORTRAIT_FRAME,
            BARBARIAN_PORTRAIT,
            BARBARIAN_PORTRAIT_PALETTE,
        )),
        AdventurerClass::Bard => Some(cached(&BARD_PORTRAIT_FRAME, BARD_PORTRAIT, BARD_PALETTE)),
        AdventurerClass::Ranger | AdventurerClass::Pathseeker => Some(cached(
            &RANGER_PORTRAIT_FRAME,
            RANGER_PORTRAIT,
            RANGER_PALETTE,
        )),
        AdventurerClass::Rogue => {
            Some(cached(&ROGUE_PORTRAIT_FRAME, ROGUE_PORTRAIT, ROGUE_PALETTE))
        }
        AdventurerClass::Druid => None,
    }
}

#[must_use]
pub fn goblin_world_frame() -> SpriteFrame {
    cached(
        &GOBLIN_WORLD_FRAME,
        GOBLIN_MATERIAL,
        GOBLIN_MATERIAL_PALETTE,
    )
}

#[must_use]
pub fn goblin_portrait_frame() -> SpriteFrame {
    cached(
        &GOBLIN_PORTRAIT_FRAME,
        GOBLIN_PORTRAIT,
        GOBLIN_PORTRAIT_PALETTE,
    )
}

fn cached(
    cell: &'static OnceLock<SpriteFrame>,
    rows: &'static [&'static str],
    palette: &'static [IndexedPaletteEntry],
) -> SpriteFrame {
    cell.get_or_init(|| indexed_sprite(rows, palette).expect("authored archetype sprite is valid"))
        .clone()
}

static WIZARD_WORLD_FRAME: OnceLock<SpriteFrame> = OnceLock::new();
static CLERIC_WORLD_FRAME: OnceLock<SpriteFrame> = OnceLock::new();
static PALADIN_WORLD_FRAME: OnceLock<SpriteFrame> = OnceLock::new();
static BARBARIAN_WORLD_FRAME: OnceLock<SpriteFrame> = OnceLock::new();
static BARD_WORLD_FRAME: OnceLock<SpriteFrame> = OnceLock::new();
static RANGER_WORLD_FRAME: OnceLock<SpriteFrame> = OnceLock::new();
static ROGUE_WORLD_FRAME: OnceLock<SpriteFrame> = OnceLock::new();
static WIZARD_PORTRAIT_FRAME: OnceLock<SpriteFrame> = OnceLock::new();
static CLERIC_PORTRAIT_FRAME: OnceLock<SpriteFrame> = OnceLock::new();
static PALADIN_PORTRAIT_FRAME: OnceLock<SpriteFrame> = OnceLock::new();
static BARBARIAN_PORTRAIT_FRAME: OnceLock<SpriteFrame> = OnceLock::new();
static BARD_PORTRAIT_FRAME: OnceLock<SpriteFrame> = OnceLock::new();
static RANGER_PORTRAIT_FRAME: OnceLock<SpriteFrame> = OnceLock::new();
static ROGUE_PORTRAIT_FRAME: OnceLock<SpriteFrame> = OnceLock::new();
static GOBLIN_WORLD_FRAME: OnceLock<SpriteFrame> = OnceLock::new();
static GOBLIN_PORTRAIT_FRAME: OnceLock<SpriteFrame> = OnceLock::new();
