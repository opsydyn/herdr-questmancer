# Changelog

All notable changes to this project will be documented here.

## [Unreleased]

### Added

- A character-sheet vocabulary for Herdr's custom sidebar rows. Questmancer now
  publishes a class sigil, an epithet, a condition, a keepsake trinket, an
  exhaustion-style vigil for summons nobody has answered, and a bag-of-holding
  spoils count, plus a party sigil line per campaign. Every value is a fact
  Questmancer already holds, and all of it degrades to ASCII. The sidebar stays
  Herdr UI: `docs/design/questmancer-sidebar-character-sheet.md` carries
  paste-ready `[ui.sidebar]` configurations.

- Two new adventurer classes, Mage and Sorcerer, with art at all three scales.
  The Mage reads by a skull staff burning green and a hood deep enough to hide
  its face; the Sorcerer by a halo standing clear above its head. Neither
  leans on the Wizard's pointed hat. `PersonaGeneration::V3` carries the
  fourteen-class roll under its own hash label, so saved personas keep the
  class they were already assigned.

- Persona garb is now visible. It was persisted, palette-mapped and guarded by
  the contrast test, then never drawn. It reads in each master's trim band, so
  two adventurers alike in every other respect are distinguishable while the
  class's own body mass stays untouched.

- Poses are visible on every class. Eleven of the twelve rendered identically
  across Working, Seeking Counsel, Returning with Spoils, Resting, Settled and
  Unknown, so the sprite never showed what an agent was doing. Returning
  adventurers now carry a chest and resting ones take a wider seated stance,
  drawn over each class's own master in that class's palette. The Barbarian
  keeps its fully authored per-pose frames as the reference.

- Authored world and portrait masters for Artificer, Runewright, Testmender
  and Pathseeker. Each borrowed another class's body: Artificer and Runewright
  wore the Wizard, Testmender the Cleric, Pathseeker the Ranger, so six of the
  twelve classes were visually indistinguishable from a sibling. Every class
  now owns its silhouette, built around its own gear, and a test proves no two
  classes share a world or portrait master.

- The Delve adopted the Hall's actor-legibility rules. Delvers now ground with
  a contact shadow against the deepest stone value, blocked delvers carry the
  same authored counsel marker, and a pane too small to hold the party inside
  a camera crop recomposes the whole party at roster scale instead of leaving
  delvers off-camera. The roster tier itself is now shared by both worlds.

- Persona palette substitution in every class world master: an adventurer's
  skin, hair, and accent recolour the authored role clusters, so same-class
  agents are visually distinct in the Guild Hall and Delve. A new
  Persona Palette Family Storybook story reviews the spread on one shared
  master, and proof tests pin silhouette, determinism, and role-colour
  uniqueness.
- A flat quiet stage behind the compact and vignette Guild Hall party, so
  material seams never run through an actor's silhouette.
- A roster rung in the Guild Hall's responsive ladder. When a party no longer
  fits at world scale, a narrow pane now recomposes the *whole* party into
  authored 8x12 masters — one per silhouette family, personalised by the same
  palette substitution — instead of dropping to a single adventurer. Roster
  actors keep their own hit regions, grounding and selection treatment, and a
  Roster Silhouette Families Storybook story reviews the art.
- An authored counsel marker: blocked adventurers now carry an outlined bell
  above their head in the canonical, compact, roster and vignette Halls,
  replacing three loose pixels that read as noise at terminal scale.
- Native card portraits for Runewright and Pathseeker, the last two classes
  without one. Every production class now owns a distinct card, proven by a
  test that no two classes resolve to the same card bytes.

### Changed

- Identity nameplates now degrade name-first: truncation shortens the
  adventurer's name and keeps the presence badge whole, falling back to a
  state glyph and age in lanes too narrow for any name.
- `Garb::Leathers`, `Garb::WorkApron`, and the Barbarian's torso leather moved
  off oak-alike browns; a redmean colour-distance test now proves every
  garb and cloth mass keeps contrast with the Hall's `OAK` and `STONE` fills.
  The guard now covers the Delve's floor, stone and moss too, which caught two
  more collisions: `Garb::Cloak` dissolved into the dungeon floor and the
  Ranger's cloth into dungeon moss. Both moved.
- Selection is now a rune ring on the floor beneath the selected adventurer
  rather than four detached corner pixels, as the art direction asks. The
  ring scales with the master, so it reads at world and roster sizes alike,
  and the selection colour is reserved: a test proves no prop, adventurer or
  dungeon fixture paints it.

### Fixed

- The goblin outbreak now actually appears. Typing `release the goblins` into
  the search prompt already set the state, and `GoblinState::is_visible` had no
  callers, so the easter egg changed nothing a player could see and the release
  notes' "rare deterministic goblin sightings" was a claim about dead code.
  Goblins now raid two hiding places in the Guild Hall and two in the Delve for
  the three-second window, and the renderer asks for its own frames so the
  window visibly closes. The outbreak rides on the presentation rather than the
  scene snapshot on purpose: it must never alter the truth Questmancer reports
  back to Herdr, and a test holds that line. Adventurer placement is untouched
  while goblins are loose.
- Four spacing and legibility defects visible in real sessions. Nameplates no
  longer sit flush against each other (`codex · WORKING 2member-car…` read as
  one string, because the collision test treated touching rectangles as
  clear). Delvers no longer stand on top of one another: station slots were
  nine pixels apart for sixteen-pixel masters, and the guard that should have
  caught it measured an 8x14 box instead of the real footprint. A crowded
  party keeps a nameplate for every adventurer, degrading to the state glyph
  instead of dropping labels entirely. Compact actors stride by a master plus
  a gutter, so neighbouring silhouettes no longer touch.
- The selection marker no longer loses half its corner runes when the selected
  adventurer stands at the edge of the room; corners are clamped inside the
  viewport so the hearth and spoils stations mark selection like every other.
- Canonical Guild Hall actors were the only ones rendered without a contact
  shadow or state signal; they now ground and mark like the compact, roster
  and vignette compositions.

- Questmancer's Guild Hall and Delve projections over one typed Herdr session
  model, with responsive wide, compact, and tiny-terminal layouts.
- Campaign, adventurer, presence, attention, persona, Summons, Chronicle, and
  returned-spoils domain language with deterministic reduction.
- Selection, search, pane observation, counsel, local acknowledgement, lazy recent
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

- Acknowledged Summons now survive a close/reopen while the same stable
  adventurer still has the same call for counsel; persistence no longer
  compares Herdr pane/output revisions with synthesized status transitions.
- Chronicle counsel entries use guild voice (`requested counsel`) throughout
  the current interface.
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
