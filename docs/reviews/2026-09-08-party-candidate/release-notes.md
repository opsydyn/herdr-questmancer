
### Fixed

- Exclude repository review packs from the crate archive. The expanded art had
  exceeded the existing 10 MiB package gate; production assets remain included.

- Herdr 0.9 / protocol 22 compatibility: subscribe before the authoritative
  snapshot and reconcile unversioned status events against current pane
  metadata, so queued old events cannot invent a completion transition.

- Scrying now keeps only the current selected adventurer's latest output.
  Superseded reads and failures cannot overwrite newer results; repeated
  refreshes coalesce while counsel continues independently.
- Roster adventurers retain recognisable state cues without names, colour or
  motion. Fresh spoils settle within three seconds, and reduced/still scenes
  do not wake for decorative animation or cleanup.
- Storybook inspection now reaches the production roster, vignette and
  status-only layouts at small sizes. Sprite galleries retain their minimum
  canvas instead of being compressed into unreadable art.
- Release automation now explicitly requests the archive workflow after
  tagging. Tags made with `GITHUB_TOKEN` previously appeared without a matching
  downloadable release; dispatch failures now fail the tagging job. Every
  release job, including the optional registry publish, checks out that tag.

### Added

- All fourteen classes now have authored work, counsel and spoils rituals with
  shared bounded timing and persona colours. Every class card centres the same
  personalised world sprite at native size. Production rituals and card visuals
  are approved; native illustrations and roster families remain in use.
  Storybook now contains 45 stories, including fourteen class pose galleries.
- Chronicle chapters: Tab inside the Chronicle requests a local last-hour guild
  recap from retained events, with a fixed UTC window and timestamped sources.
  Counts distinguish repeated events from distinct adventurers and never infer
  missing history or successful delivery from campaign closure.
- The Guild Hall cat gives one 800 ms reaction when the same known non-empty
  party becomes entirely resting. Baselines, reconnects, unknown states and
  party changes remain quiet, with no added wakes in reduced/still motion.
- Cards show the saved keepsake with a fixed description and static authored
  8x8 illustration. Compact cards retain its text within the existing footprint.

- Campaign crests derived from workspace identity, repeated below the Guild
  Hall's campaign-table actors and named on the adventurer card. Shared tables
  show separate crests only when the whole set fits; smaller Hall tiers keep
  their existing composition. No new persistence or animation wake is needed.

- A stockier Librarian world sprite and a separately authored Ledger fallback.
- Wizard, Ranger and Barbarian rituals: two working frames at 500 ms each,
  a single 600 ms counsel gesture, and returned spoils that are placed and
  settled. Their production pose galleries bring Storybook to 34 stories.
- Correlated, confirmed counsel seals the existing notice. Rejected or
  uncertain delivery remains unsealed, and confirmation does not change
  Herdr presence.

- The Guild Hall's quest board carries a quest. It is the room's focal point —
  the largest bright shape, centred, where the eye lands first — and it held
  three faint marks on an otherwise blank sheet. It now has three pinned
  notices with ink on them, brass pins, red thread running pin to pin, a wax
  seal, and a dagger driven through the board. Whatever the eye travels to
  first should reward the trip.

- A cat, asleep between the bookshelves. Nothing in the Hall was alive except
  the adventurers, and a room whose only living things are your open tasks is
  not much of a guild. It was meant to sleep by the fire, but every square of
  floor in front of the hearth is a standing slot and an adventurer stood on
  it.

- Nine dungeon tropes, now that there is a floor for scenery to stand on:
  cobwebs, stalactites, dripping water, hanging chains, a sarcophagus, a broken
  statue, glowing mushrooms, a wall lever and a rat. Deliberately not added:
  a portcullis, an arch, columns, puddles and glowing crystals, all of which
  the dungeon already had.

  Three of these exist to give the rooms height. Before them every prop either
  stood on the ground or hung flat against a wall, so the dungeon read as one
  storey tall with a ceiling nobody had drawn. The stalactites hang *into* the
  chambers rather than sitting in the ceiling band — stone drawn on stone is
  invisible, and the point of a spike is its silhouette against a room.

  Two are alive. The mushrooms glow at the camp approach and the rat is the
  only thing down there that is neither party nor furniture.

  The statue is headless on purpose: a whole statue is decoration, a broken one
  says something happened here before the party arrived.

### Changed

- Questmancer now targets Herdr `0.9.0` and protocol `22`. Older or unknown
  protocol versions are rejected before subscribing or accepting snapshots.
- Wizard, Ranger and Barbarian card fallbacks use their new personalised world
  sprites at native size, centred in the existing card canvas without scaling.

- Debug builds no longer carry full debug info. `target/` had reached 36 GB on
  disk — 24 GB of dependency debug info and 10 GB of incremental state that
  cargo grows across rebuilds and never prunes. `[profile.dev]` and
  `[profile.test]` now use `debug = "line-tables-only"`, which keeps file and
  line in panics and test failures — the only debug information this project's
  workflow actually reads, since every golden-hash and guard failure is
  diagnosed from a panic location — and drops the variable inspection nothing
  here uses. A full build of the library and every test target now costs 1.7 GB
  rather than tens of gigabytes.

  `just disk` reports what the build directory is holding, and `just
  clean-build` reclaims it. Neither touches `target/release`, so the linked
  Herdr plugin keeps running: `herdr/run.sh` prefers that binary, and its
  fallback in `bin/` had silently gone five weeks stale, so a plain `cargo
  clean` would have quietly downgraded a running Questmancer to an old build.

### Fixed

- Action confirmations never went away. `clear_action_feedback` had exactly one
  caller in the entire crate — the branch of search where a query matches a
  single adventurer — so every other message was permanent. Press `o` on an
  agent whose output exceeds `output_preview_lines` and "output preview was
  truncated" pinned itself to the bottom of the room for the rest of the
  session: still there after the preview closed, after switching between the
  Guild Hall and the Delve, and after selecting a different adventurer, by then
  describing something nobody could see.

  This was never specific to that one message. Around ten call sites set action
  feedback — "Set aside for 15 minutes.", "Draft kept.", the search position —
  and all of them behaved the same way.

  Feedback now expires six seconds after it is shown, which is longer than the
  command ribbon's three: the ribbon is a reminder you can bring back by
  moving, while this may be the only report that an action succeeded, so it has
  to survive being read. Expiry is on the clock rather than on a list of
  transitions to clear, because a list of transitions is exactly what existed —
  one entry long, and wrong for every message nobody remembered to add to it.
  Standing conditions are untouched; connection and persistence diagnostics do
  not expire.

- The Guild Hall's fire was a thumbnail. A guild hall's hearth is its emotional
  anchor, and this one was four rows of flame sitting in a recess two and a
  half times its height, easy to miss entirely at the far right of the room.
  The fire now fills the grate, over an ember bed.

- The Hall's ceiling was thirteen identical brackets in a row, which reads as
  wallpaper rather than as carpentry — once the eye resolves a perfect repeat
  it stops seeing the thing repeating. Every third beam is now a worn variant:
  same silhouette, knot and peg in different places.

- The viewport matrix's guild hall arm still carried ten hand-copied colour
  literals, the same trap that had just gone stale on the delve side. Both arms
  now derive from the renderer's own constants, and each proves it can say no —
  the delve check rejects a buffer of hall oak, the hall check rejects dungeon
  floor.

- The Delve's labyrinth was invisible. The dungeon defines seven named regions
  — entrance, west and east chambers, central junction, descending corridor,
  camp, exit landing — joined by seven doorways, all of it walkability-masked
  and tested for reachability. None of it could be seen. Every boundary in that
  layout is drawn purely as a change of surface, so the architecture is visible
  exactly as far as floor and wall colours differ, and no further. They sat 21
  and 17 apart on the same redmean scale where this codebase already demands 40
  between an adventurer and the ground beneath them.

  The consequence was not merely that the scene looked flat. With no
  perceptible floor plane, every prop in the dungeon — the bones, the chest,
  the campfire, the altar slab — read as stuck to a wall, because there was no
  ground for them to rest on. That is the whole difference between the Delve
  and the Guild Hall, which has always had a floor you can see.

  Walls are now cooler and darker, floors warmer and lighter: 21 becomes 61 and
  17 becomes 55. The palette was solved rather than eyeballed, against every
  constraint already in force — all 43 actor cloth masses stay clear of every
  dungeon surface, the Unknown delver's mist still sits above the floor in
  value, and the floor's own speckle is *quieter* than before, honouring the
  earlier decision to calm that texture. No new art; the labyrinth simply
  appears.

  A guard now asserts every floor tone clears every wall tone, so this cannot
  silently collapse again. It was verified by restoring the old palette and
  watching it fail.

- Six classes wore their eyes as a single dark bar. At 16x24 an adventurer gets
  about three pixels of face, and Mage, Rogue, Runewright, Testmender,
  Pathseeker and Sorcerer each spent two of them on adjacent eye pixels — which
  do not read as two eyes, they read as one horizontal slot, the same mark a
  visor or a blindfold makes. The Ranger already had the answer: separate them,
  `K h K h K`, so skin shows between the eyes and beneath each one. The six now
  do the same.

  The Mage was the extreme case and needed more than spacing. It had no skin at
  all — a hood, a glowing accent band where a face belongs, and nothing else, so
  at magnification it read as an empty cowl rather than a person. It now has a
  forehead, two eyes and a chin, and the focal accent moved to the staff orb it
  is already carrying.

  Deliberately unchanged: the Druid's single centred eye. Its hood leaves a
  three-pixel opening, and two eyes in that space read as a stare rather than a
  face. One eye in a deep hood is the intended silhouette.

- The Druid was left out of the sprite sweep below, and that entry's claim to
  have covered "every class master" was wrong. Fourteen masters live in
  `src/scene/assets/archetypes.rs`; the Druid's lives in
  `src/scene/assets/adventurer.rs`, because `archetypes::world_frame` returns
  `None` for that class. The audit and fix both globbed `archetypes.rs` alone,
  so they measured fourteen sprites, reported fourteen clean, and never looked
  at the fifteenth — which was still standing a pixel above its own contact
  shadow with its staff floating a column clear of any hand. Both are fixed,
  and the count is now fifteen. A sweep that derives its own scope from one
  file will report success for exactly that file.

- Every remaining class world sprite carried two of the Ranger's five defects,
  and both were structural rather than stylistic. Held props — daggers, staves,
  hammers, lutes, shields — floated one to three columns clear of the figure
  holding them, so at magnification they read as scenery that happened to be
  nearby rather than as something a character was carrying; a skin-toned hand
  now bridges each prop to its body. Feet were drawn as outline only and
  stopped short of the contact-shadow lane, so every adventurer hovered above
  the shadow the renderer painted for them; boots are now filled and reach the
  lane. Measured across all fourteen masters: 175 detached pixels and 9
  floating props before, none after. The remaining Ranger fixes — shoulders,
  neck, a face that reads — are per-class drawing work and are not in this
  change; several classes are still uniform-width silhouettes.

- The Ranger world sprite. Magnified, it showed five defects that terminal
  scale had hidden: the figure floated two pixels above its own contact
  shadow, the bow hung a full column clear of any hand, the body was a
  uniform-width column with no shoulders and no neck, the face read as a dark
  slot rather than a face, and the feet were outline-only so they merged into
  the shadow. The bow is gone — a vertical stick beside a hooded figure reads
  as the staff that Mage and Wizard carry, so the class's own prop was working
  against it. A shoulder quiver with bright fletching carries the identity
  instead, and the fletching takes the focal accent where the eye already goes.

- The release pipeline syncs `herdr-plugin.toml` itself. It had been a
  documented manual step, and it was missed twice — fixed by hand once, then
  broken again by the very next version bump — leaving `herdr plugin install`
  downloading an archive no release published. A manual step inside an
  automated pipeline is a step that eventually does not happen.

- The documented install was the contributor path. `herdr plugin install
  opsydyn/herdr-questmancer` is how a Herdr user installs a plugin — one
  command, no clone, no Rust toolchain, with Herdr running the plugin's own
  build step to fetch a checksummed binary — and neither the README nor the
  crate documentation mentioned it. Both now lead with it. The `cargo install`
  instructions were also wrong: they placed a binary the plugin launcher never
  looks for, and linked a directory with no manifest in it.
