# L1b native gate: availability check

Date: 2026-10-03. The user approved the L1b production form pack; see the
[visual approval](../2026-10-03-owned-authoring/visual-approval.md). All current
source, engineering logs, review cells/images and candidate binary matched their
qualified hashes before this documentation slice.

The next native gate cannot be run through Computer Use. The current call
`cua.getApp("Ghostty")` returned:

> Computer Use is not allowed to use the app 'com.mitchellh.ghostty' for safety reasons.

This is an access restriction, not a Questmancer defect or a passing native test.
No screen, input, native workflow, test store, process, Herdr pane/server or plugin
resource was created or operated by the check. No alternate UI automation route
was used. Native interaction/restart, managed-pane appearance and real Herdr use
remain unperformed. No cleanup is claimed for resources that were never created.

The [manual acceptance plan](../../plans/2026-10-03-owned-authoring-native.md) is
concrete and isolated: a hash-checked candidate, new physical state directory,
Herdr environment removed, five actions, attention/seen/defer, native resize and
two launches against the same store. It retains test state for evidence and
explicit cleanup. The human must run and observe it; fixture/PTY evidence cannot
replace that observation.

No product code changed, so full engineering tests were not repeated. Documentation
links, source/binary/evidence preservation and `git diff --check` are recorded in
[verification.json](verification.json). Work remains inline, uncommitted and
unreleased. L2–L5, installation, shared-server work and commit/release are separate.
