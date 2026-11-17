# Proportional dungeon acceptance

Open the retained **Questmancer dogfood** tab, dismiss any card/parchment with
Esc and press `2`. Keep the synthetic party open. Interact only with synthetic
adventurers; never observe, counsel or read an actual/unknown agent pane for
this review. Computer Use cannot inspect Ghostty; these are human checks.

1. Confirm the complete dungeon fits proportionally with the same sprite pixels,
   including the five station regions, doorway connections and selected rune.
   When a chamber's capacity is exceeded, ordinary parties should gather over
   the dungeon using full 16x24 masters. Up to sixteen fit that composition;
   denser parties retain roster art over the dungeon, not a flat green floor.
2. Resize through large, medium and small grids. Check proportional room/actors,
   sharp pixel edges, letterboxing, complete actors and readable selected footer.
   Below a complete roster's capacity, retain the existing crop/overflow contract.
3. Select using `j/k` or Tab, then click synthetic actors. Cards should match
   the clicked actor at every size. Letterboxing must not select an actor.
   The Delve has no Librarian actor; `?` still opens the Ledger.
4. With a synthetic actor selected, `o` opens its existing scrying parchment.
   Check output/scrolling, then Esc. It deliberately uses the half-block backdrop
   while open; the fitted dungeon should return on close. No automatic output
   retrieval is added to animation frames. Do not send counsel during this check.
5. Review card, Ledger, search, Chronicle, Board and Table transitions. The profile must retain the fitted room with no scale jump. Other modal
   parchments keep their fallback backdrop. No stale dungeon image should obscure
   text or portrait; no blank image should remain afterward.
6. Switch `1` Hall → `2` Delve repeatedly. Each room should fit correctly with
   consistent selection and footer, without ghost images from the other room.
7. Review working gestures, blocked lanterns, unknown cues and quiet resting
   poses for smoothness/flicker. Synthetic/fixture done states do not establish
   real-agent completion or successful delivery.

For fixed, terminal-free-source fixtures:

```sh
cargo run --features storybook --bin questmancer-storybook -- --story world.delve-fit --inspect
cargo run --features storybook --bin questmancer-storybook -- --story world.delve-fit-resting --inspect
cargo run --features storybook --bin questmancer-storybook -- --story world.delve-fit-crowded --inspect
```

These run in a terminal but require no Herdr, agent processes or persistent work.
Inside the fit studies, Tab selects, `c` opens a card, `L` opens the Ledger and
Esc closes it; clicks affect the cloned fixture only. The resting study reproduces four resting adventurers plus one unknown.
Both it and the eleven-person crowded study must keep the complete dungeon,
full sprite masters, static state cues and a click target for every adventurer.
`--no-graphics` exercises fallback. Reopen after font zoom if the host omits
fresh kernel pixel axes. Native appearance, mouse, animation and resize must be
recorded independently from test passes and protocol receipts.

See [the profile fit checklist](profile-fit.md) for coverage and click isolation.
