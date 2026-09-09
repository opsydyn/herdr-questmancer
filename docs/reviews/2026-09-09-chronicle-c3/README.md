# C3 — Chronicle capture qualification

C3 was approved on 2026-09-09. Automated composition and isolated synthetic
acceptance pass. The user visually approved the Chronicle screen pack on
2026-09-09. C1–C3 are complete locally; commit and publication remain separate.

Source: local `main`, HEAD `a26386c`, with the retained C1/C2 changes. C3 adds
qualification tests, review assets and operating guidance; it required no new
production-code change. This is an uncommitted 0.1.9 working copy, not a published
release or clean-commit qualification.

## Automated evidence

`tests/chronicle_composition.rs` runs the actual `RuntimeConnection`, supervisor,
command snapshot tasks, event adapter, reducer, persistence dispatcher and worker
against temporary Unix sockets and files. Unrelated output/sidebar/Reviewr
commands are omitted from this focused harness; their existing suites remain in
the full gate.

Four composition tests pass:

1. Hold the supervisor's topology probe independently from the command refresh.
   Its misleading `done`/revision-100 result cannot capture or replace facts;
   the qualified blocked/revision-8 result creates one valid v2 observation.
   An unversioned buffered done event is reconciled with current pane metadata.
   Snapshot-first done and matching metadata create no retroactive XP.
2. Two snapshot results overtaken by metadata trigger bounded resubscription.
   The new done baseline adds no history or XP; accepted metadata records survive.
3. An actual JSONL append failure returns the worker's error acknowledgement.
   In-memory acceptance remains distinct from durability, with no XP awarded.
4. New observations append after byte-identical legacy JSONL. A real status reward
   persists once; restart/replay and a matching done report neither rewrite the
   history nor re-award standing.

See [composition log](composition.log) and [full gate](verify.log): **656 Rust
tests / 56 runs, 28 shell tests**, formatting, all-target/all-feature Clippy with
warnings denied, workflow checks and shell syntax. The initial sandbox refused
Unix socket binding; these tests ran with the socket permission granted.

The [release build](release.log) and [package extraction/build](package.log) pass.
The [package receipt](package.json) records 215 files, 5008558 bytes
and the SHA-256. It includes the new composition suite and remains below 10 MiB.
This is a local dirty-tree artifact; it was not published.

## Isolated Herdr evidence

[Successful receipt](live-receipt.json), [captured JSONL](live-captured-chronicle.jsonl),
[initial snapshot](live-initial-snapshot.json), and [process cleanup](process-cleanup.json).

The script created a private XDG tree and Herdr 0.9.0/protocol-22 headless server,
four plain panes and test-compiled sleepers named `pi`. Fresh synthetic session
UUIDs and native session-start reports established identity; ordered lifecycle
reports provided working/blocked/idle/unknown states. These are dummy processes,
not real AI agents. Session restore and update checks were disabled.

The source-linked release binary was verified by process command and SHA-256 in
the receipt. Five v2 **pane metadata** observations persisted in order: blocked,
working, idle, unknown, working. Repeating working added no record. Closing and
reopening the owned plugin pane preserved the exact history bytes. The populated record list and chapter
were exercised through the owned pane; text reads are invocation/content
evidence, not RGB or native visual acceptance.

Cleanup released each synthetic report before closing its pane, closed the
managed pane, removed only the test-created plugin link and confirmed an empty
pane/plugin inventory. The owned server exited with code 0, its socket was removed,
and a fresh process check found none of its recorded server/pane PIDs. No shared
Herdr server, real adventurer, or user's plugin registration was modified.

## Setup findings and limits

[Attempt audit](live-attempts.json) preserves unsuccessful setup attempts instead
of presenting them as product failures. Custom reports without native identity
correctly produced no capture. Herdr's official source/session rules prevented
several provisional setups from qualifying. A copied macOS system sleeper was
killed by the OS; the final runner compiles its own minimal sleeper. A provisional
output read used an unsupported source name, corrected to `visible`. Earlier
cleanup assertions wrongly expected release to erase a still-running process's
session reference; the final runner verifies report release and subsequent pane
absence separately. Every attempted owned server exited and removed its socket.

These rules were checked against Herdr's pinned
[session reference code](https://github.com/herdrdev/herdr/blob/v0.9.0/src/agent_resume.rs)
and [terminal reporting guards](https://github.com/herdrdev/herdr/blob/v0.9.0/src/terminal/state.rs).
The final reproducible procedure is in the
[manual recipe](../../manual-test/chronicle-capture.md).

Live snapshot-only capture is not claimed from metadata events. That path is
qualified by composed socket fixtures. Real-agent completion, native terminal
visual acceptance, complete upstream history, cross-file atomicity and publication
remain separate. Herdr cannot directly synthesize explicit done through its
report-agent interface; done/reward ordering is fixture evidence only.

## Visual gate

The [screen pack](../../design/reviews/2026-09-09-chronicle-capture/README.md) contains
production reconstructions of records, chapters, sources, empty history and narrow
viewports. The user approved this copy/layout on 2026-09-09. The existing sub-80-column
chapter clipping contract remains visible and documented.
