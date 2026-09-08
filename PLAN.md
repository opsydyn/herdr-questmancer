# Questmancer plan

Questmancer turns a Herdr session into an adventurers' guild. The user is the
Questmancer, workspaces are campaigns, agents are adventurers, blocked work
raises summons, and explicit completion returns spoils. Herdr owns live facts;
Questmancer owns presentation and a small amount of durable local intent.

## Current status — 2026-09-08

Source baseline: Questmancer `0.1.9`, minimum Herdr `0.9.0`, supported protocol
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
| Complete 0.1.9 candidate preparation | Reviewed, verified and packaged; clean-commit and distribution gates remain |

The [complete candidate review](docs/reviews/2026-09-08-party-candidate/README.md)
indexes every current approval and records this expanded candidate's checks.
The latest `just verify` passed on 2026-09-08: **593 Rust tests across 54 runs,
28 shell tests, formatting, Clippy with warnings denied and script syntax**.
The release build and verified dirty-source package pass. The source package
also builds Storybook with its production assets. All current review-pack art
is approved; native room captures, real-agent transitions and acceptance from
the eventual clean release commit remain separate gates.

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

## Distribution and release gates

The [2026-09-08 publication check](docs/release-process.md#local-candidate--2026-09-08)
found only published `v0.1.0` and `v0.1.3`; the latest still has four platform
archives and `SHA256SUMS`. No `v0.1.9` tag exists in the checked remote.
The September 6 historical check found public `main` at `0.1.8` without a
matching release. Use the source-link workflow for this 0.1.9 checkout.
Earlier 0.1.3 archive and temporary installer checks are historical evidence.

The release workflow now has a locally verified explicit dispatch after tagging;
its live execution remains unverified. Closing distribution
requires a matching published release, not another historical `v0.1.0` tag.
Use [the release process](docs/release-process.md) and verify:

1. The intended clean commit passes `just verify`, `cargo build --release`,
   package checks and diff hygiene.
2. The tag matches both `Cargo.toml` and `herdr-plugin.toml`.
3. Four archives contain a root-level executable and match `SHA256SUMS`:
   x86_64/aarch64 Linux GNU and x86_64/aarch64 macOS.
4. `herdr/install.sh` installs that published version successfully.
5. Guarded Herdr `0.9.0` acceptance is repeated from that release commit,
   with current Guild Hall and Delve captures.
6. Optional Reviewr is tested only when `persiyanov.reviewr.open` is available.
7. Real-agent resting and completion are recorded only when actually observed.
   `herdr pane report-agent` supports idle, working, blocked and unknown;
   it cannot synthesize done.

Registry publication is separately gated. A crates.io API check returned 404
for Questmancer; registration, credentials and the live publish gate require
separate verification. Local tests do not establish registry availability.

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

Before distribution, commit and qualify this repair on top of `9ea8501`. No further class-art or feature expansion is
needed to resolve this regression.

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

Next: close clean-commit qualification and native/live acceptance, then seek
release authorisation. Publication is a separate gate.
Native transport/room captures, four-platform archives, checksums and the
published installer retain their own acceptance requirements above.

Keep these unpromoted ideas in the backlog:

- snapshot-only Chronicle event capture and clearer identity/closure semantics;
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
