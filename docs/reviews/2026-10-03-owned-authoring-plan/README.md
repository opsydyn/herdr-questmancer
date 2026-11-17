# L1b authoring design review

The user approved form and boundary review after the qualified L1a backend.
Work stayed inline on `main` at `9c52761`. This slice changes documentation only;
existing uncommitted L1a code and all prior review evidence are preserved.

## Review result

The [design](../../design/questmancer-owned-authoring.md) proposes a Board Actions
menu, owned campaign/decision pickers, five forms and explicit store setup. The
pickers address two observed source gaps: empty campaigns are absent from the
Board inventory, and literal answered detail is not in the shared work projection.
The [implementation proposal](../../plans/2026-10-03-owned-authoring.md) bounds
selection migration, owner lifecycle, input, recovery and qualification.

The deliberate added scope is a remembered path/source selection in state v2,
with v1 migration and acknowledged publication. It is local intent, not another
authoritative work store. Store creation and remembering that selection have
separate failure/recovery states. A serial owner handles file operations outside
rendering; typed checked queries expose answers lazily. Command acknowledgement
and cache/view publication remain separate, with no automatic duplicate submission.

Proposed bindings, text wireframes and minimum form size are reviewable design,
not implemented UI, user-research findings or production visual approval. Editing
is deliberately append/backspace plus literal bounded paste, not a full editor.
No external installation or Herdr upgrade is needed for the proposed workflow.

## Static evidence

`verification.json` records the current HEAD, unchanged source/test/script/manifest
and prior-review hashes, checked local links and `git diff --check`. Source hashes
include the uncommitted L1a modules. Existing L1a qualification logs are retained;
no behavioural test gate was rerun for this documentation-only review.

Implementation, production form visuals, native input/restart, installation,
shared-server operations, commit and release were not performed. All changes remain
uncommitted and unreleased. Next proposed action: promote the bounded L1b
implementation, then qualify and review its actual production forms separately.
