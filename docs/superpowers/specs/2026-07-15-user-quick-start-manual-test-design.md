# User quick start and manual test design

Date: 2026-07-15

## Goal

Rewrite the top-level `README.md` so a new user with a source checkout can
build, link, open, use, test, and remove `webmaster` without reading the
architecture or milestone history first.

The documented primary path must work with the repository as it exists today.
It must not advertise `herdr plugin install opsydyn/herdr-webmaster` until the
public repository and v0.1.0 release assets exist. The current supported path
is a local source link.

## Audience and assumptions

The primary reader:

- already has a source checkout of this repository;
- uses macOS or Linux;
- has Rust through `rustup`;
- has Herdr 0.7.3 or newer;
- may not know Herdr plugin terminology;
- wants a first successful session before contributor or architecture detail.

The walkthrough may explain how to start Herdr, but it must not assume that a
server owned by the user can be stopped or restarted by an automated test.

## README information architecture

The README will keep the product introduction, then lead with four user-facing
sections.

### 1. Quick start

Show one copyable source-install sequence:

1. confirm `herdr --version` is at least 0.7.3;
2. run `herdr status`;
3. when the server is not running, start `herdr server` in a separate terminal;
4. from the repository root, run `cargo build --release`;
5. link the checkout with `herdr plugin link .`;
6. verify `opsydyn.webmaster` with
   `herdr plugin list --plugin opsydyn.webmaster --json`;
7. open it with
   `herdr plugin action invoke opsydyn.webmaster.open`.

Each step will state the expected observable result. The text will explain that
`plugin link` is the development/source installation mechanism and that the
runner resolves the repository's release or debug binary.

### 2. First session

Describe the shortest useful workflow in user language:

- `1` / `F1` opens the webmaster desk;
- `2` / `F2` opens the cybercafe;
- arrow keys or `j` / `k` select an agent;
- `Enter` visits its real pane;
- `r` replies;
- `Space` marks the current attention episode seen;
- `o` refreshes only the selected output;
- `/` searches;
- `q` closes the TUI.

Explain the state language briefly: working agents build, blocked agents show
`HELP!`, completed work becomes an update, and exited panes become broken
links. Do not require synthetic events for ordinary use.

### 3. Optional manual smoke test

Provide a reproducible test that uses a dedicated, unowned plain pane, a
dedicated source, and an agent label. It will:

1. create or select an unowned plain pane explicitly (never an existing
   Codex/Claude pane and never webmaster's managed pane);
2. report a synthetic blocked agent;
3. confirm unread webmaster mail and a cafe `HELP!` pose appear without
   reopening;
4. exercise visit, reply, seen, search, refresh, and view switching;
5. close and reopen webmaster;
6. confirm view, persona, exact seen episode, and guestbook history restore;
7. return the synthetic state to working and then release the synthetic agent.

Herdr 0.7.3 cannot synthesize `done`, so the walkthrough will say that the
update-ready path requires a real agent completion event.

The test must not silently choose an arbitrary user pane. The operator creates
or selects a dedicated plain pane and captures its ID before calling
`report-agent`; reporting over an already-owned pane can be accepted by the
CLI while never appearing in the session snapshot. `HERDR_PANE_ID` identifies
webmaster's managed pane and must therefore not be used as the synthetic target.
If a webmaster pane already existed before the test, close/reopen requires
explicit operator permission; without permission, the persistence checkpoint
is reported as blocked rather than disturbing the existing pane.

### 4. Troubleshooting and removal

Cover only common first-run failures:

- server not running or incompatible Herdr version;
- `webmaster binary not found` after linking before building;
- stale `runtime.json` singleton registration;
- inspecting recent plugin logs;
- unlinking with `herdr plugin unlink opsydyn.webmaster`.

Unlinking removes the local registration, not the source checkout. Recovery
for `config.toml`, `state.json`, and `guestbook.jsonl` remains in the existing
persistence section.

## Maintainer content boundary

Contributor build commands, complete automated gates, architecture, and
historical milestone acceptance remain in the README below the user journey.
Historical statements must not be presented as instructions. The current
`Manual live acceptance` section will be refactored so its reusable user smoke
steps are not duplicated.

## Terminal Codex test handoff

The documentation work will include a ready-to-paste prompt for a terminal
Codex agent. The prompt authorizes testing, not implementation. The agent must:

- inspect `README.md`, `herdr-plugin.toml`, and the lifecycle scripts first;
- verify Herdr 0.7.3 compatibility and current server state;
- record whether the plugin was already linked and whether a webmaster pane
  already existed;
- build release, link only when necessary, verify registration, and invoke the
  plugin actions;
- use a unique synthetic source ID and explicitly selected pane;
- collect command output and ask the user for visual confirmations that cannot
  be proven from CLI state;
- exercise persistence through close/reopen only for a pane it opened, or
  after obtaining explicit permission for a pre-existing pane;
- release its synthetic agent;
- close or unlink only resources it created;
- never stop a Herdr server it did not start;
- make no repository edits;
- report pass, fail, or blocked for every checkpoint with exact evidence.

## Error handling and safety

The walkthrough will distinguish:

- a code failure from an unavailable Herdr server;
- an existing plugin registration from one created by the test;
- a pre-existing webmaster pane from one opened by the test;
- CLI-verifiable behavior from visual behavior requiring operator confirmation.

Cleanup is provenance-based. A test removes only the synthetic agent, pane,
plugin link, or server it created. It must leave pre-existing state intact.

## Acceptance criteria

The documentation change is complete when:

1. a new user can find the source install and open commands above the local
   development section;
2. every command matches Herdr 0.7.3 syntax and the repository scripts;
3. expected results are stated for build, link, registration, and open;
4. the first-session guide explains the primary keys and state language;
5. the optional smoke test includes live state, interaction, persistence, and
   cleanup checkpoints;
6. the guide never advertises unavailable release installation;
7. the terminal Codex prompt preserves pre-existing Herdr state;
8. maintainer verification remains available without dominating first use;
9. Markdown links, command blocks, and terminology pass a manual README review;
10. `cargo test --all-targets --all-features` and `bash tests/scripts.sh`
    remain green because documentation must not drift from the exercised
    command surface.
