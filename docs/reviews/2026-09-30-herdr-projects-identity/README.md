# M3a.2 identity/coverage design — 2026-09-30

**The approved documentation slice is complete.** The
[selected contract](../../design/herdr-projects-identity-coverage.md) uses explicit
registration plus retained thread records for a bounded read-only inventory.
Native incarnation guarantees stay unsupported. The
[implementation proposal](../../plans/2026-09-30-herdr-projects-adapter.md#m3a2-mappercache-implementation-proposal)
is concrete but unpromoted: pure mapping, neutral domain states, atomic v2
projection/checkpoint and synthetic restart proof before runtime attachment.

## Findings and consequences

The inspected allocation/resolve/reopen/sweep paths retain numeric thread records;
rename moves their containing project. This supports a retained-record policy,
not a native UUID guarantee. Equal creation timestamps and identical restored
records remain ambiguous. Changed creation evidence or observed replacement
requires explicit reconciliation; no hash/path/timestamp silently mints identity.

Durable mapping requires the existing cooperative lock, always emits partial
coverage and guards omissions of every known thread before reduction. The current
reducer otherwise marks the merged partial inventory Current. Latest artifact
reference slots must explicitly revoke usable links on missing/changed evidence.
No grouping/report/acknowledgement creates or resolves a human obligation.

The existing projection carries observation ordering but not registration/creation
evidence. The plan puts a bounded typed checkpoint in the same v2 atomic work-cache
envelope, advancing only after storage acknowledgement. It preserves v1 work and
local intent, future-file protection and the source-neutral reducer. The existing
inbox send is not a storage acknowledgement; producer acknowledgement and root
lease handling remain explicit M3a.3 prerequisites.

## Evidence and limits

Questmancer baseline remains dirty `main` at
`f380720ba63424b279980ff4f1c164d8fd4ef735`; M1/M2, M3a.1 and independent pane-header
work were preserved. Upstream was read in the already fetched temporary checkout
at `4e4548c3c43888e6be1c96906215998f07dc63f0` (Herdr Projects 0.2.34); it was not
built/run/installed. No real home, socket, pane, server or agent was operated.

[Inspection/validation manifest](inspection.json) records file hashes and checks.
Changed-document links, pinned source line anchors, machine-readable manifests and
`git diff --check` were checked. Source/test/configuration files were unchanged
by this slice. No tests or full engineering gate were rerun for documentation-only
changes; the retained M3a.1 receipt is separate evidence, not proof of this design's
future implementation. The specified mapping/cache vectors remain unexecuted.

No implementation, native/visual, live integration, release, publication or cleanup
acceptance is claimed. Mapper/cache implementation, M3a.3 runtime, M3b and later
roadmap phases need their own promotion.
