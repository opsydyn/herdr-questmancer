# Questmancer agent handoff

This file is the durable starting point for coding agents working in this
repository. Read it before changing code, then inspect the current worktree and
the files relevant to the requested slice. Current source always wins over an
older plan or historical review.

## Product truth

Questmancer is a Herdr `0.7.4` plugin that turns coding-agent state into a cozy,
16-bit adventurers' guild. The user is the **Questmancer**. Herdr workspaces are
**campaigns** and agents are **adventurers**.

There are two views of the same live state:

- **Guild Hall**: the warm operational home of the whole party.
- **Delve**: the dungeon view of active work.

The production UI is a scene-first RGB pixel world rendered through Ratatui
half-block cells. Contextual parchment overlays provide selection details,
counsel, search and scrying. The persistent Guild Hall Librarian owns the fixed
help handbook through the Librarian's Ledger. Do not reintroduce a
text-dashboard view or a second production renderer.

Every visual joke must communicate a real fact, transition, urgency or action.
Use the approved Questmancer vocabulary in user-facing copy: adventurer,
campaign, counsel, summons, scrying and spoils. Do not revive the old
Webmaster/cybercafe vocabulary.

## Authority and ownership

Herdr owns live topology and agent facts. Questmancer owns presentation and a
small amount of durable local intent.

- Presence and attention are separate domains.
- Sprite identity, station, pose and effects derive from Herdr state. There are
  no manual sprite controls.
- The only text Questmancer sends to an agent is counsel explicitly composed by
  the user with `r`.
- Selected output is loaded lazily. Never fetch output on an animation frame.
- Rendering never mutates or persists domain truth.
- Persistence must not copy Herdr topology, terminal output or live status.
- The managed Questmancer pane must never appear as an adventurer or receive
  focus, output or counsel commands.

## Runtime architecture

```text
Herdr socket
  -> protocol/framing clients
  -> reconnecting supervisor + event adapter
  -> typed AppEvent
  -> pure domain reducer
  -> shared Model
       -> SceneSnapshot (live facts only)
       -> ScenePlan (world, camera, stations, effects, cadence)
       -> Guild Hall or Delve RGB renderer
       -> RgbBuffer
       -> Ratatui half-block adapter
       -> contextual overlays
  -> explicit Command effects
       -> focus, counsel, output, optional Reviewr
       -> debounced state.json + append-only chronicle.jsonl
```

Important boundaries:

- `src/domain/`: typed IDs, agents, campaigns, personas, presence, attention,
  Chronicle and durable state.
- `src/update/`: pure domain events and reducer.
- `src/herdr/`: protocol types, newline framing, request/subscription clients,
  reconnect supervisor and event adaptation.
- `src/runtime_loop.rs`: async orchestration. I/O belongs here or behind command
  handlers, not inside the reducer or widgets.
- `src/command.rs`: explicit side effects such as focus, counsel, output and
  optional Reviewr invocation.
- `src/persistence/`: atomic versioned state, Chronicle JSONL, startup overlay
  and debounced worker.
- `src/scene/snapshot.rs`: presentation-independent projection of model facts.
- `src/scene/stage.rs`: deterministic semantic stage projection.
- `src/scene/assets/`: authored world masters, palettes and animation frames.
- `src/scene/render/`: RGB Guild Hall, Delve, lighting and interaction paint.
- `src/ui/scene_adapter.rs`: converts two RGB pixel rows into one Ratatui cell.
- `src/ui/scene_overlays.rs`: identity labels and contextual parchment only.
- `src/portrait.rs`: optional native card portraits and capability-safe sprite
  fallbacks. Native portraits do not replace world sprites.
- `src/ledger.rs`: the small fixed handbook and stable page identifiers used by
  the Librarian's Ledger.
- `src/storybook/`: feature-gated, terminal-free fixed production stories.
- `herdr/`: install, run and singleton lifecycle shell scripts.

## Scene-renderer invariants

The RGB scene engine is the sole production renderer. Its inputs must remain
deterministic for a fixed snapshot, viewport and time.

- World sprites use authored masters at their native scale; never shrink them
  into illegible tokens.
- Animation is semantic and bounded. Static or no-motion scenes must not wake
  merely to redraw.
- Return hit regions only for complete, visible actors.
- Connection state changes lighting or diagnostic facts without destroying the
  authored room.
- Completion theatre is a one-shot transition, not a permanent animation.
- Unknown and exited states remain truthful; never infer a successful outcome.

The two rooms deliberately have different small-viewport contracts:

- The canonical Guild Hall is `160x90` RGB pixels. Smaller viewports recompose
  the room: a capacity-checked compact whole-party layout, then a single
  priority-adventurer vignette, then status-only rendering when a `16x24`
  adventurer cannot fit. A blocked adventurer has priority unless the user has
  an explicit selection. Do not restore camera cropping for the Guild Hall.
- The Delve retains an authored camera-crop model. Its station capacity and
  overflow behaviour are tested independently.

`pixtuoid-main/` is a local reference for density, responsive composition and
terminal rendering techniques. Treat it as inspiration, not production source
or an asset library. Do not copy its code or art into Questmancer without an
explicit licensing and design decision.

## Portrait rendering

Barbarian, Rogue, Wizard and Goblin cards have native PNG portraits. Goblin
ancestry takes precedence over class. The Librarian's Ledger has a separate
native illustration. All paths must retain a non-empty authored RGB sprite
fallback.

Native images inside a Herdr-managed pane require both a compatible terminal
and Herdr's experimental graphics bridge:

```toml
[experimental]
kitty_graphics = true
```

Changing or reloading a shared Herdr server requires its owner's approval.
Follow `docs/troubleshooting/native-portrait-rendering.md`; never interpret a
compatible outer terminal alone as proof that the pane transport supports
native images.

## Development workflow

Work inline on the current checkout and current branch unless the user asks for
isolation. The current collaboration preference is no worktrees and no
subagent-driven or Superpowers workflow unless explicitly requested.

Before editing:

```bash
git status --short --branch
git log -5 --oneline
```

The worktree may contain user or prior-agent changes. Preserve them. Never
reset, checkout, stash, clean or otherwise discard changes without explicit
permission.

For behaviour changes and defects, work test-first:

1. Add one focused failing test and prove the expected failure.
2. Implement the smallest coherent vertical slice.
3. Make the focused test green.
4. Run the affected integration/property tests.
5. Run the complete verification before claiming completion.

Prefer explicit typed state, pure projections and derived values over flexible
maps or duplicated stored state. No unsafe Rust. Keep docs and operational
recipes in the same slice as behaviour they describe.

## Verification

The normal full gate is:

```bash
just verify
```

It covers formatting, Clippy with warnings denied, all targets/features, shell
behaviour tests and script syntax. Useful focused commands are:

```bash
just guild-test
just delve-test
just protocol-test
just domain-test
just persistence-test
just property-test cases=4096
just storybook-test
cargo build --release
git diff --check
```

Do not claim manual, visual, live Herdr, persistence, release or cleanup
acceptance from automated tests. Record unreviewed items as unreviewed or
blocked.

## Storybook and visual review

Use the feature-gated Storybook to inspect production assets and render paths
without Herdr, agent processes or persistent state:

```bash
just storybook
```

It owns seventeen fixed production stories. `j`/`k` select stories, `Enter`
enters inspection mode, `Esc` returns and `q` exits. Resize Ghostty while viewing
`World / Guild Hall` to review canonical, compact and vignette compositions.

Visual approval is a product gate. Passing rendering tests proves invariants,
not art quality. Compare against `reference-art/questmancer-option-a-north-star.png`
and the direction in `docs/design/questmancer-sprite-art-direction.md`.

## Local Herdr workflow

Requirements are Herdr `0.7.4`, protocol `16` and Rust `1.90.0`.

For a source-linked checkout:

```bash
cargo build --release
herdr plugin link .
herdr plugin action invoke opsydyn.questmancer.open
```

Once Herdr is linked to this checkout, a new release build does **not** require
relinking. Close and reopen the Questmancer pane so the runner starts the new
binary. The five plugin actions are:

```text
opsydyn.questmancer.open
opsydyn.questmancer.close
opsydyn.questmancer.toggle
opsydyn.questmancer.guild
opsydyn.questmancer.delve
```

## Guarded live testing

Live pane IDs, tabs, server ownership, registration and environment variables
are run-specific. Re-baseline them every time. Historical IDs are never safe
to reuse.

- Never stop a Herdr server the test did not start.
- Never unlink a plugin link the test did not create.
- Never operate on a real, unknown, Codex or Questmancer pane.
- Create a disposable plain pane for synthetic reports.
- Record every resource created by the test and clean up only those resources.
- Release a synthetic agent identity before closing its pane.
- Restore the original focus when the pane still exists.
- Inspect plugin logs and final `git status` before claiming restoration.

Herdr `0.7.4` can synthesize `idle`, `working`, `blocked` and `unknown`. It
cannot synthesize an explicit `done` transition. Fixture coverage is the honest
automated proof for completion visuals; it is not live acceptance.

Use `docs/manual-test/questmancer-scene-preview.md` for the complete guarded
procedure.

## Persistence and privacy

Questmancer is local-only at runtime: no telemetry, cloud service or network
service beyond Herdr's local socket.

- `config.toml`: user-owned, read-only configuration.
- `runtime.json`: ephemeral singleton registration.
- `state.json`: atomic, versioned local intent and persona assignments.
- `chronicle.jsonl`: append-only semantic history.

State publication is debounced, unchanged states are suppressed and shutdown
flushes acknowledged work. Preserve forward/future-version failure behaviour
and malformed-file diagnostics.

## Current product status and remaining gates

The v0.1 engineering scope is feature-complete. Do not reopen old dashboard,
cybercafe or legacy-renderer milestones as unfinished work.

Remaining release work is evidence and distribution:

- repeat guarded Herdr acceptance from the eventual clean release commit;
- capture current Guild Hall and Delve release visuals;
- publish/tag the intended repository and verify four platform archives plus
  `SHA256SUMS` and the installer;
- optionally smoke Reviewr when `persiyanov.reviewr.open` is installed;
- retain real-agent resting/completion transitions as unverified until they are
  actually observed.

Post-v0.1 ideas, including Herdr sidebar rows, belong in the backlog unless the
user explicitly promotes them into the active slice.

## Source-of-truth order

When documents disagree, use this order:

1. current source and tests;
2. `AGENTS.md` and `README.md` for operating constraints;
3. `PLAN.md` for product status and release gates;
4. current design/troubleshooting/manual-test documents;
5. historical files under `docs/superpowers/` as decision history only.

Never treat old screenshots, old pane IDs, historical plan checkboxes or prior
agent claims as fresh evidence.
