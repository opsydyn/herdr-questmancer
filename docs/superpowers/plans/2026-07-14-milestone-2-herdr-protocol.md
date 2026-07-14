# herdr-webmaster Milestone 2 Herdr Protocol Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a schema-grounded asynchronous Herdr `0.7.3` client that bootstraps from `session.snapshot`, streams lifecycle and per-pane agent events, reports compatibility failures, and reconnects with a fresh snapshot.

**Architecture:** Normal requests open one short-lived Unix socket because Herdr reads one request per connection. A separate long-lived socket owns `events.subscribe`; its request contains global lifecycle subscriptions plus one `pane.agent_status_changed` subscription per snapshotted pane. A supervisor emits transport updates through a Tokio channel and rebuilds the subscription with a fresh snapshot after disconnect or pane-topology changes.

**Tech Stack:** Rust 2024, Tokio Unix sockets and channels, Serde/serde_json, thiserror, Herdr protocol 16 fixtures generated from the locally installed Herdr `0.7.3` schema.

## Global Constraints

- Target Herdr `0.7.3` and protocol `16`; do not guess wire shapes beyond `/tmp/herdr-api.schema.json` and the tagged `v0.7.3` source.
- Send and receive newline-delimited JSON with exactly one request per ordinary connection.
- Preserve unknown JSON fields and unknown event names without crashing.
- Use a separate long-lived subscription connection; never mix pushed events into ordinary request handling.
- Resnapshot after reconnect and whenever pane topology requires rebuilding per-pane subscriptions.
- Keep protocol types independent from the application domain reducer.
- Support macOS and Linux; Windows named pipes remain out of scope.
- No unsafe Rust and no polling pane output.

---

### Task 1: Environment and schema fixtures

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/lib.rs`
- Create: `src/herdr/mod.rs`
- Create: `src/herdr/environment.rs`
- Create: `tests/environment.rs`
- Create: `tests/fixtures/herdr/pong.json`
- Create: `tests/fixtures/herdr/session_snapshot.json`
- Create: `tests/fixtures/herdr/events.jsonl`
- Create: `tests/fixtures/herdr/error.json`

**Interfaces:**
- Produces: `HerdrEnvironment::new(socket_path, bin_path)` and `HerdrEnvironment::from_lookup(|name| ...)`.
- Fixtures represent protocol `16` success, snapshot, mixed lifecycle/dotted events, unknown fields, and errors.

- [ ] **Step 1: Write failing environment tests**

```rust
#[test]
fn requires_socket_path() {
    let result = HerdrEnvironment::from_lookup(|name| match name {
        "HERDR_BIN_PATH" => Some("/usr/local/bin/herdr".into()),
        _ => None,
    });
    assert!(matches!(result, Err(EnvironmentError::Missing("HERDR_SOCKET_PATH"))));
}
```

- [ ] **Step 2: Run `cargo test --test environment` and verify unresolved `herdr::environment` imports fail**
- [ ] **Step 3: Add `thiserror = "2"`, Tokio `net/io-util/sync/test-util` features, the `herdr` module, and lookup-based environment parsing**
- [ ] **Step 4: Add schema-derived fixture files with extra unknown fields and run `jq empty tests/fixtures/herdr/*.json` plus `jq -c . tests/fixtures/herdr/events.jsonl`**
- [ ] **Step 5: Run `cargo test --test environment` and verify all environment tests pass**
- [ ] **Step 6: Commit with `git commit -m "feat: add Herdr protocol environment and fixtures"`**

### Task 2: Wire protocol types

**Files:**
- Create: `src/herdr/protocol.rs`
- Create: `tests/protocol.rs`

**Interfaces:**
- Produces: `Request<P>`, `SuccessResponse<T>`, `ErrorResponse`, `Pong`, `SessionSnapshot`, `WireEvent`, `AgentStatus`, and snapshot record types.
- `Request::new(id, method, params)` serializes `id`, `method`, and `params` in that order-independent JSON shape.

- [ ] **Step 1: Write failing fixture tests for ping, snapshot, mixed events, unknown fields, and error responses**

```rust
#[test]
fn snapshot_tolerates_unknown_fields() {
    let response: SuccessResponse<SessionSnapshotResult> =
        serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();
    assert_eq!(response.result.snapshot.protocol, 16);
    assert_eq!(response.result.snapshot.agents[0].agent_status, AgentStatus::Blocked);
}
```

- [ ] **Step 2: Run `cargo test --test protocol` and verify missing protocol types fail compilation**
- [ ] **Step 3: Implement only schema fields needed for bootstrap and later domain normalization; use Serde defaults for optional fields and a string-plus-`Value` `WireEvent` envelope for forward-compatible event names**
- [ ] **Step 4: Run `cargo test --test protocol` and verify every fixture passes**
- [ ] **Step 5: Commit with `git commit -m "feat: decode Herdr protocol 16 messages"`**

### Task 3: Async JSON-lines framing

**Files:**
- Create: `src/herdr/framing.rs`
- Create: `tests/framing.rs`

**Interfaces:**
- Produces: `write_json_line<W, T>(&mut W, &T)`, `read_json_line<R, T>(&mut BufReader<R>)`, and `read_optional_json_line<R, T>` for Tokio async readers/writers.
- Empty non-EOF lines return `FramingError::EmptyLine`; EOF returns `Ok(None)` only from the optional reader.

- [ ] **Step 1: Write failing duplex-stream tests for a message split across writes, several messages in one write, blank input, invalid JSON, and EOF**
- [ ] **Step 2: Run `cargo test --test framing` and verify unresolved framing functions fail**
- [ ] **Step 3: Implement newline writing with flush and `AsyncBufReadExt::read_line` decoding without assuming read boundaries**
- [ ] **Step 4: Run `cargo test --test framing` and verify all framing tests pass**
- [ ] **Step 5: Commit with `git commit -m "feat: add async Herdr JSON-lines framing"`**

### Task 4: Short-lived request client

**Files:**
- Create: `src/herdr/client.rs`
- Create: `tests/client.rs`

**Interfaces:**
- Produces: `HerdrClient::new(PathBuf)`, `ping()`, `snapshot()`, and generic private `request(method, params)`.
- Produces: `ClientError::{Io, Framing, Server, MismatchedId, UnexpectedResult}`.

- [ ] **Step 1: Write failing Unix-listener tests proving each request gets a new connection, request ids are checked, server errors retain code/message, and unknown result types fail clearly**
- [ ] **Step 2: Run `cargo test --test client` and verify the client type is missing**
- [ ] **Step 3: Implement atomic request ids, one Unix connection per request, typed result decoding, and protocol helpers for `ping` and `session.snapshot`**
- [ ] **Step 4: Run `cargo test --test client` and verify all request tests pass**
- [ ] **Step 5: Commit with `git commit -m "feat: add Herdr request client"`**

### Task 5: Mixed event subscription

**Files:**
- Create: `src/herdr/subscription.rs`
- Create: `tests/subscription.rs`

**Interfaces:**
- Produces: `SubscriptionRequest::for_snapshot(&SessionSnapshot)` and `HerdrSubscription::connect(socket_path, request)`.
- Produces: `HerdrSubscription::next_event() -> Result<Option<WireEvent>, ClientError>`.
- The request contains global workspace/worktree/tab/pane/layout subscriptions and one dotted `pane.agent_status_changed` entry per unique pane id.

- [ ] **Step 1: Write failing request-shape tests and a Unix-listener test that sends a `subscription_started` ack followed by raw `workspace_created` and dotted `pane.agent_status_changed` envelopes in one write**
- [ ] **Step 2: Run `cargo test --test subscription` and verify missing subscription types fail**
- [ ] **Step 3: Implement deterministic subscription ordering, ack validation, mixed envelope decoding, and clean EOF handling**
- [ ] **Step 4: Run `cargo test --test subscription` and verify lifecycle, dotted, unknown, coalesced, error-ack, and disconnect cases pass**
- [ ] **Step 5: Commit with `git commit -m "feat: stream Herdr lifecycle events"`**

### Task 6: Bootstrap and reconnect supervisor

**Files:**
- Create: `src/herdr/supervisor.rs`
- Create: `tests/supervisor.rs`

**Interfaces:**
- Produces: `ConnectionSupervisor::new(client, Backoff)` and `run(update_tx, shutdown_rx)`.
- Produces: `ConnectionUpdate::{Connected(SessionSnapshot), Event(WireEvent), Disconnected(String), Reconnecting { attempt, delay }, Resyncing, Incompatible { expected, actual }}`.
- `Backoff::delay(attempt)` starts at 250 ms, doubles, and caps at 10 seconds; tests inject 1-4 ms values.

- [ ] **Step 1: Write failing fake-server tests for initial ping/snapshot/subscribe, protocol mismatch, disconnect/reconnect/resnapshot, capped backoff, and topology-triggered resubscription**
- [ ] **Step 2: Run `cargo test --test supervisor` and verify the supervisor API is missing**
- [ ] **Step 3: Implement bootstrap ordering, exact protocol-16 validation, channel updates, cancellation, reconnect delay, and topology rebuild on pane-created/closed/moved/agent-detected events**
- [ ] **Step 4: Run `cargo test --test supervisor` and verify every deterministic reconnect scenario passes**
- [ ] **Step 5: Commit with `git commit -m "feat: supervise Herdr reconnect and resnapshot"`**

### Task 7: Documentation and live protocol smoke

**Files:**
- Modify: `README.md`
- Modify: `PLAN.md`
- Modify: `CHANGELOG.md`
- Modify: `justfile`

**Interfaces:**
- Produces: `just protocol-test` and documented Herdr `0.7.3` validation commands.

- [ ] **Step 1: Add the protocol fixture and focused test commands to `justfile`**
- [ ] **Step 2: Document the one-request-per-connection rule, mixed event envelopes, per-pane status subscriptions, and offline-server limitation**
- [ ] **Step 3: Start a temporary Herdr `0.7.3` server if no user server is running, run `herdr plugin link .`, verify `herdr plugin list --plugin opsydyn.webmaster --json`, then stop only the server started by this task**
- [ ] **Step 4: Run the complete verification gate**

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
bash tests/scripts.sh
bash -n herdr/install.sh herdr/run.sh herdr/control.sh
cargo build --release
git diff --check
```

- [ ] **Step 5: Commit with `git commit -m "docs: document Herdr protocol runtime"`**

## Milestone verification

- [ ] Protocol fixtures match installed schema version `1` and protocol `16`.
- [ ] Split/coalesced JSON-lines tests pass.
- [ ] Ordinary requests use separate connections.
- [ ] Subscription ack, mixed events, error, EOF, and topology rebuild tests pass.
- [ ] Reconnect takes a fresh snapshot and does not reuse the old stream.
- [ ] Unknown fields and unknown event names do not crash decoding.
- [ ] Full format, Clippy, test, shell, and release-build gates pass.
- [ ] Live plugin link is either verified on a temporary `0.7.3` server or reported with exact environmental evidence.

