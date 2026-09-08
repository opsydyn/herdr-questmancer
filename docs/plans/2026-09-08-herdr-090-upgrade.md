# Herdr 0.9 upgrade and sidebar opportunity

Date: 2026-09-08. The Questmancer asked to upgrade to the latest stable 0.9
before continuing native review, and highlighted conditional sidebar styling.

## What the release unlocks

[Herdr 0.9.0](https://github.com/herdrdev/herdr/releases/tag/v0.9.0), released
2026-09-07, is the current stable release. The
[announcement](https://herdr.dev/blog/connecting-the-machines/) explains the
new client-rendered outer UI and SSH machine federation.

| Change | Consequence for Questmancer |
| --- | --- |
| One Herdr window combines Local and saved SSH machines | A Questmancer instance can remain beside the campaigns on each server. Its JSON socket and pane IDs remain server-scoped; this does not provide a combined cross-machine guild. Keep machine identity visible in Herdr's own sidebar. |
| Client-local navigation and independent viewing sizes | Resize and selection evidence must identify the viewing client. Shared-tab size follows its last interacting client; revisit this in the eventual native multi-client acceptance, not by changing the room renderer. |
| Native pane graphics enabled by default | Optional class cards and the Ledger can use the normal supported transport without an experimental opt-in. Retain authored fallbacks and test the actual transport. The setting is now `terminal.kitty_graphics`; the old experimental key remains accepted. |
| Ordered conditional sidebar token styles | Use current condition and vigil text for restrained urgency and state emphasis. Rules match the displayed token's own value, so a hidden `$quest_rank` cannot recolour an agent name or another token. Rich hoard strings are not plain numbers. |
| More accurate blocked/working detection and recent reads | Existing presence and lazy scrying benefit from upstream fixes. Keep Herdr authoritative and keep user-composed counsel on its existing correlated text/submit path. |
| Stable endpoint generation and compatible client updates | This is Herdr's binary endpoint contract. Questmancer still uses the JSON API and retains its exact tested protocol gate for this slice. Future capability-based compatibility requires a separate decision. |
| Herdr Cloud and agent collaboration across machines | Roadmap proposals in the announcement, not shipped capabilities or approval for Questmancer cloud services. |

Sources: the [tagged API guide](https://github.com/herdrdev/herdr/blob/v0.9.0/docs/next/website/src/content/docs/socket-api.mdx),
[machine guide](https://github.com/herdrdev/herdr/blob/v0.9.0/docs/next/website/src/content/docs/connecting-machines.mdx),
and [sidebar rule implementation](https://github.com/herdrdev/herdr/blob/v0.9.0/src/config/sidebar/rules.rs).

## Required compatibility work

Protocol moves from 20 to 22. JSON request/event shapes used by Questmancer are
otherwise compatible in the compared schemas; new capability fields remain
additive. Real 0.9 captures are required, rather than editing old captures.

Lifecycle subscriptions no longer replay retained events. The current
snapshot-then-subscribe sequence can therefore miss a change. Pane-status
subscriptions still require known pane IDs, so discovery must be separated
from the authoritative baseline: discover IDs, acknowledge subscriptions,
then take the baseline snapshot. If its pane set differs, rebuild the
subscriptions before publishing a connected baseline.

Status events have no upstream revision. Buffered status text can therefore
predate the baseline, even though it arrives later. Reconcile such events
with an explicit `pane.get` before projecting them, using its current status
and revision. This is event-driven metadata I/O, not terminal-output polling
or an animation-frame read. A failed read must reconnect rather than invent
a transition. Already-versioned events retain their existing reducer path.

Test first: supported and unsupported versions, post-subscription baseline,
changed bootstrap topology, stale queued completion, fresh status delivery,
failure and shutdown. Then run the affected suites, full `just verify`, a
release build and the isolated production-client probe.

## Ownership and handoff

The user explicitly stopped the previously running 0.8.2 server. Its stopped
status was rechecked before installing the checksum-verified 0.9.0 executable.
The original binary is retained in this run's audit directory. Test servers,
panes, reports and registration must be isolated and cleaned up; no shared
server is started or stopped by the upgrade.

Prepare a concrete, opt-in sidebar recipe using existing tokens. Validate its
syntax with Herdr 0.9.0 and preserve readable contrast and text cues. The
user's global sidebar configuration remains theirs; a proposal does not
establish native visual approval.

Native Librarian/pilot/room acceptance remains open. This slice does not
commit, tag, publish, configure SSH machines or add new Questmancer features.

## Execution

Installed Herdr 0.9.0 from the official macOS ARM64 asset, verified SHA-256
`32b53df09872628059c789a69f02a6b8e29e14ddf26711421f3463f70c1aef17`.
An isolated server supplied protocol-22 captures and cleaned up successfully.
Questmancer now requires protocol 22, takes its authoritative baseline after
subscription acknowledgement, and reconciles unversioned status events through
current pane metadata. Wrong-pane responses fail closed. The real production
client/supervisor/reducer probe passes with owned resources cleaned up.

`just verify` passed 564 Rust tests across 50 runs and 28 shell tests, formatting,
Clippy and syntax checks. The release build and `git diff --check` pass.
The [receipt](../reviews/2026-09-08-herdr-090/README.md) retains the evidence.

The [conditional sidebar proposal](../design/questmancer-sidebar-09.md) and all
four existing recipes pass Herdr's config checker. Foregrounds clear 5.81:1
against the specified background. Native appearance remains unreviewed and
user configuration is unchanged. The next bounded design step is a native trial
of the proposed rows after user approval.
