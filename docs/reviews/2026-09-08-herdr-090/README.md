# Herdr 0.9 upgrade receipt

Date: 2026-09-08. Tested the current inline checkout based on `efcd87d`, with
all prior changes preserved. This is local compatibility evidence, not a
clean release commit or native visual acceptance.

## Installed binary and protocol

Official stable macOS ARM64 Herdr 0.9.0, published 2026-09-07; binary SHA-256:
`32b53df09872628059c789a69f02a6b8e29e14ddf26711421f3463f70c1aef17`.
The user stopped the old server before installation. The old executable was
backed up in the run's local audit directory. Final `herdr status` reports
client 0.9.0, JSON protocol 22, endpoint generation 1 and server not running;
`herdr config check` passes. User configuration was not modified.

## Verification

- [Full gate](verify.log): `just verify` exits zero; 564 Rust tests across
  50 test runs, 28 shell tests, formatting, warning-denied Clippy and shell syntax.
- [Release build](release-build.log): `cargo build --release` exits zero.
  Questmancer binary SHA-256:
  `ba8a3a719ea8dfcbb4aca694b30d3eb6d2b5e63f4abfda33417bd05b5975b466`.
- Tests first reproduced protocol rejection, stale bootstrap state, changed
  topology, unsupported post-subscription protocol, stale queued completion
  and mismatched pane responses. Focused regressions pass with the changes.
  Failure and shutdown tests cover baseline and status reads.
- [Isolated production probe](isolated-probe.json): actual client, supervisor,
  adapter and reducer exercised against Herdr 0.9.0. It covers current pane
  metadata, snapshot/subscriptions, managed-pane exclusion, recent output,
  owned-pane focus, five plugin actions, metadata, urgency sorting and
  blocked/idle/unknown/working reports. Synthetic identity and test-created
  plugin link were released; the owned server exited zero and removed its socket.
- [Sidebar validation](sidebar-validation.json): the README, three static
  recipes and new conditional recipe all pass the installed Herdr config checker.
  Explicit colours measure at least 5.81:1 against `#1e1e2e`, with faint disabled.
- `git diff --check` passes. Probe stderr is empty; the server launch log
  contains only its startup and headless-use guidance.

Real protocol captures are retained under
[`tests/fixtures/herdr/0.9.0`](../../../tests/fixtures/herdr/0.9.0/README.md).
The 0.8.2 captures remain historical rejection fixtures.

## Open acceptance

No shared server was started, no user sidebar rows were applied, and no SSH
machines were configured. Native portraits, conditional sidebar appearance,
Librarian/pilot/room visuals, real-agent completion and acceptance from the
clean eventual release commit remain open. Published archives were not rebuilt
or requalified in this slice; the September 6 distribution receipt is historical.

See the [upgrade assessment](../../plans/2026-09-08-herdr-090-upgrade.md) and
[conditional sidebar proposal](../../design/questmancer-sidebar-09.md).
