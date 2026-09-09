# Questmancer plan

Questmancer turns a Herdr session into an adventurers' guild. The user is the
Questmancer, workspaces are campaigns, agents are adventurers, blocked work
raises summons, and explicit completion returns spoils. Herdr owns live facts;
Questmancer owns presentation and a small amount of durable local intent.

## Current status — 2026-09-09

Source baseline: Questmancer `0.1.10`, minimum Herdr `0.9.0`, supported protocol
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
A guarded real-agent Chronicle acceptance session is the recommended next bounded
step if approved. Real-agent completion and live snapshot-only capture remain
separate evidence; no new product scope is promoted by this release.

Keep these unpromoted ideas in the backlog:

- further Chronicle scope beyond the approved C1–C3 capture plan;
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
