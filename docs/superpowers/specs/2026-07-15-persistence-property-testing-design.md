# herdr-webmaster persistence and property-testing design

## Decision

Milestone 6.1 adds local persistence as two deliberately different stores:

- `state.json` is an atomic, versioned snapshot of durable user intent;
- `guestbook.jsonl` is an append-only history of semantic Herdr events.

The plugin never persists the complete live `DomainState`. Herdr remains the
authority for workspaces, panes, agents, presence, focus, revisions, and output.
After startup or reconnect, persisted intent is overlaid onto a fresh Herdr
snapshot only where the referenced live identity still exists.

Property testing joins the existing example and fixture tests. It targets the
persistence boundary and domain invariants where many generated event sequences
give substantially better coverage than another list of hand-written cases.

## Scope

This slice includes:

- reading user configuration from `HERDR_PLUGIN_CONFIG_DIR/config.toml`;
- reading and atomically replacing
  `HERDR_PLUGIN_STATE_DIR/state.json`;
- appending and replaying
  `HERDR_PLUGIN_STATE_DIR/guestbook.jsonl`;
- restoring stable personas, the last valid selection, view, display
  preferences, and seen attention episodes;
- debounced persistence through one long-lived worker;
- non-fatal, visible diagnostics for malformed or unavailable local files;
- property tests for persistence and existing core domain invariants;
- tracked proptest regression cases.

This slice does not add SQLite, configuration editing, persona editing, cloud
sync, telemetry, migration from an earlier schema, release automation, terminal
recordings, or broader plugin lifecycle changes. `runtime.json` stays an
ephemeral singleton-pane registration owned by the existing lifecycle code.

## Dependencies

Use these dependency lines:

```toml
[dependencies]
toml = "1.1"

[dev-dependencies]
proptest = "1.11"
```

`proptest` remains a development dependency. Do not derive `Arbitrary` on
production types and do not add `proptest-derive`. Tests use explicit strategies
that describe valid and deliberately invalid domain data.

The project Rust baseline remains 1.90, which is above proptest 1.11's Rust
1.86 minimum.

Primary references:

- [proptest repository and MSRV policy](https://github.com/proptest-rs/proptest)
- [proptest 1.11 guide](https://docs.rs/proptest/1.11.0/proptest/)
- [proptest runner configuration](https://docs.rs/proptest/1.11.0/proptest/test_runner/struct.Config.html)
- [toml crate documentation](https://docs.rs/toml/latest/toml/)

## Files and ownership

```text
$HERDR_PLUGIN_CONFIG_DIR/
  config.toml          user-authored preferences; read-only to the plugin

$HERDR_PLUGIN_STATE_DIR/
  runtime.json         ephemeral pane registration; existing owner unchanged
  state.json           atomic durable-intent snapshot
  guestbook.jsonl      append-only semantic event history
```

If either plugin directory variable is absent, the TUI remains usable with
defaults and in-memory history. It reports that persistence is disabled instead
of failing startup. The plugin creates a missing state directory when a state
path is configured; it never creates or rewrites `config.toml`.

## Configuration

`config.toml` has the following versionless, user-authored shape:

```toml
default_view = "desk"             # desk | cafe
motion = "full"                   # full | reduced | none
character_set = "unicode"         # unicode | ascii
color_mode = "xterm256"           # xterm256 | ansi16
output_preview_lines = 80
guestbook_max_entries = 500
reviewr_action = "persiyanov.reviewr.open"
show_elapsed_time = true
```

The first four fields map to typed enums using snake-case names. The remaining
validated bounds are:

```text
output_preview_lines      10..=500
guestbook_max_entries     50..=10_000
reviewr_action            non-empty after trimming
```

Missing fields use defaults. Unknown fields are accepted so newer configuration
can be read by an older binary. A syntactically invalid file or invalid field
value rejects the complete configuration file, emits one diagnostic containing
the path and parse error, and uses safe defaults. Partial application is avoided
because a half-valid configuration is difficult for a user to reason about.

The effective startup precedence is:

```text
explicit `ui --view desk|cafe`
persisted `last_view`
configured `default_view`
built-in `desk`
```

`ui --view` therefore becomes optional. The `desk` and `cafe` actions pass an
explicit view. An ordinary `open`, a closed `toggle`, and direct `ui` invocation
omit it and allow persisted/configured preference to win.

Configuration supplies initial display preferences. A valid persisted
preference snapshot overrides it because it records the user's most recent
runtime choice. Configuration-only values such as output limits and the reviewr
action are never written to `state.json`.

## Durable state schema

`state.json` is a single pretty-printed JSON document:

```rust
pub struct PersistedStateV1 {
    pub schema_version: u32,
    pub last_view: View,
    pub preferences: DisplayPreferences,
    pub selected_persona: Option<PersonaKey>,
    pub personas: BTreeMap<PersonaKey, AgentPersona>,
    pub seen_attention: BTreeSet<AttentionEpisodeKey>,
}

pub struct AttentionEpisodeKey {
    pub persona: PersonaKey,
    pub pane_revision: u64,
    pub reason: AttentionReason,
}
```

The only supported `schema_version` is `1`. An absent file means defaults. An
unsupported future version, malformed JSON, or invalid internal relationship is
ignored as a whole with a visible diagnostic. The original file is retained for
manual recovery; startup never overwrites it merely because loading failed.

Valid internal relationships require every persona-map key to equal the
embedded `AgentPersona::key`, `selected_persona` to be absent or present in the
persona map, and every seen episode to reference a persona in that map. These
checks make cross-identity corruption fail closed.

`AttentionEpisodeKey` identifies one live attention episode rather than an
agent forever. A seen marker is restored only when persona, pane revision, and
reason all match the fresh snapshot. Herdr snapshots expose the revision but
not the original status timestamp, so `since` cannot be part of restart-stable
identity. A later blocked or completed episode advances the pane revision and
is therefore unseen.

Do not persist:

- workspace, tab, pane, or agent collections;
- current presence or focus;
- socket or protocol state;
- pane output;
- modal, search, or reply drafts;
- reviewr discovery;
- animation frames or one-shot effects;
- status messages or transient diagnostics.

The model owns one durable-intent catalog containing the persona registry and
seen-episode ledger. Loading seeds that catalog; a fresh snapshot adds newly
generated personas and prunes seen markers for episodes that no longer exist.
Persona records are not pruned automatically in this slice, so an agent can
disappear and later recover the same authored persona. `PersistedStateV1` is
derived from the model and this catalog. It is not a second mutable live-domain
copy maintained alongside `DomainState`.

## Guestbook log

Each complete line in `guestbook.jsonl` is one serialized `GuestbookEntry`.
The existing deterministic `EventId` is the deduplication key.

Replay processes lines in file order and appends valid entries through the
existing `Guestbook::append` API. The in-memory guestbook remains sorted by
`occurred_at`, deduplicated, and bounded by the effective
`guestbook_max_entries` configuration.

Malformed UTF-8, malformed JSON records, and a truncated final line do not
prevent valid records from loading. Each rejected record contributes a bounded
diagnostic containing its one-based line number. Repeated messages are folded
into a summary after the first five so a damaged file cannot flood the status
surface.

New entries are serialized under one writer, written as one record plus newline,
and flushed before the append effect is acknowledged. A failed append leaves the
entry in memory and reports a non-fatal diagnostic. It is not retried silently
because doing so could duplicate data after an ambiguous I/O failure;
deterministic IDs make an explicit later replay or repair safe.

History retention is an in-memory display bound in this slice. The JSONL file is
not compacted automatically.

## Startup and merge

Startup is ordered as follows:

1. parse plugin paths and the optional explicit view;
2. load configuration, persisted state, and guestbook concurrently;
3. start the persistence worker;
4. connect, validate, and request a fresh Herdr snapshot;
5. build a new live `DomainState` from the snapshot;
6. overlay valid durable intent;
7. subscribe and render.

Persistence loading must not wait for Herdr. Herdr connection failure must not
discard successfully loaded local data or diagnostics.

The overlay applies these rules in order:

1. Replace a generated persona only when its `PersonaKey` exists in the
   persisted persona map. Never attach a persona to a different key.
2. Resolve `selected_persona` to a current `AgentKey`. Select it only when
   exactly one live agent has that persona; otherwise keep the snapshot's valid
   selection or choose the first live agent.
3. Change `Attention::Unseen` to `Attention::Seen` only for an exact
   `AttentionEpisodeKey` match. Never manufacture an attention state.
4. Apply effective view and display preferences using the precedence above.
5. Replay valid guestbook entries, then accept live events.

The same overlay is used after every reconnect snapshot. It never replaces live
topology, presence, focus, pane revision, or output.

## Write model and shutdown

One long-lived asynchronous persistence worker owns all state and guestbook file
handles. Runtime effects send it typed messages:

```rust
enum PersistenceMessage {
    StageState(PersistedStateV1),
    AppendGuestbook {
        entry: GuestbookEntry,
        acknowledgement: oneshot::Sender<Result<(), PersistenceError>>,
    },
    Flush(oneshot::Sender<Result<(), PersistenceError>>),
    Shutdown(oneshot::Sender<Result<(), PersistenceError>>),
}
```

`Command::PersistState` captures the current persisted projection. The producer
compares it with the last staged projection and sends nothing when unchanged.
The worker coalesces state changes for 250 milliseconds, then writes only the
latest value. It has no repeating timer while clean.

Atomic replacement uses a temporary file in the same directory:

1. serialize before opening the destination;
2. create and truncate `state.json.tmp`;
3. write all bytes and `sync_all` the temporary file;
4. rename it over `state.json`;
5. best-effort sync the parent directory on supported platforms.

A failed write or rename retains the last published `state.json`, leaves the
worker alive, and emits a bounded status diagnostic. A subsequent distinct
state transition may try again.

Normal quit, terminal signal handling, and terminal teardown request
`Shutdown`, which immediately publishes the latest dirty state, flushes the
guestbook, and acknowledges completion. Shutdown uses a short bounded wait; a
filesystem failure is reported but never prevents terminal restoration.

Animation ticks do not emit persistence commands. Repeated semantic events that
leave the persisted projection unchanged do not touch disk.

## Error reporting

Persistence operations return structured diagnostics with an operation, path,
and source error. The UI retains only a bounded queue and shows the newest item
as a non-blocking status message. Diagnostics are also written to stderr after
the terminal has been safely restored.

No configuration, state, or guestbook error may panic, terminate the TUI, erase
the last valid state file, or block Herdr interaction.

## Property-testing posture

Explicit strategies live under `tests/support/strategies.rs`. Persistence
strategies generate both valid schema values and hostile byte/string inputs.
Production modules expose small deterministic functions where a property needs
to test parsing, merging, projection, or publication without running a TUI.

Pure properties use proptest's default 256 cases. Filesystem properties set
`ProptestConfig::with_cases(64)` to keep normal CI fast. Developers can override
the case count and other runner settings using proptest's standard environment
variables.

The suite adds these properties:

### Persistence

- any valid `PersistedStateV1` JSON round-trip preserves equality;
- arbitrary bytes for config, state, and guestbook loading never panic;
- state parsing returns either one valid v1 value or diagnostics, never partial
  application;
- guestbook replay is ordered, deduplicated, and bounded for arbitrary entry
  sequences and malformed interleaving;
- projecting, serializing, loading, and overlaying state is idempotent;
- overlay never changes live topology, presence, focus, or pane revision;
- selection after overlay is `None` or references a live agent;
- seen attention is restored only for an exact episode match;
- persona restoration never crosses `PersonaKey` boundaries;
- successful atomic publication exposes a complete previous or next JSON
  document, never a partially written destination.

### Core domain invariants

- persona key and appearance generation are deterministic for arbitrary agent
  identities;
- `Attention::mark_seen` is idempotent;
- applying a duplicate semantic event is idempotent;
- stale pane revisions cannot regress a newer agent state;
- after arbitrary valid pane additions, exits, closures, and snapshot
  replacements, selection is absent or live;
- `SiteStatus` always follows the documented priority regardless of agent
  insertion order;
- guestbook event IDs are stable for equal event identity inputs.

Existing hand-written tests remain the primary explanation of named behaviours.
Properties complement them; they do not replace rendering goldens, socket
fixtures, or live Herdr acceptance tests.

Proptest's source-parallel `proptest-regressions` files are committed. Any
minimal failing input found in development or CI stays in that corpus. When the
failure represents a comprehensible product rule, add a named example regression
test as well.

## Verification and acceptance

The slice is complete when:

1. absent config and state files start with safe defaults;
2. a valid config affects the initial view and display preferences;
3. an explicit `desk` or `cafe` action overrides saved and configured views;
4. view, preferences, selection, personas, and exact seen episodes survive a
   restart;
5. stale saved selection and attention cannot attach to unrelated live agents;
6. guestbook history replays across restart and tolerates malformed records;
7. rapid state changes result in one debounced atomic state publication;
8. an unchanged idle or animated UI performs no persistence writes;
9. shutdown flushes the latest staged state and guestbook before terminal
   teardown completes;
10. read-only, malformed, truncated, and unsupported-version files yield visible
    non-fatal diagnostics;
11. property failures shrink and persist in a tracked regression file;
12. `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`,
    and `cargo test --all-targets --all-features` pass;
13. the manual Herdr 0.7.3 acceptance loop still passes after persistence is
    enabled.

Implementation follows red-green-refactor. Start with schema/parser and merge
properties, then atomic state publication, guestbook replay/append, worker
integration, startup precedence, and finally lifecycle shutdown. Do not start
release engineering until these persistence acceptance criteria are green.
