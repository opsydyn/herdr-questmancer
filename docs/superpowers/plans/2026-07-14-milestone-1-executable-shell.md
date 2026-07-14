# herdr-webmaster Milestone 1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Build a testable Rust executable that safely renders and switches between empty webmaster desk and cybercafe views and is declared as a Herdr plugin.

**Architecture:** A small binary parses an initial view, owns terminal lifecycle through an RAII guard, and delegates deterministic rendering to a library. Shell entrypoints resolve the local or installed binary and manage one plugin pane through Herdr's CLI.

**Tech Stack:** Rust 2024, Ratatui 0.30, Crossterm 0.29, Clap 4, Serde, Tokio, Bash, GitHub Actions.

## Global Constraints

- Minimum Herdr version is `0.7.3`; live acceptance must not run against the installed `0.7.0` binary.
- Plugin id is `opsydyn.webmaster`; pane id is `webmaster`.
- Platforms are `macos` and `linux`.
- No unsafe code, copied 90s product assets, image protocol, database, telemetry, or network service.
- Unicode and ASCII rendering are canonical and no state is conveyed by colour alone.
- Terminal state must be restored on normal exit and panic.

---

### Task 1: Package and view model

**Files:**
- Create: `Cargo.toml`
- Create: `rust-toolchain.toml`
- Create: `rustfmt.toml`
- Create: `clippy.toml`
- Create: `.gitignore`
- Create: `src/lib.rs`
- Create: `src/app.rs`
- Create: `tests/app.rs`

**Interfaces:**
- Produces: `app::View::{Desk,Cafe}`, `app::Model::new(View)`, `app::Model::switch_to(View)`.

- [x] **Step 1: Write failing model tests**

```rust
use herdr_webmaster::app::{Model, View};

#[test]
fn starts_in_requested_view() {
    assert_eq!(Model::new(View::Cafe).view(), View::Cafe);
}

#[test]
fn switches_views() {
    let mut model = Model::new(View::Desk);
    model.switch_to(View::Cafe);
    assert_eq!(model.view(), View::Cafe);
}
```

- [x] **Step 2: Run `cargo test --test app` and verify it fails because the crate does not exist**
- [x] **Step 3: Add the package metadata and minimal `View`/`Model` implementation**
- [x] **Step 4: Run `cargo test --test app` and verify both tests pass**
- [x] **Step 5: Commit with `git commit -m "feat: scaffold webmaster domain shell"`**

### Task 2: Deterministic empty views

**Files:**
- Create: `src/ui/mod.rs`
- Create: `src/ui/theme.rs`
- Create: `src/ui/views/mod.rs`
- Create: `src/ui/views/desk.rs`
- Create: `src/ui/views/cafe.rs`
- Create: `tests/rendering.rs`

**Interfaces:**
- Consumes: `app::Model::view()`.
- Produces: `ui::render(frame: &mut Frame<'_>, model: &Model)`.

- [x] **Step 1: Add `TestBackend` tests asserting the desk title, empty-state call to action, cafe title, and 80x24 safety**
- [x] **Step 2: Run `cargo test --test rendering` and verify unresolved `ui` imports fail**
- [x] **Step 3: Implement shared chrome plus separate empty desk and cafe renderers using saturating Ratatui layouts**
- [x] **Step 4: Run `cargo test --test rendering` and verify all rendering tests pass**
- [x] **Step 5: Commit with `git commit -m "feat: render empty desk and cybercafe"`**

### Task 3: Terminal lifecycle and input loop

**Files:**
- Create: `src/main.rs`
- Create: `src/terminal.rs`
- Create: `src/ui/input.rs`
- Create: `tests/input.rs`

**Interfaces:**
- Produces: `input::action_for(KeyEvent) -> Action`, `terminal::run(View) -> anyhow::Result<()>`.
- `Action` has `Switch(View)`, `ShowHelp`, `Dismiss`, `Quit`, and `None` variants.

- [x] **Step 1: Add input tests for `1`/F1, `2`/F2, `?`, Escape, and `q`**
- [x] **Step 2: Run `cargo test --test input` and verify unresolved input symbols fail**
- [x] **Step 3: Implement key mapping, Clap `ui --view desk|cafe`, event-driven redraw, resize handling, signal-aware shutdown, and RAII terminal restoration; adaptive animation rates remain milestone 5 work because milestone 1 has no animated state**
- [x] **Step 4: Run `cargo test` and verify all model, rendering, and input tests pass**
- [x] **Step 5: Run `cargo run -- ui --view desk` manually, switch views, quit, and verify the cursor and canonical mode are restored**
- [x] **Step 6: Commit with `git commit -m "feat: add safe interactive terminal loop"`**

### Task 4: Plugin manifest and lifecycle scripts

**Files:**
- Create: `herdr-plugin.toml`
- Create: `herdr/run.sh`
- Create: `herdr/control.sh`
- Create: `herdr/install.sh`
- Create: `tests/scripts.sh`

**Interfaces:**
- `run.sh ui [--view desk|cafe]` resolves installed, release, then debug binary.
- `control.sh open|close|toggle|desk|cafe` uses `HERDR_BIN_PATH` and atomic state locking.

- [x] **Step 1: Add dependency-free shell tests with a fake Herdr executable for binary resolution, open arguments, valid-runtime focus, close, and stale-runtime cleanup**
- [x] **Step 2: Run `bash tests/scripts.sh` and verify it fails because scripts are absent**
- [x] **Step 3: Add the verified `0.7.3` manifest, runner, checksum-aware installer, and singleton controller**
- [x] **Step 4: Run `bash tests/scripts.sh` and verify all lifecycle cases pass**
- [ ] **Step 5: Run `herdr plugin link .` only after a Herdr `0.7.3` upgrade; blocked locally because Herdr is `0.7.0` and its server is offline**
- [x] **Step 6: Commit with `git commit -m "feat: declare webmaster plugin lifecycle"`**

### Task 5: Quality gates and documentation

**Files:**
- Create: `.github/workflows/ci.yml`
- Create: `deny.toml`
- Create: `justfile`
- Create: `README.md`
- Create: `LICENSE`
- Create: `CHANGELOG.md`

**Interfaces:**
- Produces: `just verify` as the canonical local/CI check.

- [x] **Step 1: Add `just verify` running format check, Clippy with warnings denied, tests, and shell syntax checks**
- [x] **Step 2: Run `just verify` and record each missing or failing gate**
- [x] **Step 3: Add CI, dependency policy, README setup/keymap/architecture/manual-run sections, license, and changelog**
- [x] **Step 4: Run the direct commands behind `just verify` and fix every code or documentation-linked command failure (`just` is not installed locally)**
- [x] **Step 5: Run `cargo run -- ui --view cafe` as the milestone smoke test**
- [x] **Step 6: Commit with `git commit -m "chore: add milestone one quality gates"`**

## Milestone verification

- [x] `cargo fmt --check`
- [x] `cargo clippy --all-targets --all-features -- -D warnings`
- [x] `cargo test --all-targets --all-features`
- [x] `bash -n herdr/install.sh herdr/run.sh herdr/control.sh`
- [ ] `just verify` (local `just` executable is unavailable; every underlying command passed directly)
- [x] Manual desk/cafe switch and terminal restoration smoke test
- [x] Confirm no live compatibility claim is made against local Herdr `0.7.0`
