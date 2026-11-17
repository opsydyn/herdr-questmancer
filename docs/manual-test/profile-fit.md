# Fitted world with an adventurer profile

Use the existing owned dogfood UI or the fixed stories below. Computer Use cannot
inspect Ghostty; this remains a human visual check. Interact only with the owned
UI and synthetic/fixed adventurers for testing. Keep the remaining resources open.

1. In the Hall, note the room's scale and position, then open an adventurer profile.
   The room must keep that same fit. Repeat in the Delve. The existing profile
   dimensions, class portrait, copy and keys are retained.
2. Close/reopen the card repeatedly. No enlarged half-block scene, blank world,
   stale parchment, old portrait or image over the card should remain. Compare
   landmarks visible on both sides of the transition.
3. Resize while the card is open through 120x34, 96x28 and 80x24 or nearby sizes.
   The room should fit the new viewport normally while the profile stays legible.
   At very small sizes the card can cover the whole room; closing it must reveal
   the fitted room on its first exposure, even if the app started with a card.
4. Click within the profile, including text and portrait. Selection must not change
   to an adventurer behind it and no output/focus/counsel action should occur.
   Only fully exposed world actors are click targets while the card is present;
   keyboard selection can still reach the entire party.
5. Check a native class portrait and authored fallback. The profile must remain
   opaque over the world, including its edges and right gutter. Check card text
   and keepsake art as well as the portrait.
6. Other modal parchments retain their existing behavior; closing them returns
   to the fitted scene/profile as appropriate. Profile coverage is not authority
   to send counsel or operate a real agent pane.

Fixed stories, with no Herdr or persistent work:

```sh
cargo run --features storybook --bin questmancer-storybook -- --story world.guild-fit-profile --inspect
cargo run --features storybook --bin questmancer-storybook -- --story world.delve-fit-profile --inspect
```

`Esc` closes the profile; `c` opens it; Tab selects a local fixture adventurer.
At 60x14 the profile initially hides the entire fitted image. Close it and verify
that the world appears, then repeat the toggle. `--no-graphics` tests fallback.
For font zoom, reopen if the host omits fresh kernel pixel axes. Record native
visibility, opacity, mouse and resize outcomes separately from producer tests.
