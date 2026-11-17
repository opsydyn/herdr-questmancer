# L1a owned work backend

The user approved the proposed L1a implementation on 2026-10-03. Work stayed
inline on `main`, starting at `9c52761`. Existing uncommitted planning documents
were preserved and their current status updated. These changes are uncommitted
and unreleased. No plugin, shared server, real agent or project home was operated.

## Result

`src/owned_work/` implements five typed human commands through a pure transition:
create campaign, create quest, request decision, answer decision, register artifact.
The source owns its records, literal answer, durable requester attribution and
bounded retry receipts. References, incarnation, source, text and expected revision
are validated. Rejection and retained identical retries do not write again.

`OwnedStore` explicitly creates/opens a versioned `owned-work.json` in an existing
physical directory. A persistent sibling `.lock` file holds the lifetime OS writer
lock; dropping the owner releases the lock, not the file. Reads reject symlinks,
duplicate keys, future/malformed/inconsistent data and files above 16 MiB. Commands
check the physical root, lock inode, selected file inode and content digest.
Publication synchronises the temporary file and containing directory. Initial
creation uses an exclusive hard link; replacement uses rename. A post-replacement
failure retains an indeterminate command and blocks further commands until the
same file/receipt is reconciled and durability re-established. Explicit reopening
also establishes durability before a receipt can acknowledge a retry.

`src/sources/local_work.rs` produces Complete baselines for the existing reducer
and acknowledged cache path. It rejects an older store or exhausted generation.
Cache failure retains the previous projected facts and requests reconciliation;
it cannot undo a committed answer. Tests preserve external facts, local intent,
Projects observer checkpoints and the v2 cache format. The cache contains no literal
answer text, live binding, terminal output or raw session identity.

## Evidence

The focused restart test was added before implementation and failed because the
owned modules did not exist. A second failing test caught requester attribution
depending on an evictable receipt; attribution now lives with the decision.
Logs and machine-readable preservation/gate data accompany this receipt.

- Focused owned suite: 16 tests pass, including the child-process lock probe.
- Full `just verify`: 775 Rust tests across 63 runs and 28 shell tests pass;
  formatting, all-feature Clippy with warnings denied and script syntax pass.
- Property and release results are recorded in `verification.json`.
- Five internal fault tests exercise pre-replacement failure, indeterminate
  acknowledgement, reconciliation, restart and identical-but-replaced inodes.
- The existing Board/Table models and overlays render owned facts through production
  paths without changing work truth. This is automated projection evidence.
- Of 213 baseline source/test/script/manifest files, 208 are byte-identical. The
  five changed existing files are module exports, strict JSON visibility and test
  recipes; renderer, Herdr, runtime, cache format, manifests and old tests are intact.

An initial broad library run hit ten existing tests' sandbox-denied Unix socket
binds. The authorised full gate ran with disposable sockets and passed. An earlier
compile exhausted the disk; only this checkout's generated incremental cache was
removed, and qualification ran with `CARGO_INCREMENTAL=0`. Both failures are retained
as evidence, separate from the final passing gate.

## Deliberate limits and next work

L1a is backend engineering proof. There is no ordinary-startup attachment,
configuration selection, command socket, CLI writer or user authoring form. No
new agent commands, dependency activation, lifecycle edits, Chronicle writes,
awards, orchestration or authoritative replay were added. The store is an atomic
snapshot, not a historical journal. Every owned refresh currently uses a Complete
baseline rather than introducing an incremental event protocol.

Answers and requester detail remain authoritative backend data; L1b needs a typed
selected-detail query and explicit committed/pending/indeterminate feedback.
Filesystem calls are synchronous library operations; L1b must own their scheduling
outside rendering. Replacement checks protect observed changes and the writer lock
is cooperative; this is not a defence against arbitrary non-cooperating filesystem
writers. Native interaction, visual approval, live-source use, other native hosts,
installation and release were not performed.

Next proposed work: review minimal L1b Board authoring forms and the typed
command/detail boundary before implementing the user workflow. See the
[owned contract](../../design/questmancer-owned-work.md) and
[milestone plan](../../plans/2026-10-01-owned-work.md).
