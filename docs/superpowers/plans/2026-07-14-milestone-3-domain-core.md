# herdr-webmaster Milestone 3 Domain Core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> `superpowers:subagent-driven-development` or `superpowers:executing-plans` to
> implement this plan task-by-task with red-green-refactor.

**Goal:** Normalize Herdr snapshots and events into a typed, deterministic
domain model shared by the webmaster desk and cybercafe.

**Architecture:** Protocol records stop at a normalization boundary. Domain
state uses typed IDs, keeps agent presence separate from user attention,
derives site status, and owns deterministic personas and bounded guestbook
history. A pure reducer consumes timestamped semantic events and returns state
plus effect commands; widgets and socket handlers do not mutate domain state.

**Tech Stack:** Rust 2024, Serde, BLAKE3, existing Herdr protocol fixtures, and
table-driven reducer tests.

## Constraints

- Preserve visible agents and seen attention when replacing a reconnect
  snapshot where identities still match.
- Never infer attention from colour or UI selection.
- Derive `SiteStatus`; do not persist or independently mutate it.
- Persona identity uses native session reference, then workspace root plus
  agent name, then workspace plus pane ID.
- Persona handle and appearance use independent labelled hashes.
- Guestbook IDs derive from event kind, pane, revision, and timestamp.
- Duplicate and stale revisions must not create repeated attention or history.
- Unknown panes request a resnapshot rather than inventing incomplete agents.
- No wall clock, socket I/O, filesystem I/O, or randomness inside the reducer.

---

### Task 1: Typed IDs, time, presence, and attention

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/lib.rs`
- Create: `src/domain/mod.rs`
- Create: `src/domain/ids.rs`
- Create: `src/domain/agent.rs`
- Create: `src/domain/attention.rs`
- Create: `tests/domain_types.rs`

- [ ] Write failing tests for non-interchangeable IDs, protocol-to-presence
  conversion, unseen-to-seen attention, and elapsed timestamps.
- [ ] Add BLAKE3 and serializable newtypes for workspace, tab, pane, agent,
  persona, event, and timestamp values.
- [ ] Implement `Presence`, `AttentionReason`, and `Attention` transitions.
- [ ] Run `cargo test --test domain_types` and commit.

### Task 2: Stable persona generation

**Files:**
- Create: `src/domain/persona.rs`
- Create: `tests/persona.rs`

- [ ] Write failing tests for identity preference order, restart stability,
  handle/appearance independence, and trait diversity.
- [ ] Implement `PersonaKey::for_agent`, labelled BLAKE3 selection, original
  handle vocabulary, and terminal-neutral appearance traits from the visual
  bible.
- [ ] Run `cargo test --test persona` and commit.

### Task 3: Agent and site normalization

**Files:**
- Create: `src/domain/site.rs`
- Create: `src/domain/state.rs`
- Create: `tests/normalization.rs`

- [ ] Write failing fixture tests that normalize snapshot agents/workspaces,
  prefer agent-session identity, and preserve custom status and revisions.
- [ ] Implement `Agent`, `Site`, `DomainState::from_snapshot`, and derived site
  priority: needs webmaster, update ready, updating, online, offline.
- [ ] Run `cargo test --test normalization` and commit.

### Task 4: Guestbook identity and bounded history

**Files:**
- Create: `src/domain/guestbook.rs`
- Create: `tests/guestbook.rs`

- [ ] Write failing tests for stable event IDs, duplicate rejection, ordering,
  and maximum-history eviction.
- [ ] Implement `GuestbookEntry`, `GuestbookEvent`, and `Guestbook` without
  persistence I/O.
- [ ] Run `cargo test --test guestbook` and commit.

### Task 5: Pure reducer and effect boundary

**Files:**
- Create: `src/update/mod.rs`
- Create: `src/update/event.rs`
- Create: `src/update/reducer.rs`
- Create: `tests/reducer.rs`

- [ ] Write failing transition tests for working to blocked, blocked to done,
  blocked to idle, done unseen to seen, pane exit, workspace close, duplicates,
  unknown panes, and snapshot replacement.
- [ ] Implement `AppEvent`, `Command`, and pure `update(DomainState, AppEvent)`.
- [ ] Preserve seen attention/personas across snapshot replacement; emit
  guestbook commands once and request resnapshot for unknown pane events.
- [ ] Run `cargo test --test reducer` and commit.

### Task 6: Milestone documentation and verification

**Files:**
- Modify: `README.md`
- Modify: `PLAN.md`
- Modify: `CHANGELOG.md`
- Modify: `justfile`

- [ ] Document domain invariants and add `just domain-test`.
- [ ] Run format, Clippy, every Rust/shell test, release build, and diff checks.
- [ ] Commit the milestone documentation.

## Acceptance

- Presence and attention are independently testable.
- Snapshot fixture becomes typed sites and agents with stable personas.
- Every required site-status priority is covered.
- Repeated/stale events do not duplicate attention or guestbook records.
- Reconnect snapshot replacement retains user-seen state for matching agents.
- Unknown-pane events emit a resnapshot command without panicking.
- The reducer is deterministic for equal state, event, and timestamp inputs.
