# Herdr Projects inventory: configuration and acceptance boundaries

The opt-in reader supports one explicitly registered local project in the inspected
Herdr Projects 0.2.34 layout. It reads records and report references, not execution,
planning, human questions or project memory. Installed-source and live usefulness
acceptance have not been performed. Herdr Projects requires Herdr 0.9.1; this
reader does not install it or alter Questmancer's Herdr 0.9.0/protocol-22 transport.

## Selection

The source is disabled when the table is absent or `enabled = false`. To select
a project, the owner supplies this table in their plugin `config.toml`:

```toml
[herdr_projects]
enabled = true
source_id = "projects:example-original"
campaign_id = "example"
project_path = "/absolute/physical/path/to/project"
format = "herdr-projects-0.2.34"
identity_policy = "registered_retained_records"
```

Use the physical absolute path: symlink components and parent traversal are
rejected. No project-home discovery occurs. The format and identity policy are
explicit commitments; the reader does not inspect an installed upstream binary
to certify its version. Unknown table keys, invalid tokens, relative paths and
unsupported format/policy reject configuration. Existing top-level future config
fields retain their previous compatibility behaviour.

Tokens identify this registered retained project, not its name or path. Never
reuse them for a copy, restore or replacement. Native durable identity remains
unsupported: identical records after an unobserved replacement cannot be distinguished.
A renamed retained project can keep its tokens only after the owner confirms
continuity and explicitly changes the locator. Configuration is read on startup;
there is no hot rebind. Close/reopen the Questmancer pane to apply a selection.
Do not restart or reload a shared Herdr server for this reader.

A durable plugin state directory is required. Work and its identity checkpoint
share `work-state.json` v2. Valid v1 is preserved and migrates only on acknowledged
publication; corrupt/future files remain protected and disable source attachment.
No source path, terminal output, binding, report body or source-side intent is cached.
Disabling the reader retains last-known cached work; it does not delete or finish it.

## What to expect

Use `b` for the Quest Board or `w` for the War Table. Literal recorded grouping,
closure reason, source error and labels appear in the existing selected details.
Open/starting threads are Unknown; resolved threads are Closed, without a success
check. Waiting-on-you does not create a summons. Dependencies, structured requests
and live bindings remain unsupported. Enter cannot focus an unverified source pane.
Legacy Herdr summons, counsel and agent observation keep their existing behaviour.

The producer reads off the UI thread, with one capture and one acknowledgement
outstanding. Healthy polls are five seconds apart. Transient failure delays start
at five seconds, doubling to at most thirty seconds. Unchanged mapped facts do not
rewrite the cache or redraw; error/freshness changes become a visible diagnostic.
Capture has a two-second cooperative budget checked between bounded reads.
Shutdown cancels and joins the owned capture before persistence flush. A stalled
OS filesystem call cannot be forcibly interrupted; this is not a hard real-time
deadline. Only local regular files are supported; remote filesystem acceptance
has not been performed.

| Observation | Retained state and remedy |
| --- | --- |
| Busy/missing lock, malformed/unstable/oversized required input | Unavailable; last facts remain, automatic bounded retry |
| Known retained thread omitted | Reconciliation required; reject the whole inventory, retry until all known records return |
| Storage publication fails | Reconciliation required; checkpoint/facts do not advance, retry |
| Known thread creation changes, selected root disappears/replaces, registration conflicts | Stop attachment and latch reconciliation; owner confirms/rebinds the retained project and restarts, or uses fresh tokens for replacement work |
| Restored cache | Cached/unverified, no bindings; only a new acknowledged baseline makes it current |

The reader never creates a source lock, writes records, acknowledges reports,
resolves threads, opens URLs, spawns agents or issues execution commands. Current
means the selected facts were observed under the partial-coverage contract; it
does not prove current agent execution, successful delivery or a complete inventory.

## Synthetic qualification and later live gate

`just source-test` exercises exporter/mapper and producer/consumer against
disposable trees. The [runtime receipt](../reviews/2026-09-30-herdr-projects-runtime/README.md)
records owned-resource, restart, identity, cancellation, source-byte and full-gate
evidence. Its [diagnostic images are approved](../reviews/2026-09-30-herdr-projects-runtime/visual-approval.md).
They use production cells; native input and live sources are not established by
those tests.

A later live trial needs its own explicit operational scope. Baseline actual
installed binaries/protocol, selected records and file hashes; use an owned
disposable project/server/panes and the [guarded scene procedure](questmancer-scene-preview.md).
Exercise edits, unavailable/recovery, restart and cleanup only owned resources.
Never start real agents or alter an unknown/shared server as an implicit part
of this recipe. The [bounded acceptance plan](../plans/2026-09-30-herdr-projects-acceptance.md)
defines the proposed trial and has not been promoted or executed. M3b
bindings/requests and later roadmap milestones remain separate.
