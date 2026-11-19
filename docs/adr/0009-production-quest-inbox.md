---
status: accepted
date: 2026-11-18
decision-makers: Questmancer owner
---

# Keep Quest Inbox review decisions in owned work

## Context

The Storybook study uses examples whose evidence, triage and settlement live in
memory. Production already has source-qualified quests and review obligations,
an incarnation-keyed attention cache, a serial owned-work writer and
ephemeral, verified Herdr bindings. Artifact registration supplies a reference.
The current owned command cannot establish that a required check ran against
the candidate under review. The review needs that contract before a settlement
can survive restart and remain meaningful.

## Decision

Build on the existing Board and its work projection. Use the source-qualified
quest ID for a row and the obligation incarnation for its triage and review
actions. A review has a pinned repository, base, candidate and required check.
The owned backend validates and records the explicit human outcome for that
review. Recording the decision and resolving that incarnation share a commit.
Use the existing serial writer and receipt recovery path. If persisted types
change, specify and test a versioned migration and preserve protected-file
behaviour before shipping.

| Fact | Owner and treatment |
| --- | --- |
| Campaign, quest, review request and settlement | The owned backend for its own quests. Upstream sources retain authority over their work. |
| Commit and change summary | The selected Git repository at immutable object IDs. |
| Check results and reports | The check runner supplies attributable results for the pinned revision. Reports are attributed context. The trusted receipt entry point must be separate from adventurer report intake. |
| Seen and deferred state | Existing local intent, scoped to the current obligation incarnation. |
| Adventurer, pane, session and focus | Herdr. The association is optional, validated live and discarded on identity loss. |

Store the minimum evidence references and the human's decision. Keep diff
bodies, check logs, live binding identities and terminal output out of durable
Inbox data. Collect the selected evidence on demand. Where a repository cannot
be read or a check receipt cannot be validated, keep the review unresolved and
show the reason. Settlement applies to the recorded candidate and review
incarnation. A later candidate needs its own review.

## Options considered and trade-offs

An improved Board with evidence links is the smallest fallback if grouping
provides little benefit. It reuses existing navigation and leaves evidence
comparison across surfaces. A second standalone inbox store would provide a
separate extension point but make attention and resolution harder to reconcile
with the Board after concurrent updates or restart. Herdr already supplies
execution and pane navigation. Its agent activity does not supply the reviewed
revision or the human decision needed here.

The owned path pays for strict evidence capture and migration testing. It keeps
operation local and allows review when a pane is absent. The initial check
receipt is an operator-attested local record with an explicit trust boundary.
It does not claim independent remote verification. Branch heads can move during
review, so the immutable revision and capture time remain visible even after a
refresh. The existing work model remains the source for shared attention order.

## Opt-in startup

Keep `inbox_on_start` in user-owned configuration. Startup opens the existing
Inbox overlay over the selected world after cache restoration. The overlay's
return target is transient and explicit world launches take precedence. This
keeps both the persisted world selection and the Herdr launch contract intact.

## Decision challenge

| Proposal claim | Current evidence | Gap to close |
| --- | --- | --- |
| Grouping by quest reduces navigation. | The Storybook keyboard path works with three fixtures. | Measure next-action accuracy and pane opens on three real work items. |
| Reviewers can rely on matching checks. | Storybook refuses an older fixture check. Git and the Herdr/Pi/Codex executables are present locally. | Qualify a real repository/base/head and independently attributed check receipt. Verify that evidence can be collected without trusting agent reports. |
| Triage and settlement can recover after restart. | Existing code persists incarnation-scoped intent and owns decision answers with serial command receipts. | Prove the new review command and its evidence policy across reopen, retries and partial publication. |

The strongest reason to stop is collection cost. If assembling evidence takes
more effort than the pane visits it saves, a grouped UI adds another place to
maintain. The smallest credible slice is the three-case baseline with evidence
collected from real work. Repeatable provenance and unassisted next-action
results inform rollout in the [production plan](../plans/2026-11-18-production-quest-inbox.md).

**Verdict** — `proceed` with the Board-first build, as subsequently promoted by
the Questmancer. After the scripted dogfood, qualify opt-in startup locally and
defer human review until after the art audit.
