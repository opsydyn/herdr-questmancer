# Complete Questmancer 0.1.9 candidate review

2026-09-08. The Questmancer approved the final Mage/Sorcerer card visuals and
preparation of the complete candidate. All promoted party-delight work is
implemented and visually approved. This report covers the expanded, uncommitted
checkout on `main`, based on `c3720a9`; that earlier clean qualification does
not qualify these later changes as a clean release commit.

## Review outcome

The bounded review found and corrected a packaging blocker: review packs made
the crate **18,542,974 bytes**, exceeding the existing **10,485,760-byte** CI
limit. The actual archive-size assertion failed before the fix. `Cargo.toml`
now excludes `docs/design/reviews` and `docs/reviews` from the crate while
retaining them in Git. The final verified archive is **4,965,526 bytes**.
All 101 Rust source and native PNG files under `src` match their packaged
copies byte for byte. The default crate and packaged Storybook both build.

The handoff had accumulated contradictory pending approvals and old test
counts. `AGENTS.md`, `PLAN.md`, the README, art guidance and release process
now state the current result and separate outstanding release gates. Curated
notes are consolidated under `[0.1.9]`, leaving `[Unreleased]` empty. Running
the workflow's exact AWK extractor confirms the release body includes both
the earlier candidate and every new party slice, with current Herdr/protocol
requirements.

No other blocking finding was identified in this bounded source review. It
covered the post-`c3720a9` application/interaction/runtime wiring, chapter
projection and overlay, cat transition and deadlines, keepsake card layout,
ritual registry/assets and persona substitution, all-class card centring,
Storybook inventory, focused regressions and the existing release workflow.
This is a local agent review with automated evidence; it does not establish
native terminal appearance, real-agent lifecycle or publication acceptance.

The reviewed boundaries remain intact:

- Chronicle chapters count retained events within a fixed UTC window and link
  timestamped sources. Snapshot refreshes append no history; identity events
  are not asserted to be reliable joins. Chapters add no commands or persistence.
- The cat reacts once for the same known non-empty party becoming entirely
  resting. Membership and connection changes establish a new baseline;
  reduced/still rendering has no added animation wakes.
- Keepsakes use existing saved assignments, static authored art and fixed copy.
- All fourteen rituals use shared timing and persona routes; cards reuse exact
  static native-size world pixels. Roster art and native illustrations remain.
- Explicit user-composed counsel remains the only agent text route. Rendering
  adds no output fetch or persisted live topology.

## Approval index

All rows below were visually approved by the Questmancer, most recently on
2026-09-08. Images remain in their original approval packs; they were not
regenerated during this candidate review.

| Scope | Record |
| --- | --- |
| Librarian, pilot, rooms, heraldry and earlier cards | [Consolidated review](../../design/reviews/2026-09-08-consolidated/README.md) |
| Six keepsakes | [Card details](../../design/reviews/2026-09-08-keepsakes/README.md) |
| One-shot cat reaction | [Production reaction](../../design/reviews/2026-09-08-cat-reaction/README.md) |
| Factual Chronicle chapters | [Chapter screens](../../design/reviews/2026-09-08-chronicle-chapters/README.md) |
| Bard, Artificer, Testmender | [Production rituals](../../design/reviews/2026-09-08-tool-ritual-production/README.md), [cards](../../design/reviews/2026-09-08-tool-card-fallbacks/README.md) |
| Cleric, Paladin, Druid | [Production rituals](../../design/reviews/2026-09-08-cleric-paladin-druid-production/README.md), [cards](../../design/reviews/2026-09-08-cleric-paladin-druid-card-fallbacks/README.md) |
| Rogue, Pathseeker, Runewright | [Production rituals](../../design/reviews/2026-09-08-rogue-pathseeker-runewright-production/README.md), [cards](../../design/reviews/2026-09-08-rogue-pathseeker-runewright-card-fallbacks/README.md) |
| Mage, Sorcerer | [Production rituals](../../design/reviews/2026-09-08-mage-sorcerer-production/README.md), [final cards](../../design/reviews/2026-09-08-mage-sorcerer-card-fallbacks/README.md) |

Storybook now has **45 stories**: two worlds, 37 asset views and six interactions.
The asset views include fourteen class pose galleries. Existing native card
illustrations and independent roster families remain part of the production UI.

## Current qualification

See [verification.json](verification.json) for source hashes, artifact identity,
commands and evidence boundaries. Logs are retained alongside this report.

| Check | Result |
| --- | --- |
| `just verify` | 593 Rust tests across 54 runs; 28 shell tests; formatting, Clippy with warnings denied and shell syntax passed |
| `cargo build --release` | Passed on this macOS ARM64 host |
| `cargo package --allow-dirty --locked --offline` | Passed, including Cargo's packaged-source build; 205 files, 4,965,526 bytes |
| Packaged Storybook | `cargo build --manifest-path target/package/questmancer-0.1.9/Cargo.toml --locked --offline --features storybook --bin questmancer-storybook` passed |
| Final documentation/workflow checks | 28 shell tests and workflow contracts passed; exact release-note extraction checked |
| Diff hygiene | `git diff --check` passed |
| Isolated Herdr 0.9 runtime | Passed; test-owned server, four synthetic identities and one managed plugin pane; cleanup passed |

The [isolated runtime receipt](isolated-runtime.json) records the newly created
resources and release-binary hash. The existing guarded
[qualification script](../2026-09-08-release-candidate/isolated_plugin_check.py)
was reused without reusing historical IDs or server state. It confirmed
working/blocked/idle/unknown metadata, managed-pane exclusion, singleton open,
and working → blocked → working transitions. Guild/Delve action logs exited
zero; those are invocation checks, not room-appearance proof. Every synthetic
identity was released before its pane closed. The test link was removed,
owned focus restored before cleanup, the owned server exited zero, and its
socket disappeared. No shared server or unrelated pane was operated on.

## Remaining gates and next bounded step

The next recommended step is a **local candidate commit followed by clean-commit
qualification**, when authorised. Include the accumulated implementation,
review artifacts and current handoff changes; review the final staged scope
before committing. Repeat `just verify`, `cargo build --release`, clean
`cargo package --locked --offline` without `--allow-dirty`, and guarded isolated
Herdr qualification. Store those new receipts outside the checkout so they
cannot dirty the candidate. No commit, tag, push, workflow dispatch or
publication was performed during this preparation.

Native Ghostty/Herdr graphics and current room captures remain unverified.
Prior Computer Use safety rejection is not resolved by these RGB/Ratatui packs
or headless integration. Real-agent resting/completion and optional Reviewr
retain their separate acceptance gates; synthetic reports cannot express done.

The fresh read-only GitHub check found only published `v0.1.0` and `v0.1.3`;
the `v0.1.9` tag lookup returned 404. Publication still requires explicit
authorisation, followed by the four current platform archives, checksum and
installer checks. Historical 0.1.3 checks do not qualify 0.1.9. Registry settings
and credentials remain unverified. Follow [the release process](../../release-process.md).
