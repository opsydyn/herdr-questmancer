# Chronicle chapters — 2026-09-08

The Questmancer approved implementation after approving the cat reaction.
The Questmancer approved this parchment flow on 2026-09-08. Native transport
observations remain separate.

[Production review sheet](chronicle-chapters.png) shows the unchanged selected
record list, the whole-guild chapter, source entries reached by scrolling and
an empty later window. It uses fixed fixture names/times and actual production
Ratatui cells reconstructed with Menlo, not native terminal screenshots.

## Reading a chapter

Press `c` to open the existing Chronicle, then `Tab` for a last-hour guild
chapter. `Tab` returns to the record list. Use `j`/`k`, arrows or the wheel to
scroll; `Esc` closes. Requesting another chapter captures a new UTC window.
Nothing sends counsel, focuses adventurers or changes selected identity.

The inclusive start/end window is fixed when requested. Chapters count retained
EventIds within it, not people or successful projects. Repeated spoils returns
by one adventurer remain multiple events. Source summaries preserve recorded
names, with timestamps and source IDs underneath. Source order is deterministic
for timestamp ties. Long source text wraps to the full 76-column parchment;
terminals narrower than 80 columns clip horizontally until widened.

The projection is local and on demand. Its request is ephemeral. No stored
chapter, reading checkpoint, new history event, model-generated prose service,
network request, achievement or persistence migration is introduced. UTC dates
use the already locked `time` crate, now a direct dependency; its 1.88 MSRV is
below this repository's Rust 1.90 requirement. Unrepresentable dates retain
explicit epoch milliseconds instead of a fabricated calendar date.

## Evidence limits

The Chronicle is bounded and does not record whether every event in a window
was captured. Every chapter says it covers retained records only. Empty windows
say no retained events, not that nothing happened.

Normal Herdr 0.9 status events are reconciled against authoritative pane
metadata and can append Chronicle history. Refreshed snapshots themselves
preserve but do not append history.
Workspace removal does not emit CampaignClosed. The legacy AdventurerJoined
enum also records unknown whereabouts; chapters call this an identity event,
and preserve its original summary. A retained campaign-closure record can be
counted, but never becomes a claim of successful delivery. No missing event is
reconstructed from current topology or presence.

Snapshot-only capture and clearer identity event taxonomy remain separate
correctness follow-ups. Any future capture work must preserve startup/reconnect
baselines and deduplication. These chapters do not repair historical gaps.
The next planned delight batch is Bard, Artificer and Testmender class art.

## Verification

Focused tests cover explicit inclusive boundaries, future entries, repeated
events, stable source order, bounded/missing history, all event templates,
legacy unknown semantics, extreme timestamps, full long-source text, keyboard
routing, fixed request time, scroll bounds and unchanged persisted state.
The first UI regression failed because Tab did not request a chapter. A later
scroll regression failed at a trailing blank separator; the final scroll
position now retains the last source ID.

`just verify` passes 591 Rust tests across 54 runs and 28 shell tests,
formatting, Clippy with warnings denied and shell syntax. Release build and
review regeneration also pass. [Verification receipt](verification.json). Product visual approval and native transport observations remain
separate. This work is uncommitted after the clean `c3720a9` candidate; it has
not been tagged or published.

Regenerate with a Python containing Pillow:

```bash
python3 docs/design/reviews/2026-09-08-chronicle-chapters/regenerate.py
```
