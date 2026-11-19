# Questmancer character lineup audit

This review captures the pre-revision lineup. The
[native portrait pass](../2026-11-18-native-portrait-alignment/README.md)
records the selected revisions.

**Wizard and Mage are the reference portraits for the native lineup.** Judge
both body proportions and how the artwork handles detail. The references use
compact bodies, short limbs, strong outlines and broad clusters of colour.
Their signature props stand out at card size. A shared ground line and canvas
margin help compare silhouettes with different framing.

The first portrait pass should pair Artificer and Rat Royalty.
In the supplied Artificer card, the long torso and legs, fine leather folds,
and dense buckles and machinery make the dwarf read as a taller, more
illustrative character. The revision should keep the goggles, wrench and one
cyan workshop accent. Shorten the body and legs, thicken the silhouette and
group the belt, straps and metal into larger, simpler pixel clusters.
Rat Royalty has a compact stance, with fine fur shading, ermine spots, gold
filigree and jewellery competing for attention. Keep the crown, sceptre, cape
and tail. Use a few broad purple, gold and cream shapes to carry the costume.

- [Full lineup](01-lineup.png) compares 15 selectable adventurers and the
  Librarian. It shows the native PNG, card fallback, world sprite, small
  roster sprite and actual fallback picker preview where applicable.
- [Portrait framing study](02-framing.png) compares the existing canvases with
  the same artwork scaled and aligned to a shared ground line.
- [Full native style comparison](04-native-style.png) repeats Wizard and Mage
  in every row, with each figure fitted to the same bounds and ground line.
- [Hall, Delve and Inbox](03-in-context.png) shows the production fitted rooms
  with all 15 characters and the Inbox at 84 and 52 columns. The wide queue
  shows the teal-sealed scroll. The compact queue has the full text width.

## Framing and proportion

The native PNGs use a 384×512 canvas. Pixels with alpha at least 128 occupy
64% of the Wizard's canvas height, 77% of the Mage's, 95% of the Sorcerer's
and 73% of the Bard's. The Mage occupies essentially the full width and
reaches the left edge. The shared ground line makes the effect of canvas
padding easier to see. Artificer's long body and fine materials remain
visible within the matched bounds, as do the Sorcerer's longer coat and
Bard's narrow stance. The fine details in Rat Royalty's existing costume
stay visibly denser than the reference cards. Use framing for padding and
placement and revise the illustration where its proportions or detail drift
from the references.

## Per-character decisions

| Character | Decision | What to carry forward |
| --- | --- | --- |
| Barbarian | Keep | The squat stance, broad head and axe are close to the reference silhouettes and pixel clusters. |
| Bard | Revise the native illustration | Shorten the torso and legs. Keep the feather, plum hat, lute and lively stance. The existing world and picker sprites already read compactly. |
| Cleric | Simplify native detail | Preserve the stocky body, bright tabard and sun staff. Group gold edging and fabric highlights into broader areas. |
| Druid | Simplify native detail | Keep the green hood, branch crown and living staff. Consolidate cloak studs, leaf edges and jewellery shadows. |
| Paladin | Simplify native detail | Preserve the enclosed helm, shield and compact stance. Treat engraved armour and gold highlights as larger clusters. |
| Ranger | Simplify native detail | Keep the broad hood and bow. Simplify gradients in the cape and the repeated lines of the arrows and quiver. |
| Rogue | Keep | The low hood, angled stance, blade and broad shadow clusters sit close to the Mage reference. |
| Wizard | Reference portrait | Preserve the illustration, compact figure and broad blue and gold clusters. Check its padding and placement alongside other cards. |
| Artificer | Revise the native illustration first | Shorten the torso and legs. Simplify the leather and machinery into larger clusters. Keep the goggles, wrench and cyan accent. Check those cues in the scene and picker once the portrait direction is established. |
| Runewright | Simplify native detail | Keep the compact stance and blocky rune hammer. Group the engravings and brocade, then strengthen the hammer cue in the scene sprite and picker. |
| Testmender | Revise the native illustration | Shorten the coat and legs. Keep the bright goggles and a primary tool, and combine straps and vials into clearer groups. |
| Pathseeker | Simplify native detail | Keep the staff, compact hood and travel gear. Merge the many leather folds, curls and small pouches into broader shapes. |
| Mage | Reference portrait | Preserve the illustration, compact hood, skull staff and green glow. Check the staff's side margin in the card. |
| Sorcerer | Revise the native illustration | Shorten the coat and legs and simplify the fine hair and fabric shading. Keep the swept hair, pale streak and cyan orb. |
| Rat Royalty | Simplify the native illustration first | Keep the crown, red-purple cape, sceptre and tail. Reduce the filigree, fur texture, ermine spots and gem highlights. |
| Librarian | Simplify native detail | Keep the stocky silhouette, orange hair and books. Group robe folds, spines and fur highlights into broader patches. |

Use the Artificer and Rat Royalty studies to establish the treatment for both
proportion and detail. Follow with the Sorcerer, Bard and Testmender silhouette
passes, then the remaining detail passes. Judge each candidate at card size
against both references and confirm that its strongest identifying cues still
carry into the fallback and roster.

## Evidence

The [exporter](export.rs) uses fixed Storybook personas, the production fit
renderer for both rooms and the production picker with its half-block fallback.
The native column shows the PNG bytes registered by Questmancer. The framing
study changes the comparison canvas only. The final human Ghostty review still
covers the native presentation, navigation and visual acceptance.

Regenerate the sheets with Python and Pillow available. The exporter reads the
checked-out assets, so use the pre-revision assets to reproduce this baseline.

```sh
python3 docs/design/reviews/2026-11-18-lineup-audit/regenerate.py
```
