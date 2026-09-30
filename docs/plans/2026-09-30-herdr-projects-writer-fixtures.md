# Proposed upstream-writer fixture qualification

Status: proposed, not promoted or implemented. The [M3a.4 prerequisite audit](../reviews/2026-09-30-herdr-projects-acceptance/README.md)
found no installed Herdr Projects, Herdr 0.9.0 below the pinned plugin minimum,
and normal thread creation's automatic ticker startup. The promoted operational
trial remains blocked. This is a smaller evidence step with different acceptance
limits, not a workaround claiming that trial passed.

## Outcome

Prove that records produced by the inspected Herdr Projects 0.2.34 writer functions
are accepted and projected literally by Questmancer. Current fixtures are authored
by Questmancer tests; using the upstream writer closes a distinct schema/locking
assumption before spending effort on an installed runtime.

## Bounded work

1. Copy the clean pinned source at `4e4548c3c43888e6be1c96906215998f07dc63f0`
   into one new temporary directory and record its original hashes. Inspect any
   upstream agent instructions. Build with its locked dependency versions; do not
   install or update binaries. Missing dependencies or host build failures are
   recorded prerequisites, not grounds to change the source pin.
2. Add only a test-only harness in that temporary copy. Reuse upstream project
   creation, thread allocation/update, atomic writes and locking. Use explicit
   test paths and upstream's test environment/fake runner; do not repurpose the
   process home or run the production CLI. No production upstream code is patched.
   Record the harness diff and compiled test identity. Any attempted external
   command fails the harness rather than reaching Herdr or an agent.
3. Produce one disposable project and at most three retained records: initial
   allocation, title/report-reference update and literal closure. Preserve the
   upstream-produced bytes and writer provenance. A locally generated report is
   labelled fixture content, not agent output or delivered work.
4. Feed those physical records to Questmancer's existing reader and acknowledged
   mapper/cache path. Verify strict decode, Partial/unsupported capabilities,
   Unknown/Closed semantics, updated report-reference hash, unchanged recapture
   and restart reconciliation. Reuse the existing composed harness, avoiding new
   production entry points or CLI/source controls. If repository tests change,
   add one focused failing compatibility test before a fix and run affected suites
   plus `just verify`.
5. Preserve the exact writer bytes, harness provenance, results and owned-resource
   cleanup in a new receipt. Remove only temporary resources created by this run
   after collecting evidence. Keep original qualification and approval manifests
   intact. Work inline in Questmancer; no worktrees or upstream contribution.

## Acceptance limits

No installed-source version, production CLI workflow, real agent, live Herdr,
native input, attention usefulness or authoritative identity is qualified by this
slice. It creates no agent/ticker/coordinator process, source command capability,
binding, summons, planning, memory ownership, replay, success inference or XP.
The adapter remains a read-only projection with Partial coverage.

After this evidence step, propose a separate compatible-runtime scope with exact
temporary binaries and an explicit account of any upstream workflow launches.
Do not upgrade the user's Herdr or treat this proposal as permission to do so.
