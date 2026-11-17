# L1b: owned work authoring

Date: 2026-10-03. The user approved review of the next forms after L1a backend
qualification, then promoted its implementation. The forms and runtime now exist;
[qualification](../reviews/2026-10-03-owned-authoring/README.md) is separate from
production visual approval and native/live acceptance. The proposal below records
the approved design; implementation differences are listed at the end. Read the [owned contract](questmancer-owned-work.md)
and [L1a receipt](../reviews/2026-10-03-owned-work-backend/README.md) first.

The largest obstacle is reaching the first useful record. The Board currently
lists quests and obligations, so an empty campaign cannot be selected there.
The backend is not attached at startup, and answered detail has no UI query.
Adding five buttons alone would leave these gaps in the human workflow.

## Recommendation and evidence

Keep authoring inside the existing Board parchment. Add an Actions menu, an
owned-campaign picker and five small forms. Keep the Hall/Delve underneath;
the War Table remains a projection of dependency facts. There is no new room,
dashboard, renderer or execution control in this slice.

| Step | Current source / expected progress | Cost | Proposed correction |
| --- | --- | --- | --- |
| First use | No selected owner at ordinary startup | Cannot create work through UI | Explicit Create/Open store setup; remember selection as local intent |
| Create campaign | Board inventory contains quests and obligations, not empty campaigns | New campaign has no selectable work row | Small owned-campaign picker in Actions; allow zero-quest campaigns |
| Answer | Shared obligation has resolution but no literal answer | Cannot inspect the answer after restart | Lazy typed selected-detail query to the owner |
| Save | Backend and projection cache have separate acknowledgements | A delayed view could invite duplicate submission | Separate saved, view-pending and indeterminate feedback |
| Read/edit | Board already wraps by display-cell width; current input ignores paste | Long question/answer and narrow panes need deliberate handling | Scrollable form, bounded paste and explicit focus |

These are source observations and expert design inferences, not user research.
The screenshot truncation repair and prior Board approval remain separate evidence;
these new forms have not been drawn or tried in a native terminal.

## Entry and campaign context

- Proposed Board-only `a`: **Actions**. Existing `b`, `w`, `!`, Enter, Space and
  `s` retain their meanings. Board Enter still observes a verified live binding.
- Actions starts with **Owned campaign: <name>** and a Change entry. The picker
  includes only campaigns belonging to the selected owner, including empty ones.
  Show an ID suffix for duplicate names; keep full source-qualified IDs internally.
- Offer Create campaign, Create quest, Request decision, Answer selected decision,
  Register artifact and Store details/setup. Hide no authority distinction:
  unavailable actions have a short reason. External selections remain readable.
- Opening Actions on an owned item selects its campaign. Opening it on an external
  item must not silently redirect an answer or artifact to an owned campaign.
  Creation requires an explicitly chosen owned campaign; there is no import.
- Create campaign needs no prior campaign. On success, retain that campaign as the
  authoring context and offer Create quest. Context is ephemeral, not a new work row.
- Answer selected decision is available only for a matching owned, open decision
  from a verified owner. It always shows its question and campaign. No generic
  "resolve" command and no conversion of seen/defer into an answer.

The menu and pickers use literal action names. Quest kinds may show the existing
Expedition/Commission/Inspection/Trial labels alongside their exact machine kind.
New quests remain Planned; choosing Ship does not start an adventurer.

## Explicit store setup and restart

Use one remembered `OwnedStoreSelection { path, source }` in local intent, not a
new configuration writer or separate registry service. Propose a state schema v2
with an optional selection and explicit v1 migration preserving all current intent.
`state.json` stores this pin only: no campaigns, answers, live topology or receipts.
`config.toml` stays read-only. Work cache v2 and owned snapshot v1 stay unchanged.

On first use, Actions offers **Create local store** and **Open existing store**.
Create proposes `owned-work.json` in the supplied plugin state directory; show the
absolute destination and an explicit Create button. Generate a random source once
for this setup attempt. The parent must already exist physically and meet L1a
selection checks. No parent creation, symlink resolution, missing-store replacement
or fallback store in the working directory. If persistence is unavailable or
protected, explain why durable setup is unavailable before creating anything.

Open requires an absolute path and expected source identity, displayed in Store
details and retained after creation. It validates through `OwnedStore::open` before
remembering the pin. This is the bounded recovery path; no arbitrary directory
browser, multiple-store manager or automatic identity adoption is proposed.

The setup sequence has two independent acknowledgements:

1. Create/open the authoritative store and obtain a validated owner.
2. Publish the selection through the intent worker with file/directory durability
   acknowledgement, then publish its Complete work baseline through the cache.

If step 2 fails, say **Store created; selection not remembered** (or **Store open;
selection not remembered**). Retain path/source in the setup result. Retry remembers
that same selection; it never creates another store. Require remembered selection
and an acknowledged baseline before enabling the five authoring actions. A crash
between steps leaves an existing store recoverable through explicit Open, not an
empty replacement. These are separate writes, not an atomic two-file transaction.

Ordinary startup without a pin never creates or opens an owner. With a valid pin,
open exactly that selection and rebuild the owned projection. Missing, busy,
replaced, malformed or future-version selections show a diagnostic and disable
authoring. Retain cached facts as unverified; do not guess a new source. Explicit
setup can replace a pin only after a validated Open/Create and user selection.
Do not delete the old source's cache/history or reinterpret it as the new store.
Changing selection is disabled while an operation is outstanding. Fence the old
epoch, release its owner and mark its retained facts disconnected; only the new
acknowledged baseline can make the new selection Current.

This adds a migration and a narrowly acknowledged intent write to L1b. It is
preferred over requiring users to hand-edit random IDs into configuration after
their first campaign. Do not weaken future-version or malformed-state protections
to enable setup. Older binaries must reject v2 rather than discard the selection.

## Forms and keyboard contract

| Form | Required input | Optional input / constraints | Commit button |
| --- | --- | --- | --- |
| Campaign | Name | Objective | Create campaign |
| Quest | Owned campaign, title, explicit kind | Description; no assignment/dependencies | Create quest |
| Decision request | Owned campaign, question | Quest in that campaign, context; human responsibility | Request decision |
| Decision answer | Fixed decision ID/incarnation, literal answer | Read-only question, context and requester | Save answer |
| Artifact | Owned campaign, title, explicit kind | Quest in that campaign, URI; no validation claim | Register artifact |

Campaign/quest choices come from verified owner metadata and retain typed IDs.
The backend still validates every reference. A selected quest may prefill a visible
optional association; changing campaign clears it. Kind selectors require a visible
choice; neither Unspecified nor lifecycle inference is offered. Registration does
not open the URI, run Git/CI or assert that an artifact exists.

Each form has a fixed heading, campaign/source context, a scrollable body and a
footer. Wrap question, context, field content, URI and status by display cells.
Keep the focused field or button visible. No side-by-side form at narrow widths.
At less than 28 columns or 12 rows show a resize message, preserve draft/focus and
disable submission; resizing is not cancellation. Treat these as proposed form
limits, not changes to the smaller Board or world contracts.

- Tab/Shift-Tab moves among fields, selectors, commit and Cancel. Enter operates
  only the focused selector/button; Enter in multiline text inserts a newline.
- Up/Down moves selector/menu choices or scrolls read-only question/context when
  that section is focused. Multiline editing in L1b is append/backspace with a
  visible end caret and scrolling; no promise of an arbitrary cursor editor.
- Ctrl-U clears only the focused editable field. Backspace respects UTF-8 boundaries.
  Bracketed paste appends literal bounded text to that field; pasted keys never
  become menu actions. Restore terminal paste mode on all exit paths.
- Enforce backend byte bounds (8,192 per text field), nonblank required text and
  permitted controls before submitting. Reject an over-limit paste as a whole with
  feedback, preserving the prior text; do not silently truncate it. Single-line
  fields reject newline/control input. Backend validation remains authoritative.
- Esc/Cancel leaves an unsent form and returns to Actions. Keep one volatile draft
  while revisiting Actions; clearly state that exit/restart discards it. An explicit
  Discard draft action clears it. Starting another form with a draft requires an
  explicit discard choice; do not lose text through navigation or reuse it elsewhere.
- Submission accepts a key Press, never a Repeat. Freeze the exact request after
  acceptance, including ID, expected revision, actor, timestamp and payload. Do not
  generate another ID or refresh its timestamp during retry.

Local attribution is the literal display identity **Questmancer**, presented as a
local human action, not authentication or a Herdr agent identity. No settings or
persona system is needed for L1b. Authoring drafts and counsel drafts are independent.

### Review wireframes

These describe information and focus, not pixel placement or approved art.

```text
Quest Board / Actions
Owned campaign: Replace authentication layer   [Change]
  Create campaign
> Create quest
  Request decision
  Answer selected decision       Select an owned open decision
  Register artifact
  Decisions                       Open and answered records
  Store details

Up/Down: choose   Enter: open   Esc: Board
```

```text
Answer decision
Campaign: Replace authentication layer
Source: owned:<suffix>     Question 1 of 1

Should the old token endpoint remain for backwards compatibility?
Context: The web migration needs a compatibility policy.
Requested by: Questmancer

Answer [focused]:
Keep it for one release and document its removal date.|

[Save answer]   [Cancel]
Tab: field/action   Enter: newline   Esc: keep draft
```

```text
Answer decision                     (narrow parchment)
Campaign: Replace
authentication layer

Answer saved.
The Board has not updated yet.

[Retry view update]
This does not submit another answer.

Tab: action   Enter: retry
Esc: Board
```

## Owner, effects and selected detail

Add one bounded owner task to `RuntimeLifecycle`, independent of Herdr connectivity
and the optional Projects producer. It holds the store lock and serialises synchronous
filesystem operations on a dedicated worker thread. Rendering and pure reducers
never read files, block on sync or allocate identities/time. No filesystem polling
on animation frames. Preserve both live and offline event loops.

Proposed request vocabulary: Setup(Create/Open), Submit(Command), ReadDecision,
RefreshView and Reconcile. Use typed selection epoch, request ID and outcomes.
One command may be outstanding; do not queue mutations while setup, commit,
indeterminate reconciliation or view publication is pending. Read requests coalesce
to the latest selection; every response checks its epoch and request before display.
Keep owner metadata and command outcomes separate from acknowledged shared work facts.
Use a separate typed owned-effect path; do not disguise backend commands as
`AgentCommand` or send them through the Herdr command dispatcher.

`ReadDecision` includes source-qualified ID, incarnation, expected store revision
and selection/request tags. Add a narrow backend checked-read operation which
validates the held directory/lock/file identity and digest before reading acknowledged
state. `.state()` alone is not proof that the selected file is still current.
Return a typed detail/result with requester, question/context, optional literal
answer/actor/time and observed revision. Missing, stale or indeterminate results
cannot enable Answer. A changed revision requires a fresh read and user review.

Fetch detail only when selecting/opening the relevant owned decision or after its
commit; never on animation frames. Reject late results for another selection,
incarnation or owner epoch. Literal answers remain solely in the owned store and
volatile selected-detail state, not the work cache, Chronicle or logs. Board
navigation must keep resolved decisions inspectable through a small **Decisions**
picker in Actions, including campaign-level decisions; answering removes attention,
not historical access. This picker filters owned decisions by campaign and state.

After a durable receipt, produce `LocalWorkSource::baseline` and use the existing
acknowledged work publication boundary. Apply against the latest projection so
external updates and seen/defer are preserved. Invalidate obsolete proposals and
retry from current facts; retain Projects checkpoints. Bindings stay absent.

Shutdown stops new requests, drains the accepted operation, joins the owner and
flushes persistence. Do not detach a filesystem writer or claim cancellation after
replacement. Blocking OS calls are not forcibly cancellable: if shutdown cannot
establish an outcome, report it and rely on receipt reconciliation at next Open.
No hard wall-clock shutdown guarantee is inferred from task cancellation.

## Feedback and recovery

| Known result | User-facing meaning | Permitted next action |
| --- | --- | --- |
| Draft | Nothing submitted | Edit, keep or discard draft |
| Saving | Exact command is outstanding | Read status; no edit/resubmit; leaving hides the form, not the request |
| Rejected before publication | Not saved, with specific reason | Preserve text; reload references/revision and require explicit resubmission |
| Stale revision | Draft needs current facts | Refresh and review; never silently change expected revision then retry |
| Indeterminate | May have saved | Check outcome with same request; block new mutations until reconciled |
| Receipt acknowledged, cache pending/failed | Saved; view not updated | Retry view publication only; never resubmit the command |
| Receipt and view acknowledged | Saved and Board updated | Return to relevant campaign/item; answer detail remains inspectable |
| Selected store lost/replaced | Owner unavailable | Preserve draft, disable commands; explicit same-selection recovery |

Esc may return to the Board during an outstanding operation, with a persistent
owned-operation notice in Board/Actions and an explicit Return to result entry.
Do not overwrite that outcome with transient focus or source diagnostics. On restart,
read the pinned store and its receipt/state, rebuild the view and query answer detail.
Volatile drafts are not restored; do not invent a cancelled or failed outcome for
a command whose acknowledgement was lost.

## Qualification and scope limits

An unassisted task after implementation: create a campaign and quest, ask a policy
question, answer it, register a report, close/reopen, then find the same answer.
Observe discovery, accidental duplicate attempts and interpretation of delayed
publication. No measured baseline or five-second usability claim exists yet.
If the campaign cannot be found or "view pending" is interpreted as unsaved, revise
the entry/feedback before expanding to lifecycle or orchestration.

Automated tests prove input/state/storage invariants. Fixture screenshots and
native-terminal interaction remain distinct review gates. The proposed next slice
is [L1b implementation](../plans/2026-10-03-owned-authoring.md), subject to promotion.
No assignment, spawning, lifecycle editing, URI opening, imports, knowledge system,
journal, replay, Firstmate or new Projects integration is included.


## Implemented boundary and deliberate compromises

L1b is implemented after promotion. The owner transports typed inventory/revision
metadata rather than the full store; checked detail carries one literal answer.
Neither rendering nor the pure input handler reads files or allocates random/time
identity. One request is outstanding on bounded channels. A monotonically increasing
request tag and serial selection changes provide fencing without another epoch
counter; no old-owner request can coexist with a later setup. Late tags are rejected.

All owned updates remain Complete baselines through acknowledged work publication.
Backend save, remembering selection and cache publication are separate writes.
Unchanged prior revision during reconciliation proves the command was not committed
and retains its draft. A saved command stays frozen through view retry; resolved
answers remain in Decisions. Store/result also lists campaign-level spoils.

The fixed action title stays visible while campaign/source context scrolls. Tab or
editing follows focus; Up/Down or wheel releases that anchor to read context. Editing
is append/backspace with bounded literal paste, not a full cursor editor. Setup uses
manual physical path/source entry, not automatic identity adoption. `PersistedStateV1`
retains its historical Rust type name for compatibility but writes state schema v2
and explicitly migrates v1; work cache v2 and owned snapshot v1 are unchanged.

Filesystem work runs on a dedicated serial thread. Normal shutdown drains the accepted
operation and joins it before persistence flush; blocking OS calls have no forced
cancellation guarantee. Unexpected teardown closes the result receiver and joins the
worker. The writer lock remains cooperative. No dependency or runtime service added.
The automated PTY recipe supplies complete terminal protocol replies; fixture cell
images use production overlays but are not native screenshots or visual approval.


## Subsequent presentation approval — 2026-10-03

The user approved the production form fixture images; see the
[approval record](../reviews/2026-10-03-owned-authoring/visual-approval.md).
The original proposal and qualification paragraphs above retain their historical
scope. Native Ghostty interaction is still unperformed: Computer Use denied access
at the availability check. The [manual native plan](../plans/2026-10-03-owned-authoring-native.md)
uses a new physical store and no Herdr connection; managed-pane/live use, installation,
commit and release remain separate. No product behaviour changed in this approval slice.
