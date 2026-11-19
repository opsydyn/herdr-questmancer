# Production Quest Inbox plan

Proposed on 2026-11-18. The Board-first build, scripted dogfood, opt-in startup
and native portrait pass are complete. Human Ghostty review remains pending.

## Outcome

When several adventurers are active, the Questmancer can see which quest needs
human attention, what action is due and what evidence supports it. A review can
be settled explicitly for the revision that was actually checked. Reopening the
app restores deferred summons and recorded review decisions.

Before the production Inbox, the reviewer checked changes, tests and
adventurer reports across the Board, terminal panes and Git. Storybook
exercised the keyboard flow and stale-result refusal with fixture data.
The scripted production trial measured evidence-pane visits, with unassisted
decision time still open for the final review.

[Quest Inbox study](../reviews/2026-11-18-quest-inbox/README.md) ·
[Owned work contract](../design/questmancer-owned-work.md) ·
[Owned-work review decision](../adr/0009-production-quest-inbox.md)

## Phase 0 · Baseline three real reviews

Time-box to one 60-minute dogfood session. Use one Pi quest, one Codex quest and
one follow-up review with evidence from an earlier commit. Choose real,
bounded changes with a known acceptance criterion and a measurable check. Use a
separate Questmancer profile and owned store for the exercises. Start with the
current Quest Board and manual review workflow. Sketch the same three rows and
their evidence on a review sheet using the real facts. Keep the original order
of actions in the notes so the second pass is understood as an informed replay.

| Case | Expected reviewer action | Evidence to record |
| --- | --- | --- |
| Current revision | Compare the scoped change and tests, then decide whether the acceptance criterion is satisfied. | Quest ID, source of each fact, Git base and head, check revision and result, outcome, terminal pane opens. |
| Earlier passing result | Identify the mismatch and request or run checks for the candidate revision. | Both revisions, the next action the reviewer identifies, whether the old green result would have been accepted. |
| Deferred summons | Defer a real human request, restart with the same profile and recover the request and its deadline. | Seen/deferred state before and after restart, return to the due queue on wake, outcome of a pane change or loss if observed. |

For each case, record whether the reviewer identified the correct next action
unassisted, the time to that decision and the number of terminal pane opens.
Count deliberate revisits to a pane. Record missing provenance and the effort
needed to collect each evidence item. Both agent types must be represented.

Advance if the review sheet supports all three correct decisions, would save at
least one otherwise necessary pane visit in two cases, and shows an exact
candidate and check revision for both revision cases. Treat estimated savings as
a small-sample pilot signal. If evidence is hard to obtain reliably, fix that
seam first. If grouping adds little value, improve the existing Board. An
incomplete session leaves the launch decision open.

The [2026-11-18 production dogfood](../reviews/2026-11-18-production-inbox-dogfood/README.md)
exercised these cases with real Pi and Codex turns on changes in a disposable
Git repository. The scripted comparison saved one evidence-pane visit in each
revision case. The host observed the deferral wake at its original deadline.
Unassisted human next-action accuracy remains unmeasured.

## Phase 1 · Ship review and triage inside the Quest Board

The local build exposes the Inbox with `b`, then `i`, and starts reviews from
an owned quest with `v`. The engineering suite and an isolated offline PTY run
passed. The PTY run exercised the real Git diff, stale receipt rejection,
deferral across restart, explicit settlement, compact resize and clean exit.
The managed Herdr dogfood also passed the scripted workflow and resize checks.
Human Ghostty acceptance remains pending.

Proposed appetite after the baseline is five engineering days in two slices.
Use three for revision-bound review and durable settlement, then two for the
Inbox surface and terminal qualification. If the evidence path needs custom CI,
a daemon or general workflow automation, stop and shrink the evidence scope.

### Review and settlement

1. Start with one explicitly selected local Git repository and Questmancer-owned
   quests. The reviewer creates a review request against a named criterion, an
   exact Git base and a committed candidate. The request has its own incarnation
   and uses the existing review obligation. A replacement candidate has a new
   review incarnation and preserves the earlier decision as history.
2. Show the changed-file summary for the pinned base and candidate, the required
   check result, and a short attributed Pi or Codex report when supplied. Run
   read-only Git collection on demand, outside the renderer. Import one bounded
   result format from an explicitly trusted local runner. The operator runs one
   approved project check against a clean checkout of the candidate. The receipt
   identifies the repository, full candidate ID, check recipe, runner, timestamp
   and result. An adventurer's report supplies context. Show any registered PR
   link. Live PR status is a follow-on.
3. Enable **Settle review** after the reviewer sees the criterion, pinned
   change and matching required result. Require a deliberate confirmation that
   names the quest and revision. Treat unavailable, failed, mismatched and
   inconclusive required checks as unresolved. Refresh the candidate before
   submitting when a moving ref has been selected. If it moved, show the new
   candidate and require a new review. A successful settlement records the
   reviewed immutable revision even if a branch moves immediately after it.
4. Commit the settlement and matching review-obligation resolution together
   through the owned backend. Include the reviewer, timestamp, review
   incarnation, reviewed revisions and evidence references. Recheck the expected
   owned revision and complete evidence at the command boundary. An identical
   retry returns its existing receipt. An indeterminate save uses the existing
   check-outcome flow. A durable commit whose view publication failed reports
   **Saved, view pending** and reconciles on refresh. Settling acknowledges that
   specific review. Quest lifecycle, merge state and adventurer presence each
   retain their own authority.

### Inbox surface

- Add **Inbox** as a page within the existing Board parchment. Propose `i` in
  the Board and show it in the legend. Preserve one row per quest and show a
  separate row for a campaign summons without a quest. Order actionable rows
  using the existing attention rules. Show the next action, then other summons
  on selection. Show a due count by quest, deferred requests with wake times,
  and waiting work only when the source supplies a reason. Label cached or
  unavailable evidence at the point where the reviewer needs it.
- Reuse `!` for urgent work, `j`/`k` for selection, Space for seen, `s` for
  timed defer or restore in the Inbox, `Enter` for review and Esc to return.
  Keep **Observe adventurer** an explicit secondary action. It is available
  only with a verified live association. The pilot can use an explicit
  ephemeral selection of an existing adventurer for an owned quest. Invalidate
  it on disconnect or identity replacement and revalidate before focus. The
  full review remains available without a live pane. Use a visible settlement
  action within review so the Board's existing `a` authoring action stays clear.
- Reuse the existing seen/defer cache and 15-minute deferral keyed by obligation
  incarnation. A seen item stays due. A deferred item returns at its deadline.
  A new review is eligible without inheriting a previous review's defer. At
  compact sizes show action, revision and reason first, with scrollable evidence
  and reachable controls. An undersized terminal can still return to the Board
  or room.

## Phase 2 · Offer an opt-in Inbox home

The Board-first engineering checks and scripted Herdr dogfood are complete.
The `inbox_on_start = true` preference opens the Inbox over the usual room on
ordinary launches. Explicit Guild Hall or Delve launches bypass it. Esc
returns to the room, `b` opens the Board, and an empty Inbox offers store
setup with `a`. Store creation remains explicit. The wide queue shows an
authored magical scroll through the existing half-block renderer.

Startup qualification passed in a disposable Herdr 0.9.5 server using the
release build. The exercise covered the omitted setting, enabled setup flow,
Esc and Board navigation, both explicit room launches and an explicit switch
in an existing pane. It verified that startup created no store. `just verify`
passed 871 Rust tests and 28 shell checks. The 4,096-case property run and the
release build also passed.

## Qualification and boundaries

| Gate | Required evidence |
| --- | --- |
| Domain and persistence | A previously created store opens, and a settlement survives restart. Revision and incarnation mismatches, stale cache, malformed receipts, adventurer reports offered as checks, concurrent writers and repeated commands fail closed or reconcile. A new candidate returns to review. |
| Composition | A current result settles exactly once, an old passing result stays open, a deferral survives restart and wakes, and an absent or replaced pane cannot receive focus. Cache publication failure preserves the authoritative receipt. |
| Terminal | Repeat the three cases through the production entry point in a real Herdr terminal. Exercise keyboard navigation, compact resize, disconnected/reconnect behavior, room return and explicit Guild/Delve launch. Capture the next-action and pane-open measures again. |
| Engineering | `just verify`, `just property-test 4096`, release build and `git diff --check`. Record local startup coverage before moving to the art audit. |

The pilot scope is one local repository, one project check recipe and one
Questmancer profile. Broader lifecycle management, automatic PR polling,
event-based wake rules, Claude, remote machines, chained agents and a shared
inbox are follow-on decisions. The [production walkthrough](../manual-test/quest-inbox.md)
covers the Board path and the local check receipt.

## Follow-on · Art-direction audit

The [full art-direction audit](../design/reviews/2026-11-18-lineup-audit/README.md)
includes a per-character decision, matched framing study and production
render comparisons. The [native portrait pass](../design/reviews/2026-11-18-native-portrait-alignment/README.md)
shows the revised artwork at card size for the final review.

## Final review · Ghostty

Review Inbox startup, unassisted next actions and the artwork together in
Ghostty after the opt-in build and art audit. Use that evidence for rollout.
