# M3a.4 installed-source and isolated-runtime acceptance

Status: promoted by “Commit and move to next”. The prerequisite audit is complete;
the operational trial is blocked before resource creation. See the
[audit receipt](../reviews/2026-09-30-herdr-projects-acceptance/README.md).
Herdr Projects is absent from PATH and local plugin registration; the installed
Herdr reports 0.9.0, below the pinned source's 0.9.1 requirement. Normal upstream
thread creation/adoption starts a ticker and cannot satisfy this trial's no-launch
scope. No live trial, installation or source mutation was performed.

The user previously approved M3a.3's synthetic diagnostic visuals, closing the
[runtime slice](2026-09-30-herdr-projects-runtime.md). The steps below retain the
promoted trial's scope. A smaller [upstream-writer fixture proposal](2026-09-30-herdr-projects-writer-fixtures.md)
is the recommended next step; it has not been promoted.

## Outcome and scope

Establish whether the accepted read-only inventory projection works with records
written by the pinned upstream implementation and through an owned Herdr runtime,
including recovery and restart. This tests the remaining integration assumptions
before adding requests, bindings or other roadmap features.

Work inline, preserving the dirty checkout. Use one new physical temporary project
with at most three retained thread records, isolated source/config/state homes and
fresh registration tokens. Runtime testing may use one test-owned server, one
workspace and one Questmancer pane. Add a disposable plain pane only if upstream's
supported record-writing path requires it. No coordinator, ticker or agent launch
is included. No real project home, existing plugin registration or shared server
is modified; no installation, upgrade, relink of an existing plugin or release
is included. Questmancer remains a reader.

## 1. Establish prerequisites before mutation

Record resolved binary paths, hashes, versions and supported isolation options.
Compare Herdr Projects with the [inspected 0.2.34 contract](../reviews/2026-09-30-herdr-projects-source/README.md)
and commit `4e4548c3c43888e6be1c96906215998f07dc63f0`. Re-inspect the command
handlers for the intended project/thread operations: help output alone does not
prove they avoid automatic agent, ticker or coordinator startup.

The inspected plugin requires Herdr 0.9.1, while Questmancer's current transport
targets 0.9.0/protocol 22. Verify the actual protocol and plugin compatibility;
do not infer compatibility from a greater version number. Check that all socket,
configuration, plugin registration and source-home writes can be contained in
test-owned locations. Freeze the Questmancer source and candidate binary hashes.

If the compatible binaries are absent, the installed layout differs, isolation
is incomplete or supported record creation requires excluded launches, stop and
report the concrete prerequisite. Do not install, upgrade, patch upstream or
silently replace upstream-created records with handwritten fixtures. Approval
of this plan authorises the bounded trial only when these prerequisites hold.

## 2. Exercise upstream-written records

Before creating resources, initialise a ledger of owned paths, processes, sockets,
registration and pane IDs. Use supported upstream operations whose inspected
effects fit the scope to create the disposable project and retained records.
Record the exact commands, record contents and upstream writer provenance.

Observe baseline inventory, a title/report-reference change and literal source
closure. The Board/Table must preserve attribution, neutral Unknown/Closed states
and Partial coverage. Closure must not create successful delivery, XP or summons;
unsupported bindings must not permit focus. If an upstream report writer cannot
be exercised without a launched agent, record that case as unperformed.

During intervals with no upstream operation, compare source-tree file hashes
before and after Questmancer capture. Separate authorised upstream changes from
reader activity so source-byte invariance has a meaningful comparison.

## 3. Exercise the owned runtime and restart

Follow the resource and cleanup rules in the [guarded scene procedure](../manual-test/questmancer-scene-preview.md),
using freshly observed IDs and only this trial's server/registration/panes.
Use the [inventory configuration guide](../manual-test/herdr-projects-inventory.md)
with isolated durable state. Record the actual plugin runner and binary.

Exercise baseline -> upstream change -> writer-busy/unavailable -> recovery ->
Questmancer close/reopen -> cached/unverified -> new acknowledged baseline.
Use only supported upstream locking operations or a separately identified owned
test lock; the latter is fault injection, not evidence of upstream concurrency.
Check that unchanged observations preserve cache bytes and that work survives
restart without becoming current until a new baseline is accepted.

Replace only the disposable selected root to prove attachment stops and retains
facts. Owner reconciliation uses fresh tokens for replacement work and a new
Questmancer start; replacement must never attach silently. Record timings and
diagnostics against the documented poll/backoff policy, without claiming a hard
filesystem deadline. No Questmancer source commands, bindings or execution effects
are introduced.

## 4. Keep interaction evidence explicit

Inspect production Board/Table details at the existing 48x28 and 120x38 review
sizes, including long diagnostics, scrolling, selection and `Esc`. If a native
terminal is available, record the terminal, pane transport and actual inputs.
If native access is unavailable, retain PTY/cell evidence as runtime engineering
evidence and leave native interaction acceptance open. The approved fixture images
do not close that gate. No real-work or five-second usefulness claim follows from
this three-record trial.

## 5. Cleanup and receipt

Close only ledger-owned panes and release any owned synthetic identity first.
Stop only the test-owned server, unlink only registration created by this trial,
and remove only owned temporary resources after preserving evidence. Restore
original focus only when the same original pane still exists. Confirm process,
socket and registration cleanup, inspect plugin logs and final `git status`.

The receipt records prerequisite versions/protocols, exact hashes and commands,
resource ownership, before/after source and cache hashes, observed transitions,
native evidence or its absence, cleanup and any failed/unperformed cases. A
failure produces a bounded diagnosis and proposed next step; it does not expand
the trial into implementation or upgrade work. Existing automated qualification
remains separate; rerun affected/full gates if any code is subsequently authorised
and changed. M3b, memory, replay, clean packaging and release remain unpromoted.
