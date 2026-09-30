# Questmancer agent handoff

This file is the durable starting point for coding agents working in this
repository. Read it before changing code, then inspect the current worktree and
the files relevant to the requested slice. Current source always wins over an
older plan or historical review.

## Product truth

Questmancer `0.1.10` is a Herdr `0.9.0` / protocol `22` plugin that turns
coding-agent state into a cozy, 16-bit adventurers' guild. The user is the **Questmancer**. Herdr workspaces are
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
  A separate versioned work cache may retain source-owned campaign/quest/obligation
  facts and local attention intent. Restore these as unverified; never cache live
  bindings or session identity.
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
       -> ScenePlan (world, camera, stations, effects)
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
- `src/sidebar.rs`: display-only Questmancer tokens for optional Herdr sidebar
  marginalia. It derives role, omen and campaign strings from live domain facts.
- `src/ledger.rs`: the small fixed handbook and stable page identifiers used by
  the Librarian's Ledger.
- `src/storybook/`: feature-gated, terminal-free fixed production stories.
- `herdr/`: install, run and singleton lifecycle shell scripts.

## Scene-renderer invariants

The RGB scene engine is the sole production renderer. Its inputs must remain
deterministic for a fixed snapshot, viewport and time.

- World sprites use authored masters at their native scale; never shrink them
  into illegible tokens.
- Animation is semantic and bounded. `SceneFrame.next_frame_in` owns render
  deadlines; authored frame counts and timing live with the assets. Static or
  no-motion scenes must not wake merely to redraw.
- Return hit regions only for complete, visible actors.
- Connection state changes lighting or diagnostic facts without destroying the
  authored room.
- Completion theatre is a one-shot transition, not a permanent animation.
- Unknown and exited states remain truthful; never infer a successful outcome.

The two rooms deliberately have different small-viewport contracts:

- The canonical Guild Hall is `160x90` RGB pixels with eleven party slots.
  It falls through capacity-checked compact `16x24` actors (minimum `64x40`),
  authored `8x12` roster actors (minimum `20x27`), a single `16x24` priority
  vignette, then status-only rendering. Selection wins vignette priority;
  otherwise a blocked adventurer wins. Do not restore Hall camera cropping.
- The Delve keeps its authored camera crop, but below `100` pixels wide or
  `56` pixels high it uses the same roster tier when the viewport is at least
  `20x27` and the entire party fits. Otherwise it keeps the crop and its
  independently tested station/overflow contract; it has no Hall vignette.
- Both rosters retain static tool, lantern, Z, check and question-mark state
  cues when names cannot fit. Full-motion fresh spoils end at three seconds;
  reduced/still rosters require no decorative or cleanup wake.
- The Librarian is visible and clickable in canonical and compact Halls. The
  roster, single-adventurer vignette and status-only tiers omit that actor;
  `?` still opens the Ledger. The revised Librarian has a stocky `16x24`
  world sprite and an independently authored `24x32` Ledger fallback. Its
  production visual review remains separate from class-art approval.

## Sidebar marginalia

Herdr owns the sidebar and the user's global sidebar configuration. Questmancer
reports eleven namespaced tokens through Herdr metadata:

- agent display: `$quest_sigil`, `$quest_role`, `$quest_epithet`,
  `$quest_condition`, `$quest_omen`, `$quest_trinket`, `$quest_vigil`,
  `$quest_hoard`;
- campaign display: `$quest_campaign`, `$quest_party`, `$quest_hoard`;
- internal ordering: `$quest_rank`, derived urgency rather than display copy.

With the user-enabled `sidebar_urgency_order = true` setting (default false),
Questmancer also requests Herdr's transient `agent.view.set` sort. It never
filters agents; it reissues the sort after reconnect and attempts to clear its
own view on shutdown. Metadata display and opt-in ordering are separate
capabilities. Neither changes the user's global sidebar configuration.

The plugin uses source `plugin:opsydyn.questmancer`, never writes `title`,
`display_agent`, `state_labels`, focus or task data, and never edits
`ui.sidebar.*.rows`. Metadata refresh events are inert so reports cannot create
a refresh loop. Reports are reissued after a socket reconnect and otherwise
only when the derived projection changes.

`pixtuoid-main/` is a local reference for density, responsive composition and
terminal rendering techniques. Treat it as inspiration, not production source
or an asset library. Do not copy its code or art into Questmancer without an
explicit licensing and design decision.

## Portrait rendering

All fourteen classes have distinct world masters, portrait fallbacks and
native PNG cards: Artificer, Barbarian, Bard, Cleric, Druid, Mage, Paladin,
Pathseeker, Ranger, Rogue, Runewright, Sorcerer, Testmender and Wizard. Cards
are selected by class. Every class fallback reuses its current personalised
`16x24` world sprite, centred without scaling inside the existing `24x32`
canvas. All card alignments are visually approved; Mage and Sorcerer completed
the set on 2026-09-08 under
`docs/design/reviews/2026-09-08-mage-sorcerer-card-fallbacks/`.
Goblin and Orc art is reserved for future event/NPC storytelling and must not replace an
ordinary adventurer's class portrait. The
Librarian's Ledger has a separate native illustration. All paths must retain a
non-empty authored RGB sprite fallback.

Native images inside a Herdr-managed pane require both a compatible terminal
and Herdr's graphics bridge (enabled by default in 0.9):

```toml
[terminal]
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
just property-test 4096
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

It owns fifty-eight fixed production stories: two worlds, thirty-seven asset
views and nineteen interactions. `j`/`k` and Up/Down select within the current
category; `h`/`l` and Left/Right change category. `Enter` enters inspection,
`Esc` returns and `q` exits. Resize Ghostty while viewing `World / Guild Hall`
to review canonical, compact, roster, vignette and status-only compositions;
review the Delve crop and roster independently. In inspection mode, world,
card and interaction stories reach every positive production size. Sprite
galleries retain their `80x28` minimum. The 2026-09-06 PTY resize receipt does
not establish native visual acceptance; Computer Use currently refuses
Ghostty access for safety reasons.

Visual approval is a product gate. Passing rendering tests proves invariants,
not art quality. Compare against `reference-art/questmancer-option-a-north-star.png`
and the direction in `docs/design/questmancer-sprite-art-direction.md`.

## Local Herdr workflow

Requirements are Herdr `0.9.0`, protocol `22` and Rust `1.90.0`.

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

Herdr `0.9.0` can synthesize `idle`, `working`, `blocked` and `unknown`. It
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
- `work-state.json`: versioned source work facts, revision tombstones and local
  seen/deferred intent. Atomic acknowledged publication; no Herdr topology,
  terminal output, live binding or raw session identity.

State publication is debounced, unchanged states are suppressed and shutdown
flushes acknowledged work. Preserve forward/future-version failure behaviour
and malformed-file diagnostics.

## Current product status and remaining gates

The v0.1 engineering scope and the approved party-delight sequence are complete.
Do not reopen dashboard, cybercafe or legacy-renderer milestones. Work inline;
new product ideas remain in the backlog until the user promotes them.

All current review-pack visuals are approved: Librarian, Hall/Delve, campaign
heraldry, keepsakes, cat reaction, Chronicle chapters, and rituals plus matching
card fallbacks for all fourteen classes. The last card approval was 2026-09-08.
See `docs/reviews/2026-09-08-party-candidate/README.md` for the approval index,
review findings and current candidate evidence. Storybook has 58 stories:
two worlds, 37 asset views and nineteen interactions.

Questmancer `v0.1.9` is published from clean commit `98f557d`. Qualification
passed 600 Rust tests across 54 runs, 28 shell tests, the release build, verified
clean-source packaging and fresh isolated Herdr runtime/cleanup. The GitHub
workflow passed all four builds; downloaded archives match `SHA256SUMS`, and
the actual Herdr installer passed in temporary storage on macOS ARM64. Review
packs remain in Git and are excluded from the crate's existing 10 MiB limit.
See `docs/reviews/2026-09-08-published-release/README.md` and its receipt.

The user approved Hall/Delve/sidebar appearance and confirmed restored native
Artificer, Bard and Librarian illustrations. Exhaustive resize/native-card
coverage, execution of other platform binaries on native hosts, optional Reviewr
and real-agent resting/completion remain unverified. The crates.io job was
skipped; registry publication remains separate. The 0.1.9 phase is complete.
On 2026-09-09 the user approved C1 from the Chronicle capture plan. Snapshot
refreshes now carry an ephemeral connection epoch, request ID and fact generation;
only current responses with unchanged subscription membership may replace facts.
Requests coalesce to one active plus one pending; two superseded responses or a
changed subscription pane set request a quiet baseline through our own supervisor.
Snapshot cancellation is separate from counsel/focus and selected-output tasks.
C2 was subsequently approved and implemented. Qualified refreshes now append
zero-XP observations; startup/reconnect/resync baselines remain quiet. Complete
session/terminal evidence qualifies subject history; fallback/ambiguous identities
remain quiet. Workspace-close hints require snapshot confirmation, and campaign
removal never means successful delivery. New unknown status has an explicit
whereabouts category. New writes use v2 envelopes with immutable source identity;
mixed v1/v2 replay preserves legacy records and never re-awards standing. Raw
session metadata stays ephemeral. Terminal replacement also invalidates scrying
and same-party cat comparisons. See
`docs/reviews/2026-09-09-chronicle-c1/README.md`,
`docs/reviews/2026-09-09-chronicle-c2/README.md`,
`docs/design/questmancer-chronicle-capture.md` and
`docs/plans/2026-09-09-chronicle-capture.md`. C3 was approved and qualified with
four composed socket/runtime/persistence tests and an isolated Herdr server with
four owned dummy Pi processes. Five v2 metadata observations persisted; repeats
and restart added no history. The full gate passes 656 Rust tests / 56 runs and
28 shell tests. See `docs/reviews/2026-09-09-chronicle-c3/README.md`. The changed
Chronicle screens were visually approved by the user on 2026-09-09. C1–C3
are published in `v0.1.10` from clean `9f2f58f`. Release qualification passed
657 Rust tests / 57 runs, 28 shell tests, source packaging and isolated runtime
cleanup. All four downloaded archives, checksums and the actual macOS ARM64
Herdr installer passed. Documentation and approved Chronicle images are deployed.
See `docs/reviews/2026-09-09-published-release/README.md`. Real-agent completion
and live snapshot-only capture remained separate at publication; crates.io was skipped.
The user subsequently approved guarded real-agent acceptance. A fresh isolated Pi
run confirmed working/done pane-metadata records, one spoil, 10 XP, unchanged
history/XP after restart, and complete owned-resource cleanup. See
`docs/reviews/2026-09-09-real-agent-chronicle/README.md`. Live snapshot-only
capture, other agent integrations and native completion visuals remain unverified.

## Native portrait regression repair — 2026-09-08

The Questmancer accepted the sidebar, Hall and Delve screenshots, but reported
native card and Librarian art regressing to fallbacks. Herdr 0.9 managed-plugin
panes acknowledged Kitty while omitting CSI 16t replies and PTY pixel dimensions.
Plain panes did not reproduce the failure. Startup now fills missing PTY pixel
axes from the own-pane `pane.graphics.info` response before the existing Picker
query. It preserves the character grid/known axes, rejects zero/overflow and
bounds the request to 500 ms. It never forces a protocol or adds frame requests.

The repair passes 600 Rust tests across 54 runs, 28 shell tests and the release
build. A fresh managed-plugin probe prepares all fourteen native cards and the
Librarian. The user confirmed restored Artificer, Bard and Librarian illustrations
in Ghostty after reopening the repaired binary; consult
`docs/reviews/2026-09-08-native-portrait-regression/README.md` and its receipt for
that result. The repair was subsequently committed, clean-qualified and published
at `98f557d` as `v0.1.9`; consult the publication receipt for final evidence.

## Current presentation contracts — 2026-09-08

The four-row sidebar is applied. Campaign heraldry uses native-scale crests
below canonical Hall campaign-table actors and the same named identity on cards.
It derives only from workspace IDs, adds no durable state or wakeups, and omits
table crest sets when all cannot fit. Smaller Hall tiers omit them.

Six existing saved keepsake assignments select static 8x8 card illustrations
and fixed copy. Detailed cards reserve their lower text section for these;
compact cards retain name and description without artwork. Persona generation,
persistence and the card footprint are unchanged.

The canonical Hall cat performs one 800 ms head-lift/stretch after the same
known non-empty party becomes entirely resting. It has no persistent state or
extra wakes in reduced/still motion. Reconnects and membership changes reset
the baseline.

Tab inside the Chronicle requests a last-hour recap with a fixed UTC window,
retained-event counts and timestamped sources; Tab returns to records. Chapters
add no persistence or agent commands. Qualified refreshes now append explicit
observations; connection baselines remain quiet. The old joined event also
represented unknown whereabouts, so legacy chapters still call it an identity
event. New whereabouts, presence, membership and campaign-removal categories are
counted separately. Preserve these evidence limits.

All fourteen classes use authored rituals with existing persona substitution:
two 500 ms working frames, one 600 ms counsel gesture, spoils placed at 1000 ms
and quiet by three seconds. Confirmed counsel seals the existing notice without
changing Herdr presence. Every class card centres its personalised static world
master in the existing 24x32 canvas. Roster families and native PNGs remain in use.

## Source-of-truth order

When documents disagree, use this order:

1. current source and tests;
2. `AGENTS.md` and `README.md` for operating constraints;
3. `PLAN.md` for product status and release gates;
4. current design/troubleshooting/manual-test documents;
5. historical files under `docs/superpowers/` as decision history only.

Never treat old screenshots, old pane IDs, historical plan checkboxes or prior
agent claims as fresh evidence.

## M1 durable obligations — 2026-09-30 (unreleased)

The user approved the ownership contract and Quest Board concept. `src/work/`
adds source-qualified work beside the live Herdr domain. Sources own work and
resolution; Questmancer owns only presentation and local seen/deferred intent.
The legacy Chronicle remains observation history, not an authoritative work log.

`b` opens the Quest Board parchment; `!` cycles human obligations then existing
eligible summons. In the Board, `j/k` or Tab/Shift-Tab select work, Up/Down or
wheel scroll details, Space marks seen, `s` defers 15 minutes, Enter observes only
a verified live binding, and Esc closes. Seen/deferred never means source-resolved.

Ordinary startup does not install FixtureSource. Tests and seven Storybook states
exercise the boundary. Cached work is unverified on restart; source gaps/conflicts
retain prior facts and require a baseline. Complete coverage records revision
retirement so stale entities cannot return. Source updates and local intent are
shown as durable only after the persistence worker acknowledges publication.

See `docs/reviews/2026-09-30-durable-obligations/README.md` for exact qualification
and the recorded 2026-09-30 production visual approval after the truncation repair.
M1 is complete for its fixture-backed scope and unreleased. Native terminal
acceptance remains separate. M2 was subsequently promoted on 2026-09-30.
Adapters, commits, releases and shared-server changes require their own promotion. Preserve the unrelated pane-header repair.


## M2 War Table — 2026-09-30 (unreleased)

The user promoted the fixture-backed War Table. `src/work/graph.rs` projects
source-qualified campaign dependency facts separately from the live SceneSnapshot.
The static RGB map is composed by `src/scene/war_table.rs`, painted by
`src/scene/render/war_table.rs` inside the existing world path, and labelled by
`src/ui/war_table.rs`. It is a contextual surface, not a third world renderer.

`w` opens/closes the Table from the Hall, Delve or Board. `j/k` cycles quests;
Tab/Shift-Tab cycles work campaigns; Up/Down or wheel scrolls selected details;
`b` returns the selected quest to the Board; Enter observes only a verified
binding; `!` reuses M1's queue. Table navigation is ephemeral and emits no
persistence or source answer. Counsel drafts remain preserved.

Links run prerequisite to dependent. Cards retain literal source lifecycle and
historical assignment; `?` counts source-open human requests and `R` counts
human review requests, including locally deferred ones. They are source facts,
not a second alert policy. Source freshness remains explicit. Cyclic or oversized
campaigns use a labelled paged inventory; narrow panes retain scrollable details
and all keyboard access. Below 28 columns or 12 rows the overlay is status-only.
No readiness, success or task assignment is inferred from graph placement.

Six new fixed Table stories bring Storybook to 58 stories. See
`docs/plans/2026-09-30-war-table.md` and `docs/reviews/2026-09-30-war-table/README.md`
for the bounded scope and evidence. M2 is engineering-qualified and its production
visuals were approved by the user on 2026-09-30. It is complete and unreleased
within its fixture-backed scope. Native terminal and live-source acceptance remain
separate and unperformed.
M3 source inspection and planning were subsequently approved on 2026-09-30;
M3a.1 was subsequently promoted with “Let’s continue”. M3a.2 mapper/cache was
then promoted and implemented as recorded below. Runtime was subsequently promoted;
later roadmap phases remain unpromoted.

## M3 source assessment and plan — 2026-09-30

Read `docs/reviews/2026-09-30-herdr-projects-source/README.md` and
`docs/plans/2026-09-30-herdr-projects-adapter.md` before adapter work. Herdr Projects
0.2.34 was inspected at `4e4548c3c43888e6be1c96906215998f07dc63f0`; source was
fetched and read, not built/run/installed. It requires Herdr 0.9.1. No real project
home, shared server or live pane was operated for this assessment.

Its JSON thread rows are not a coherent versioned project/decision protocol.
Placement kinds, recorded attention groups and resolved thread status cannot
be coerced into quest kinds, durable human decisions or successful completion.
The proposal is M3a.1 snapshot/exporter contract proof, then explicit opt-in
inventory integration; structured requests and verified bindings are separate
M3b contracts. The user subsequently promoted M3a.1 with “Let’s continue”.
`src/sources/herdr_projects/` implements its experimental read-only snapshot DTO,
strict decoder and bounded exporter. All coverage is Partial; native durable
identity, dependencies, structured obligations and bindings are unsupported.
See `docs/design/herdr-projects-snapshot-contract.md` and
`docs/reviews/2026-09-30-herdr-projects-exporter/README.md`. No runtime attachment,
work mapping, cache migration or source commands were added. Qualification passed
722 Rust tests across 61 runs, 28 shell tests, the 4,096-case property gate and
release build. M3a.1 is complete for its synthetic proof and unreleased.

The user subsequently promoted the M3a.2 identity/coverage design slice.
Read `docs/design/herdr-projects-identity-coverage.md` before mapping work. It
selects explicit registration plus retained records; creation is checkpoint
evidence, not a native incarnation. Changed evidence requires reconciliation,
not a guessed new quest. All observations remain Partial; durable mapping needs
the existing cooperative lock and must reject known-thread omissions before
the reducer marks retained facts Current. Never write a source lock or sidecar.
Projection plus typed registration/creation checkpoint must publish atomically
in a proposed v2 work cache. No mapper, migration or runtime code was added by
this documentation-only slice; its design receipt is
`docs/reviews/2026-09-30-herdr-projects-identity/README.md`. The adapter plan now
specified that next code slice. The user subsequently promoted mapper/cache
implementation. It now provides pure registered mapping, neutral
Unspecified/Unknown/Closed/Archived values, latest-reference revocation and an
atomic v2 projection/checkpoint cache with v1 migration. Ordinary intent/fixture
writes preserve checkpoints; obsolete/duplicate proposals cannot downgrade newer
facts. The full gate passes 739 Rust tests across 61 runs and 28 shell tests;
the 4,096-case property gate passes. See
`docs/reviews/2026-09-30-herdr-projects-mapper/README.md` for final evidence and
production review images. No source attachment/configuration/polling is installed.
Identity failures are classified for explicit rebind; the M3a.3 producer must latch
them and enforce root leases/cancellation. Source labels are attributed context,
not invented worker identity. Native identity stays Unsupported and coverage
Partial. The user subsequently approved the M3a.2 production visuals and promoted
M3a.3 opt-in runtime integration. See
`docs/plans/2026-09-30-herdr-projects-runtime.md`,
`docs/manual-test/herdr-projects-inventory.md` and
`docs/reviews/2026-09-30-herdr-projects-runtime/README.md`.

The disabled-by-default `[herdr_projects]` table requires explicit tokens, an
absolute physical path, pinned format and retained-record policy. Startup loads
the atomic checkpoint; a protected cache or missing durable directory disables
attachment. RuntimeLifecycle owns the independent producer/inbox in both live
and offline loops, and stops/joins it before persistence flush. SourceInbox has
one acknowledged observation outstanding; the consumer maps against current work
and advances its checkpoint only after publication. A held read-only descriptor
lease survives failed reads and is checked at capture and before consumption.
Observed root loss/replacement or hard identity failure latches reconciliation;
transient read/coverage/storage failures retain work and retry. All completions
are generation-fenced before changing newer health. Healthy polls are five seconds
apart; retries back off to thirty seconds. The two-second capture budget and
cancellation are cooperative between bounded reads, not a hard deadline on a
stalled OS filesystem call. Unchanged probes do not write or redraw. One ephemeral
source diagnostic remains visible even while another cached campaign is selected.
No source files, bindings, summons, execution commands, Chronicle or XP are added.
The full gate passes 754 Rust tests across 62 runs and 28 shell tests; the
4,096-case property gate and release build pass. The runtime source suite has
12 all-feature tests, plus the existing exporter/mapper suite.
The user subsequently approved the diagnostic images; see
`docs/reviews/2026-09-30-herdr-projects-runtime/visual-approval.md`. M3a.3 is
complete within its synthetic inventory scope and remains unreleased.
`docs/plans/2026-09-30-herdr-projects-acceptance.md` proposes the next bounded
installed-source/isolated-runtime trial; that operational scope has not been
promoted or executed. Native/live acceptance, M3b, upstream contributions and
M4–M7 remain separate or unpromoted.
