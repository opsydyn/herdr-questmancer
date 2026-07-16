# Summons Acknowledgement Identity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Preserve an acknowledged Summons across restart without comparing Herdr pane-output revisions to locally synthesized status-transition revisions.

**Architecture:** Durable acknowledgement identity becomes the stable pair `(PersonaKey, GuildSummons)`. `DurableIntent::overlay` remains the single reconciliation boundary: it restores matching unread attention and retains only keys represented by the current live summons projection.

**Tech Stack:** Rust 1.90, Serde, Proptest, existing JSON persistence and Ratatui application model.

## Global Constraints

- Do not change Herdr-owned presence, topology, output, pane revision or protocol values.
- Do not add a database, timestamp heuristic, local generation counter or host API assumption.
- Existing development JSON containing `pane_revision` must remain readable; newly written JSON must omit it.
- An observed transition away from a Summons or to a different Summons must clear the old acknowledgement.
- Run no live Herdr mutation in this implementation task; live retest happens only after independent review.

---

### Task 1: Key durable acknowledgements by persona and Summons

**Files:**
- Modify: `src/persistence/state.rs`
- Modify: `tests/persisted_state.rs`
- Modify: `tests/persistence_worker.rs` if its fixtures construct `AttentionEpisodeKey`
- Modify: `tests/support/strategies.rs`
- Modify: any compiler-identified test fixture that constructs `AttentionEpisodeKey`

**Interfaces:**
- Consumes: `PersonaKey`, `GuildSummons`, `GuildAttention`, `DurableIntent::overlay`, `PersistedStateV1::capture`.
- Produces: `AttentionEpisodeKey { persona, summons }` and backwards-readable development JSON.

- [ ] **Step 1: Add failing persistence tests**

Add exact tests that:

```rust
#[test]
fn overlay_restores_seen_summons_when_snapshot_revision_changes() {
    let state = captured_state();
    let mut domain = support::fixture_domain();
    domain.agents.values_mut().next().unwrap().pane_revision = 0;
    let mut model = Model::new(View::Guild);
    model.durable_intent_mut().seed(&state).unwrap();

    model.replace_domain(domain);

    assert!(matches!(model.selected_agent().unwrap().attention, GuildAttention::Read { .. }));
}

#[test]
fn new_state_json_omits_transport_revision_from_seen_attention() {
    let json = serde_json::to_value(captured_state()).unwrap();
    let episode = &json["seen_attention"][0];
    assert!(episode.get("pane_revision").is_none());
}
```

Also add coverage proving:

- an observed domain with no Summons removes the stored key;
- a different Summons does not inherit the old acknowledgement;
- JSON containing the legacy extra `pane_revision` field still deserializes.

- [ ] **Step 2: Run RED**

Run:

```bash
cargo test --test persisted_state overlay_restores_seen_summons_when_snapshot_revision_changes
cargo test --test persisted_state new_state_json_omits_transport_revision_from_seen_attention
```

Expected: the revision-drift assertion remains unread and the serialized key still contains `pane_revision`.

- [ ] **Step 3: Implement the minimal persistence correction**

Change the public key to:

```rust
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct AttentionEpisodeKey {
    pub persona: PersonaKey,
    pub summons: GuildSummons,
}
```

Update `attention_episode` and all fixtures to construct only these fields. Do
not change `GuildAttention`, live agent revision, reducer transition behavior or
schema version.

- [ ] **Step 4: Run focused GREEN and compatibility checks**

Run:

```bash
cargo test --test persisted_state --test startup --test persistence_worker
PROPTEST_CASES=4096 cargo test --test property_domain --test persisted_state
```

Expected: all focused persistence, compatibility and property tests pass.

- [ ] **Step 5: Run the complete quality gate**

Run:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
bash tests/scripts.sh
ruby tests/workflow_contract.rb .github/workflows/release.yml .github/workflows/ci.yml
cargo build --release
git diff --check
```

Expected: all commands pass and `target/release/questmancer` is rebuilt for the later live retest.

- [ ] **Step 6: Commit**

```bash
git add src/persistence/state.rs tests
git commit -m "fix: persist acknowledged Summons by stable identity"
```

Write implementation evidence to
`.superpowers/sdd/questmancer-task-9-attention-report.md` and leave tracked files
clean.
