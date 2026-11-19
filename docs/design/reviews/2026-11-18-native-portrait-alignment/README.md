# Questmancer native portrait alignment

The production cards use Wizard and Mage as the 16-bit references for compact
bodies, short limbs, dark outlines and broad colour clusters. The
[before-and-after sheet](before-after.png) compares all twelve redraws in their
production canvas proportions. The [full lineup](lineup.png) includes all
fifteen selectable classes and the Librarian. Both sheets composite the PNGs
on parchment for comparison ahead of the final Ghostty review.

| Pass | Characters | Identifying details |
| --- | --- | --- |
| First pair | Artificer, Rat Royalty | Artificer has a large wrench, cyan goggles and a shorter apron. Rat Royalty has a gold crown, sapphire sceptre, broad red-purple cape and curled tail. |
| Proportions | Sorcerer, Bard, Testmender | Silver streak and cyan orb, plumed hat and lute, and green goggles and glass probe. |
| Detail | Cleric, Druid, Paladin, Ranger, Runewright, Pathseeker, Librarian | Sun staff and book, branch hood and living staff, shield and helm, bow and hood, rune hammer, walking staff and bedroll, and orange hair and books. |
| Kept | Wizard, Mage, Rogue, Barbarian | Their existing silhouettes and pixel work already fit the direction. |

The replacement class images live in [`src/assets/portraits`](../../../../src/assets/portraits)
and the Ledger uses [`src/assets/librarian.png`](../../../../src/assets/librarian.png).
The native Adventurer card and character picker already share the embedded
class images. The [original audit](../2026-11-18-lineup-audit/README.md) records
how the signature details carry into the authored half-block card, Hall,
Delve and roster sprites. This pass replaces the PNG illustrations. The
Runewright's small hammer silhouette is a separate sprite follow-up.

## Asset preparation

[Generation prompts and checksums](generation.json) record each built-in
imagegen redraw. Each used the original class image and the Wizard and Mage
references. Preparation trimmed transparent margins at alpha 16, resized the
figure proportionally with nearest-neighbour sampling and placed it on a new
transparent canvas. Class images keep their 384×512 frame. Art at alpha 128 or
above uses a common ground line at pixel 438, with width and height caps of
336 and 360 pixels, subject to one-pixel rounding. The Librarian retains its
1024×1024 canvas and fits the square Ledger artwork.

All sixteen images are distinct RGBA PNGs. The staged replacements have
clear margins at alpha 128, and the four kept images match their previous
checksums. The Sorcerer checksum guard in the asset suite tracks the new
native image. The fixture's authored Storm Channeler sprite rows retain their
original values.

## Qualification

`just verify` passed 871 Rust tests across 68 runs and 28 shell tests, including
the native picker, the PNG and fallback checks, and the viewport suites. The
Sorcerer checksum test rejected the previous PNG and passed with the selected
redraw. The locked release build and `git diff --check` passed.

The final human Ghostty review covers the card and character picker, the
Librarian's Ledger and the opt-in Inbox together with the release build.
