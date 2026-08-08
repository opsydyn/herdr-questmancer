//! Your agents have entered the dungeon. You are the Questmancer.
//!
//! Questmancer is a [Herdr](https://herdr.dev) plugin that turns a coding-agent
//! session into a living 16-bit adventurers' guild. Herdr workspaces become
//! campaigns and agents become adventurers: working agents travel the Delve,
//! blocked agents call for counsel, and completed work returns with spoils.
//!
//! It answers one question faster than a list can — **which agent needs me?**
//!
//! # Installing
//!
//! Questmancer is a Herdr plugin rather than a standalone program, and Herdr
//! installs plugins itself:
//!
//! ```bash
//! herdr plugin install opsydyn/herdr-questmancer
//! herdr plugin action invoke opsydyn.questmancer.open
//! ```
//!
//! Herdr fetches the repository, reads `herdr-plugin.toml` and runs the
//! plugin's build step, which downloads the prebuilt binary for your platform
//! and checks it against the release checksums. No clone, no Rust toolchain.
//!
//! To work on Questmancer instead, link a checkout — Herdr links a plugin by
//! *directory*, needing the manifest and the `herdr/` scripts, not just an
//! executable:
//!
//! ```bash
//! git clone https://github.com/opsydyn/herdr-questmancer
//! cd herdr-questmancer
//! cargo build --release
//! herdr plugin link .
//! ```
//!
//! `cargo install questmancer` builds the same binary onto your `PATH`, which
//! is useful for running it directly. It is not an install of the plugin:
//! `cargo install` places binaries and nothing else, so the manifest Herdr
//! needs is absent, and the plugin's launcher resolves its binary from inside
//! the plugin directory rather than from `PATH`.
//!
//! Requires Herdr `0.8.0`, which speaks protocol `19`. Herdr refuses a client
//! whose protocol differs, so the plugin and the server move together.
//!
//! # The two rooms
//!
//! One RGB pixel world, rendered into a terminal pane as half-block cells.
//!
//! - **Guild Hall** — the warm operational home for the whole party. Stations
//!   for campaign tables, the counsel bell, the hearth and the spoils bench.
//! - **Delve** — the same live state seen as a dungeon, with delvers at the
//!   stations their work has taken them to.
//!
//! Both rooms share one selection. Parchment overlays handle counsel, search,
//! scrying and the Chronicle without replacing the scene with a dashboard, and
//! the room recomposes rather than clipping when the pane gets small: a party
//! too large for world scale is redrawn at roster scale rather than losing
//! adventurers off-camera.
//!
//! # Finding what needs you
//!
//! `!` jumps to the next adventurer waiting on a human, ranked by what the
//! party actually needs — an unanswered call for counsel first, then one
//! somebody has seen, then the quieter summons, and within a rank whoever has
//! waited longest. `s` sets a summons aside for a while when the answer is
//! "not now". `?` opens the Librarian's Ledger, whose keyring is generated
//! from the real binding table and cannot drift from it.
//!
//! With `sidebar_urgency_order = true`, Herdr's own agent list leads with the
//! same ranking, so the answer reaches you without this pane being open.
//!
//! # What Questmancer will not do
//!
//! Herdr and the coding agent remain authoritative. Questmancer projects their
//! state into a scene; it never invents one. An adventurer's identity,
//! location, pose and attention all derive from what Herdr reports, the only
//! text ever sent to an agent is counsel you composed with `r`, and
//! Questmancer never selects, focuses, reads or counsels its own pane.
//!
//! # A note on this library
//!
//! The published artefact is the `questmancer` binary. This library exists so
//! integration tests — which are separate crates and therefore need `pub` —
//! can reach the renderer, the domain model and the Herdr client.
//!
//! **It is not a supported API.** Items are public for testability, not for
//! reuse, and they change without ceremony. Depend on the binary's behaviour,
//! which the Herdr plugin contract covers, rather than on these types. If you
//! want part of this surface stabilised for another Herdr plugin, open an
//! issue and say which part and why — that is a decision worth making
//! deliberately rather than by accident of visibility.

#![forbid(unsafe_code)]

pub mod app;
pub mod cli;
pub mod command;
pub mod config;
pub mod domain;
pub mod herdr;
pub mod interaction;
pub mod ledger;
pub mod persistence;
pub mod portrait;
pub mod rank;
pub mod runtime;
pub mod runtime_loop;
pub mod scene;
pub mod sidebar;
#[cfg(feature = "storybook")]
pub mod storybook;
pub mod terminal;
pub mod ui;
pub mod update;
