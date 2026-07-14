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

Milestones 1 and 2 are implemented: the Rust executable, empty desk/cafe
projections, safe terminal lifecycle, plugin manifest, singleton actions, and
the schema-grounded Herdr protocol runtime. Domain normalization and agent
interactions arrive in the next milestones described in [PLAN.md](PLAN.md).

The plugin requires Herdr `0.7.3` because its runtime design depends on
`session.snapshot`, the protocol schema command, and the current agent event
surface.

## Local development

Requirements:

- Rust `1.90.0` (installed automatically by `rustup` from
  `rust-toolchain.toml`)
- Herdr `0.7.3` or newer for plugin linking and live integration
- `just` is optional; every recipe is also a normal shell command

Build and run directly:

```bash
cargo build
cargo run -- ui --view desk
cargo run -- ui --view cafe
```

Link a development checkout after building and while Herdr `0.7.3+` is running:

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
| `?` | help (reserved for milestone 4) |
| `q` / `Ctrl-C` | close the TUI |

The full v0.1 key map will add selection, focus, reply, seen state, output
refresh, search, and optional reviewr actions.

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

The fixture suite does not require a running Herdr server. Live plugin linking
does: start `herdr server` in another terminal before `herdr plugin link .`.
The local milestone-2 acceptance run linked `opsydyn.webmaster` successfully
against Herdr `0.7.3` / protocol `16` and then stopped its temporary server.

## Privacy

Webmaster is local only. It has no telemetry, cloud sync, or network service.
The install script contacts GitHub only to download an explicitly requested
release artifact.

```text
best viewed with ratatui
80x24 minimum
this site is always under construction
```
