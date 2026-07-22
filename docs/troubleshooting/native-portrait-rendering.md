# Native portrait rendering through Herdr

This note records the failure mode and recovery sequence discovered while
adding `ratatui-image` portraits to Questmancer's expanded Adventurer card.
It applies to terminal images rendered inside a Herdr-managed pane, where the
outer terminal and Herdr form one graphics transport.

## Confirmed working result

The native-card path is confirmed for Barbarian, Rogue, Wizard and Goblin
through Ghostty and Herdr 0.7.4 using Kitty graphics. Bard, Druid, Paladin and Orc
are registered with the same transparent PNG contract and automated coverage;
their live native render remains a manual-review item. The world remains rendered by
Questmancer's RGB half-block scene adapter. Only the expanded card uses the
native image protocol.

Current implementation:

- `ratatui-image` 11.0.6 with its Crossterm integration;
- `image` 0.25.6 with PNG decoding only;
- embedded assets under `src/assets/portraits/*-card.png`;
- capability and prepared-protocol owner: `src/portrait.rs`; and
- canonical fallback: the class's authored 24x32 RGB portrait master.

## Failure symptom

The Adventurer parchment and text rendered correctly, but the portrait region
was completely empty. Earlier runs showed the authored RGB sprite in the same
space, proving that card sizing and layout were not the cause.

The empty region is an important diagnostic signal. A native image widget
reserves its cells so Ratatui does not paint ordinary content over the image.
If an intermediary then discards the native graphics escape sequence, neither
the image nor the fallback is visible.

## Root cause

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

The fallback deliberately does not use `ratatui-image`'s Halfblocks renderer.
Questmancer already owns a higher-quality, deterministic portrait master for
that path.

## Herdr configuration

Herdr's local Kitty bridge is experimental and disabled by default. Enable it
in `~/.config/herdr/config.toml`:

```toml
[experimental]
kitty_graphics = true
```

Validate and apply the configuration without stopping the persistent server:

```bash
herdr config check
herdr server reload-config
herdr status
```

The reload returned `status: applied`, but the already-attached Herdr client
continued to expose its previous graphics capability. Exit only the attached
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
2. Confirm `experimental.kitty_graphics = true` and run `herdr config check`.
3. Attach a fresh Herdr client after enabling the bridge.
4. Open Questmancer and create disposable agents only; never repurpose an
   unrelated live agent for testing.
5. Select a Barbarian, Bard, Druid, Paladin, Rogue, Wizard, Goblin or Orc adventurer with an embedded
   portrait.
6. Confirm the card shows the transparent PNG while the Guild Hall remains the
   RGB half-block world.
7. Disable or bypass native capability in Storybook and confirm the authored
   sprite occupies the same card region.
8. Confirm classes without a PNG still show their authored portrait master.
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
2. **Herdr bridge:** confirm `experimental.kitty_graphics = true`.
3. **Fresh attachment:** reattach the Herdr client after enabling the bridge.
4. **Binary selection:** confirm the release binary was rebuilt.
5. **Asset contract:** confirm the PNG decodes and the class is mapped.
6. **Capability query:** trust the query result; do not add terminal-name
   heuristics.
7. **Action ordering:** ensure the plugin close completed before reopening it.

This sequence produced the confirmed native portrait path while retaining a
non-blank fallback for every unsupported identity and transport.
