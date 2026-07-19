# Questmancer

> Your agents have entered the dungeon.
>
> You are the Questmancer.

Questmancer turns a Herdr session into a living adventurers' guild. Working
agents delve through chambers of code. Blocked agents call for counsel.
Completed work returns as spoils awaiting inspection.

It is a local Ratatui interface for scanning a session, reading selected output,
counselling blocked agents, and returning to their panes. One shared model drives
two views:

- **Guild Hall** is one inhabited Great Room: campaigns share the hall while
  attention, history, selected output and review actions live at stable
  landmarks.
- **Delve** is the spatial view: each Herdr workspace becomes a connected dungeon
  and each agent occupies a chamber with a state-specific pose.

Both views keep the same selection and actions. Narrow terminals use compact,
actionable layouts rather than dropping controls. Unicode and xterm-256 are the
default presentation; ASCII, ANSI-16, reduced-motion, and no-motion modes are
first-class fallbacks.

## Requirements

- Herdr `0.7.4` using protocol `16`
- Rust `1.90.0` (selected by `rust-toolchain.toml`)
- `jq` for older-link migration and the optional fake-agent walkthrough below
- `just` only if you want the contributor shortcuts
- Ruby with standard-library YAML support only for contributor workflow checks

Questmancer currently ships from source. From this checkout:

```bash
cargo build
herdr plugin link .
herdr plugin action invoke opsydyn.questmancer.open
```

`herdr plugin link .` deliberately skips release download. The plugin runner
uses `bin/questmancer`, then `target/release/questmancer`, then
`target/debug/questmancer`, choosing the first executable it finds.

### Cutting over an older development link

Close and unlink the previous local plugin before linking Questmancer. This
discovers the older development link by its manifest name and local source, so
the migration remains readable without carrying its qualified identifier into
current release surfaces.

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

The `close` is best-effort because an older pane may already be gone. An
`unlink` error is meaningful: inspect `herdr plugin list` before continuing.
The commands above were checked against the installed Herdr `0.7.4` CLI.

## Actions and views

```text
opsydyn.questmancer.open
opsydyn.questmancer.close
opsydyn.questmancer.toggle
opsydyn.questmancer.guild
opsydyn.questmancer.delve
```

`open` restores the saved/configured view when it creates a pane and focuses an
existing Questmancer pane. `guild` and `delve` switch an existing pane or create
one in the requested view. `toggle` focuses Questmancer unless invoked from its
own pane, where it closes it.

## The Great Room

The Guild Hall is one hall with many Campaign Tables, not a dashboard of
separate panels. Every workspace is a campaign with its own banner, seal and
table, while the guild shares the same landmarks:

- **Guild Door** shows whether paths to Herdr are opening, joined, lost or
  incompatible.
- **Quest Wall** holds campaign banners, rollups and attention.
- **Campaign Tables** hold deterministic expedition tokens for adventurers who
  are still away delving.
- **Counsel Bell** receives blocked adventurers asking for help.
- **Hearth** gives idle adventurers a truthful place to rest.
- **Chronicle Lectern** records a bounded history of guild events.
- **Scrying Alcove** shows the selected adventurer and recent output.
- **Spoils Desk** receives completed unseen work and offers Reviewr when that
  integration is available.

These are Truthful Stations: each live adventurer appears exactly once. Working,
unknown and acknowledged-complete adventurers remain tokens at their Campaign
Table; blocked adventurers are projected at the Counsel Bell; idle adventurers
rest at the Hearth; unseen completed work returns at the Spoils Desk. Departed
adventurers leave no body behind—the Guild Door and Chronicle retain the event.

The room changes camera rather than identity as the terminal narrows. At 120
columns and wider, the whole Great Room and every campaign table coexist. From
80–119 columns, the camera crops around the selected campaign while preserving
the Door, Quest Wall and Hearth. Below 80 columns, `Tab` pans a landmark camera
around that same room. Selection, search, observe, counsel, acknowledge, output
refresh and Spoils inspection keep the bindings below in every valid context.

You can also run the binary without Herdr to inspect its offline layout:

```bash
cargo run -- ui --view guild
cargo run -- ui --view delve
```

## Keys

| Key | Action |
|---|---|
| `1` / `F1` | Guild Hall |
| `2` / `F2` | Delve |
| `Tab` | Cycle the active Guild Hall region |
| `j` / `Down` | Select the next adventurer |
| `k` / `Up` | Select the previous adventurer |
| `g` / `G` | Select the first / last adventurer |
| `Enter` | Observe the selected adventurer's pane |
| `r` | Compose counsel for the selected adventurer |
| `Space` | Acknowledge the selected unread Summons locally |
| `o` | Refresh the selected adventurer's recent output |
| `/` | Search adventurer, handle, visible presence, class, ancestry, or campaign |
| `v` | Inspect Spoils with the configured Reviewr action when available |
| `?` | Show in-app help |
| `Esc` | Dismiss the active modal |
| `q` / `Ctrl-C` | Close Questmancer when no text modal is open |

Counsel and search accept normal text and `Backspace`. `Enter` submits,
`Ctrl-U` clears, and `Esc` cancels. The footer advertises only actions valid for
the current selection. Questmancer never focuses, reads, or replies to its own
managed pane.

## Configuration

After linking, locate the user-owned configuration directory with:

```bash
herdr plugin config-dir opsydyn.questmancer
```

Create `config.toml` in that directory. A complete file using the defaults is:

```toml
default_view = "guild"            # guild | delve
motion = "full"                   # full | reduced | none
character_set = "unicode"         # unicode | ascii
color_mode = "xterm256"           # xterm256 | ansi16
output_preview_lines = 80          # 10..=500
chronicle_max_entries = 500        # 50..=10000, in-memory bound
reviewr_action = "persiyanov.reviewr.open"
show_elapsed_time = true
```

Unknown fields are accepted for forward compatibility. An invalid TOML
document, enum, bound, or blank `reviewr_action` rejects the whole file and
Questmancer starts with defaults while reporting the error.

Initial-view precedence is an explicit `ui --view guild|delve`, saved
`last_view`, configured `default_view`, then built-in `guild`. The explicit
`guild` and `delve` plugin actions take precedence. Saved display preferences
win over configuration because they represent the last accepted runtime state;
output limits, elapsed-time display, and the Reviewr action remain
configuration-only.

## Local state, ownership, and privacy

Herdr supplies separate plugin directories. Questmancer uses them as follows:

| Path | Owner | Purpose |
|---|---|---|
| `$HERDR_PLUGIN_CONFIG_DIR/config.toml` | You | Read-only configuration; Questmancer never creates or rewrites it |
| `$HERDR_PLUGIN_STATE_DIR/runtime.json` | Lifecycle controller | Ephemeral singleton-pane registration |
| `$HERDR_PLUGIN_STATE_DIR/state.json` | Persistence worker | Atomically replaced, versioned durable user intent |
| `$HERDR_PLUGIN_STATE_DIR/chronicle.jsonl` | Persistence worker | Append-only semantic event history |

`state.json` contains the last view and selection, display preferences, generated
persona catalogue and assignments, and exact Summons acknowledgement episodes.
It does not copy Herdr-owned live output or topology. The in-memory Chronicle
projection is bounded by `chronicle_max_entries`, but
`$HERDR_PLUGIN_STATE_DIR/chronicle.jsonl` is append-only and has no automatic
size or age bound; it grows until you remove it. Persisted Chronicle entries can
contain the adventurer key and host agent name, campaign/workspace ID, pane ID,
pane revision, timestamp, event kind, and summary.

If a plugin directory is unavailable, its store is disabled and the TUI still
runs in memory. A damaged `state.json` is rejected as a whole and state writes
remain disabled until restart so evidence is not overwritten. Chronicle damage
is isolated per line. Close Questmancer before repairing either file; copy the
original somewhere safe first. Cleanup uses the configured plugin state paths:
removing `$HERDR_PLUGIN_STATE_DIR/state.json` resets saved intent, removing
`$HERDR_PLUGIN_STATE_DIR/chronicle.jsonl` clears the unbounded on-disk history,
and removing stale `$HERDR_PLUGIN_STATE_DIR/runtime.json` allows the next `open`
action to recreate the pane registration.

Questmancer is local-only at runtime. The application has no telemetry, cloud
sync, or network service, and its runtime communication stays on Herdr's local
socket. Source builds may download Rust dependencies, CI downloads actions and
toolchains, and `herdr/install.sh` downloads a release archive and its checksum
from the explicitly configured GitHub repository.

## Fake-agent walkthrough

Use a dedicated plain Herdr pane. Do not target a Codex pane, the Questmancer
pane, or another agent-owned pane: Herdr can accept the report while ownership
rules keep that synthetic source out of the session snapshot.

1. Create and focus a disposable plain pane in Herdr.
2. Capture its ID, then report state transitions with one source and increasing
   sequence numbers:

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

herdr pane report-agent "$PANE_ID" \
  --source "$SOURCE_ID" \
  --agent smoke-adventurer \
  --state working \
  --message "manual walkthrough complete" \
  --seq 3

herdr pane release-agent "$PANE_ID" \
  --source "$SOURCE_ID" \
  --agent smoke-adventurer \
  --seq 4
```

Confirm working, blocked/Summons, counsel, acknowledge, search, and output refresh
in both views. The Herdr `0.7.4` command accepts `idle`, `working`, `blocked`,
and `unknown`, but the live acceptance run found that a synthetic `idle` report
was normalized to `done` in `session.snapshot`. It therefore did not prove the
resting projection. The command does not accept a `done` literal. Verify resting
and returned spoils with a real agent transition or the fixture and rendering
tests; do not describe either synthetic path as accepted live behavior.

When finished, release the synthetic source as above. Close only the disposable
pane you created:

```bash
herdr pane close "$PANE_ID"
```

The guarded Herdr `0.7.4` acceptance record, including the exact pass and
blocked results, persistence proof, and cleanup audit, is in
[`docs/manual-test/questmancer-0.1.0.md`](docs/manual-test/questmancer-0.1.0.md).

## Release packaging

Tags named `v<version>` build four archives. Every archive contains one
executable named `questmancer` at its root.

| Target | Asset |
|---|---|
| `x86_64-unknown-linux-gnu` | `questmancer-v0.1.0-x86_64-unknown-linux-gnu.tar.gz` |
| `aarch64-unknown-linux-gnu` | `questmancer-v0.1.0-aarch64-unknown-linux-gnu.tar.gz` |
| `x86_64-apple-darwin` | `questmancer-v0.1.0-x86_64-apple-darwin.tar.gz` |
| `aarch64-apple-darwin` | `questmancer-v0.1.0-aarch64-apple-darwin.tar.gz` |

The release job downloads all four matrix artifacts, verifies the complete
matrix, creates `SHA256SUMS`, and publishes the archives and checksum together.
`herdr/install.sh` selects the host target, downloads the matching archive and
`SHA256SUMS`, verifies SHA-256, then installs `bin/questmancer`. Set
`QUESTMANCER_REPOSITORY=owner/repository` to test the installer against a fork.

## Developer Storybook

Review Questmancer's sprites, widgets, fixed Great Room scenes, Delve variants
and compatibility modes without starting Herdr by running `just storybook`.

```bash
just storybook
```

The Storybook is a developer-only Cargo feature. It reads no Herdr environment,
connects to no socket and writes no plugin state.

Use j/k to move between stories, h/l to change categories, Enter to inspect the
production canvas, Esc to return, ? for help and q to quit.

For the current class-art review, inspect **Sprite Scout & Shadow World** and
then **Sprite Scout & Shadow Masters**. They show Bard, Ranger and Rogue at
their authored 16x24 world size and beside their separate 24x32 portrait
masters. This is a review-only asset lane: it does not prompt agents, call
Herdr, write state, or replace the existing production sprites.

Run its focused automated checks with `just storybook-test`.

```bash
just storybook-test
```

## Experimental scene-first preview

The production Questmancer pane still uses the existing UI renderer. The
scene-first renderer is an opt-in developer preview used to review the RGB
pixel world against the approved [north-star reference art](reference-art/README.md).
It is not declared in `herdr-plugin.toml`, is not packaged into releases, and
does not replace the plugin pane.

From an existing Herdr shell with `HERDR_SOCKET_PATH` exported:

```bash
just scene-preview-test
just scene-preview
```

The preview renders live Herdr state but deliberately ignores normal
Questmancer controls. It cannot reply to agents, focus panes, inspect output,
mark summons read, or change persisted state. Use Codex CLI and Herdr for
those actions. The [guarded manual guide](docs/manual-test/questmancer-scene-preview.md)
and [cutover evidence ledger](docs/superpowers/reviews/2026-07-17-scene-first-cutover.md)
define the review process; production cutover is **not yet decided**.

## Contributor checks

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
bash tests/scripts.sh
bash -n herdr/install.sh herdr/run.sh herdr/control.sh
cargo build --release
git diff --check
```

Focused shortcuts include `just protocol-test`, `just domain-test`,
`just guild-test`, `just delve-test`, `just persistence-test`, and
`just property-test`. `just verify` runs the normal local gate; `just
release-check` adds the release build and whitespace check. The shell gate uses
Ruby's standard-library YAML parser to validate active CI and release workflow
semantics; GitHub-hosted runners already provide it.

## Unlink and clean up

Close the managed pane before unlinking the development plugin:

```bash
herdr plugin action invoke opsydyn.questmancer.close 2>/dev/null || true
herdr plugin unlink opsydyn.questmancer
```

Unlinking removes the development registration, not your checkout or durable
configuration. Remove `bin/questmancer` only if you ran `just install-local`.
Keep or back up the Herdr-managed configuration and state described above
unless you intentionally want to reset Questmancer.

Architecture and visual rationale live in the approved
[creative direction](docs/superpowers/specs/2026-07-15-questmancer-creative-direction.md)
and the plans under `docs/superpowers/plans/`.
