# M3a.1 Herdr Projects snapshot/exporter proof — 2026-09-30

The user said “Let’s continue” after M3 source inspection/planning, promoting the
bounded M3a.1 contract/exporter proof. It is implemented inline and engineering-qualified: M3a.1 is complete within
its synthetic contract-proof scope and unreleased. This receipt covers synthetic source-tree tests,
not a live adapter, native UI acceptance or release.

## Baseline and delivered boundary

Questmancer `main`, HEAD `f380720ba63424b279980ff4f1c164d8fd4ef735`, ahead six;
the accepted M1/M2 and independent pane-header work remain uncommitted and preserved.
The full gate qualifies this combined dirty checkout. See [file status](git-status.txt).
Herdr Projects was inspected at `4e4548c3c43888e6be1c96906215998f07dc63f0`, release
0.2.34. No installed source version, real home, pane, agent, ticker, shared server
or plugin registration was operated. Only test-owned files/processes were used.

- `src/sources/herdr_projects/contract.rs`: typed version-1 export/decoder,
  literal upstream enums, explicit supported/unsupported capabilities and strict
  validation, including duplicate thread keys and report evidence consistency.
- `files.rs`: read-only descriptor-relative NOFOLLOW traversal and regular-file
  checks, including every ancestor, directory and allowlisted file. Reads have
  byte/count bounds; no create/write flags or metadata-derived external path.
- `read.rs`: two byte captures, existing read-only shared non-blocking project
  lock, directory/lock identity checks, strict record parsing and home-report
  SHA-256 evidence. No domain reducer, I/O clock, source command or output reader.
- `capture` returns a source DTO only. No startup configuration/CLI action,
  SourceInbox attachment, work mapping, observer revision/cursor or persistence
  change is introduced. The caller supplies explicit identities/path/time.
- Source format/revision is a declared pinned layout, not certification of an
  installed binary. Unknown selected source enum variants are rejected; source
  fields outside the supported subset are ignored, not copied into the DTO.

The [frozen contract](../../design/herdr-projects-snapshot-contract.md) and
[synthetic JSON example](snapshot-v1.json) describe what the boundary actually
supports. It does not promise an upstream-standard protocol. `sha2` is now an
explicit dependency; existing rustix/time dependencies enable filesystem/parsing
features. Cargo.lock records the added time-macros dependency at 0.2.31; the host
uses Rust 1.90.0. No dependency was updated by an unbounded upgrade command.

## Engineering evidence

| Check | Outcome | Evidence |
| --- | --- | --- |
| Focused all-feature source suite | Passed: 15 tests before final diagnostic/portability edits; final gate covers them | [Focused](focused.log) |
| Clippy, all targets/features, warnings denied | Passed before final diagnostics/test portability edits; final gate covers those edits | [Clippy](clippy.log) |
| `just verify` | Passed: 722 Rust tests across 61 runs, 28 shell tests, formatting, Clippy and script syntax | [Full gate](verify.log) |
| `just property-test 4096` | Passed: 107 tests across six runs; generators configured for 4,096 cases | [Properties](property.log) |
| Release build and diff check | Passed | [Build](release-build.log), [machine receipt](verification.json) |

Test-first evidence: the source boundary was initially missing ([red](red.log)).
A subsequent focused decoder test failed because ordinary map deserialization
accepted duplicate thread keys ([duplicate-key red](duplicate-red.log)); the
custom bounded visitor now rejects them. A separate red test established that
literal multiline goal/error context must survive while terminal escape/control
characters remain invalid ([context red](context-red.log)).

Tests prove source-byte/file inventory invariance for successful captures;
missing lock does not create one; malformed/unknown/mismatched required records
fail the whole capture; symlinked roots/files/reports and path traversal fail;
report hash mismatch/missing report remain explicit; false capabilities, future
versions and inconsistent diagnostics fail; file/inventory/document bounds hold;
and 1/10/100-thread captures are deterministic for fixed input/time.

The generated property covers duplicate identity keys with conflicting Open and
Resolved records: no later closure may be selected by last-write-wins decoding.
An independent test-owned writer process holds the same exclusive file lock used
by upstream, atomically replaces paired project/thread records, and briefly leaves
a torn pair inside its critical section. Captures either return Busy or observe
matching pairs; the final Pair 40 is seen. This demonstrates cooperative locking
on this macOS host, not a native source transaction under arbitrary writers.

## Findings and limitations

1. Every DTO has Partial coverage, including under the cooperative lock.
   Equal reads and individual atomic writes cannot establish complete coverage,
   unseen intermediate events, arbitrary-writer exclusion or replay. Omissions
   cannot resolve/retire work. A future consumer must retain prior facts and
   diagnose known-entity disappearance rather than mark them current.
2. Every DTO declares durable identity unsupported. A test removes/replaces a
   numeric thread ID while retaining the same creation timestamp; that timestamp
   cannot certify a new incarnation. No hash or path is promoted to native identity.
   Before durable mapping, accept a source registration/retained-record contract
   with explicit limits, or obtain a native project/thread incarnation export.
3. Source grouping is last-recorded context. WaitingOnYou does not create a
   Decision; Resolved remains a source closure reason, not successful work.
   Structured requests, dependencies, live binding and revision-bound validation
   are all unsupported. Legacy attention and M1/M2 domain/rendering are unchanged.
4. Home report matching is only byte/hash agreement. Bodies are discarded; no
   PR/check/review/merge assertion or source command is executed. TOML diagnostics
   use the parser's message without rendering the whole source excerpt.
5. No live/native/Linux-host acceptance, source version certification or poll/
   shutdown qualification is claimed. Bounded bytes do not bound filesystem
   latency. Full M3 attention/binding acceptance remains outstanding. No release.

Recommended next bounded work: settle the durable identity and coverage contract
for M3a.2 before neutral-state/cache mapping or runtime integration. M3b structured
requests and verified pane bindings remain separate. No subsequent slice started.
