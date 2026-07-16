# Changelog

All notable changes to this project will be documented here.

## [Unreleased]

### Added

- Questmancer's Guild Hall and Delve projections over one typed Herdr session
  model, with responsive wide, compact, and tiny-terminal layouts.
- Campaign, adventurer, presence, attention, persona, Summons, Chronicle, and
  returned-spoils domain language with deterministic reduction.
- Selection, search, pane visit, counsel, local acknowledgement, lazy recent
  output, and optional Reviewr actions shared by both views.
- Original deterministic fantasy adventurers with ancestry, class, keepsake,
  chamber, and profile recognition anchors in Unicode and ASCII modes.
- Connected campaign dungeons, Guild Hall architecture, state-specific props,
  bounded semantic animation, and rare deterministic goblin sightings.
- Protocol-16 request/subscription clients with capped reconnect,
  resubscription, topology refresh, and last-visible-state preservation.
- Typed local configuration for initial view, motion, character set, colour
  mode, output bound, Chronicle bound, Reviewr action, and elapsed time.
- Atomically replaced versioned user intent and tolerant append-only Chronicle
  history with debounced writes, unchanged-state suppression, bounded
  diagnostics, and shutdown flush.
- Source-first Herdr `0.7.4` setup, migration, fake-agent, recovery, privacy,
  contributor, release, and cleanup documentation.
- Four-target Linux/macOS release packaging with root-level `questmancer`
  executables and release-wide SHA-256 checksums.

### Changed

- Product identity, binary, plugin actions, views, lifecycle recipes, and local
  persistence vocabulary now consistently use Questmancer.
- Contributor checks now cover all targets and features, lifecycle behavior,
  shell syntax, release compilation, and diff hygiene in CI.

### Fixed

- Completion effects stop at their exact semantic boundary; static,
  reduced-motion, and no-motion sessions do not create needless frame work.
- Monotonic scheduling prevents render latency and wall-clock adjustments from
  shifting animation phases.
- Managed-pane exclusion prevents Questmancer from selecting, reading, or
  sending commands to itself.
- Wide text and operational overlays remain intact when decorative goblins are
  composed into occupied architecture.
- Invalid durable state fails closed without overwriting recovery evidence;
  malformed Chronicle records do not hide valid surrounding history.
