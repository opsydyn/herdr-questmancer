# Questmancer Pane Sprite Viewport Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Show each recognized adventurer as a compact Kitty-graphics sprite, name, and truthful condition inside the first reserved row of its Herdr terminal pane.

**Architecture:** Herdr owns a one-row pane-viewport reservation and paints an image layer plus text in that row. Questmancer owns the projection of live adventurer facts and publishes the sprite/text payload through a dedicated Herdr graphics API. The managed Questmancer pane, empty panes, unsupported graphics clients, stale identities, resize events, and teardown all have explicit fallback or clear behavior.

**Tech Stack:** Rust, Tokio, Serde JSON socket API, Ratatui/Herdr pane renderer, Kitty graphics PNG/RGBA transport, Questmancer authored 8×8 or 12×12 PNG assets.

**Spec:** Approved design in the conversation on 2026-09-10; current contracts in `docs/manual-test/chronicle-capture.md`, `docs/troubleshooting/native-portrait-rendering.md`, and the repository `AGENTS.md`.

## Global Constraints

- Preserve Herdr ownership of pane layout, terminal content, focus, and pane identity.
- Preserve Questmancer’s managed-pane exclusion; it must never receive an adventurer banner.
- Use live Herdr facts only; do not persist topology, terminal output, or the rendered sprite.
- Keep the existing text-only fallback when Kitty graphics are unavailable.
- Do not inject escape sequences into agent terminal output.
- Keep `terminal.kitty_graphics = true` as the supported graphics setting; retain authored fallbacks.
- Maintain Herdr `0.9.0` / protocol `22` compatibility for the current release; this feature requires a coordinated Herdr protocol version bump or capability negotiation.
- Existing uncommitted documentation changes in this checkout must be preserved.

---

### Task 1: Herdr reserved pane-header contract

**Files:**
- Modify: Herdr source checkout, protocol schema, pane layout state, config model, and pane renderer (exact paths to be resolved in the Herdr repository).
- Test: Herdr protocol, layout, and renderer tests in the Herdr repository.
- Modify: `docs/troubleshooting/native-portrait-rendering.md` only if the Herdr capability name or fallback guidance changes.

**Interfaces:**
- Produces a negotiated pane capability named `pane_viewport_header` (or the final Herdr naming chosen during implementation) with `height_rows: 1` and Kitty graphics support.
- Produces a reserved first viewport row that is excluded from the embedded terminal PTY grid and is repainted on resize, focus changes, metadata changes, and client redraw.

- [ ] **Step 1: Write failing Herdr tests**

Add tests proving that enabling the feature reduces the terminal viewport by exactly one row, leaves the pane border geometry unchanged, and falls back to the current full viewport when the feature is disabled or unsupported.

- [ ] **Step 2: Run the focused Herdr tests**

Run the Herdr repository’s focused layout and renderer tests. Expected: failures showing the missing capability/config and missing reserved-row behavior.

- [ ] **Step 3: Implement the reservation**

Add an opt-in config value with a default that preserves current behavior. Route the reserved row through Herdr’s pane compositor rather than through PTY output. Ensure PTY resize reports continue to describe only the terminal content area.

- [ ] **Step 4: Run focused tests and resize checks**

Verify canonical, narrow, single-pane, split-pane, zoomed, hidden-sidebar, and unsupported-client cases. Expected: one reserved row only when enabled and no terminal-content overlap.

- [ ] **Step 5: Commit the Herdr contract**

Commit the Herdr-side reservation and tests independently so Questmancer can target a concrete protocol revision.

### Task 2: Herdr pane-header text and image API

**Files:**
- Modify: Herdr socket protocol, pane metadata state, Kitty graphics layer, and pane compositor (Herdr repository).
- Test: Herdr socket/API, graphics lifecycle, clipping, and teardown tests (Herdr repository).
- Modify: Questmancer `src/herdr/protocol.rs` and `src/herdr/client.rs` after the Herdr API is fixed.

**Interfaces:**
- Consumes: Task 1’s one-row reservation and capability negotiation.
- Produces: `pane.viewport_header.set` and `pane.viewport_header.clear` (final names subject to Herdr convention), accepting a short UTF-8 label, optional PNG/RGBA bytes, source identity, sequence, and optional TTL.

- [ ] **Step 1: Write failing API tests**

Cover accepted payloads, source/sequence ordering, TTL expiry, clear-on-close, stale identity rejection, resize repaint, image decode failure, unsupported graphics, and Unicode display-width truncation.

- [ ] **Step 2: Run focused API tests**

Expected: failures for the new method, state, and renderer integration.

- [ ] **Step 3: Implement header state and rendering**

Store the latest accepted header per pane and source. Paint the image at a fixed 8×8 or 12×12 placement in the reserved row, render the label beside it, clip by terminal display width, and clear atomically on replacement or teardown. Never write the payload into the PTY stream.

- [ ] **Step 4: Verify graphics fallback and lifecycle**

Run the focused tests plus a manual Ghostty check with `terminal.kitty_graphics = true`, then disable graphics and confirm the text-only fallback. Confirm old images disappear after replacement, close, release, resize, and reconnect.

- [ ] **Step 5: Commit the API**

Commit the Herdr API and renderer separately from Questmancer integration.

### Task 3: Questmancer compact pane projection

**Files:**
- Modify: `src/sidebar.rs` or a new focused module for pane-header projection.
- Modify: `src/command.rs` and `src/runtime_loop.rs` for publish/clear effects.
- Modify: `src/herdr/protocol.rs` and `src/herdr/client.rs` for the negotiated Herdr API.
- Add: `src/scene/assets/` class marker assets if existing authored assets cannot be reused at 8×8/12×12.
- Test: `tests/command.rs`, `tests/sidebar.rs`, and a new focused pane-header test module.

**Interfaces:**
- Consumes: Herdr Task 2 API and the existing `SidebarProjection` live-agent facts.
- Produces: `PaneHeaderProjection { pane_id, identity_digest, label, sprite, condition }` with deterministic text and a stable source/sequence.

- [ ] **Step 1: Write failing projection tests**

Assert the exact compact format `⌖ Ari · Pathseeker · working`, safe empty/unknown handling, deterministic class sprite selection, managed-pane exclusion, identity-change invalidation, and no projection from disconnected or ambiguous agents.

- [ ] **Step 2: Run focused Questmancer tests**

Run `cargo test --test pane_header` (or the final focused target). Expected: failures until the projection and publisher exist.

- [ ] **Step 3: Implement the pure projection**

Derive the label from the existing live agent/persona projection. Keep sprite bytes authored and immutable; do not copy the sprite or topology into persistence.

- [ ] **Step 4: Implement publish/clear effects**

Publish only when the derived payload changes. Clear on agent disappearance, identity replacement, managed-pane exclusion, Herdr disconnect, and shutdown. Coalesce updates and never publish from an animation wake.

- [ ] **Step 5: Run focused tests**

Verify request payloads contain only the dedicated header fields, preserve source sequencing, and never write `title`, `display_agent`, or terminal text.

### Task 4: Integration, documentation, and visual acceptance

**Files:**
- Modify: `docs/manual-test/questmancer-scene-preview.md`.
- Modify: `docs/troubleshooting/native-portrait-rendering.md`.
- Modify: `README.md` and the sidebar/metadata design documentation if user configuration is required.
- Add: dated review receipt under `docs/reviews/`.

**Interfaces:**
- Consumes: Herdr capability revision and Questmancer publisher from Tasks 1–3.
- Produces: a reproducible guarded acceptance procedure and evidence receipt.

- [ ] **Step 1: Write failing contract/documentation checks**

Require the docs to state the reserved-row behavior, Kitty prerequisite, text fallback, managed-pane exclusion, clear lifecycle, and that this is terminal viewport content rather than pane-border chrome.

- [ ] **Step 2: Run documentation and existing focused checks**

Run the relevant Rust/documentation contract tests. Expected: failure until examples and capability names match the implementation.

- [ ] **Step 3: Add the guarded manual procedure**

Use a fresh owned Herdr server, one disposable recognized adventurer per state, unique source IDs, no real user panes, and temporary storage. Inspect the actual pane viewport in Ghostty, resize it, disable graphics, replace the identity, close/reopen the plugin, and verify cleanup.

- [ ] **Step 4: Record visual acceptance**

Capture the compact sprite line at canonical and narrow pane sizes. Record whether native Kitty rendering, text fallback, resize repaint, stale-image clearing, and managed-pane exclusion passed. Keep native visual evidence separate from automated renderer tests.

- [ ] **Step 5: Run the complete gate**

Run `just verify`, `cargo build --release`, `git diff --check`, and the Herdr repository’s complete verification. Do not claim cross-repository release readiness until both gates pass.

- [ ] **Step 6: Commit documentation and receipt**

Commit the Questmancer integration and evidence separately from the Herdr feature commit, then recommend release qualification only after the user approves that next gate.

## Self-review checklist

- The plan keeps Herdr layout/rendering and Questmancer projection independently testable.
- No task writes into PTY output or changes pane identity/title ownership.
- The one-row reservation prevents sprite overlap with shell output.
- Capability fallback, resize, stale identity, TTL, teardown, and managed-pane cases are covered.
- No production code has been changed by writing this plan.

## Repair contract — 2026-09-11

The approved repair replaces the incomplete graphics-label extension. The final
optional API is `pane.viewport_header.info` plus one atomic
`pane.viewport_header.sync` containing a producer's complete desired set.
An empty set clears it. This combines ordered set/clear in one request, avoiding
cross-request replacement races. Every entry carries the exact live terminal
and agent-session tuple. The server validates source ordering and identities,
uses a 30-second lease, and owns both text and the reserved image layer.

Questmancer reuses its authored personalised 8×12 roster sprites without scaling,
not the invalid placeholder image. A single worker coalesces changes and renews
active leases every ten seconds; identical renewal does not dirty Herdr's scene.
Missing/disabled capability is inert. Header data never uses terminal input,
ordinary graphics placement, persisted state, pane titles or sidebar rows.

The earlier small commits and check claims did not qualify this feature. Repair
qualification must cover server composition, client-surface transport, retained
redraw, socket lifecycle and both complete repository gates before any new
completion claim. Native visual acceptance remains a separate recorded result.

Repair evidence is retained in
[the 2026-09-11 receipt](../../reviews/2026-09-11-pane-header-repair/README.md).
Questmancer's full gate and both native release builds pass. Herdr's native tests,
lint, documentation contracts and render profiles pass, but its complete check
stops at unavailable Windows C SDK headers. Two owned isolated runtime runs pass
with complete resource cleanup. Native Ghostty visual acceptance remains blocked;
no shared-server rollout or publication has occurred. Repair changes remain
uncommitted for review. Earlier unchecked plan tasks are not implied complete.
