# Questmancer plan

Questmancer turns a Herdr session into an adventurers' guild. The user is the
Questmancer, workspaces are campaigns, agents are adventurers, blocked work
raises summons, and explicit completion returns spoils. Herdr owns live facts;
Questmancer owns presentation and a small amount of durable local intent.

## Current release — 0.2.0, 2026-11-17

Version 0.2.0 is published from clean `11ddd02`, with all four archives, checksums,
the actual macOS ARM64 installer and deployed docs verified. It includes the Quest Board/War Table, explicit local owned-work
authoring, Herdr 0.9.4+ compatibility, revised class/Librarian art, proportional
Hall/Delve fitting, profile masking and automatic party density through 64 actors.
Pi integration v9 is installed and real working/resting capture is qualified with
Herdr 0.9.5 / Pi 1.5.2; existing sessions need manual reload. Storybook has 84 states.

The release checklist and fresh evidence live in
[the 0.2.0 receipt](docs/reviews/2026-11-17-release-0.2.0/README.md).
Historical slice sections below describe their state at the recorded date;
their “unreleased” labels are superseded by this published release scope.

Remaining product work: native owned-authoring acceptance; exhaustive native
resize/profile/mouse/animation review; blocked Pi and other integration coverage;
stale integration-banner clearing; explicit lifecycle/delegation design. Herdr
Projects installed-source qualification remains parked. Other platform binaries
require their own native execution checks. Registry publication remains separate.

## Core UX sequence — 2026-11-18

The [core UX plan](docs/plans/2026-11-18-core-ux.md) records the ordered
durability, party identity, status, Herdr and world-art work after 0.2.0.
The [room studies](docs/reviews/2026-11-18-core-ux/README.md) carry the visual
comparison and review prompts. Keep their Storybook evidence separate from
native acceptance.

## Historical baseline — 2026-09-09

Source baseline: Questmancer `0.1.10`, minimum Herdr `0.9.4`, supported protocol
`22`, Rust `1.90.0`. The original v0.1 engineering scope is complete; do not
reopen the retired dashboard, legacy renderer or old branding milestones.
The checkout contains approved follow-on work and existing changes that must
be preserved. Work inline unless the user requests isolation.

[A party worth knowing](docs/plans/2026-09-05-party-delight.md) owns the current
creative and correctness sequence:

| Slice | Status |
| --- | --- |
| Three-class storyboard and long-torso correction | Visually approved 2026-09-05; direction implemented in the production pilot |
| Scrying result ordering (2a) | Implemented and verified; stale reads/failures cannot replace current output |
| Roster state cues and bounded completion (2b) | Implemented, verified and visually approved 2026-09-05 |
| Operating-guidance reconciliation (2c) | Complete 2026-09-05; current source and dated publication checks are documented |
| Librarian proportion refresh | Implemented; consolidated visual review approved 2026-09-08 |
| Three-class production pilot | Implemented; consolidated visual review approved 2026-09-08 |
| Pilot card fallback continuity | Implemented, verified and visually approved 2026-09-05; cards reuse the new personalised sprites at native size |
| Terminal resize pass | Storybook correction and 70 passing executable PTY resize checks approved 2026-09-06; native Ghostty visual review blocked by Computer Use |
| Herdr 0.8.2 release preflight (2026-09-06) | Historical isolated compatibility, package and published 0.1.3 archive checks; release dispatch repair remains local |
| Herdr 0.9 upgrade | Installed 0.9.0 / protocol 22; subscription baseline and status reconciliation updated; isolated production-client checks passed; conditional sidebar recipe proposed |
| Campaign heraldry | Implemented and visually approved 2026-09-08 |
| Keepsakes, cat reaction and Chronicle chapters | Implemented, verified and visually approved 2026-09-08 |
| All fourteen class rituals and matching card fallbacks | Implemented, verified and visually approved 2026-09-08 |
| Questmancer 0.1.9 release | Published from clean `98f557d`; four archives, checksums and actual installer verified |

The [complete candidate review](docs/reviews/2026-09-08-party-candidate/README.md)
indexes every current approval and records this expanded candidate's checks.
The final clean release commit `98f557d` passed **600 Rust tests across 54 runs,
28 shell tests, formatting, Clippy with warnings denied and script syntax**,
the release build, clean source-package build and isolated Herdr qualification.
All current review-pack art is approved. The user also accepted current
Hall/Delve/sidebar appearance and restored native Artificer, Bard and Librarian
illustrations. See the [published release record](docs/reviews/2026-09-08-published-release/README.md)
for separate workflow, archive and actual installer evidence.

## Implemented architecture and capabilities

```text
Herdr snapshot + events
  -> protocol clients + reconnecting supervisor
  -> typed events + pure reducer
  -> shared Model
       -> SceneSnapshot (live facts)
       -> ScenePlan + presentation intent
       -> Guild Hall or Delve RGB painter
       -> half-block adapter + contextual overlays
  -> explicit focus, counsel, output and optional Reviewr commands
  -> debounced state.json + append-only chronicle.jsonl
```

- One RGB production renderer with contextual selection, counsel, search,
  scrying, Chronicle and Librarian's Ledger overlays. There is no second
  dashboard or scene-preview binary.
- Protocol 22 request/subscription clients, bounded reconnect, fresh snapshots,
  managed-pane exclusion, and truthful unknown/exited handling.
- Typed presence and attention, stable persona generations, campaigns,
  bounded Chronicle history, summons acknowledgement/snooze and urgency jumps.
- Correlated counsel text/submit outcomes, per-adventurer drafts, bounded
  selected-output reads and independent counsel work. No output request or
  persistence write originates from an animation wake.
- Fourteen classes with distinct `16x24` world masters, `24x32` portrait
  fallbacks and native class cards; five `8x12` roster families plus dedicated
  pilot roster masters. Wizard, Ranger, Barbarian, Bard, Artificer and
  Testmender, Cleric, Paladin, Druid, Rogue, Pathseeker, Runewright, Mage and
  Sorcerer have
  authored working, counsel and spoils sequences with shared render deadlines.
  All fourteen classes' card
  fallbacks now centre the same personalised world sprite in the existing
  `24x32` canvas without stretching.
  Native graphics are confined to cards and the Ledger illustration.
- Original Hall/dungeon architecture, material lighting, selected floor rings,
  semantic completion effects and a bounded hidden goblin interaction.
- Eleven sidebar tokens, optional Herdr-owned urgency sorting, and local guild
  standing awarded only for recorded spoils/campaign closure.
- Atomic versioned user intent, debounced persistence, acknowledged shutdown
  flushes, and terminal restoration after normal exit, error, signal or panic.
- Forty-five Storybook stories through production paths: two worlds,
  thirty-seven asset views and six interactions. Current `j`/`k` navigation
  remains category-local; `h`/`l` changes category.

The Librarian is an independent help NPC. Canonical and compact Halls reserve
its complete clickable sprite; smaller tiers retain keyboard access to the
Ledger. The [Librarian refresh](docs/design/reviews/2026-09-05-librarian/README.md)
adds a stocky world sprite and independent Ledger fallback; its consolidated
visual review was approved on 2026-09-08.

## Responsive contracts

All sizes below are RGB pixels; two pixel rows occupy one terminal row.

| Room | Current selection of layout |
| --- | --- |
| Guild Hall | Canonical at least `160x90` with at most eleven adventurers; otherwise capacity-checked compact at least `64x40`, then authored roster at least `20x27`, then a single `16x24` vignette, then status-only |
| Delve | Authored camera crop; below width `100` or height `56`, use roster when at least `20x27` and the whole party fits; otherwise retain crop and independent station overflow |

The Hall's vignette prioritises explicit selection, then blocked presence.
Neither room shrinks a world master to make it fit. Roster states use shared
shape cues; full-motion fresh spoils stop at three seconds and leave a stable
completed cue. Reduced/still rosters have no decorative or cleanup timer.
Newer facts and socket boundaries interrupt old completion theatre.

## 0.1.10 published — 2026-09-09

Chronicle C1–C3 and the approved screen pack are published in
[v0.1.10](https://github.com/opsydyn/herdr-questmancer/releases/tag/v0.1.10) from
clean `9f2f58f`. README, launch-page gallery, handbook and release notes are current.
Qualification passed 657 Rust tests / 57 runs, 28 shell tests, verified source
packaging and fresh isolated Herdr runtime/cleanup. All four release archives,
checksums and the actual macOS ARM64 installer passed. The website is deployed;
crates.io was skipped. See the
[publication receipt](docs/reviews/2026-09-09-published-release/README.md).

## Real-agent Chronicle acceptance — 2026-09-09

The approved isolated Pi run passed: genuine working/done metadata produced two
v2 records, one spoil and 10 XP; restart preserved history and XP. Owned-resource
cleanup passed. See the [receipt](docs/reviews/2026-09-09-real-agent-chronicle/README.md).
Live snapshot-only capture and native completion visuals remain separate.
The next recommendation is a bounded design for live snapshot-only acceptance.

## Earlier 0.1.9 distribution and remaining evidence

[Questmancer v0.1.9](https://github.com/opsydyn/herdr-questmancer/releases/tag/v0.1.9)
is published from clean commit `98f557d`. Both manifests match the tag. The
release workflow passed all gates and four target builds; the downloaded
macOS/Linux Intel/ARM archives each contain one root-level executable and match
`SHA256SUMS`. The actual Herdr installer passed in temporary storage and its
macOS ARM64 binary reports `questmancer 0.1.9`. See the
[publication receipt](docs/reviews/2026-09-08-published-release/README.md).
This closes the historical manifest/archive distribution gap.

Remaining evidence is bounded: execution of the other three binaries on their
native hosts, exhaustive terminal resize/native-card coverage, optional Reviewr,
and real-agent resting/completion. Synthetic Herdr reports cannot express done.
The crates.io job was skipped; registry setup/publication remains separate.

## Native acceptance regression — 2026-09-08

The user accepted current sidebar, Hall and Delve appearance in Ghostty, while
identifying a regression in native adventurer/Librarian artwork. The
[repair record](docs/reviews/2026-09-08-native-portrait-regression/README.md)
reproduces missing pixel geometry specifically in managed-plugin panes. The
implemented startup repair passes 600 Rust tests across 54 runs, 28 shell tests,
the release build and native preparation in a managed plugin. It has been
loaded and the user confirmed restored native Artificer, Bard and Librarian
illustrations in Ghostty. The record tracks that result separately.
The earlier 593-test candidate preparation remains historical above.

The repair was clean-qualified and published as `v0.1.9` at `98f557d`. No further
class-art or feature expansion is needed to resolve this regression.

## Approved sequence and remaining work — 2026-09-08

The consolidated review was approved and the earlier candidate was qualified
at clean commit `c3720a9`. Subsequent approvals covered keepsakes, the cat
reaction, Chronicle chapters and all remaining class rituals and card fallbacks.
The final Mage/Sorcerer card review was approved on 2026-09-08. This completes
the promoted party-delight sequence. The
[candidate report](docs/reviews/2026-09-08-party-candidate/README.md) contains the
approval links, current verification and packaging evidence, and bounded audit.
The Questmancer authorised the complete local candidate commit and roadmap
assessment on 2026-09-08. Preparation evidence is retained as a dated receipt;
clean-commit qualification is recorded separately after the commit. See the
[current roadmap assessment](docs/reviews/2026-09-08-roadmap-assessment.md).

The 0.1.9 phase is complete and published. On 2026-09-09 the Questmancer approved
C1 in the [Chronicle capture plan](docs/plans/2026-09-09-chronicle-capture.md).
Snapshot freshness, correlation, coalescing and subscription-baseline qualification
are implemented locally; [the receipt](docs/reviews/2026-09-09-chronicle-c1/README.md)
records verification and limits. The user subsequently approved C2: typed
observations, incarnation qualification, compatible v1/v2 replay and readable
Chronicle copy are implemented locally; see the
[C2 receipt](docs/reviews/2026-09-09-chronicle-c2/README.md). Snapshot observations
and campaign removal earn zero XP; connection baselines remain quiet.
C3 was approved; complete capture composition and isolated synthetic acceptance
passed. See the [C3 receipt](docs/reviews/2026-09-09-chronicle-c3/README.md).
The [Chronicle screens](docs/design/reviews/2026-09-09-chronicle-capture/README.md)
were visually approved on 2026-09-09. C1–C3 are complete and published in 0.1.10.
The subsequent approved real-agent Pi acceptance is recorded above. Live
snapshot-only capture remains separate evidence; no new product scope is promoted.

## Living control plane planning — 2026-09-27

The user agreed with the architecture assessment and promoted durable human
obligations and campaign work context to roadmap/design planning. On 2026-09-30
the user approved the roadmap, architecture contract and first-slice plan.
The user subsequently approved the
[Quest Board concept](docs/design/reviews/2026-09-30-quest-board/README.md),
authorising M1 implementation. The fixture-backed slice is implemented inline,
engineering-qualified and production-visually approved on 2026-09-30;
[qualification and production visual review](docs/reviews/2026-09-30-durable-obligations/README.md)
remain distinct gates. The
[design contract](docs/design/questmancer-living-control-plane.md),
[phased roadmap](docs/plans/2026-09-27-living-control-plane-roadmap.md) and
[M1 implementation plan](docs/plans/2026-09-27-durable-obligations.md) define the
proposed extension beside the existing live Herdr domain.

M1 proves a fixture-backed obligation survives progress, agent exit and restart,
with a derived attention queue and selected-work parchment. War Table, live
adapters, Forge evidence, source knowledge and historical reconstruction are
separate later phases. M1 is complete and unreleased; native terminal acceptance
remains separate. The user subsequently promoted M2 War Table on 2026-09-30.
M2 is engineering-qualified and production-visually approved on the same date;
it is complete and unreleased within its fixture-backed scope. Native terminal
and live-source acceptance remain separate and unperformed. Evidence is tracked in the
[M2 plan](docs/plans/2026-09-30-war-table.md) and
[receipt](docs/reviews/2026-09-30-war-table/README.md). M3 source inspection and
planning were subsequently approved on 2026-09-30. The
[source assessment](docs/reviews/2026-09-30-herdr-projects-source/README.md) and
[adapter plan](docs/plans/2026-09-30-herdr-projects-adapter.md) propose a versioned
snapshot/exporter proof before a reduced-capability inventory adapter. M3
source-contract proof M3a.1 was subsequently authorised with “Let’s continue” and
is engineering-qualified and complete for its synthetic contract-proof scope,
unreleased; [receipt](docs/reviews/2026-09-30-herdr-projects-exporter/README.md).
The user subsequently approved the identity/coverage design slice. The
[selected contract](docs/design/herdr-projects-identity-coverage.md) uses explicit
registration/retained records, locked partial observations and omission guards;
native identity stays unsupported. This documentation slice is complete;
the user subsequently promoted mapper/cache implementation. It is implemented
and engineering-qualified with 739 Rust tests/61 runs, 28 shell tests and the
4,096-case property gate. The
[mapper receipt](docs/reviews/2026-09-30-herdr-projects-mapper/README.md) records
evidence; its state-cue images were subsequently approved. M3a.3 runtime integration
was then promoted and implemented: disabled-by-default selection, owned directory
lease, acknowledged producer/consumer, bounded polling/retries and cooperative
shutdown. Protected caches disable attachment; identity failures latch, transient
failures retain work, and unchanged probes neither write nor redraw. See the
[runtime plan](docs/plans/2026-09-30-herdr-projects-runtime.md),
[operating guide](docs/manual-test/herdr-projects-inventory.md) and
[runtime receipt](docs/reviews/2026-09-30-herdr-projects-runtime/README.md).
Engineering qualification passes 754 Rust tests/62 runs, 28 shell tests, the
4,096-case property gate and release build.
The user subsequently [approved the diagnostic images](docs/reviews/2026-09-30-herdr-projects-runtime/visual-approval.md).
M3a.3 is complete within its synthetic inventory scope and remains unreleased.
The [installed-source/isolated-runtime acceptance plan](docs/plans/2026-09-30-herdr-projects-acceptance.md)
was promoted by “Commit and move to next”. Its prerequisite audit is complete;
the operational trial is blocked by missing Herdr Projects, installed Herdr 0.9.0
versus upstream's 0.9.1 minimum, and thread creation's automatic ticker startup.
See the [audit receipt](docs/reviews/2026-09-30-herdr-projects-acceptance/README.md).
The accumulated work is committed at `4923a05`, with a fresh passing full gate.
The smaller [upstream-writer fixture plan](docs/plans/2026-09-30-herdr-projects-writer-fixtures.md)
was proposed, not promoted; further Projects qualification is now parked under
the owned-work direction below.
Installed-source/native/live acceptance, M3b and M4–M7 remain outstanding or
unpromoted. Existing pane-header repair, visual qualification and release evidence
remain independent.

On 2026-10-01 the user agreed to owning our project/work backend, with Questmancer
as its interface and Herdr providing execution. The
[owned-work contract](docs/design/questmancer-owned-work.md) and
[L1 milestone plan](docs/plans/2026-10-01-owned-work.md) are the current next-work
documents. L1a proposes an authoritative local store, five typed human commands,
decision answers and source/cache restart proof. L1b adds reviewed authoring forms
to complete the user workflow. The cache and Chronicle retain their existing
meanings. L1a backend proof was approved and implemented on 2026-10-03; its
[receipt](docs/reviews/2026-10-03-owned-work-backend/README.md) records qualification.
L1b form and boundary review was subsequently approved; its
[design](docs/design/questmancer-owned-authoring.md) and
[implementation proposal](docs/plans/2026-10-03-owned-authoring.md) are written.
L1b implementation was then promoted and is implemented with explicit startup
selection, five forms, checked answer detail and separate save/view recovery. See the
[engineering and form review receipt](docs/reviews/2026-10-03-owned-authoring/README.md).
The user subsequently approved the [form images](docs/reviews/2026-10-03-owned-authoring/visual-approval.md).
Native/live use remains unperformed: Computer Use refuses Ghostty access. The
[isolated native plan](docs/plans/2026-10-03-owned-authoring-native.md) is prepared
for human execution; its [receipt](docs/reviews/2026-10-03-owned-authoring-native/README.md)
records the limit. Projects
acceptance, writer fixtures and M3b are parked; no adapter code or old receipts
are removed. No external project installation or Herdr upgrade is a dependency.
L2–L5, including any separate orchestration module, need their own plans/promotion.

Keep these unpromoted ideas in the backlog:

- Chronicle scope outside the separately proposed living-control-plane roadmap;
- durable mementos or a trophy shelf with a separate persistence design;
- additional Hall station/hierarchy redesign;
- Storybook navigation across categories;
- ancestry silhouettes and unused appearance attributes.

Guarded sidebar-token and optional urgency-order acceptance must preserve the
user's global configuration and every unrelated pane/server.

## Engineering rules

No unsafe Rust, telemetry, cloud service, database or copied product art.
No manual sprite controls or persisted live topology/output. Native image
capabilities never replace the RGB world renderer. Start behaviour changes
with a focused failing test; keep documentation and operational recipes in
that slice. Visual, terminal, live, persistence and release evidence remain
separate. Follow [AGENTS.md](AGENTS.md) for ownership and guarded testing.

## Herdr 0.9.4 compatibility — 2026-10-14 (unreleased)

The user promoted updating plugin compatibility to the latest Herdr. Protocol 22
is retained. The removed pane-graphics geometry workaround is retired in favour
of standard terminal detection; lost subscription history preserves its server
diagnostic and requests a new connection baseline. Official 0.9.4 captures and
an isolated linked-plugin run qualify the transport independently of native
visual acceptance and publication. See [the current receipt and capability
assessment](docs/reviews/2026-10-14-herdr-094/README.md). The installed Herdr
application and user-owned servers are separate from this source qualification.

## Installed Herdr upgrade — 2026-10-15

The user approved upgrading. Herdr 0.9.5 (the latest stable 0.9.4 hotfix) is now
installed, with current Claude/Codex integrations and protocol 22. A fresh
isolated production-plugin trial and automated owned-authoring/restart trial
passed with owned cleanup. No shared server was running or restarted. The
[upgrade receipt](docs/reviews/2026-10-15-herdr-upgrade/README.md) records the
binary digest, configuration preservation and acceptance limits. The manual
native trial now references this candidate; human execution remains the next
gate. Commit/publication and L2–L5 remain separate.


### 2026-11-17 — proportional Hall production integration (unreleased)

The owner explicitly promoted the approved Storybook fit. The production Hall
now fits its existing canonical RGB scene through negotiated Kitty output with
shared mouse geometry, bounded owned-image cleanup and existing semantic frame
deadlines. Half-block fallback, all parchment/card paths and the Delve remain
available. Station-capacity checks now prevent silently omitted party members.
Full qualification: 815 Rust tests / 66 runs, 28 shell tests, release build,
four isolated release-binary PTY scenarios and an exact native-payload pixel
comparison. Only the verified retained dogfood UI was refreshed; synthetic
party, personas, authoritative work and focus were preserved. Native visual,
resize and animation acceptance remain human gates. No commit or release.
See `docs/reviews/2026-11-17-production-hall-fit/README.md` and ADR-0002.


### 2026-11-17 — proportional Delve integration (unreleased)

The owner promoted the Hall fit/fallback behavior to the dungeon. Both rooms
now share `WorldFit`, the existing source-pixel transform and two-ID transport
cache. Saturated Delve stations choose a complete roster whenever it fits;
scrying and other parchments retain their existing output path. Two fixed Delve
studies bring Storybook to 71 stories. Full qualification passes 821 Rust tests /
66 runs, 28 shell tests, release/Storybook builds, Rustdoc, five isolated PTY
scenarios and two byte-exact native image comparisons. The verified existing
UI is refreshed and left on the Delve; the remaining synthetic pane, persona
records and authoritative work were preserved. Three historical synthetic panes
were already absent and were not recreated. Current busy-party station overflow
uses the full roster fallback; native visual acceptance remains human review.
See ADR-0003 and `docs/reviews/2026-11-17-production-delve-fit/README.md`.
No commit, release or shared-server change.


### 2026-11-17 — crowded dungeon regression repair (unreleased)

The owner's screenshot rejected the broad flat-roster overflow behavior. ADR-0004
supersedes that policy: saturated ordinary parties now gather over the authored
dungeon with up to sixteen full 16x24 masters, centered rows and literal state
cues. Dense/small rosters also retain dungeon backgrounds. Fixed backdrop
centering preserves reduced/still no-cleanup behavior; gathered completion
remains quiet at three seconds. The resting-crowd study brings Storybook to 72.
Final qualification: 825 Rust tests / 66 runs, 28 shell tests, release/Storybook
builds, Rustdoc, five isolated PTY scenarios and two exact image comparisons.
The owned dogfood UI is refreshed with the fitted adapter active and Delve open;
remaining resources, persona records and authoritative work were preserved.
Native appearance awaits human review. See
`docs/reviews/2026-11-17-delve-crowd-repair/README.md`. No commit or release.

### 2026-11-17 — fitted rooms beneath adventurer profiles (unreleased)

The owner's screenshot exposed the profile guard reverting both rooms to
half-block output. ADR-0005 retains the same proportional image and transform
beneath the unchanged profile through shared masked row slicing. Covered actor
targets are removed and profile clicks are consumed before they can request
hidden adventurer output. Initial full occlusion, first exposure, repeated
toggles, native portrait layering and resize are covered. Two fixed profile
studies bring Storybook to 74 stories. Full qualification passes 830 Rust tests /
66 runs, 28 shell tests, release/Storybook builds, Rustdoc, eight executable PTY
scenarios and two exact image comparisons. The owned dogfood UI is refreshed;
remaining synthetic resources, all persona records, owned-work bytes and focus
are preserved. Native profile appearance and interaction await human review.
See `docs/reviews/2026-11-17-profile-fit-repair/README.md`. No commit or release.

### 2026-11-17 — resting-party composition review (repair pending)

New owner screenshots reject the Hall's bottom-packed compact fallback and the
Delve's gathered row. Current-source reproduction confirms that six resting
adventurers plus one unknown disable Hall fitting (five canonical placements for
seven actors), while the Delve still fits but gathers everyone into one row.
Forty exports and 64 passing focused tests demonstrate the missing composition
acceptance contract. Local storage has 13 GiB free; no cleanup was performed.
This is assessment only: production and retained live resources are untouched.
Next: Storybook-first station-preserving overflow positions, then production
correction and full qualification. See
`docs/design/reviews/2026-11-17-party-layout-regression/README.md`.

### 2026-11-17 — larger-party UX comparison (experimental)

The owner requested room for more adventurers, including smaller sprites.
Eight Storybook-only studies compare seven full/small adventurers and 24/32
authored small adventurers in both complete rooms, with stable seats, state
cues, selection and existing full-size profiles. Storybook has 82 stories.
Verification passes 834 Rust tests / 66 runs and 28 shell tests; 24 source
renders and four executable PTY scenarios provide bounded producer evidence.
Native readability, small-class art and spatial-grouping decisions remain for
human review before automatic density or production promotion. The current
production regressions and live resources are unchanged. See ADR-0006 and
`docs/design/reviews/2026-11-17-party-capacity/README.md`.


### 2026-11-17 — automatic fitted-room density (unreleased)

The owner approved automatic tier switching. Both native fitted rooms retain
16x24 masters through eleven visible adventurers, use authored 8x12 staggered
seats for 12–32 and denser 8x12 seats for 33–64. Stable seating and literal state
cues replace the crowded Hall rejection and Delve row; profiles retain full-size
detail. Existing graphics/geometry/modal fallbacks and bounded overflow remain.
Two production automatic stories bring Storybook to 84. Qualification passes
841 Rust tests / 67 runs, 28 shell tests, release/Storybook/Rustdoc builds, eight
executable PTY scenarios, 42 source exports and two exact payload comparisons.
Only the verified retained UI was refreshed; current view, seven-person party,
remaining synthetic resources, saved personas, owned work and focus were
preserved. Native visual/input/resize/animation review remains a human gate.
See ADR-0007 and `docs/reviews/2026-11-17-automatic-density/README.md`.
No commit, release or shared-server change.
