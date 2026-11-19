# Production Quest Inbox dogfood

2026-11-18 · Herdr 0.9.5, protocol 22 · Pi 1.5.2 · Codex CLI 0.160.0

| Field | Receipt |
| --- | --- |
| result | complete |
| claim | Three scripted review cases passed in the production Board Inbox. |
| evidence | Live managed-pane input, real Pi and Codex turns, host Git checks, durable files, timed wake and focus confirmation. |
| remaining uncertainty | Unassisted human review, Ghostty appearance and the cost of routinely preparing receipts. |
| next | Build and qualify opt-in startup, then audit the art. Human review comes last. |

The trial used the release binary built from `a55c833` plus the local Board Inbox changes.
Its SHA-256 was `33a910b1fa794bcd8386732f90a084de1c0ca1b6743c4ed6437d283b81ad108d`.
The operator used a disposable Herdr server, a source-linked Questmancer plugin,
a separate profile and one local Git exercise repository. Pi and Codex supplied
short label changes in real turns. The operator applied the returned text as data,
committed it and ran the required checks against clean revisions. The agent reports
supplied context. Check provenance came from the host executions.

## Revision reviews

Both comparisons started on the Board with the quest selected. The Board path
opened one shell pane for the diff and check revisions and then returned to
Questmancer. The Inbox path used the managed Questmancer pane throughout. These
counts track visits to evidence panes; they exclude the return to Questmancer.
Evidence was prepared once for both paths, so the counts describe the review
interaction with receipts available.

| Quest | Git base | Candidate | Passing check | Observed outcome | Evidence-pane visits, Board → Inbox |
| --- | --- | --- | --- | --- | --- |
| Pi · Repair stale check copy | `6ec76c61105a` | `f1a507fd7b86` | `f1a507fd7b86` | The required check and candidate matched. The operator confirmed settlement and the owned review retained the exact candidate and reviewer. | 1 → 0 |
| Codex · Clarify scope action copy | `f1a507fd7b86` | `62df7605b7e1` | `dd202aa69c2e` | Import refused the earlier passing receipt. The review stayed open and the Inbox asked for the required check. | 1 → 0 |

The changed-file summary included `labels.json` for Pi and `labels.json` plus
`review-note.md` for Codex. The host verified that the Codex check passed on the
intermediate commit before adding the review note. The review decision therefore
depended on the exact revision even with a real earlier pass for the same recipe.
The comparison scripts knew each expected outcome, and the Inbox ran second.
Timing from the scripts cannot establish how quickly a person would decide.

## Timed deferral

The scope decision was deferred from the managed Inbox for 15 minutes. The
same deadline, **21:36:04.456 UTC**, reappeared after reopening the profile.
Without a keypress, the Inbox's due count rose from one quest to two. The first
sample after the transition was **124 ms** after the deadline. Opening the row
then showed **Answer decision**. The Board showed the same action, with zero
visits to another pane in either flow. The authoritative request stayed open
through deferral, restart and wake.

## Managed terminal and reconnect

- Questmancer opened the Board and read the Inbox with the managed pane at 104
  and 58 columns. A real Herdr client changed the viewport. Managed-pane keys
  were sent with the Herdr pane API. Escape returned from the review to the
  Inbox at the narrower width.
- Observe resolved the linked, qualified Pi session and Herdr confirmed focus
  on its pane. The Codex turn used the ephemeral CLI and had exited by review
  time. The Codex evidence and review remained available through the Inbox.
- Restarting explicitly into Delve and Guild restored the same owned profile.
- In a separate process with a copy of the trial profile, a local socket bridge
  dropped the live connection. The Inbox and defer entry stayed readable. The
  release binary attempted reconnect, then obtained a fresh snapshot and event
  subscription when the bridge reopened. The managed pane continued its timer
  throughout.

## Cleanup and rollout

The managed process exited and removed its runtime registration. The trial
released the Pi identity, closed its panes, unlinked its plugin and stopped its
private server. Its API socket and temporary Pi credential reference were
removed. The existing shared server and party were outside the trial.

The prepared-evidence comparison saved an evidence-pane visit in both revision
cases. The [rollout plan](../../plans/2026-11-18-production-quest-inbox.md)
places opt-in startup and the art audit before the final human Ghostty review.
