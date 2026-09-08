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
| Librarian proportion refresh | Implemented; world and independent Ledger fallback await visual approval |
| Three-class production pilot | Implemented; production pose, playback and counsel review awaiting visual approval |
| Pilot card fallback continuity | Implemented, verified and visually approved 2026-09-05; cards reuse the new personalised sprites at native size |
| Terminal resize pass | Storybook correction and 70 passing executable PTY resize checks approved 2026-09-06; native Ghostty visual review blocked by Computer Use |
| Herdr 0.8.2 release preflight (2026-09-06) | Historical isolated compatibility, package and published 0.1.3 archive checks; release dispatch repair remains local |
| Herdr 0.9 upgrade | Installed 0.9.0 / protocol 22; subscription baseline and status reconciliation updated; isolated production-client checks passed; conditional sidebar recipe proposed |
| Campaign heraldry | Implemented: deterministic table crests and matching card identity; final visual sign-off deferred |
| Keepsakes, guild memory and remaining classes | Follow-on candidates; not part of the heraldry slice |

The [storyboard](docs/design/reviews/2026-09-05-party-storyboard/README.md) and
[production roster sheets](docs/design/reviews/2026-09-05-roster-states/README.md)
record the two earlier visual approvals. The [production pilot pack](docs/design/reviews/2026-09-05-party-pilot/README.md)
and Librarian refresh await their own approval. The latest full `just verify`
passed on 2026-09-08: **570 Rust tests across 51 test runs, 28 shell tests,
formatting, Clippy with warnings denied and script syntax**. This covers the
current checkout with preserved earlier edits. Fixture inspection and automated
checks do not certify live Herdr transitions, native transport or the eventual
release commit.

The [Herdr upgrade and release receipt](docs/reviews/2026-09-06-release-readiness/README.md)
records the historical 0.8.2 binary, isolated compatibility checks and distribution
preflight. The [0.9 upgrade](docs/plans/2026-09-08-herdr-090-upgrade.md) records
the current protocol and sidebar proposal. Native room captures and acceptance from a clean release commit
remain pending.

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
  pilot roster masters. Wizard, Ranger and Barbarian have authored working,
  counsel and spoils sequences with shared render deadlines. Their card
  fallbacks now centre the same personalised world sprite in the existing
  `24x32` canvas without stretching.
  Native graphics are confined to cards and the Ledger illustration.
- Original Hall/dungeon architecture, material lighting, selected floor rings,
  semantic completion effects and a bounded hidden goblin interaction.
- Eleven sidebar tokens, optional Herdr-owned urgency sorting, and local guild
  standing awarded only for recorded spoils/campaign closure.
- Atomic versioned user intent, debounced persistence, acknowledged shutdown
  flushes, and terminal restoration after normal exit, error, signal or panic.
- Thirty-four Storybook stories through production paths: two worlds,
  twenty-six asset views and six interactions. Current `j`/`k` navigation
  remains category-local; `h`/`l` changes category.

The Librarian is an independent help NPC. Canonical and compact Halls reserve
its complete clickable sprite; smaller tiers retain keyboard access to the
Ledger. The [Librarian refresh](docs/design/reviews/2026-09-05-librarian/README.md)
adds a stocky world sprite and independent Ledger fallback; visual approval
is pending.

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

The [current publication check](docs/release-process.md#distribution-status--2026-09-06)
found the public repository and published `v0.1.0` and `v0.1.3` releases.
Latest `v0.1.3` lists four platform archives and `SHA256SUMS`. The September 6 check found public `main` at `0.1.8`, with no published
`v0.1.8` release. This checkout now prepares the unused `0.1.9` candidate. Current default plugin installation therefore lacks its
matching archive; use the documented source-link workflow for this checkout.
The four published 0.1.3 archives now pass checksum, layout and architecture
checks; its macOS ARM64 installation passes in a temporary directory.

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

## Approved next sequence — 2026-09-08

The Questmancer approved the sequence, then explicitly moved visual sign-off to
last so engineering can progress. Current state:

1. Operating notes corrected; four-row sidebar applied with the original backed up.
2. Librarian and three-class pilot remain implemented; their visual decisions
   are collected in the final review rather than blocking the next slice.
3. Local release preparation passes full verification, release build and
   packaged-source verification. Clean-commit, archive/installer and publication
   acceptance remain separate.
4. Campaign heraldry is implemented; see the
   [slice record](docs/plans/2026-09-08-campaign-heraldry.md).
5. Complete the [final review queue](docs/reviews/2026-09-08-final-review.md), then
   qualify the eventual clean release candidate and publish only when authorised.

## Remaining product review and backlog

- Complete the [approved party sequence](docs/plans/2026-09-05-party-delight.md)
  in bounded slices. Review the revised Librarian and three-class production
  pilot sheets and playback in the deferred final visual review.
- Review remaining [Hall station and hierarchy work](docs/design/guild-hall-art-direction.md)
  independently; the roster approval does not approve an entire room redesign.
- Storybook flat-list navigation across categories remains a proposed change;
  current category-local navigation is documented, not silently reimplemented.
- Repeat guarded sidebar-token and optional urgency-order acceptance, retaining
  the user's global configuration and every unrelated pane/server.
- Keep mementos and expanded class rituals in the backlog
  until explicitly promoted.

## Engineering rules

No unsafe Rust, telemetry, cloud service, database or copied product art.
No manual sprite controls or persisted live topology/output. Native image
capabilities never replace the RGB world renderer. Start behaviour changes
with a focused failing test; keep documentation and operational recipes in
that slice. Visual, terminal, live, persistence and release evidence remain
separate. Follow [AGENTS.md](AGENTS.md) for ownership and guarded testing.
