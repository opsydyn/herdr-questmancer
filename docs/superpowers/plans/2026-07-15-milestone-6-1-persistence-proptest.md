# Milestone 6.1 Persistence and Property Testing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> `superpowers:subagent-driven-development` (recommended) or
> `superpowers:executing-plans` to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Persist webmaster intent and guestbook history safely across restarts,
apply it only to matching live Herdr state, and use proptest to enforce the
persistence and core-domain invariants.

**Architecture:** `state.json` is a versioned atomic snapshot of durable user
intent, while `guestbook.jsonl` is an append-only semantic history. One
long-lived Tokio worker serializes both stores and debounces state replacement;
startup overlays validated durable intent onto each fresh Herdr snapshot without
replacing live topology or presence. Explicit proptest strategies exercise pure
parsers, projection/overlay, reducer sequences, and filesystem publication.

**Tech Stack:** Rust 2024 with Rust 1.90, Tokio, Serde/serde_json, toml 1.1,
proptest 1.11, tempfile, Herdr 0.7.3 protocol 16, Ratatui 0.30.

## Global Constraints

- Use `toml = "1.1"` as a runtime dependency and `proptest = "1.11"` only as a
  development dependency.
- Do not add `proptest-derive` or derive `Arbitrary` on production types.
- Herdr is authoritative for live workspaces, panes, agents, presence, focus,
  revisions, and output; never deserialize those from `state.json` into the
  live domain.
- `runtime.json` remains ephemeral singleton registration state owned by
  `src/runtime.rs` and `herdr/control.sh`.
- Configuration is read-only at
  `$HERDR_PLUGIN_CONFIG_DIR/config.toml`; the plugin never creates it.
- Durable state lives at `$HERDR_PLUGIN_STATE_DIR/state.json`; history lives at
  `$HERDR_PLUGIN_STATE_DIR/guestbook.jsonl`.
- Missing plugin directories disable their corresponding persistence without
  preventing offline or live startup.
- View precedence is explicit CLI/action view, persisted `last_view`, configured
  `default_view`, then built-in `desk`.
- State replacement is debounced for exactly 250 ms and uses same-directory
  temporary write, file sync, atomic rename, and best-effort parent sync.
- Guestbook appends are single-writer, newline-terminated, flushed, ordered,
  deduplicated by `EventId`, and bounded in memory only.
- `AttentionEpisodeKey` is exactly persona, pane revision, and reason. Do not add
  `since`: Herdr snapshots do not expose the original status timestamp.
- Persistence failures are non-fatal, bounded, visible, and never erase the last
  published valid state.
- Animation ticks and unchanged persisted projections perform no writes.
- No unsafe code, database, telemetry, network service, or release-engineering
  expansion in this slice.
- Every behavior change follows red-green-refactor and ends in a focused commit.

## File map

```text
src/config.rs                         config schema, validation, and path discovery
src/persistence/mod.rs                public persistence types and diagnostics
src/persistence/state.rs              v1 schema, validation, capture, and overlay
src/persistence/atomic_json.rs        state load and atomic publication
src/persistence/guestbook_jsonl.rs    byte-safe JSONL replay and append
src/persistence/worker.rs             debounced single-writer Tokio task
src/persistence/startup.rs            concurrent local load and precedence
tests/support/mod.rs                   integration-test support exports
tests/support/strategies.rs            explicit proptest strategies
tests/config.rs                        config/path examples and hostile-input property
tests/persisted_state.rs               schema, projection, and overlay properties
tests/atomic_state.rs                  publication examples and filesystem property
tests/guestbook_persistence.rs         replay/append examples and properties
tests/persistence_worker.rs            debounce, dedupe, error, and shutdown tests
tests/property_domain.rs               core domain invariant properties
tests/startup.rs                       effective startup and reconnect integration
```

---

### Task 1: Typed configuration and persistence paths

**Files:**
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `src/app.rs`
- Modify: `src/lib.rs`
- Create: `src/config.rs`
- Create: `tests/config.rs`
- Modify: `tests/cli.rs`

**Interfaces:**
- Produces: `WebmasterConfig::parse(&[u8]) -> Result<Self, ConfigError>`.
- Produces: `PersistencePaths::from_lookup`, `config_path`, `state_path`, and
  `guestbook_path`.
- Produces: serializable snake-case `View`, `Motion`, `CharacterSet`,
  `ColorMode`, and `DisplayPreferences`.
- Consumed by: Tasks 2, 5, and 6.

- [ ] **Step 1: Add dependencies and write failing configuration examples.**

  Add `toml = "1.1"` to `[dependencies]`, `proptest = "1.11"` to
  `[dev-dependencies]`, and add Tokio's `fs` feature. In `tests/config.rs`, cover
  complete defaults, every accepted enum, unknown-field tolerance, all numeric
  bounds, empty `reviewr_action`, whole-file rejection, and optional paths:

  ```rust
  use herdr_webmaster::{
      app::{CharacterSet, ColorMode, Motion, View},
      config::{PersistencePaths, WebmasterConfig},
  };

  #[test]
  fn parses_a_complete_configuration() {
      let config = WebmasterConfig::parse(br#"
          default_view = "cafe"
          motion = "reduced"
          character_set = "ascii"
          color_mode = "ansi16"
          output_preview_lines = 120
          guestbook_max_entries = 750
          reviewr_action = "persiyanov.reviewr.open"
          show_elapsed_time = false
          future_field = "accepted"
      "#).unwrap();

      assert_eq!(config.default_view, View::Cafe);
      assert_eq!(config.preferences.motion, Motion::Reduced);
      assert_eq!(config.preferences.character_set, CharacterSet::Ascii);
      assert_eq!(config.preferences.color_mode, ColorMode::Ansi16);
      assert_eq!(config.output_preview_lines, 120);
      assert_eq!(config.guestbook_max_entries, 750);
      assert!(!config.show_elapsed_time);
  }

  #[test]
  fn empty_reviewr_action_rejects_the_whole_file() {
      let error = WebmasterConfig::parse(b"reviewr_action = '   '").unwrap_err();
      assert!(error.to_string().contains("reviewr_action"));
  }

  #[test]
  fn discovers_each_plugin_directory_independently() {
      let paths = PersistencePaths::from_lookup(|name| match name {
          "HERDR_PLUGIN_CONFIG_DIR" => Some("/tmp/config".into()),
          "HERDR_PLUGIN_STATE_DIR" => Some("/tmp/state".into()),
          _ => None,
      });
      assert_eq!(paths.config_path().unwrap(), std::path::Path::new("/tmp/config/config.toml"));
      assert_eq!(paths.state_path().unwrap(), std::path::Path::new("/tmp/state/state.json"));
      assert_eq!(paths.guestbook_path().unwrap(), std::path::Path::new("/tmp/state/guestbook.jsonl"));
  }
  ```

- [ ] **Step 2: Verify RED.**

  Run `cargo test --test config --test cli`. Expected: compilation fails because
  `config` does not exist and `Command::Ui` still requires a concrete default
  view.

- [ ] **Step 3: Implement typed parsing and validation.**

  Derive `Serialize`/`Deserialize` with `#[serde(rename_all = "snake_case")]` on
  the four enums and `DisplayPreferences`. Implement this exact public shape in
  `src/config.rs`:

  ```rust
  #[derive(Clone, Debug, Eq, PartialEq)]
  pub struct WebmasterConfig {
      pub default_view: View,
      pub preferences: DisplayPreferences,
      pub output_preview_lines: u32,
      pub guestbook_max_entries: usize,
      pub reviewr_action: String,
      pub show_elapsed_time: bool,
  }

  impl Default for WebmasterConfig {
      fn default() -> Self {
          Self {
              default_view: View::Desk,
              preferences: DisplayPreferences::default(),
              output_preview_lines: 80,
              guestbook_max_entries: 500,
              reviewr_action: "persiyanov.reviewr.open".to_owned(),
              show_elapsed_time: true,
          }
      }
  }

  impl WebmasterConfig {
      pub fn parse(bytes: &[u8]) -> Result<Self, ConfigError>;
  }

  #[derive(Clone, Debug, Default, Eq, PartialEq)]
  pub struct PersistencePaths {
      config_dir: Option<PathBuf>,
      state_dir: Option<PathBuf>,
  }

  impl PersistencePaths {
      pub fn from_env() -> Self;
      pub fn from_lookup(lookup: impl FnMut(&str) -> Option<String>) -> Self;
      pub fn config_path(&self) -> Option<PathBuf>;
      pub fn state_path(&self) -> Option<PathBuf>;
      pub fn guestbook_path(&self) -> Option<PathBuf>;
  }
  ```

  Deserialize through a private `ConfigFile` with `#[serde(default)]`, convert
  it to `WebmasterConfig`, validate `10..=500`, `50..=10_000`, and trimmed
  non-empty action, and return one `ConfigError` rather than partially applying
  values.

- [ ] **Step 4: Make the CLI view optional.**

  Change `Command::Ui` to `view: Option<View>` with `#[arg(long, value_enum)]`.
  Update `tests/cli.rs` so bare `ui` equals `Command::Ui { view: None }` and
  `--view cafe` equals `Some(View::Cafe)`. Do not change terminal startup or
  shell actions yet.

- [ ] **Step 5: Add hostile-input property coverage.**

  Add this property to `tests/config.rs`:

  ```rust
  use proptest::prelude::*;

  proptest! {
      #[test]
      fn arbitrary_config_bytes_never_panic(bytes in proptest::collection::vec(any::<u8>(), 0..4096)) {
          let _ = WebmasterConfig::parse(&bytes);
      }
  }
  ```

- [ ] **Step 6: Verify and commit.**

  Run `cargo fmt --all`, `cargo test --test config --test cli`, and
  `cargo clippy --all-targets --all-features -- -D warnings`. Expected: all
  pass. Commit with `feat: add typed webmaster configuration`.

### Task 2: Durable schema, catalog, capture, and overlay

**Files:**
- Modify: `src/app.rs`
- Modify: `src/lib.rs`
- Create: `src/persistence/mod.rs`
- Create: `src/persistence/state.rs`
- Create: `tests/support/mod.rs`
- Create: `tests/support/strategies.rs`
- Create: `tests/persisted_state.rs`

**Interfaces:**
- Produces: `PersistedStateV1`, `AttentionEpisodeKey`, `DurableIntent`,
  `PersistedStateV1::capture`, `PersistedStateV1::validate`, and
  `DurableIntent::overlay`.
- Produces: `Model::durable_intent`, `Model::durable_intent_mut`, and automatic
  overlay/synchronization from `Model::replace_domain`.
- Consumed by: Tasks 3, 5, and 6.

- [ ] **Step 1: Write failing schema and relationship tests.**

  In `tests/persisted_state.rs`, construct a fixed model from the Herdr snapshot
  fixture and assert schema version `1`, selection by `PersonaKey`, persona-map
  key equality, and seen-episode identity:

  ```rust
  mod support;

  use herdr_webmaster::{
      app::{Model, View},
      persistence::{AttentionEpisodeKey, PersistedStateV1},
  };

  #[test]
  fn capture_contains_only_durable_intent() {
      let mut model = Model::new(View::Cafe);
      model.replace_domain(support::fixture_domain());
      let agent = model.selected_agent().unwrap();
      let expected_persona = agent.persona.key.clone();
      let expected_revision = agent.pane_revision;
      let expected_reason = agent.attention.reason().unwrap();
      model.mark_selected_attention_seen();

      let state = PersistedStateV1::capture(&model);

      assert_eq!(state.schema_version, 1);
      assert_eq!(state.selected_persona, Some(expected_persona.clone()));
      assert_eq!(state.personas[&expected_persona].key, expected_persona);
      assert!(state.seen_attention.contains(&AttentionEpisodeKey {
          persona: expected_persona.clone(),
          pane_revision: expected_revision,
          reason: expected_reason,
      }));
  }
  ```

  Add named invalid-state tests for a mismatched embedded persona key, selected
  persona missing from the map, and seen episode referencing a missing persona.

- [ ] **Step 2: Verify RED.**

  Run `cargo test --test persisted_state`. Expected: compilation fails because
  the persistence schema and durable model catalog do not exist.

- [ ] **Step 3: Implement exact schema types and validation.**

  Create `src/persistence/mod.rs` and `state.rs` with:

  ```rust
  pub const STATE_SCHEMA_VERSION: u32 = 1;

  #[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
  pub struct AttentionEpisodeKey {
      pub persona: PersonaKey,
      pub pane_revision: u64,
      pub reason: AttentionReason,
  }

  #[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
  pub struct PersistedStateV1 {
      pub schema_version: u32,
      pub last_view: View,
      pub preferences: DisplayPreferences,
      pub selected_persona: Option<PersonaKey>,
      pub personas: BTreeMap<PersonaKey, AgentPersona>,
      pub seen_attention: BTreeSet<AttentionEpisodeKey>,
  }

  #[derive(Clone, Debug, Default, Eq, PartialEq)]
  pub struct DurableIntent {
      selected_persona: Option<PersonaKey>,
      personas: BTreeMap<PersonaKey, AgentPersona>,
      seen_attention: BTreeSet<AttentionEpisodeKey>,
  }

  impl PersistedStateV1 {
      pub fn capture(model: &Model) -> Self;
      pub fn validate(&self) -> Result<(), StateValidationError>;
  }

  impl DurableIntent {
      pub fn seed(&mut self, state: &PersistedStateV1);
      pub fn overlay(&mut self, domain: &mut DomainState);
  }
  ```

  Extend `AttentionReason` with `Hash`, `Ord`, and `PartialOrd` derives so the
  episode key has a lawful deterministic `BTreeSet` order.

  `overlay` must restore personas by exact key, select only one matching live
  agent, mark only exact unseen episodes seen, retain all learned personas, and
  prune seen keys not represented by a current attention episode. It must never
  mutate sites, pane IDs, presence, focus, revisions, or output-bearing model
  state.

- [ ] **Step 4: Integrate the catalog into `Model`.**

  Add `durable_intent: DurableIntent` to `Model`. `replace_domain` must call
  `durable_intent.overlay(&mut domain)` before assigning the domain. Add a
  focused `mark_selected_attention_seen` method that runs the existing reducer,
  replaces the domain, and therefore synchronizes the ledger. Keep the reducer
  pure; do not move persistence types into `DomainState`.

- [ ] **Step 5: Add explicit strategies and round-trip/overlay properties.**

  `tests/support/strategies.rs` must export strategies for typed IDs,
  `AttentionReason`, personas, `PersistedStateV1`, agents, and small valid
  domains. Add properties equivalent to:

  ```rust
  proptest! {
      #[test]
      fn valid_state_json_round_trips(state in support::persisted_state()) {
          let bytes = serde_json::to_vec(&state).unwrap();
          let decoded: PersistedStateV1 = serde_json::from_slice(&bytes).unwrap();
          prop_assert_eq!(decoded, state);
      }

      #[test]
      fn overlay_cannot_replace_live_facts(
          mut domain in support::domain_state(),
          state in support::persisted_state(),
      ) {
          let before = support::live_facts(&domain);
          let mut intent = DurableIntent::default();
          if state.validate().is_ok() {
              intent.seed(&state);
          }
          intent.overlay(&mut domain);
          prop_assert_eq!(support::live_facts(&domain), before);
          prop_assert!(domain.selected_agent.as_ref().is_none_or(|key| domain.agents.contains_key(key)));
      }
  }
  ```

  Configure pure properties with proptest's default 256 cases. Keep generated
  source-parallel regression files under version control.

- [ ] **Step 6: Verify and commit.**

  Run `cargo fmt --all`, `cargo test --test persisted_state`, and
  `cargo clippy --all-targets --all-features -- -D warnings`. Expected: all
  pass. Commit with `feat: model durable webmaster intent`.

### Task 3: Atomic state loading and publication

**Files:**
- Modify: `src/persistence/mod.rs`
- Create: `src/persistence/atomic_json.rs`
- Create: `tests/atomic_state.rs`

**Interfaces:**
- Produces: `parse_state`, `load_state`, and `publish_state`.
- Produces: structured `PersistenceDiagnostic` and `PersistenceError` carrying
  operation, path, optional one-based line, and source message.
- Consumed by: Task 5 worker and Task 6 startup.

- [ ] **Step 1: Write failing load and validation tests.**

  Cover missing file as `Ok(None)`, valid v1, malformed JSON, schema `2`, every
  invalid relationship from Task 2, arbitrary bytes, and preservation of the
  original invalid file. Use this public contract:

  ```rust
  pub fn parse_state(path: &Path, bytes: &[u8])
      -> Result<PersistedStateV1, PersistenceDiagnostic>;

  pub async fn load_state(path: &Path)
      -> Result<Option<PersistedStateV1>, PersistenceDiagnostic>;

  pub async fn publish_state(
      path: &Path,
      state: &PersistedStateV1,
  ) -> Result<(), PersistenceError>;
  ```

- [ ] **Step 2: Verify RED.**

  Run `cargo test --test atomic_state`. Expected: compilation fails because the
  atomic store does not exist.

- [ ] **Step 3: Implement byte parsing and non-destructive loading.**

  `parse_state` must deserialize, require schema version `1`, then call
  `validate`. `load_state` maps only `ErrorKind::NotFound` to `Ok(None)` and
  converts every other error to a path-bearing diagnostic. No read failure may
  remove or rewrite either the destination or a leftover temporary file.

- [ ] **Step 4: Write failing publication tests.**

  Assert pretty JSON plus trailing newline, replacement of a valid old document,
  creation of a missing parent directory, cleanup/reuse of `state.json.tmp`, and
  retention of the prior destination when temporary-file creation or rename
  fails. Use a read-only parent on Unix for the named permission regression.

- [ ] **Step 5: Implement atomic publication.**

  Serialize before opening any file. Create the state directory, write
  `state.json.tmp` in the same directory with `tokio::fs::File`, call
  `sync_all`, rename over the destination, then best-effort sync a
  `std::fs::File::open(parent)` inside `tokio::task::spawn_blocking`. On failure,
  best-effort remove only the temporary file and retain the last destination.

- [ ] **Step 6: Add the 64-case filesystem property.**

  Use `ProptestConfig::with_cases(64)` and alternate two valid generated states
  while a reader repeatedly parses the destination. Assert every successful
  read equals the complete previous or next state and never partial JSON:

  ```rust
  proptest! {
      #![proptest_config(ProptestConfig::with_cases(64))]

      #[test]
      fn atomic_publication_never_exposes_partial_json(
          first in support::persisted_state(),
          second in support::persisted_state(),
      ) {
          let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
          runtime.block_on(support::assert_atomic_publication(first, second));
      }
  }
  ```

- [ ] **Step 7: Verify and commit.**

  Run `cargo fmt --all`, `cargo test --test atomic_state`, and
  `cargo clippy --all-targets --all-features -- -D warnings`. Expected: all
  pass. Commit with `feat: publish atomic webmaster state`.

### Task 4: Byte-safe guestbook replay and append

**Files:**
- Modify: `src/domain/guestbook.rs`
- Modify: `src/persistence/mod.rs`
- Create: `src/persistence/guestbook_jsonl.rs`
- Create: `tests/guestbook_persistence.rs`

**Interfaces:**
- Produces: `ReplayResult { guestbook, diagnostics }`, `replay_guestbook`,
  `load_guestbook`, and `append_guestbook`.
- Consumed by: Task 5 worker and Task 6 startup.

- [ ] **Step 1: Write failing replay examples.**

  Cover chronological reordering, duplicate IDs, maximum eviction, malformed
  UTF-8, malformed JSON between valid lines, a non-newline-terminated final
  record, absent file, and more than five errors. Assert diagnostic line numbers
  and one folded summary:

  ```rust
  #[test]
  fn malformed_records_do_not_hide_valid_history() {
      let bytes = b"{\"bad\":true}\n\xff\n{\"also\":\"bad\"}";
      let replay = replay_guestbook(Path::new("guestbook.jsonl"), bytes, 500);
      assert!(replay.guestbook.entries().is_empty());
      assert_eq!(replay.diagnostics[0].line, Some(1));
      assert_eq!(replay.diagnostics[1].line, Some(2));
      assert_eq!(replay.diagnostics[2].line, Some(3));
  }
  ```

- [ ] **Step 2: Verify RED.**

  Run `cargo test --test guestbook_persistence`. Expected: compilation fails
  because the JSONL module does not exist.

- [ ] **Step 3: Implement replay over bytes, not `str::lines`.**

  Split on `b'\n'`, reject a non-empty unterminated final slice as truncated,
  decode each complete line independently, and append valid entries through
  `Guestbook::append`. Retain at most five individual diagnostics plus one
  summary count. Add `Guestbook::with_entries(maximum_entries, entries)` only if
  it delegates to `append` and preserves the existing invariants.

- [ ] **Step 4: Write failing append tests and implement append.**

  Assert one compact serialized record plus one newline, parent creation,
  ordered multiple appends, and a path-bearing error for an unwritable file.
  `append_guestbook(path, entry)` must use one `OpenOptions` append handle, write
  all bytes, and `sync_data` before returning success.

- [ ] **Step 5: Add arbitrary interleaving properties.**

  Generate valid entries, duplicates, arbitrary invalid byte records, and a
  random bound `1..100`. Assert replayed entries are unique by ID, chronological,
  at most the bound, and equal to folding only valid complete records through
  `Guestbook::append`. Use 256 cases because replay is pure.

- [ ] **Step 6: Verify and commit.**

  Run `cargo fmt --all`,
  `cargo test --test guestbook --test guestbook_persistence`, and
  `cargo clippy --all-targets --all-features -- -D warnings`. Expected: all
  pass. Commit with `feat: persist bounded guestbook history`.

### Task 5: Debounced single-writer persistence worker

**Files:**
- Modify: `src/persistence/mod.rs`
- Create: `src/persistence/worker.rs`
- Create: `tests/persistence_worker.rs`

**Interfaces:**
- Produces: `PersistenceWorker::start`, `PersistenceClient::stage_state`,
  `PersistenceClient::append_guestbook`, `PersistenceClient::flush`, and
  `PersistenceClient::shutdown`.
- Produces: a bounded `mpsc::Receiver<PersistenceDiagnostic>` for UI reporting.
- Consumed by: Task 7 terminal/runtime integration.

- [ ] **Step 1: Write failing debounce and deduplication tests.**

  Use `#[tokio::test(start_paused = true)]` and `tempdir`. Stage three distinct
  states without advancing time, prove there is no destination at 249 ms, then
  advance one millisecond and assert only the newest state was published. Stage
  the same projection again and assert destination metadata is unchanged:

  ```rust
  #[tokio::test(start_paused = true)]
  async fn coalesces_state_to_the_latest_value_after_250_milliseconds() {
      let directory = tempfile::tempdir().unwrap();
      let paths = WorkerPaths::new(
          Some(directory.path().join("state.json")),
          Some(directory.path().join("guestbook.jsonl")),
      );
      let (mut client, _diagnostics, worker) = PersistenceWorker::start(paths);
      let first = support::fixed_persisted_state(View::Desk);
      let latest = support::fixed_persisted_state(View::Cafe);

      assert!(client.stage_state(first).unwrap());
      assert!(client.stage_state(latest.clone()).unwrap());
      tokio::time::advance(Duration::from_millis(249)).await;
      tokio::task::yield_now().await;
      assert!(!directory.path().join("state.json").exists());

      tokio::time::advance(Duration::from_millis(1)).await;
      tokio::task::yield_now().await;
      assert_eq!(load_state(&directory.path().join("state.json")).await.unwrap(), Some(latest));
      client.shutdown().await.unwrap();
      worker.await.unwrap();
  }
  ```

- [ ] **Step 2: Verify RED.**

  Run `cargo test --test persistence_worker`. Expected: compilation fails
  because the worker does not exist.

- [ ] **Step 3: Implement the typed worker protocol.**

  Use this exact message boundary:

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

  #[derive(Clone, Debug, Eq, PartialEq)]
  pub struct WorkerPaths {
      pub state: Option<PathBuf>,
      pub guestbook: Option<PathBuf>,
  }

  #[derive(Debug)]
  pub struct PersistenceClient {
      sender: mpsc::UnboundedSender<PersistenceMessage>,
      last_staged: Option<PersistedStateV1>,
  }

  impl PersistenceClient {
      pub fn stage_state(
          &mut self,
          state: PersistedStateV1,
      ) -> Result<bool, PersistenceError>;
  }
  ```

  `PersistenceWorker::start` returns the client, a bounded diagnostic receiver,
  and its `JoinHandle`. `stage_state` compares against `last_staged` before an
  infallible unbounded-channel send and returns whether it staged a new value.
  The worker keeps one optional dirty state and one resettable 250 ms sleep; it
  does not arm a repeating timer when clean. Guestbook appends execute in
  message order and acknowledge after `sync_data`.

- [ ] **Step 4: Add failing shutdown, disabled-path, and error tests.**

  Prove `flush` publishes dirty state without waiting for the debounce,
  `shutdown` publishes then exits, `None` paths remain successful no-ops, a
  guestbook append is acknowledged, and unwritable paths produce diagnostics
  while the worker accepts a later distinct state. Assert the diagnostic queue
  remains bounded when many failures occur.

- [ ] **Step 5: Implement flush, shutdown, and non-fatal diagnostics.**

  State publication failures keep the worker alive and clear that attempted
  dirty value; only a later distinct `stage_state` retries. Send the newest
  bounded diagnostic with operation/path/message. `shutdown` flushes state,
  completes prior append messages by FIFO ordering, replies once, and terminates
  the task.

- [ ] **Step 6: Verify and commit.**

  Run `cargo fmt --all`, `cargo test --test persistence_worker`, and
  `cargo clippy --all-targets --all-features -- -D warnings`. Expected: all
  pass. Commit with `feat: add debounced persistence worker`.

### Task 6: Concurrent startup load and deterministic precedence

**Files:**
- Modify: `src/app.rs`
- Modify: `src/lib.rs`
- Modify: `src/config.rs`
- Modify: `src/command.rs`
- Modify: `src/interaction.rs`
- Modify: `src/persistence/mod.rs`
- Create: `src/persistence/startup.rs`
- Modify: `src/runtime_loop.rs`
- Modify: `src/ui/views/desk.rs`
- Modify: `tests/command.rs`
- Modify: `tests/interaction.rs`
- Create: `tests/startup.rs`
- Modify: `tests/runtime_loop.rs`
- Modify: `tests/desk_rendering.rs`

**Interfaces:**
- Produces: `load_startup(paths, explicit_view) -> StartupData`.
- Produces: `RuntimeSettings` and a fully restored `StartupData::model`.
- Produces: config-driven output line count, reviewr action, guestbook bound, and
  elapsed-time visibility.
- Consumed by: Task 7 terminal entrypoint.

- [ ] **Step 1: Write failing precedence and load tests.**

  In `tests/startup.rs`, create all sixteen combinations of explicit/persisted/
  configured/built-in view and assert:

  ```rust
  #[test]
  fn view_precedence_is_explicit_then_persisted_then_configured_then_desk() {
      assert_eq!(effective_view(Some(View::Desk), Some(View::Cafe), View::Cafe), View::Desk);
      assert_eq!(effective_view(None, Some(View::Cafe), View::Desk), View::Cafe);
      assert_eq!(effective_view(None, None, View::Cafe), View::Cafe);
      assert_eq!(effective_view(None, None, View::Desk), View::Desk);
  }
  ```

  Add async examples for absent files, valid files, invalid config plus valid
  state, future state plus valid guestbook, malformed guestbook diagnostics,
  independent missing config/state directories, and plugin-disabled startup.

- [ ] **Step 2: Verify RED.**

  Run `cargo test --test startup`. Expected: compilation fails because
  `StartupData` and `effective_view` do not exist.

- [ ] **Step 3: Implement concurrent raw reads and ordered interpretation.**

  Define:

  ```rust
  #[derive(Clone, Debug, Eq, PartialEq)]
  pub struct RuntimeSettings {
      pub output_preview_lines: u32,
      pub reviewr_action: String,
      pub show_elapsed_time: bool,
  }

  #[derive(Debug)]
  pub struct StartupData {
      pub model: Model,
      pub settings: RuntimeSettings,
      pub paths: WorkerPaths,
      pub diagnostics: Vec<PersistenceDiagnostic>,
  }

  pub async fn load_startup(
      paths: PersistencePaths,
      explicit_view: Option<View>,
  ) -> StartupData;

  pub const fn effective_view(
      explicit: Option<View>,
      persisted: Option<View>,
      configured: View,
  ) -> View;
  ```

  Use `tokio::join!` to read config bytes, state bytes, and guestbook bytes
  concurrently. Parse config first after reads complete so its
  `guestbook_max_entries` bounds replay. Seed the durable catalog and persisted
  preferences only for a valid v1 state, install replayed guestbook history,
  then build the model using the exact view precedence.

  Refactor `bootstrap_model` to accept the restored `Model` plus the optional
  `HerdrEnvironment`; it may set connection/status state but must not replace
  restored view, preferences, durable intent, or guestbook history.

- [ ] **Step 4: Apply configuration-only runtime settings.**

  Add `settings: RuntimeSettings` to `Model` or pass it through one owned startup
  runtime object; do not put it in `DomainState`. Replace hard-coded output line
  counts in `src/runtime_loop.rs` and `src/interaction.rs` with
  `model.settings().output_preview_lines`. Make desk elapsed rendering omit the
  duration cleanly when `show_elapsed_time` is false. Keep reviewr copy and key
  unchanged.

- [ ] **Step 5: Make reviewr discovery/invocation configurable.**

  Replace command-module constants with the configured qualified ID. Split on
  the final `.` only when invoking; discovery compares the complete string. A
  non-splittable but non-empty configured value yields `ReviewrAvailable(false)`
  and a non-fatal invocation error rather than a panic. Extend `tests/command.rs`
  and `tests/runtime_loop.rs` with a custom `acme.diff.inspect` action.

- [ ] **Step 6: Add startup hostile-input property coverage.**

  Generate three arbitrary byte buffers for config, state, and guestbook, write
  them to a temp directory, call `load_startup`, and assert it never panics,
  always yields a valid view/settings/model, and any selection is absent or
  live. Use `ProptestConfig::with_cases(64)` because this property uses files.

- [ ] **Step 7: Verify and commit.**

  Run `cargo fmt --all`,
  `cargo test --test startup --test runtime_loop --test command --test desk_rendering`,
  and `cargo clippy --all-targets --all-features -- -D warnings`. Expected: all
  pass. Commit with `feat: restore persisted webmaster startup`.

### Task 7: Runtime effects, lifecycle actions, diagnostics, and shutdown

**Files:**
- Modify: `src/main.rs`
- Modify: `src/runtime.rs`
- Modify: `src/runtime_loop.rs`
- Modify: `src/terminal.rs`
- Modify: `src/interaction.rs`
- Modify: `src/update/event.rs`
- Modify: `herdr/run.sh`
- Modify: `herdr/control.sh`
- Modify: `tests/interaction.rs`
- Modify: `tests/runtime.rs`
- Modify: `tests/runtime_loop.rs`
- Modify: `tests/scripts.sh`
- Modify: `tests/persistence_worker.rs`

**Interfaces:**
- Produces: `RuntimeEffects { desk, persistence }` from every reducer path.
- Produces: terminal dispatch for `PersistState` and `AppendGuestbook`.
- Produces: bounded persistence shutdown before terminal restoration.

- [ ] **Step 1: Write failing runtime-effect tests.**

  Assert a blocked event yields one JSONL append and one staged state, mark-seen
  yields a staged state, actual view/selection changes yield staged state,
  duplicate/stale events yield neither, animation-only clock changes yield
  neither, and snapshot reconnect stages only after overlay:

  ```rust
  #[test]
  fn blocked_transition_routes_history_and_state_to_persistence() {
      let mut model = support::connected_model();
      let effects = apply_connection_update(
          &mut model,
          support::blocked_update(8),
          Timestamp::from_millis(2_000),
      );
      assert_eq!(effects.persistence.iter().filter(|effect| effect.is_guestbook_append()).count(), 1);
      assert_eq!(effects.persistence.iter().filter(|effect| **effect == Command::PersistState).count(), 1);
  }
  ```

- [ ] **Step 2: Verify RED.**

  Run `cargo test --test runtime_loop --test interaction`. Expected: assertions
  fail because persistence commands are currently discarded.

- [ ] **Step 3: Preserve pure reducer effects end-to-end.**

  Add:

  ```rust
  #[derive(Clone, Debug, Default, Eq, PartialEq)]
  pub struct RuntimeEffects {
      pub desk: Vec<DeskCommand>,
      pub persistence: Vec<crate::update::Command>,
  }

  pub struct ActionReduction {
      pub control: ControlFlow<(), ()>,
      pub commands: Vec<DeskCommand>,
      pub persistence: Vec<crate::update::Command>,
  }
  ```

  Make `apply_connection_update`, `apply_command_result`, and interaction
  reduction return/retain the domain persistence commands. Translate only
  `RequestSnapshot` to `DeskCommand::RefreshSnapshot`. Emit `PersistState` for
  real view/selection/preference mutations and not for no-ops. Preserve
  `AppendGuestbook` values exactly once.

- [ ] **Step 4: Dispatch persistence effects from both terminal loops.**

  Start the worker from `StartupData.paths`. For each effect batch, schedule desk
  commands and process persistence commands in order: capture/stage the current
  model for `PersistState`, and await acknowledged append for
  `AppendGuestbook`. Add persistence diagnostics as a `tokio::select!` branch
  that updates the non-blocking model status. The offline loop still persists
  local view/selection/seen changes when state paths exist.

- [ ] **Step 5: Write failing action-default and registration tests.**

  Update shell expectations so `open` and a closed `toggle` do not pass
  `WEBMASTER_INITIAL_VIEW`; `desk` and `cafe` still pass explicit values and
  switch existing panes. Assert the Rust runtime registration records the
  resolved effective view, not an unresolved CLI `Option<View>`.

- [ ] **Step 6: Implement lifecycle precedence.**

  `herdr/control.sh` must use an `open_pane default false` branch that omits the
  `--env` argument, while explicit branches retain it. `herdr/run.sh` adds
  `--view` only when `WEBMASTER_INITIAL_VIEW` is exactly `desk` or `cafe`.
  Change `terminal::run` to accept `Option<View>`, load startup before entering
  raw mode, resolve the view, then call `RuntimeRegistration::from_env` with the
  resolved `View`. `main.rs` passes the optional CLI value and owns no separate
  registration.

- [ ] **Step 7: Add failing shutdown tests and implement bounded shutdown.**

  With paused Tokio time, stage dirty state, request quit and signal shutdown,
  and prove the latest state plus prior guestbook append are durable. Wrap worker
  shutdown in a one-second timeout. Collect diagnostics, shut down Herdr tasks,
  shut down persistence, explicitly drop `TerminalGuard`, then print collected
  diagnostics to stderr. A persistence timeout or filesystem error must not
  prevent cursor/raw-mode restoration.

- [ ] **Step 8: Verify and commit.**

  Run `cargo fmt --all`,
  `cargo test --test interaction --test runtime --test runtime_loop --test persistence_worker`,
  `bash tests/scripts.sh`, `bash -n herdr/run.sh herdr/control.sh`, and
  `cargo clippy --all-targets --all-features -- -D warnings`. Expected: all
  pass. Commit with `feat: integrate persistent webmaster runtime`.

### Task 8: Core invariant properties, documentation, and milestone gate

**Files:**
- Modify: `tests/support/strategies.rs`
- Create: `tests/property_domain.rs`
- Modify: `.gitignore`
- Modify: `README.md`
- Modify: `PLAN.md`
- Modify: `CHANGELOG.md`
- Modify: `justfile`

**Interfaces:**
- Consumes: all persistence and domain APIs from Tasks 1-7.
- Produces: the tracked regression posture and Milestone 6.1 operator guidance.

- [ ] **Step 1: Add the remaining core-domain properties.**

  In `tests/property_domain.rs`, implement named properties for deterministic
  persona key/appearance, idempotent `Attention::mark_seen`, duplicate-event
  idempotence, stale-revision monotonicity, valid selection after arbitrary
  topology changes, site-status priority independent of insertion order, and
  stable guestbook IDs. The reducer properties must compare both state and
  emitted commands:

  ```rust
  proptest! {
      #[test]
      fn marking_attention_seen_is_idempotent(attention in support::attention()) {
          let once = attention.clone().mark_seen();
          let twice = once.clone().mark_seen();
          prop_assert_eq!(once, twice);
      }

      #[test]
      fn stale_revisions_never_regress_state(
          state in support::domain_with_one_agent(),
          stale_revision in 0_u64..100,
      ) {
          let current = state.agents.values().next().unwrap().pane_revision;
          prop_assume!(stale_revision < current);
          let event = support::status_event(&state, stale_revision, AgentStatus::Done);
          let (next, commands) = update(state.clone(), event);
          prop_assert_eq!(next, state);
          prop_assert!(commands.is_empty());
      }
  }
  ```

- [ ] **Step 2: Verify property shrinking and regression persistence.**

  Run `PROPTEST_CASES=1024 cargo test --test property_domain --test persisted_state`.
  Expected: all properties pass. Temporarily invert one local assertion, run its
  focused test to confirm a minimal input is written under a source-parallel
  `proptest-regressions` path, restore the assertion, rerun, and commit the
  generated regression file if proptest retains one. Do not manufacture an
  empty regression file. Ensure `.gitignore` does not exclude
  `proptest-regressions`.

- [ ] **Step 3: Update operator and contributor documentation.**

  Add the complete config example, file ownership, precedence, corruption
  behavior, local-only/no-telemetry statement, and recovery instructions to
  `README.md`. Document `PROPTEST_CASES`, regression files, focused persistence
  tests, and the three persistence paths. Mark only the persistence subsection
  of Milestone 6 complete in `PLAN.md`; leave release automation and recordings
  open. Add the slice to `CHANGELOG.md` and focused `just` recipes for
  persistence/property checks.

- [ ] **Step 4: Run the full automated gate from a clean diff.**

  Run exactly:

  ```bash
  cargo fmt --all --check
  cargo clippy --all-targets --all-features -- -D warnings
  cargo test --all-targets --all-features
  PROPTEST_CASES=1024 cargo test --test property_domain --test persisted_state
  bash tests/scripts.sh
  bash -n herdr/install.sh herdr/run.sh herdr/control.sh
  cargo build --release
  git diff --check
  ```

  Expected: every command exits `0`; shell tests report their updated exact pass
  count; no test creates untracked runtime/config/state/guestbook files outside
  temporary directories.

- [ ] **Step 5: Run live Herdr 0.7.3 acceptance.**

  Run `herdr --version`, `herdr status`, `cargo build`, `herdr plugin link .`, and
  `herdr plugin action invoke opsydyn.webmaster.open`. Generate blocked, seen,
  done, and restarted-pane states using the README's fake-agent commands. Verify
  view/preferences/persona/seen restoration, guestbook replay, no duplicated
  history after reconnect, focus/reply, and selected-output reads. If no Herdr
  server is available, record that as an environmental acceptance blocker and
  do not describe the live gate as passed.

- [ ] **Step 6: Review and commit the milestone.**

  Review `git diff --stat`, `git status --short`, and the commits since
  `f2bfd94`. Confirm there are no unrelated edits, generated state files,
  placeholders, or release-scope changes. Commit documentation/regressions with
  `docs: complete persistence milestone`, then run
  `git status --short --branch` and require a clean feature branch.

## Acceptance mapping

- Config parsing, defaults, bounds, and precedence: Tasks 1 and 6.
- Versioned durable schema and relationship validation: Task 2.
- Stable persona, selection, and exact seen-episode restoration: Tasks 2 and 6.
- Atomic non-destructive state publication: Task 3.
- Ordered, bounded, tolerant guestbook replay and append: Task 4.
- Debounce, unchanged-state suppression, and no idle writes: Task 5.
- Runtime command routing, diagnostics, offline use, and shutdown flush: Task 7.
- Persistence and core-invariant proptest coverage: Tasks 1-8.
- Full automated and live Herdr 0.7.3 verification: Task 8.
