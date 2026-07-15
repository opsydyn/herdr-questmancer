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

Milestones 1 through 5 and the Milestone 6.1 local-persistence slice are
implemented. The live webmaster desk turns Herdr
snapshots and events into sites, webmaster mail, a guestbook, and one selected
agent's recent output. Its async terminal runtime handles input, connection
updates, commands, redraws, and structured shutdown without blocking the desk.

The cybercafe projects that same model as an actionable room of original pixel
characters: seated agents occupy workstations and the selected contributor gets
a separately composed full-body profile. It remains usable at 80x24, pages dense
herds to keep the selection visible, and falls back to a compact list below 80
columns. The IT Crowd reference supplied during design was used only as a
fidelity reference; no character, outfit, logo, pose, or composition was copied.

If Herdr disconnects, webmaster keeps the last visible state on screen, shows
its reconnect attempt, and refreshes from a new snapshot after reconnecting.
Selected output is loaded lazily: when the selection or that pane's revision
changes, or when the webmaster explicitly presses `o`. It is never polled on
render ticks.

Cafe animation is derived from an injected runtime clock. It samples wall time
once to retain epoch-shaped domain timestamps, then advances exclusively from a
Tokio monotonic origin. A single resettable sleep schedules only the next visible
frame; the desk, static cafe states, and no-motion cafe are event-driven and do
not redraw just because time passed.

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

## Local persistence and configuration

Webmaster uses Herdr's plugin directories when they are available. The files
have deliberately separate owners:

| Path | Owner | Purpose |
|---|---|---|
| `$HERDR_PLUGIN_CONFIG_DIR/config.toml` | user | read-only configuration; webmaster never creates or rewrites it |
| `$HERDR_PLUGIN_STATE_DIR/runtime.json` | lifecycle controller | ephemeral singleton-pane registration |
| `$HERDR_PLUGIN_STATE_DIR/state.json` | webmaster persistence worker | atomically replaced, versioned durable user intent |
| `$HERDR_PLUGIN_STATE_DIR/guestbook.jsonl` | webmaster persistence worker | append-only semantic event history |

If a directory variable is absent, its store is disabled. The TUI still starts
with safe defaults and in-memory guestbook history; persistence never becomes a
requirement for using the desk or cafe.

A complete `config.toml` is:

```toml
default_view = "desk"             # desk | cafe
motion = "full"                   # full | reduced | none
character_set = "unicode"         # unicode | ascii
color_mode = "xterm256"           # xterm256 | ansi16
output_preview_lines = 80          # 10..=500
guestbook_max_entries = 500        # 50..=10000, in-memory display bound
reviewr_action = "persiyanov.reviewr.open"
show_elapsed_time = true
```

Missing fields use these defaults and unknown fields are accepted. An invalid
TOML document, enum, bound, or blank `reviewr_action` rejects the complete file
instead of partially applying it. Webmaster reports the path and error, then
uses defaults.

The initial view precedence is explicit `ui --view desk|cafe`, saved
`last_view`, configured `default_view`, then the built-in desk. The `desk` and
`cafe` plugin actions are explicit; ordinary `open`, a closed `toggle`, and bare
`ui` allow the saved/configured view to win. Saved display preferences override
configuration because they represent the most recent accepted runtime state.
Configuration-only output limits, elapsed-time display, and reviewr action are
never written to `state.json`.

`state.json` stores schema version 1, the last view, display preferences,
selected persona, the authored persona catalog, and exact seen-attention
episodes. Herdr remains authoritative for workspaces, panes, live agents,
presence, focus, revisions, and output. The guestbook log stores one compact
JSON record per newline. Replay keeps valid complete records in chronological,
deduplicated, bounded in-memory history even when another line is malformed or
the final line is truncated; the JSONL file itself is not compacted.

All persistence is local. Webmaster sends no telemetry, performs no cloud sync,
and adds no network service. The only network access in this repository is the
install script downloading an explicitly requested release from GitHub.

### Corruption and recovery

Persistence errors are non-fatal and appear in the TUI status surface and on
stderr after terminal restoration. If an existing `state.json` cannot be read,
parsed, or validated, it is rejected as a whole and state publication is
disabled for that process lifetime; guestbook appends remain enabled. This
prevents the first live snapshot from overwriting evidence needed for recovery.
Guestbook damage is isolated per record and reports one-based line numbers,
with repeated diagnostics folded after five records.

To recover, close webmaster first, copy the affected files somewhere safe, and
then fix or remove only the damaged file. Removing `state.json` resets saved
view, preferences, selection, personas, and seen markers. Removing
`guestbook.jsonl` clears history. Fix `config.toml` in place or remove it to use
defaults. Restart webmaster after correcting or removing `state.json` to
re-enable durable snapshot writes. If singleton control points to a pane that no
longer exists, remove only `runtime.json`; the next `open` recreates it. Do not
edit a file while the TUI is running because the persistence worker owns state
and guestbook writes.

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
exposes the fully qualified action configured by `reviewr_action`.

The desk and cafe use the same typed action path for selection, visit, reply,
seen, search, refresh, and optional reviewr launch. At 120 columns and above the
cafe shows a workstation grid plus the selected full-body profile; from 80 to
119 it uses the full grid; below 80 it uses the compact vertical list.

## Cybercafe state language

| Herdr state | Visible cafe signal | Full-motion cadence |
|---|---|---|
| working | `BUILDING`, typing pose, CRT cursor and modem | 6 fps |
| blocked | `HELP!`, raised hand and help card | 2 fps |
| done, unseen | `UPDATE READY`, eight-frame confetti transition | 8 fps for 1 second |
| done, seen | `DONE`, relaxed seated pose | event-driven |
| idle | `IDLE`, screensaver pose | 1 fps |
| exited | `BROKEN LINK`, broken CRT and empty chair | event-driven |
| unknown | `UNKNOWN` and `?` marker | event-driven |
| focused | `LIVE` and a lit desk lamp, without replacing state | follows state |

Colour is supplementary: every state has a text label and silhouette or marker.
Unicode half-block art is canonical, with a pure ASCII projection and an
ANSI-16 palette available to constrained terminals.

The display preference model supports:

```text
motion: full | reduced | none
character set: unicode | ascii
colour mode: xterm-256 | ansi-16
```

`full` enables semantic state animation; `reduced` freezes rapid effects but
keeps the slow idle screensaver; `none` is entirely event-driven. The defaults
are `full`, `unicode`, and `xterm-256`; the local configuration and durable state
described above can override and restore them.

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

The last fully completed live run was against Herdr `0.7.3` / protocol `16` on
2026-07-14, including the Milestone 5 cafe on final commit `9d5d257`. It
verified a stable live subscription, a blocked `HELP!` transition without a
restart, search, refresh, local seen state, exact isolated reply delivery, and
pane focus. The plugin commands require a running server.

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
opening reviewr. Note the selected persona handle, switch view with `1`/`2`, and
press `Space` so the exact blocked episode is seen.

Close and reopen the webmaster pane to exercise a restarted plugin pane:

```bash
herdr plugin action invoke opsydyn.webmaster.close
herdr plugin action invoke opsydyn.webmaster.open
```

Confirm the chosen view and configured preferences return, the same persona is
selected with the same handle, the blocked episode remains seen, and prior
guestbook entries replay exactly once. A temporary transport interruption
should retain the visible desk under a reconnecting banner, resnapshot after
recovery, and not duplicate replayed history. Recheck `Enter`, `r`, and `o`
after the reconnect to cover focus, exact reply delivery, and selected-output
reads.

Press `2` (or invoke `opsydyn.webmaster.cafe`) and confirm the same blocked
agent is visible as `HELP!` with a raised-hand workstation pose. Exercise the
same selection, visit, reply, seen, search, and refresh actions at 80x24. The
Milestone 5 automated gate covers working, blocked, done, idle, exited, reduced
motion, no motion, Unicode, ASCII, xterm-256, ANSI-16, dense herds, and tiny
areas; a fresh live cafe smoke is intentionally part of the release acceptance.

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
`unknown`; it cannot synthesize `done`. Use a real agent completion event for
the update-ready path, mark it seen, restart webmaster again, and confirm that
the exact completion episode restores while a later pane revision is unseen.

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

The theatre layer derives poses, deterministic frames, and the earliest next
phase boundary across the complete cafe model. This matters when 6 fps typing
and 8 fps completion effects interleave, and ensures completion stops at exactly
one second even if input arrives just before that boundary. The terminal owns
one cancellation-safe resettable sleep. Deadlines map the sampled model
timestamp back onto that same monotonic origin, so time spent rendering cannot
shift a semantic frame boundary; an already-past boundary wakes immediately. It
drops the timer in event-driven modes, creates no per-frame tasks, performs no
output reads or persistence on animation wakes, and re-derives the deadline
after input, runtime, and clock events.

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

Run persistence examples and property suites with:

```bash
just persistence-test
just property-test
```

The direct high-case-count property command used by the Milestone 6.1 gate is:

```bash
PROPTEST_CASES=1024 cargo test --test property_domain --test persisted_state
```

`PROPTEST_CASES` overrides Proptest's normal case count. When Proptest finds a
failure, it shrinks the input and writes a source-parallel regression seed such
as `tests/property_domain.proptest-regressions` (or a file below a
`proptest-regressions` directory). These files are intentionally tracked and
must not be deleted or ignored after the assertion is fixed.

Focused persistence coverage includes configuration and path discovery,
versioned state validation/overlay, atomic JSON publication, tolerant guestbook
replay/append, worker debounce and shutdown, and startup precedence. Tests use
temporary directories; they must not create `config.toml`, `state.json`,
`guestbook.jsonl`, or `runtime.json` in the checkout.

Run the focused operational desk suite with `just desk-test`.

Run the focused pixel-art, theatre, cafe, and scheduler suite with:

```bash
just cafe-test
```

Run the complete Milestone 5 gate, including the release build, with:

```bash
just milestone5-verify
```

Equivalent commands are:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
bash tests/scripts.sh
bash -n herdr/install.sh herdr/run.sh herdr/control.sh
cargo build --release
git diff --check
```

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
