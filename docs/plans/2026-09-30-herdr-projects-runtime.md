# M3a.3 opt-in runtime integration

The user approved the M3a.2 production review and promoted the next proposed
runtime slice. Work inline and preserve the existing dirty tree. No live homes,
agent launches, installation, upstream edits or shared-server changes are included.

1. Parse a disabled-by-default `[herdr_projects]` selection with explicit source
   and campaign tokens, absolute project path, pinned format and retained-record
   policy. Startup loads the existing atomic cache/checkpoint once. Invalid
   configuration or protected cache cannot enable a producer.
2. Extend SourceInbox with a bounded, acknowledged source-observation path.
   Map in the consumer against the current model, publish the atomic pair, then
   acknowledge the checkpoint. Only one capture/observation is outstanding.
3. Own a read-only descriptor lease on the selected root. Check it before and
   after each capture; observed loss/replacement and hard identity errors latch
   reconciliation. Transient parse/lock/coverage/publication failures retain
   work and retry; recovery starts a fresh observer generation. Never rediscover
   or silently attach a replacement. Restart/rebind remains an explicit owner
   operation, under the documented continuity assumption.
4. Poll off the UI thread every five seconds; bounded failure backoff reaches
   thirty seconds. Use a two-second cooperative capture budget and cancellation
   checks between bounded regular-file reads. Shutdown cancels and joins the
   owned producer/capture before persistence flush. OS filesystem calls cannot
   be forcibly interrupted; do not claim a hard deadline for a stalled filesystem.
5. RuntimeLifecycle owns this source independently of Herdr, including offline
   mode. Use the existing Board/Table and freshness; retain one ephemeral source
   diagnostic for first-read failures and latched reconciliation. No bindings,
   structured summons, source commands, inferred success, Chronicle or XP effects.
6. Test-first disposable trees: baseline -> edit -> busy/parse/disconnect ->
   retained recovery -> restart, omission recovery, identity latch, root replacement,
   bounded queues/acknowledgements, timeout/cancellation and source-byte invariance.
   Measure capture at 1/10/100 threads. Run affected suites, full verification,
   4,096-case property gate, release build and production diagnostic exports.

M3a.3 is engineering-qualified; the [receipt](../reviews/2026-09-30-herdr-projects-runtime/README.md)
records 754 Rust tests/62 runs, 28 shell tests, the 4,096-case property gate and
release build. The user subsequently
[approved the diagnostic images](../reviews/2026-09-30-herdr-projects-runtime/visual-approval.md).
This completes the synthetic inventory slice; it remains unreleased.
Installed-source compatibility, native input, live Herdr and five-second usefulness
remain unperformed gates. The next [bounded acceptance plan](2026-09-30-herdr-projects-acceptance.md)
was promoted. Its [prerequisite audit](../reviews/2026-09-30-herdr-projects-acceptance/README.md)
stopped before resource creation; the operational trial remains blocked. M3b and
later roadmap milestones remain unpromoted.
