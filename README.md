# Questmancer

> Your agents have entered the dungeon.
>
> You are the Questmancer.

Questmancer turns a Herdr session into a living, 16-bit adventurers' guild.
Herdr workspaces become campaigns and coding agents become adventurers. Working
agents travel through the Delve, blocked agents call for counsel, and completed
work returns with spoils.

The production interface is one RGB pixel world with two rooms:

- **Guild Hall** is the warm operational home for the whole party.
- **Delve** is the dungeon view of the same live Herdr state.

Both rooms retain the same selection and commands. An in-world selection rune
marks the selected adventurer; contextual parchment overlays handle counsel,
search and scrying without replacing the scene with a dashboard. A persistent
Librarian in the Guild Hall opens the fixed Librarian's Ledger handbook.

## Requirements

- Herdr `0.7.4`, protocol `16`
- Rust `1.90.0` (selected by `rust-toolchain.toml`)
- `jq` for the migration and guarded smoke-test recipes below
- `just` only for contributor shortcuts

## Install from this checkout

Questmancer currently supports source linking:

```bash
cargo build --release
herdr plugin link .
herdr plugin action invoke opsydyn.questmancer.open
```

The linked plugin runner resolves `target/release/questmancer`, then
`bin/questmancer`, then `target/debug/questmancer`. After a new
`cargo build --release`, reopen the Questmancer pane; relinking is unnecessary
while Herdr is still linked to this checkout.

### Cutting over an older development link

Close and unlink the previous local plugin before linking Questmancer. The
recipe discovers it by manifest name and local source:

```bash
previous_plugin=$(
  herdr plugin list --json |
    jq -r 'first(.result.plugins[] | select(.name == "webmaster" and .source.kind == "local")) | .plugin_id // empty'
)
if [[ -n $previous_plugin ]]; then
  herdr plugin action invoke "$previous_plugin.close" 2>/dev/null || true
  herdr plugin unlink "$previous_plugin"
fi
cargo build --release
herdr plugin link .
herdr plugin action invoke opsydyn.questmancer.open
```

An `unlink` error is meaningful: inspect `herdr plugin list` before continuing.

## Plugin actions

```text
opsydyn.questmancer.open
opsydyn.questmancer.close
opsydyn.questmancer.toggle
opsydyn.questmancer.guild
opsydyn.questmancer.delve
```

`open` restores and focuses the singleton Questmancer pane. `guild` and `delve`
open that pane in the requested room, or switch the existing pane. `toggle`
focuses Questmancer unless invoked from its own pane, where it closes it.

## Controls

| Key | Action |
|---|---|
| `1` / `F1` | Enter the Guild Hall |
| `2` / `F2` | Enter the Delve |
| `j` / `Down` | Select the next adventurer |
| `k` / `Up` | Select the previous adventurer |
| `g` / `G` | Select the first / last adventurer |
| `Enter` | Observe the selected adventurer's Herdr pane |
| `r` | Open the counsel parchment |
| `Space` | Acknowledge the selected unread summons locally |
| `o` | Refresh the selected adventurer's recent output |
| `!` | Jump to the next adventurer waiting on you |
| `Tab` | Move to the next campaign's party |
| `c` | Open the Chronicle |
| `/` | Search the party and campaigns |
| `n` / `N` | Walk to the next / previous search match |
| `v` | Inspect spoils through Reviewr when available |
| `m` | Cycle motion: full, reduced, still |
| `u` | Switch between Unicode and ASCII glyphs |
| `p` | Switch between truecolour and 16 colours |
| `?` | Open or close the Librarian's Ledger |
| `Esc` | Dismiss the active parchment |
| `q` / `Ctrl-C` | Close Questmancer when no text parchment is open |

`!` cycles the adventurers who are waiting on a human, most urgent first: an
unanswered call for counsel, then one already seen but unresolved, then the
quieter summons, and within each rank whoever has waited longest. Deferred
summons are skipped until their snooze expires. When nobody is waiting the
selection stays put and Questmancer says so. While at least one adventurer is
waiting, the command ribbon carries the count.

`m`, `u` and `p` change motion, glyphs and colour depth while running. Each
reports the setting it landed on and is written to durable state, so a change
made at runtime survives a restart without editing the configuration file.

Search reports how many adventurers matched and `n`/`N` walk them, wrapping.
Matches for adventurers who have since left the party are dropped rather than
selected.

`c` opens the Chronicle: the guild's own record of who joined, set out, asked
for counsel, returned with spoils, rested, departed, and which campaigns
closed. With an adventurer selected it shows that adventurer's history;
otherwise the whole guild's, newest first. `Esc` or `c` closes it. The
Chronicle is a reading surface — no key acts on the party while it is open.

`Tab` moves the selection into the next campaign's party and wraps. With the
whole party on one campaign it stays put and says so.

Counsel and search accept ordinary text. `Enter` submits, `Ctrl-U` clears, and
`Esc` cancels. Questmancer never selects, focuses, reads, or counsels its own
managed pane.

The persistent Librarian is a non-agent Guild Hall character. Click him—or
press `?` anywhere—to open the same four-page handbook. Use left/right to page
through Welcome, Reading the Party, Questmancer's Keyring and Safe Chronicle;
`Esc` closes it. The Librarian cannot receive counsel, focus or output commands.

The Keyring page is generated from the binding table itself rather than
written by hand, and a test sweeps every key the input handler accepts and
fails on any action the Keyring does not describe. The table above and the
Keyring cannot drift apart from what the keys actually do.

Herdr and the coding-agent CLI remain authoritative. Questmancer does not
manually steer sprites: their identity, location, pose and attention effects
derive from Herdr state. The only text sent to an agent from Questmancer is
explicit counsel composed with `r`; all other sprite behaviour is projection.

## Configuration

Locate the user-owned configuration directory with:

```bash
herdr plugin config-dir opsydyn.questmancer
```

Create `config.toml` there when defaults are not sufficient:

```toml
default_view = "guild"            # guild | delve
motion = "full"                   # full | reduced | none
character_set = "unicode"         # unicode | ascii
color_mode = "xterm256"           # xterm256 | ansi16
output_preview_lines = 80          # 10..=500
chronicle_max_entries = 500        # 50..=10000
reviewr_action = "persiyanov.reviewr.open"
show_elapsed_time = true
```

Invalid configuration is reported visibly and safe defaults are used. Display
compatibility settings remain accepted while the RGB scene is the sole
production renderer.

### Optional Herdr sidebar marginalia

Questmancer can add a small, truthful marginal note to Herdr's own sidebar.
This is opt-in: add rows to your existing user-owned Herdr `config.toml`; the
plugin never rewrites global Herdr configuration or semantic agent state.

```toml
[ui.sidebar.agents]
rows = [
  ["state_icon", "workspace", "tab"],
  ["agent", "$quest_role", "$quest_omen"],
]

[ui.sidebar.spaces]
rows = [
  ["state_icon", "workspace"],
  ["branch", "git_status", "$quest_campaign"],
]
```

`$quest_role` is the adventurer's stable ancestry and class, `$quest_omen` is
the current presence in guild language, and `$quest_campaign` is the live
party and summons roll-up. The feature uses Herdr's display-only metadata
channel, so it never changes an agent title, status label, focus or task.

`rows` replaces the corresponding Herdr sidebar rows: merge this example with
any existing sidebar customisation, then run `herdr config check` and reload
the configuration by the method appropriate for your Herdr server. Questmancer
does not use `rows_by_agent`; its dynamic personas remain presentation data.

## Local state and privacy

Questmancer is local-only at runtime. It has no telemetry, cloud sync or network
service; runtime communication stays on Herdr's local socket.

| Path | Purpose |
|---|---|
| `$HERDR_PLUGIN_CONFIG_DIR/config.toml` | User-owned read-only configuration |
| `$HERDR_PLUGIN_STATE_DIR/runtime.json` | Ephemeral singleton-pane registration |
| `$HERDR_PLUGIN_STATE_DIR/state.json` | Atomic, versioned local intent and persona assignments |
| `$HERDR_PLUGIN_STATE_DIR/chronicle.jsonl` | Append-only semantic event history |

The state store does not copy Herdr-owned output or topology. The Chronicle file
has no automatic size bound and grows until the user removes it.

## Developer Storybook

The feature-gated Storybook renders the production scene engine without Herdr,
agent processes or persistent state:

```bash
just storybook
```

It contains twenty-five fixed production stories, each owned exactly once:

- Guild Hall
- Delve
- all eight classic world masters
- the Barbarian v2 legacy comparison and complete semantic pose family
- all eight classic portrait masters
- native class-led Artificer, Barbarian, Bard, Cleric, Druid, Paladin, Ranger, Rogue, Testmender and Wizard card portraits with authored-sprite fallbacks
- Goblin world and portrait Easter egg
- persistent Librarian world and Ledger fallback sprites
- selected adventurer
- counsel parchment
- search parchment
- scrying parchment
- Librarian's Ledger
- narrow viewport

Use `j`/`k` or arrow keys to move through stories and `q` to exit. Run its
focused checks with `just storybook-test`. The Storybook header reports
`native Kitty`, `native Sixel`, `native iTerm2`, or `authored sprite fallback`
so the active card rendering path is explicit.

Native Kitty portraits inside a Herdr-managed pane also require Herdr's
experimental graphics bridge:

```toml
[experimental]
kitty_graphics = true
```

After changing that setting, validate it with `herdr config check` and reload
the server configuration with `herdr server reload-config`. Without the bridge,
Questmancer deliberately uses its authored RGB sprite; the outer terminal being
Kitty-compatible is not sufficient on its own.

See [Native portrait rendering through Herdr](docs/troubleshooting/native-portrait-rendering.md)
for the complete failure analysis, recovery sequence and guarded smoke test.

## Guarded live smoke test

Use a disposable plain Herdr pane. Never target a Codex pane, the Questmancer
pane, or another agent-owned pane.

```bash
PANE_ID=$(herdr pane current | jq -r '.result.pane.pane_id')
SOURCE_ID="questmancer-smoke-$(date +%s)-$$-$RANDOM"

herdr pane report-agent "$PANE_ID" \
  --source "$SOURCE_ID" \
  --agent smoke-adventurer \
  --state working \
  --message "mapping the dungeon" \
  --seq 1

herdr pane report-agent "$PANE_ID" \
  --source "$SOURCE_ID" \
  --agent smoke-adventurer \
  --state blocked \
  --message "needs counsel" \
  --seq 2

herdr pane release-agent "$PANE_ID" \
  --source "$SOURCE_ID" \
  --agent smoke-adventurer \
  --seq 3
```

Herdr `0.7.4` can report `idle`, `working`, `blocked` and `unknown`; it cannot
synthesize an explicit `done` transition. Do not claim live completion-state
acceptance from this recipe.

The full environment-preserving procedure is in the
[production manual acceptance guide](docs/manual-test/questmancer-scene-preview.md).
Visual decisions and unreviewed states are recorded separately in the
[cutover evidence ledger](docs/superpowers/reviews/2026-07-17-scene-first-cutover.md).

## Contributor checks

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
PROPTEST_CASES=4096 cargo test --test scene_pixel_properties --test scene_stage_properties
bash tests/scripts.sh
bash -n herdr/install.sh herdr/run.sh herdr/control.sh
cargo build --release
```

For an offline production run, use `just run guild` or `just run delve`.

## Uninstall a local link

Close Questmancer first, then remove only this plugin link:

```bash
herdr plugin action invoke opsydyn.questmancer.close
herdr plugin unlink opsydyn.questmancer
```

Do not stop Herdr to uninstall a plugin.

```text
best viewed with ratatui
80x24 minimum
the guild hall is always open
```
