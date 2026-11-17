# L1b owned authoring

The user promoted the reviewed implementation on 2026-10-03. Work stayed inline
on `main`, beginning at `9c52761`. Prior dirty L1a, planning and evidence were
preserved. The combined work remains uncommitted and unreleased. No installation,
shared Herdr server, real adventurer or external project home was operated.

## Result

Board `a` opens owned Actions, campaign/decision pickers, explicit store setup and
five parchment forms. Empty campaigns are selectable; answered records remain
reachable. A serial owner runs independently of Herdr connectivity. Rendering and
pure input never perform filesystem I/O or generate command identity/time.

Setup remembers one physical absolute path and source in local state v2. Legacy
v1 intent migrates without losing existing intent. No pin creates no store;
protected, missing or replaced selections disable authoring. Open does not adopt
an arbitrary source. Store creation, selection acknowledgement and cache/view
publication remain separate writes with distinct recovery states.

Commands freeze one ID, revision, timestamp and payload. Saved/view pending retries
only publish the existing snapshot. Indeterminate recovery reconciles that command;
an unchanged prior revision preserves the unsaved draft. Checked lazy detail reads
validate store identity/digest, source, incarnation and revision. Literal answers
never enter the shared work cache, Chronicle or logs. Normal shutdown drains and
joins the owner before persistence flush; OS filesystem calls cannot be forcibly
cancelled. One outstanding request and monotonic tags fence responses.

Forms wrap by display cells, accept bounded literal paste and support multiline
append/backspace. Fixed action titles remain visible during scroll. Tab/editing
follows focus; Up/Down or wheel can read context. Below 28 columns or 12 rows,
drafts survive and submission is disabled. Esc retains one volatile draft; explicit
discard clears it and quit loses it. The existing Board observe, seen/defer,
counsel and external-authority contracts remain intact.

## Engineering evidence

The accompanying [verification receipt](verification.json) records command results,
source/binary hashes and baseline preservation. Focused red logs cover lifecycle,
Storybook, narrow action-heading visibility, unsaved reconciliation, wheel reading,
keyring documentation drift and a clear status-only resize surface. No new dependencies
or production renderer added. All 291 prior review files are byte-identical. Of 217 baseline source/test/script/manifest files, 189 are unchanged;
28 existing files and five new paths form this extension over the prior L1a source.

- `just verify`: 792 Rust tests across 64 runs, 28 shell tests, formatting,
  all-target/all-feature Clippy with warnings denied and script syntax pass.
- `just property-test 4096`: 151 tests across eight runs pass, including frozen
  submitted drafts under repeated keys, paste and resize.
- Release build passes. It is a candidate binary, not an installed/published release.
- Owner/input tests cover missing restart without replacement, checked inode
  replacement, protected intent before creation, remembering-only retry, cache
  failure after a committed answer, exact-revision recovery and late request fencing.
- The [disposable macOS PTY receipt](pty-receipt.json) exercises actual keys and
  bracketed paste: Create store, Create campaign/quest, Request/Answer decision,
  Register artifact, quit/reopen and read the same answer. Authority and acknowledged
  cache revisions 0–5 match; reopening leaves authority bytes unchanged. Both
  processes exit and the temporary directory is removed. Zero Herdr resources.

The PTY trial is automated host input/storage evidence, not an unassisted user test
or native art review. Initial attempts were inconclusive because the harness omitted
the terminal-status reply and stripped cursor movement without reconstructing the
grid. The final harness supplies the full replies and reconstructs changed cells;
no portrait/raw-input production change was required. Temporary tracing was removed.
Image inspection also caught scene glyphs leaking behind the below-minimum resize
message; a focused failing regression now passes with the overlay cleared. The
final full gate also corrected a Clippy warning in a new rendering test and the
existing shell documentation contract's old 58-story expectation. Tests run with
authorised disposable sockets; shared services were not touched.

## Production form review — approved 2026-10-03

The user subsequently replied **Approved**. The [approval record](visual-approval.md)
pins the accepted images and preserves the original engineering receipt. Native
acceptance remains separate; its [availability check](../2026-10-03-owned-authoring-native/README.md)
was blocked by Computer Use denying Ghostty access.

Eight fixed owned stories bring Storybook to 66 (two worlds, 37 assets, 27
interactions). These PNGs reproduce production RGB rendering, identity labels and
parchment overlays from Ratatui cell buffers using Menlo. They are fixture images,
not native terminal screenshots. JSON cells and generators are retained. The extra
narrow editing image shows the same story after focus scrolling.

| Surface | Review image |
| --- | --- |
| Actions | [actions](visuals/actions.png) |
| Empty campaign picker | [empty campaign](visuals/empty-campaign.png) |
| Answer form | [answer](visuals/answer.png) |
| Narrow question/context | [narrow reading](visuals/answer-narrow.png) |
| Narrow answer/caret | [narrow editing](visuals/answer-narrow-editing.png) |
| Saved; view pending | [view pending](visuals/view-pending.png) |
| Uncertain result | [indeterminate](visuals/indeterminate.png) |
| Answered detail | [answered](visuals/answered.png) |
| Below minimum | [resize](visuals/below-minimum.png) |

Review action/context readability, focus/navigation, answer/save wording and whether
pending states clearly describe what is already durable. The fixed title and scroll
tests prove reachability and bounds, not art quality or five-second comprehension.

Run `just storybook` for production inspection. `render-review.py` rebuilds the
Storybook library, exports cells with Rust and draws PNGs with Python/Pillow on
macOS. Run from the repo root with a Pillow-equipped Python. `pty-workflow.py`
requires the current release candidate and uses only its own temporary store.
The [operating recipe](../../manual-test/owned-work-authoring.md) explains setup,
recovery, keyboard editing and the separate native acceptance procedure.

## Limits and next gate

Production form fixture visuals are approved. Native Ghostty/managed-pane authoring,
real Herdr use, other native hosts and unassisted usability remain unperformed.
Earlier approved Hall/Delve/Board art is not approval for these forms. The review
does not establish installation, release or authoritative historical replay.

Complete baselines are retained rather than adding a new event protocol. State's
historical Rust type name remains `PersistedStateV1`, although writes are schema
v2. Editing is append/backspace, and manual recovery uses exact path/source entry.
The lock remains cooperative. No lifecycle editing, dependency activation,
assignment, agent spawning, URI opening, journal or orchestration was added.

Next bounded gate: the [isolated manual native trial](../../plans/2026-10-03-owned-authoring-native.md).
Computer Use cannot operate Ghostty, so native execution needs human observation. L2–L5 and commit/publication remain separately
promoted work. See the [approved design](../../design/questmancer-owned-authoring.md)
and [implementation plan](../../plans/2026-10-03-owned-authoring.md).

## Repeated PTY evidence — 2026-10-15

The upgrade check reran the automated PTY harness. That harness writes its restart
screen to this review directory: `restart-pty-screen.txt` now contains the October
15 synthetic campaign ID and answer timestamp, not the original October 3 screen.
The structural `pty-receipt.json` remains identical to the original `logs/pty.log`
JSON. Original approved production review images are unchanged. The repeated run
is recorded separately under [the upgrade review](../2026-10-15-herdr-upgrade/README.md).
It establishes no native acceptance.
