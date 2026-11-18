# Full class art pass

All fourteen classes now have dedicated `8x12` sprites. These are authored at
roster size and keep each class recognizable when the room gets crowded. The
Hall and Delve still display presence separately through their state glyphs.

| Class | Cues carried across scales |
| --- | --- |
| Cleric | Sun mace, cream vestments and a rectangular prayerbook. The full-size Sunward Chaplain still provides the reference. |
| Paladin | Winged steel helm, scarlet cape and broad ivory shield with gold trim. |
| Testmender | Green goggles, pale moustache, leather field apron and the needle and patch from the existing ritual. |
| Bard | Plum cap with a warm feather and a round lute held across the body. |
| Rogue | Open violet hood, bright edges on the blades and a low stance. |
| Pathseeker | Forked staff with a hanging green lantern, travel pack and route gear. |

The full-size `16x24` changes hold the same foot anchor across work, counsel,
rest and spoils. The `24x32` profile fallback centres the corresponding world
sprite. The original portraits are the reference for this pass.

- [Portraits and sprite comparisons at both sizes](six-class-comparison.png)
- [All fourteen small sprites in colour and flat silhouette](roster-silhouettes.png)
- [Full parties in the Hall and Delve at 120×34 and 80×24 cells](full-party-rooms.png)
- [Canonical Delve reference scene with the revised Bard](canonical-delve.png)
- [Earlier Mage and caster comparison](../2026-11-17-caster-sprites/README.md)

Use `world.guild-fit-casters` and `world.delve-fit-casters` in Storybook. Each
party has one of every class. Tab selects an adventurer and `c` opens the
profile. The comparison sheets use production sprite and fitted room exports.

The broadened silhouette regression test first found the shared Cleric and
Paladin shape. It now checks all fourteen silhouettes pairwise using only
transparency. The scene tests cover fixed foot anchors, profile fallback
continuity and presence timing. Production projections showed all fourteen
actors in each room at both pane sizes. The canonical Delve comparison against
the previous reference changed six pixels, all within the Bard's bounds.
Four isolated Storybook runs checked resize, keyboard selection and all six
refreshed profiles in both rooms. `just verify` passed 842 Rust tests and 28
shell tests. The release, Storybook and Rustdoc builds passed. Native visual
review remains open.
