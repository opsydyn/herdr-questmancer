# Native portrait rendering through Herdr

This note records the failure mode and recovery sequence discovered while
adding `ratatui-image` portraits to Questmancer's expanded Adventurer card.
It applies to terminal images rendered inside a Herdr-managed pane, where the
outer terminal and Herdr form one graphics transport.

## Current scope and historical evidence

Questmancer `0.1.10` requires Herdr `0.9.0` / protocol `22`. All fourteen
classes have native PNG cards and authored `24x32` portrait fallbacks;
`src/portrait.rs` owns the exact routes. The Librarian's Ledger has a separate
native illustration and an independently authored `24x32` Ledger fallback.
Goblin and Orc illustrations are reserved event art, not ordinary ancestry
routes. Native protocols are used for cards and the Ledger illustration;
both worlds use the RGB half-block renderer.

Historical acceptance recorded Barbarian, Rogue, Wizard and Goblin native art
through Ghostty and Herdr `0.7.4` with Kitty graphics. Preserve that evidence
as historical: it does not certify all current cards or the current Herdr
transport. Repeat current live acceptance separately from decoding and
fallback tests.

Current implementation:

- `ratatui-image` 11.0.6 with its Crossterm integration;
- `image` 0.25.6 with PNG decoding only;
- embedded assets under `src/assets/portraits/*-card.png`;
- capability and prepared-protocol owner: `src/portrait.rs`; and
- canonical fallback: authored class RGB art in the `24x32` portrait canvas.
  All fourteen classes reuse their personalised `16x24` world sprite at native
  size, centred with four-pixel margins.

## Herdr 0.9 managed-pane regression — 2026-09-08

The user reported native illustrations regressing to sprite fallbacks in
Ghostty, for both the adventurer card and Librarian. Reopening the plugin did
not recover them. Fresh, test-owned panes isolated the difference:

- a plain pane answered Kitty and CSI 16t, with 8x18 host cells;
- a managed-plugin pane answered Kitty but omitted CSI 16t and had zero PTY
  pixel dimensions despite a valid character grid;
- `ratatui-image` therefore discarded the native capability when it could not
  determine font size. All embedded PNGs still decoded and prepared normally.

Before querying graphics, Questmancer now requests `pane.graphics.info` for its
own managed pane only when PTY pixel geometry is missing. It fills missing
pixel axes from the host-reported cell size, preserving the latest character
grid and every already-known pixel axis. Zero/overflowing sizes are rejected,
and the local request is capped at 500 ms. No terminal-name heuristic or
forced graphics protocol is introduced: the subsequent Picker query remains
authoritative. Failure retains the authored fallback.

This is startup transport preparation, not a per-frame request. It does not
change Herdr configuration, focus, agents, persistent state or the scene
renderer. See the [regression receipt](../reviews/2026-09-08-native-portrait-regression/README.md)
for reproduction, validation and the separate visual result. Earlier clean
qualification at `9ea8501` predates this repair.

## Historical empty-region failure

The Adventurer parchment and text rendered correctly, but the portrait region
was completely empty. Earlier runs showed the authored RGB sprite in the same
space, proving that card sizing and layout were not the cause.

The empty region is an important diagnostic signal. A native image widget
reserves its cells so Ratatui does not paint ordinary content over the image.
If an intermediary then discards the native graphics escape sequence, neither
the image nor the fallback is visible.

## Historical root cause

`TERM_PROGRAM=ghostty` described the outer terminal, but did not prove that the
complete pane transport supported Kitty graphics. Questmancer temporarily used
that environment variable to promote Ratatui's Halfblocks capability result to
Kitty. Herdr's experimental Kitty bridge was disabled, so Herdr did not forward
or render the image while Ratatui still reserved the portrait cells.

The false assumption was:

```text
Ghostty supports Kitty graphics
    therefore a program inside a Herdr pane supports Kitty graphics
```

The correct model is:

```text
Questmancer -> Herdr managed pane -> attached Herdr client -> Ghostty
```

Every link must support the selected protocol.

## Required invariant

The result of `ratatui_image::picker::Picker::from_query_stdio()` is
authoritative. Questmancer must not upgrade a Halfblocks result based on
`TERM`, `TERM_PROGRAM` or knowledge of the outer terminal.

- Kitty, Sixel or iTerm2 result plus an available class asset: render the PNG.
- Halfblocks, failed detection or failed PNG preparation: render the authored
  RGB sprite.
- Missing PNG for a class: render its authored RGB sprite.
- No capability combination may leave the portrait region empty.

The fallback uses Questmancer's authored RGB portrait through its scene
adapter, rather than `ratatui-image`'s Halfblocks renderer. The Librarian uses
its independently authored `24x32` Ledger fallback.

## Herdr configuration

Herdr 0.9 enables the Kitty bridge by default. The current setting in
`~/.config/herdr/config.toml` is:

```toml
[terminal]
kitty_graphics = true
```

The legacy `experimental.kitty_graphics` key is accepted, including an explicit
false; `terminal.kitty_graphics` takes precedence. Preserve intentional settings.
Run `herdr config check` before applying changes. Herdr's 0.9 configuration
reference requires a server restart or client reattach after changing graphics.
For remote sessions, server configuration controls parsing/API availability and
local client configuration controls outer-terminal output. A successful config
reload alone is not proof of graphics transport.

Obtain the owner's approval before changing or restarting a shared server.
Reattach the viewing client first when diagnosing outer-terminal output; test
native rendering afterward. Do not restart a shared server without approval.

In the historical `0.7.4` failure, reload returned `status: applied`, but the
already-attached Herdr client continued to expose its previous capability. Exit only the attached
client, leave the persistent server running, then attach again from Ghostty:

```bash
herdr
```

Do not stop a shared Herdr server merely to refresh this capability. Existing
workspaces and agent panes belong to the persistent server.

## Build and plugin refresh

For a locally linked plugin, use a release build:

```bash
cargo build --release
```

`herdr/run.sh` resolves `target/release/questmancer` before
`target/debug/questmancer`. A plain `cargo build` therefore does not update the
binary used by the plugin when an older release binary already exists.

After the build, close Questmancer, wait for the close action to finish, then
open it again. Do not invoke close and open concurrently: plugin actions are
asynchronous and a late close can remove the newly opened pane.

```bash
herdr plugin action invoke opsydyn.questmancer.close
herdr plugin log list --plugin opsydyn.questmancer --limit 1
herdr plugin action invoke opsydyn.questmancer.guild
herdr plugin log list --plugin opsydyn.questmancer --limit 1
```

Both final log records should be `succeeded` with empty stderr.

## Guarded smoke test

1. Confirm Herdr client and server are compatible with `herdr status`.
2. Confirm `terminal.kitty_graphics = true` and run `herdr config check`.
3. Attach a fresh Herdr client after enabling the bridge.
4. Open Questmancer and create disposable agents only; never repurpose an
   unrelated live agent for testing.
5. Review each of the fourteen class cards in Storybook. For live checks, use
   only an observed disposable adventurer's derived class; there are no manual
   persona controls. Review reserved Goblin/Orc art in Storybook separately.
6. Confirm the card shows the transparent PNG while the Guild Hall remains the
   RGB half-block world.
7. Disable or bypass native capability in Storybook and confirm the authored
   sprite occupies the same card region.
8. Open the Librarian's Ledger and confirm its native illustration and
   fallback independently. All current classes have PNGs; decode/preparation
   failure must still leave a non-empty authored fallback.
9. Release synthetic agent reports and close only test-created panes after
   acceptance.

Useful automated checks:

```bash
cargo test portrait --lib
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
```

The portrait tests must preserve the key regression contract: an intermediary
Halfblocks result cannot be promoted to Kitty and cannot displace the authored
fallback.

## Diagnostic order for future failures

When a native portrait is missing, inspect in this order:

1. **Empty or fallback?** Empty suggests a false-positive native protocol;
   fallback suggests capability detection or asset preparation declined native
   rendering safely.
2. **Managed pane geometry:** check the current repair and its receipt above;
   a Kitty acknowledgement without CSI 16t or PTY pixels reproduces this regression.
3. **Herdr bridge:** confirm `terminal.kitty_graphics = true`.
4. **Fresh attachment:** reattach the Herdr client after enabling the bridge.
5. **Binary selection:** confirm the release binary was rebuilt.
6. **Asset contract:** confirm the PNG decodes and the class is mapped.
7. **Capability query:** trust the query result; do not add terminal-name
   heuristics.
8. **Action ordering:** ensure the plugin close completed before reopening it.

This sequence produced the confirmed native portrait path while retaining a
non-blank fallback for every unsupported identity and transport.
