# webmaster

> your agents are building a tiny internet.
>
> you are the webmaster.

`webmaster` turns a [Herdr](https://herdr.dev) session into a 90s control
centre and cybercafe. Working agents put sites under construction. Blocked
agents contact the webmaster. Completed work becomes a site update. Dead panes
become broken links.

Underneath the blinking text and tiny CRTs is a fast interface for seeing who
needs you, reading their output, replying, and jumping back into the work.

## Project status

Milestones 1 through 4 are implemented. The live webmaster desk turns Herdr
snapshots and events into sites, webmaster mail, a guestbook, and one selected
agent's recent output. Its async terminal runtime handles input, connection
updates, commands, redraws, and structured shutdown without blocking the desk.

If Herdr disconnects, webmaster keeps the last visible state on screen, shows
its reconnect attempt, and refreshes from a new snapshot after reconnecting.
Selected output is loaded lazily: when the selection or that pane's revision
changes, or when the webmaster explicitly presses `o`. It is never polled on
render ticks.

The compatibility baseline is Herdr `0.7.3` / protocol `16` because the runtime
depends on `session.snapshot`, the protocol schema command, and the current
agent event surface.

## Local development

Requirements:

- Rust `1.90.0` (installed automatically by `rustup` from
  `rust-toolchain.toml`)
- Herdr `0.7.3` or newer for plugin linking and live integration
- `just` is optional; every recipe is also a normal shell command

Build and run directly. Without Herdr's plugin environment the TUI starts in a
useful offline mode, so the layout and keys can be explored without a server:

```bash
cargo build
cargo run -- ui --view desk
cargo run -- ui --view cafe
```

Link a development checkout after building and while Herdr `0.7.3` / protocol
`16` is running:

```bash
cargo build
herdr plugin link .
herdr plugin action invoke opsydyn.webmaster.open
```

`herdr plugin link` intentionally skips the release download step. The runner
resolves `bin/herdr-webmaster`, then `target/release/herdr-webmaster`, then
`target/debug/herdr-webmaster`.

## Keys

| Key | Action |
|---|---|
| `1` / `F1` | webmaster desk |
| `2` / `F2` | cybercafe |
| `Tab` | cycle the active desk region |
| `j` / `Down` | select the next agent |
| `k` / `Up` | select the previous agent |
| `g` / `G` | select the first / last agent |
| `Enter` | visit (focus) the selected agent's pane |
| `r` | compose a reply to the selected agent |
| `Space` | mark the selected agent's attention seen locally |
| `o` | refresh the selected agent's recent output |
| `/` | search agent, handle, status, or site |
| `v` | focus the selected pane, then open reviewr when available |
| `Esc` | dismiss the active modal |
| `q` / `Ctrl-C` | close the TUI when no modal is open |

Reply and search modals accept normal text and `Backspace`. `Enter` sends the
reply or runs the search, `Ctrl-U` clears the input, and `Esc` cancels without
sending. The footer only advertises actions that apply to the current
selection. In particular, `v` appears only when the connected Herdr session
exposes `persiyanov.reviewr.open`.

## Plugin actions

```text
opsydyn.webmaster.open
opsydyn.webmaster.close
opsydyn.webmaster.toggle
opsydyn.webmaster.desk
opsydyn.webmaster.cafe
```

The controller uses `$HERDR_BIN_PATH`, an atomic lock directory, and
`$HERDR_PLUGIN_STATE_DIR/runtime.json` to avoid duplicate panes and recover
from stale pane state.

## Manual live acceptance

This procedure was last completed against Herdr `0.7.3` / protocol `16` on
2026-07-14. It verified a stable live subscription, blocked mail without a
restart, search, exact reply delivery, and pane focus. The plugin commands
require a running server.

```bash
herdr status
cargo build
herdr plugin link .
herdr plugin action invoke opsydyn.webmaster.open
```

From a different Herdr pane, publish a blocked test agent using that pane's
real ID:

```bash
PANE_ID="$(herdr pane current | jq -r '.result.pane.pane_id')"
herdr pane report-agent "$PANE_ID" \
  --source manual-acceptance \
  --agent acceptance-agent \
  --state blocked \
  --message "Need webmaster input" \
  --custom-status "waiting for reply"
```

Back at the desk, confirm the blocked transition appears as unread webmaster
mail without reopening the TUI. Exercise selection, `Enter`, `r`, `Space`,
`/`, and `o`; if the footer offers `v`, confirm it focuses this pane before
opening reviewr. A temporary transport interruption should retain the visible
desk under a reconnecting banner and resnapshot after recovery.

Return the synthetic agent to working and close the desk when finished:

```bash
herdr pane report-agent "$PANE_ID" \
  --source manual-acceptance \
  --agent acceptance-agent \
  --state working \
  --custom-status "implementing reply"
herdr plugin action invoke opsydyn.webmaster.close
```

Herdr `0.7.3`'s `report-agent` command accepts `idle`, `working`, `blocked`, or
`unknown`; it cannot synthesize `done`. Use a real agent completion event when
accepting the update-ready path.

## Architecture

One pure domain model feeds both views. Ratatui widgets are projections, not
owners of session state. The Herdr transport opens a fresh socket for each
ordinary request and a separate long-lived socket for event subscriptions.
It validates protocol `16`, refreshes with `session.snapshot` after disconnect
or pane-topology changes, and preserves unknown event names for the reducer.

Herdr `0.7.3` emits two event-envelope styles on the subscription stream:
snake-case lifecycle names such as `workspace_created`, and dotted scoped
events such as `pane.agent_status_changed`. Because agent-status subscriptions
are scoped, webmaster rebuilds one entry per unique pane after each snapshot.

The domain boundary keeps presence (`working`, `blocked`, `done`, `idle`,
`exited`) separate from the webmaster's seen/unseen attention. Site status is
derived, guestbook events are deterministic and bounded, and native agent
session identity keeps original personas stable when panes move. Equal state,
events, and injected timestamps always produce equal reducer output.

See the [design](docs/superpowers/specs/2026-07-14-herdr-webmaster-design.md),
[pixel-art bible](docs/superpowers/specs/2026-07-14-pixel-art-design.md), and
[protocol plan](docs/superpowers/plans/2026-07-14-milestone-2-herdr-protocol.md).

## Verification

```bash
just verify
```

Without `just`:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
bash tests/scripts.sh
bash -n herdr/install.sh herdr/run.sh herdr/control.sh
```

Run the focused protocol suite and regenerate the installed schema with:

```bash
just protocol-test
herdr api schema --output /tmp/herdr-api.schema.json
```

Run the focused domain suite with `just domain-test` (or the corresponding
`cargo test --test ...` command in the `justfile`).

Run the focused operational desk suite with `just desk-test`.

The fixture suite does not require a running Herdr server. Live plugin linking
does: start `herdr server` in another terminal before `herdr plugin link .`.
The local milestone-4 acceptance run linked `opsydyn.webmaster`, exercised the
live desk loop against Herdr `0.7.3` / protocol `16`, and then stopped its
temporary server.

## Privacy

Webmaster is local only. It has no telemetry, cloud sync, or network service.
The install script contacts GitHub only to download an explicitly requested
release artifact.

```text
best viewed with ratatui
80x24 minimum
this site is always under construction
```
