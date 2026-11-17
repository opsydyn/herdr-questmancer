# Owned work authoring

L1b is included in 0.2.0. Use the current source and
[qualification receipt](../reviews/2026-10-03-owned-authoring/README.md); fixture
images and automated PTY input do not establish native visual approval.

## First use

Open the Quest Board with `b`, then Actions with `a`. Create local store proposes
`owned-work.json` in the supplied plugin state directory. Review its physical
absolute path, Tab to the explicit Create button and press Enter. The parent
directory must already exist. Without a remembered pin, startup creates no store.

Store details/result displays the exact path and source identity. Keep both for
manual Open recovery. Setup first opens/creates authority, then remembers selection
in `state.json`, then acknowledges its projection in `work-state.json`. Configuration
is read-only. A split acknowledgement can leave a real store without a remembered
pin; retry remembering the same selection, or explicitly Open it after restart.
Never delete a store to repair a diagnostic or copy an arbitrary source ID into it.

Create a campaign (name, optional objective); it becomes the authoring context.
Change owned campaign includes empty campaigns. Create a quest with its title and
explicit kind. Request decision records a question, optional context and optional
quest. Decisions lists open and answered records for the chosen campaign; open a
record, inspect its question, then explicitly compose and Save answer. Register
artifact records title, kind, optional quest and URI. Registration does not open
the URI or certify the artifact. Store details also lists campaign-level spoils.

An external Board selection clears authoring context. Choose an owned campaign
explicitly; external records cannot be answered or mutated through these forms.
Quest creation remains Planned. No agent is started, assigned or messaged.

## Input and recovery

- Tab/Shift-Tab moves between fields and buttons. Enter inserts a newline in
  multiline fields or activates the focused commit/Cancel button.
- Up/Down changes a choice or scrolls text. Wheel scrolls read-only context;
  editing and Tab return to following the focused field. Ctrl-U clears that field.
- Paste is literal, limited to 8,192 bytes per field. Unsupported controls or an
  over-limit paste reject the whole addition, retaining prior text. Single-line
  fields reject newlines. Editing is append/backspace, not an arbitrary cursor editor.
- Esc returns to Actions, then Board. One unsent draft remains volatile; Resume
  draft returns to it, Discard clears it, and quit loses it. A new form requires
  discarding an existing draft. Resize preserves it. Below 28 columns or 12 rows
  submission is disabled.
- Saving freezes one exact command. Leaving its form does not cancel it. An
  uncertain result requires Check outcome before another mutation. A confirmed
  unsaved result retains the draft for review; no automatic resubmission occurs.
- Saved/view pending means the authority already changed. Retry view republishes
  that snapshot only. Remember pending retries the same pin only. Rejection retains
  the draft and requires checked current facts before an explicit resubmission.

Return through Actions/Board with Esc, close the overlay, then `q` to quit.
Shutdown drains and joins accepted filesystem work before persistence flush.
Blocking OS calls have no forced cancellation guarantee. The sibling `.lock`
file remains after exit; release is the OS lock, not deletion of the file.

## Restart and native review

With a remembered pin, reopening validates the exact path/source and republishes
the owned baseline. Missing, replaced, busy, malformed or future-version selections
disable authoring; no replacement is created. Cached facts stay unverified until
acknowledged. Select a campaign, then Decisions, and read the saved answer. Literal
answers live only in owned authority and volatile selected detail, never the cache
or Chronicle. Seen and deferred do not answer a decision.

For native acceptance, use a deliberately selected test store in an existing
physical temporary directory. Record every file/process created. Exercise all
five actions, `!` and seen/defer, answer, artifact, narrow resize, quit/reopen and
answer retrieval; review save/recovery wording. Inspect the Storybook owned states
for pending and uncertain outcomes rather than forcing faults in real data. Do
not install/reload a plugin, change a shared Herdr server or operate real agents as
part of this recipe without that scope being promoted. Remove only test-owned
files after the owner exits; record cleanup and keep native visuals separate.

The disposable automated counterpart is saved in the review pack as
`pty-workflow.py`. It supplies terminal capability/status replies, drives keys and
bracketed paste against the current built binary, checks authority/cache revision
acknowledgements and restart answer text, then removes its temporary directory.
It strips Herdr environment and creates no Herdr resource. Run it from the repo
root with Python 3 after `cargo build --release`. It is host input/storage evidence,
not unassisted usability, native image quality or real Herdr integration evidence.
