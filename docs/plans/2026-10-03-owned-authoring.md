# L1b implementation: owned authoring

Date: 2026-10-03. The user subsequently promoted this reviewed implementation.
The slice is implemented; [qualification and review evidence](../reviews/2026-10-03-owned-authoring/README.md)
records the actual gates. Form visuals, native/live use, commit and release remain separate.
Work inline, without worktrees or subagents. Preserve existing uncommitted L1a,
planning and historical evidence. The [form design](../design/questmancer-owned-authoring.md)
is the concrete review target; [L1](2026-10-01-owned-work.md) remains the milestone.

## Outcome and appetite

Complete the five-action human workflow in Board parchment with explicit store
setup, restart and readable answers. One selected store, one owner, one outstanding
mutation. Reuse the existing RGB scene, work protocol and acknowledged cache.
No daemon, database, agent commands, new renderer or autonomous workflow.

## Implementation order after promotion

1. **Focused red workflow test.** Drive proposed Actions/form input against a
   disposable owner and existing persistence worker: Create store/campaign/quest,
   Request/Answer decision, Register artifact, close/reopen and inspect the same
   answer. Start with one failing setup/create-and-remember test; add each next
   behavioural test before its implementation, rather than a large inert test suite.
2. **Selection intent and checked queries.** Add typed path/source pin to state v2;
   migrate v1 without losing personas, attention, experience or preferences. Unknown
   versions and malformed files remain protected. Add narrowly acknowledged
   selection publication with directory-sync failure reporting. Preserve selection
   on later ordinary intent writes. Add checked owner metadata/decision reads and
   typed stale/missing/indeterminate outcomes; no UI file access or cache answers.
3. **Owned runtime lifecycle.** Dedicated serial filesystem worker, bounded typed
   requests/results, generation fencing, one outstanding command and independent
   live/offline operation. Wire explicit Create/Open and remembered startup Open.
   Reuse LocalWorkSource and acknowledged publication, preserving Projects
   checkpoints, source freshness and local attention intent. Show commit and cache
   failure separately. Join the owner before persistence shutdown; document OS-call
   cancellation limits and retained receipts.
4. **Board entry and forms.** Add proposed Board `a`, campaign/decision pickers,
   form focus/editor state and five typed submissions. Preserve Board Enter observe,
   `!`, Space/seen, `s`/defer and counsel drafts. Add bounded literal paste and mode
   restoration, key-repeat suppression for submission, multiline scroll and narrow
   resize preservation. No file I/O in rendering or interaction reduction.
5. **Recovery and answered detail.** Freeze pending requests; retain rejected drafts;
   no implicit stale-revision resubmission. Indeterminate retries reconcile the exact
   receipt. View retry only republishes committed state. Detail responses check
   source, incarnation, revision, request and selection epoch. Resolved decisions
   remain reachable from Decisions, including those without quests.
6. **Qualify and review.** Run affected tests, full `just verify`,
   `just property-test 4096`, release build and `git diff --check`. Add fixed
   Storybook cases using production overlays and no persistent state. Prepare wide,
   narrow and below-minimum review images. Record engineering, visual, native/restart
   and installation gates separately; do not install or operate shared Herdr as an
   automatic consequence of green tests. Update handbook, README, design/status and
   operational recipe in the implementation slice.

## Decisive tests

- No pin: ordinary startup creates no owned file/lock. Valid pin: restart opens the
  exact owner. Missing/busy/replaced/future/malformed selection never creates a
  replacement. Protected intent blocks setup before authoritative creation.
- Create succeeds but selection publication fails: retain path/source, report the
  split result, retry remembering only; explicit Open recovers after restart.
- Empty campaign selectable; duplicate names keep identity; external selection
  cannot answer/register against an implicit owned target.
- Five actions produce the exact backend records and replayed identical request
  has one effect. Answer resolves only its matching incarnation, remains readable
  after restart and has no open attention entry after acknowledged publication.
- Cache failure after answer shows saved/view pending; retry cannot answer twice.
  Indeterminate save forbids a new mutation and reconciles the same request.
- Late detail/command/source results cannot overwrite a newer selection or health.
  Root/file replacement invalidates checked read and answer enablement.
- Board and forms preserve long content at narrow display-cell widths, including
  wide Unicode and multiline text. Resize preserves draft/focus and cannot submit.
  Press/Repeat sequences produce one command; over-limit paste preserves old text.
- Dirty draft, rejected save, navigation during saving and quit have explicit
  outcomes. Authoring does not send counsel, focus, spawn, open a URI, infer quest
  readiness or alter external facts. Existing counsel and source regressions pass.
- Live/offline shutdown and persistence failures use disposable files/sockets;
  accepted operations are not detached. Keep native acceptance separate.

Property tests should target command uniqueness under input/retry sequences and
selection/request fencing. Do not mirror every form constructor or assert incidental
pixel coordinates. Add rendering tests only for meaningful visibility/scroll bounds.

## Expected paths and compromises

`src/owned_work/` (checked queries/owner), `src/app.rs`, `src/interaction.rs`,
`src/ui/input.rs`, Board/scene overlays, `src/terminal.rs`, `src/runtime_loop.rs`,
local source/cache integration as necessary, persistence state/worker and Storybook.
Tests will span owner, input, work, persistence and rendering boundaries.
Keep Cargo dependencies unchanged unless an evidenced need is separately reviewed.

The pin needs state v2 migration and stronger acknowledgement than current ordinary
intent publication; this is explicit scope, not permission to rewrite persistence.
The owner must coexist with the current single optional Projects inbox; add a typed
owned result path without displacing it or building a generic adapter framework.
L1a synchronous calls require dedicated scheduling. L1b remains append/backspace
editing, not a full terminal editor. Manual Open recovery needs path/source identity;
preserve both visibly if remembering creation fails.

## Completion boundary

Backend and runtime/input automation may establish engineering qualification. The
five-action workflow, answer retrieval and restart must then be exercised through
actual input; production form visuals require user review. A terminal-free fixture
is not native acceptance, and neither is installation or release. Report any unmet
gate plainly. L2–L5 and committed publication remain separately promoted work.

Production form images were subsequently approved; see the
[approval record](../reviews/2026-10-03-owned-authoring/visual-approval.md). Next gate:
[isolated native use](2026-10-03-owned-authoring-native.md), currently blocked for
automation by Computer Use refusing Ghostty. Manual execution remains unperformed.
The historical [design review](../reviews/2026-10-03-owned-authoring-plan/README.md)
remains documentation-only evidence; it is not superseded by an implementation claim.
