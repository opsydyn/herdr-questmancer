# Native Ghostty/Herdr acceptance trial — 2026-11-16

- result: blocked
- claim: The rebuilt candidate and isolated synthetic party pass automated gates; native appearance and human interaction await review because Computer Use denies Ghostty access.
- evidence: `verification.json`, `receipt.json`, `capability-results.jsonl`; fresh release build and full gate; six owned foreground receivers and a managed probe pane retained at the user's request.
- remaining uncertainty: Native cards/Ledger, animation playback and relative placement, visibility propagation, mouse/resize/input and owned authoring/restart have not been observed in Ghostty.
- next: Attach the isolated session, refresh the owned Questmancer pane's graphics detection, and complete the human checklist below.

## Host-captured results

Installed Herdr is 0.9.5 / protocol 22 and Ghostty is 1.4.0. A fresh release build
passed after authorized dependency downloads. The current full gate passed 789
Rust tests / 64 runs, 28 shell tests, format, denied-warning Clippy and shell syntax.
The first sandbox run could not bind seven disposable Unix sockets; its authorized
rerun passed. Raw logs are `/tmp/questmancer-native-build-20261116.log` and
`/tmp/questmancer-native-verify-20261116-authorized.log`.

The isolated server started empty. Six synthetic adventurers use owned `/bin/cat`
foreground receivers. Working, blocked, resting and unknown projections, repeated
open, Guild/Delve/Guild actions, working/blocked/working transitions and managed-pane
exclusion passed. No user configuration, normal server or real agent was operated.
Source changes from earlier owned-work/compatibility work are preserved.

Computer Use returned: `Computer Use is not allowed to use the app
'com.mitchellh.ghostty' for safety reasons.` No alternate Ghostty UI automation
was used. This prevents automated native visual acceptance; it does not invalidate
headless protocol results or provide native approval.

## Attach and native review

Run in Ghostty:

```bash
python3 /tmp/qmnative-8hk1qa2m/control.py attach
```

This command strips inherited Herdr context and uses only the trial's temporary
config/state/cache. It attaches to the retained test server, not the normal server.
From a second terminal, **before entering any authoring draft**, run:

```bash
python3 /tmp/qmnative-8hk1qa2m/control.py refresh
```

Refresh closes only the recorded trial Questmancer pane and opens a new instance
so its standard terminal query runs after attachment. The initial headless startup
cannot establish native portrait capability. Do not use refresh with an unsaved
draft: the recorded pane is intentionally restarted. The trial ledger is updated.

1. Review Hall (`1`) and Delve (`2`): six actors, truthful state cues, coherent
   selection and names. Review each room at wide and narrow native sizes.
2. Select an adventurer and inspect its card. Open `?` for the Ledger. Record
   native versus authored fallback, layout and mouse/keyboard behaviour separately.
3. Select the "Capability probe" tab, press Return, and review the bounded demo.
   It uses only its own image IDs, deletes them afterward, restores raw mode and
   restores visibility reporting if the probe enabled it. Parent green/purple
   changes and a yellow child to its right require actual observation.
4. Repeat the probe after moving between visible/hidden outer tabs as useful.
   A report is not proof that visibility tracks individual Herdr panes. Missing
   feature-query support is unknown, not proof that animation itself is absent.
5. For owned authoring, use the trial's Board `b` → Actions `a`. Create the proposed
   local store in this temporary trial state. Create a campaign and quest, request
   and explicitly answer a decision, register a synthetic artifact reference, then
   review narrow/below-minimum input and restart. Follow the detailed
   [human checklist](../../plans/2026-10-03-owned-authoring-native.md), using the
   current candidate hash from `verification.json`. No authority is prepopulated.

No clipboard read, OS clipboard permission, real-agent counsel, external source,
URI opening or production renderer change is part of this trial. Graphics-off
native appearance requires a separate owned configuration run; the previous
headless fallback evidence does not qualify it.

## Headless capability baseline

`capability-results.jsonl` contains protocol replies from the managed probe before
native attachment:

| Probe | Observed reply | Qualified claim |
| --- | --- | --- |
| Cell geometry CSI 16t | No reply | Geometry through a native attached client remains unverified |
| Visibility mode/query | Mode recognised and reports returned | Herdr parser responds; actual outer/inner visibility transitions unverified |
| Basic Kitty | OK | Basic image query acknowledged |
| Animation frame upload | OK with frame index | Frame accepted; visible playback unverified |
| Parent/child placement | OK | Placement accepted; visible position/lifecycle unverified |
| Enhanced animation/relative feature queries | EINVAL: dimensions required | Feature-query extension not established; do not infer full feature support |

Sources for diagnostic commands: [Kitty graphics specification](https://sw.kovidgoyal.net/kitty/graphics-protocol/).
The scripts are diagnostic producers, not another Questmancer renderer. Native
review is still pending. New user-triggered responses append to the live trial
file; the repository copy is the headless baseline only.

## Retention and eventual cleanup

Trial root: `/tmp/qmnative-8hk1qa2m`. The resource ledger is its `receipt.json`.
The running server, receiver panes, managed Questmancer and probe pane are retained
for the requested review. Cleanup is deferred, not claimed. To clean up later:

```bash
python3 /tmp/qmnative-8hk1qa2m/control.py cleanup
```

It verifies the original socket identity and recorded pane set, releases synthetic
identities and stops only this test-owned server. Unexpected resources require a
fresh ownership audit. Files remain for evidence; remove the trial directory only
after recording results and confirming all owned processes have exited.

Recommendation: **needs follow-up** for human native observations.
