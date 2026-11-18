# Mage and caster sprite review

The Mage's native portrait supplies the dark violet hood, red eyes, bone staff
and green magic. The previous world sprite showed a visible face and a plain
robe. Its revised `16x24` frames use a shadowed face, a silver-edged hood, a
flared cloak and green light from the staff and censer. The `24x32` profile
fallback centres the same working sprite at native scale. The native portrait
remains the reference artwork.

| Class | Read at `8x12` |
| --- | --- |
| Mage | Round hood, red eyes, tall bone staff and green flame |
| Sorcerer | Swept hair, sloped shoulders and cyan storm focus |
| Wizard | Bent pointed hat, broad brim and upright staff |
| Druid | Antlered hood, pale beard and living branch |
| Artificer | Flat goggle line, raised wrench and tool satchel |
| Runewright | Squared hood, pale beard, short hammer and rune tablet |

[Portrait, world sprite, roster silhouettes and Mage poses](caster-sprites.png)
uses actual sprite pixels with nearest-neighbour enlargement. The palette keeps
the class-defining details visible across different adventurer appearances.
[Hall and Delve comparison](caster-rooms.png) shows fourteen adventurers at
`120x34` and `80x24` cells. Each room has one of every class and varied presence.
The smaller sprites rely on separate status glyphs for work, counsel and rest.

Run the fitted comparisons in Storybook:

```sh
cargo run --features storybook --bin questmancer-storybook -- --story world.guild-fit-casters --inspect
cargo run --features storybook --bin questmancer-storybook -- --story world.delve-fit-casters --inspect
```

Tab selects an adventurer and `c` opens their profile. The existing
`asset.mage-poses` story shows the Mage's full-size poses. Live parties get the
new art through their saved classes. The one-of-each party is a fixed preview.

The silhouette regression test reproduced Mage and Sorcerer sharing the exact
transparency mask. It now checks that all six caster shapes are pairwise distinct
without using colour. Existing tests cover pose anchors and the unscaled fallback.
Production fit exports confirmed all fourteen actors at both pane sizes in both
rooms. Four isolated Storybook PTY runs opened the Mage profile, resized the
pane, checked the visible image cells and restored terminal state on exit.
`just verify` passed 842 Rust tests and 28 shell tests. The release, Storybook
and Rustdoc builds passed. Native visual review remains open.
