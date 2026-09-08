# Questmancer: a party worth knowing

Date: 2026-09-05
Status: steps 1 and 2b visually approved on 2026-09-05; steps 2a and 2b
implemented and fully verified. Step 2c is complete. The Librarian refresh
and three-class production pilot are implemented and fully verified; their
production visuals were approved on 2026-09-08. Keepsakes, the cat reaction,
Chronicle chapters, and rituals with matching card fallbacks for all fourteen
classes are implemented and visually approved. See the
[complete candidate review](../reviews/2026-09-08-party-candidate/README.md).
Manual terminal/live and published release acceptance remain separate.

## Product outcome

Make the Questmancer recognise individual adventurers, feel useful when they
need counsel, enjoy their return, and remember the guild's shared history.

The central experience is:

**Recognise an adventurer → offer counsel → observe work resume → welcome
returned spoils → remember the event.**

Every flourish must help someone read a real state, understand an action, or
recognise a persona or campaign. Long waits stay calm. Completion never implies
that tests passed, a change was merged, or the Questmancer approved the work.

## Assessment baseline

Source was inspected on `main` at `efcd87d`, seven commits ahead of the local
`origin/main` reference. Eleven existing modified files include counsel
delivery, interaction, runtime and overlay changes. Preserve those edits and
re-baseline before implementation; do not treat this plan as permission to
commit or discard them.

The current source contains:

- Questmancer `0.1.8`, minimum Herdr `0.8.0`, supported protocol `19`.
- Fourteen classes, authored `16x24` world masters, independent `24x32`
  portrait fallbacks, and five `8x12` roster silhouette families.
- Thirty-two Storybook stories through production rendering paths.
- Working-frame animation for Barbarian; static working masters for the other
  thirteen classes. Most classes use shared pose decorations for rest/spoils.
- A guild standing score, Chronicle, fixed Librarian handbook, keepsakes,
  native class cards, cat and goblin presentation.

The 2026-09-04 assessment reproduced older output overwriting newer scrying
output, and ambiguous states at a `30x30` RGB viewport. Small Delve rendering
also omitted fresh-spoils theatre. These findings must become focused tests
when their fixes start.

The visual assessment also identified **long, narrow torsos and undersized
heads/faces** in several world masters. This conflicts with the approved
stocky character grammar. Correcting those proportions is an explicit part of
the first storyboard pass, before adding animation; more frames must not lock
in the elongated bodies.

The assessment's `just verify` pass was 514 Rust tests and 26 shell tests,
plus formatting, Clippy and script syntax. That is prior assessment evidence,
not acceptance of any work proposed below. Live Herdr, native portrait
transport and terminal visual acceptance remain separate checks.

At assessment, `AGENTS.md`, `PLAN.md` and parts of the art documentation
contained older compatibility, story-count and viewport descriptions. Step 2c
reconciled those with current source while preserving ownership and safety
rules.

## Sequence and decision points

| Step | Deliverable | Depends on | Exit condition |
| --- | --- | --- | --- |
| 1. Storyboard the party | Wizard, Ranger and Barbarian experience sheets | Current assessment | Complete: storyboard and proportion direction approved 2026-09-05 |
| 2. Make the loop dependable | Separate scrying and small-scene fixes; current operating guidance | State cues from step 1 for the visual fix | Focused failures reproduced, fixes verified, small scenes visually readable |
| 3. Build the three-class pilot | Production working rituals, counsel feedback and returned-spoils presentation | Steps 1 and 2 | Full loop accepted in both rooms and all supported presentation modes |
| 4. Give campaigns and personas character | Heraldry, then keepsake details, then one room reaction | Accepted pilot | Each small addition improves recognition or communicates a real transition |
| 5. Let the guild remember | Factual Chronicle chapters; separately designed durable mementos | Accepted pilot; history and persistence design for mementos | Every claim traces to an observed event; restart and truncation behaviour is explicit |
| 6. Expand and capture | Remaining classes in reviewed batches; release evidence | Accepted shared art grammar | Automated, terminal visual and live acceptance recorded separately |

The [step 1 review pack](../design/reviews/2026-09-05-party-storyboard/README.md)
is visually approved. Steps 2a and 2b are complete, including visual approval
of the production roster sheets. Operating-guidance reconciliation in step 2c
is also complete. The promoted parts of steps 4–6 are now implemented and
visually approved: heraldry, keepsakes, the cat reaction, Chronicle chapters
and all fourteen class rituals/cards. Durable mementos remain unpromoted.
Native/live and distribution acceptance remain separate; see the
[current roadmap assessment](../reviews/2026-09-08-roadmap-assessment.md).

## 1. Storyboard the party

Prepare a review sheet for each of **Wizard, Ranger and Barbarian**. They cover
robe/staff, lighter gear and a broad body, with Barbarian providing the existing
animation reference. Review that reference too rather than treating it as the
final quality ceiling.

| Moment | Wizard | Ranger | Barbarian | Meaning |
| --- | --- | --- | --- | --- |
| Working | Turns a stubborn page | Checks a map | Adjusts grip on a tool | Herdr reports working; the ritual does not infer the coding task |
| Seeking counsel | Closes the book, raises the shared lantern signal | Lowers the map, raises the signal | Sets the tool down, raises the signal | Herdr reports blocked; shared marker remains immediately recognisable |
| Work resumes | Takes up the book | Returns to the map | Takes up the tool | A later Herdr working event; delivery acknowledgement alone is insufficient |
| Returned spoils | Sets down a sealed folio | Brings a rolled map | Carefully places a tiny parcel | Herdr reports done; contents remain unverified |
| Resting | Book tucked away | Map stowed | Relaxed stance | Herdr reports idle, without implying success |

Include static unknown, settled completion and departure examples alongside
the five moments. Unknown must remain visibly uncertain; exited adventurers
must leave the live party. Treat the sealed parcel as visual metaphor: this
pilot introduces no persisted inventory or review-approval state.

Deliver:

- Native-size silhouette and colour studies, with optional enlarged inspection.
- Before/after proportion studies that explicitly shorten the long torsos,
  broaden the body and give the head/face more of the visible figure. Follow
  the approved guide: head roughly 40–50% of visible body height, torso about
  as wide as it is tall, little or no neck, and short two- or three-pixel feet.
  Redraw within the existing `16x24` frame and preserve the foot anchor; do not
  solve the problem by scaling the whole sprite down.
- World masters beside roster and portrait fallbacks, using the existing art
  direction's large heads, short legs, clear gear and stable foot anchors.
- A timeline for working frames and one-shot transitions, including interruption
  by a newer status, disconnection or departure.
- Static alternatives for reduced and still motion; the meaning must survive
  without animation or colour alone.
- One still showing the real card and nameplate space, so the art does not
  earn room by pushing useful text out of the pane.

Begin with two working frames per pilot class and short transitions within the
existing three-second completion window. Exact pixels and timing are review
decisions. Long walks through the rooms, new room geometry and a broad native
portrait repaint are outside this pilot.

Use the existing Storybook for any executable studies and production-path
comparison. Static sheets can precede code; do not introduce another renderer
or sprite editor. Update the sprite art direction with the accepted decisions
after visual review.

## 2. Make the loop dependable

### 2a. Scrying result ordering

Implementation record, 2026-09-05: `OutputRequest` travels through every output
command and result. The model owns one selection-bound context and accepts
only its current pending request, with a nondecreasing output revision.
Selection/agent/pane changes and connection boundaries invalidate that context.
Reconnect reloads unchanged selections. The runtime keeps one output read in
flight plus the latest pending request; connection changes cancel output work
without cancelling counsel tasks. This state remains transient.

The revision 20 → 10 regression failed with revision 10 before the fix and now
passes. A socket-backed refresh-burst regression also failed before coalescing
and now passes, including counsel completion while output is stalled.
Focused library/app/command/interaction/runtime suites pass. Overlay tests now
settle an actual request before checking text and scrolling; the static
Scrying Storybook fixture also settles its authored response without socket
I/O, with a regression proven red before that fixture correction.

`just verify` passed on `efcd87d` plus the preserved existing edits and this
slice: **531 Rust tests across 48 test runs, 26 shell tests, formatting,
Clippy with warnings denied and script syntax**. `git diff --check` also
passed. Live Herdr and manual terminal acceptance have not been run; no
release build, commit or publication is claimed by this gate.

The original slice contract follows:

Start with a failing regression for revision 20 followed by revision 10 on the
same selected pane. Add cases for an older failure after a newer success,
selection away and back, pane removal, and reconnect with old requests pending.

Use a typed output request identity carried through the command and result,
with current pane/selection and connection validity. Accept results only for
the current eligible request; apply revision ordering within the relevant pane
and connection lifetime. An older failure must not clear a newer preview.
Bound or coalesce superseded output requests without cancelling counsel work.

Primary files: `src/app.rs`, `src/command.rs`, `src/runtime_loop.rs`,
`src/interaction.rs`; tests in their matching integration suites. Preserve the
existing counsel correlation and retry semantics. Do not add output polling
on animation frames or persistent output storage.

### 2b. Readable state at roster scale

Implemented on 2026-09-05. The [three-sheet production review](../design/reviews/2026-09-05-roster-states/README.md)
shows both rooms, truecolour/ANSI 16, final terminal-cell layouts and the
three-second deadline. The Questmancer visually approved these production
compositions on 2026-09-05, following the earlier approval of their shape
direction. Live terminal acceptance remains separate.

Evidence from this slice:

- Shared five-pixel tool, lantern, Z, completion check and question mark on
  existing authored roster bodies. Settled completion remains distinct.
- Shared full-motion gutter effects end at the semantic deadline, including
  the final 1 ms wake. Reduced/still frames remain pixel-identical across the
  deadline. The real runtime scheduler disarms at settlement.
- Newer presence and connection loss cancel theatre. A transient presentation
  cutoff suppresses retained transitions after reconnect without clearing or
  persisting summons; a new done event can still play.
- Complete hit regions, two full rows, selection rings and connection facts
  retain separate space. Labels respect odd half-block bounds, cues and rings,
  and use ASCII symbols when selected. ANSI 16 state frames remain distinct.
- The focused regressions were observed failing before their fixes. Final
  `just verify` passed: 542 Rust tests across 49 suites and 26 shell tests,
  formatting, strict Clippy and script syntax. Review regeneration and image
  layout checks passed. Live and terminal acceptance are separate.

Original scope:

Reproduce Working/Idle/Unknown aliasing and missing Delve completion at
`30x30` RGB pixels before changing the paint path. Add shared shape cues for
working, counsel, resting, completed and unknown states that remain useful
when identity labels cannot fit. Keep settled completion distinguishable after
the one-shot effect ends.

Restore bounded fresh-spoils presentation in the Delve roster. Full motion
must terminate on its semantic deadline. Reduced/still modes retain a stable
state cue without a decorative redraw loop. Newer facts interrupt old theatre.
Preserve complete actor hit regions, selection priority and capacity handling.

Primary files: `src/scene/render/roster.rs`, `interaction.rs`, `guild_hall.rs`,
`delve.rs`, `src/scene/stage.rs`, and `src/ui/scene_overlays.rs`. Add tests to
the scene, runtime and overlay suites, including final Ratatui buffers where
labels, glyphs and colour conversion matter.

### 2c. Reconcile operating guidance

Approved for execution on 2026-09-05. Reconcile current source, document
historical claims as history, and record publication independently. Do not
change product behaviour or deploy a release in this slice.

Completed on 2026-09-05. Current compatibility, fourteen native class cards,
thirty-two Storybook stories, sidebar reporting, room tiers and Librarian
coverage are reconciled across the operating documents. The dated release
check records published `v0.1.3` assets and the missing matching `v0.1.8`
publication for current `main`; it does not claim installer acceptance.

Validation: 26 shell checks including documentation/workflow contracts,
eight focused Storybook/Librarian tests, local document links and
`git diff --check` passed. The Storybook count contract now matches the
existing 32-story catalogue. No production Rust changed in this slice;
the preceding full gate remains the 542-test result recorded above.

Update current compatibility, fourteen-class coverage, Storybook inventory,
native portrait scope, sidebar capability and both rooms' actual responsive
tiers in `AGENTS.md`, `PLAN.md`, `README.md` and relevant design/manual-test
documents. Keep historical records recognisable as history. Confirm release
publication separately before changing release-status claims or tag recipes.

The two fixes remain independently reviewable. Documentation reconciliation
must not expand into unrelated product changes.

### Librarian follow-on review

The Questmancer authorised the Librarian and then the pilot with “Yes librarian
then move on” on 2026-09-05. The original `16x24` sprite and padded Ledger
fallback are retained as a review baseline; production now has a broader head,
shorter purple robe, clearer spectacles and books, feet on row 21, and a
separately authored `24x32` Ledger fallback. The native illustration is unchanged.

The [two-sheet review](../design/reviews/2026-09-05-librarian/README.md)
uses production assets, room painters and Ratatui buffers. Visual approval is
pending. Focused asset, Hall, Ledger and Storybook checks passed after a red
regression for grounding and use of the larger portrait canvas. Before the
pilot changes, `just verify` passed with 543 Rust tests across 49 suites,
26 shell tests, formatting, strict Clippy and script syntax. The help-NPC
role and complete clickable station remain unchanged.

## 3. Build and review the three-class pilot

Implemented on 2026-09-05. The [production review pack](../design/reviews/2026-09-05-party-pilot/README.md)
contains five sheets and two playback GIFs, rendered by current production
assets, room painters and Ratatui overlays. The implementer inspected the
artifacts; Questmancer visual approval is pending. Manual terminal, live
Herdr and native transport acceptance remain separate.

The current implementation has:

- Eight authored moments per class in the approved `16x24` proportions and
  three dedicated `8x12` roster bodies, using existing persona colour roles.
- Two 500 ms working frames; one 600 ms counsel gesture followed by stillness;
  class-specific spoils placed after 1000 ms and quiet completion at 3000 ms.
- Shared per-asset timing and next-visible-pixel deadlines in both rooms.
  `ScenePlan.cadence` is removed; `SceneFrame.next_frame_in` stays authoritative.
- Reduced/still poses with no decorative or cleanup timer, and no replay of
  retained counsel/return facts across repeated reports or reconnection.
- A typed red seal only after correlated counsel text and submission succeed.
  The existing notice expires after six seconds. Presence and recovery
  semantics remain independent; confirmed counsel does not resume work.
- Thirty-four fixed Storybook stories, including three production pose
  galleries. Storybook time remains fixed; GIF playback samples production
  frames at explicit times.

Focused tests first failed for the new timing, quiet completion, counsel
sealing and story inventory. Additional rendered and runtime cases cover foot
anchors, parcel placement, the exact final deadline, repeated reports and
reconnect for all three classes. The final `just verify` passed on `efcd87d`
plus the preserved dirty tree and this slice: **550 Rust tests across 50 test
runs, 26 shell tests, formatting, strict Clippy and script syntax**.
Review exporters compile with warnings denied; 80 local documentation links,
review-script syntax, GIF timing and `git diff --check` also pass. No release build, live Herdr action, commit or publication is claimed.

The original implementation contract follows; expansion beyond this pilot
still depends on its visual acceptance.

Implement the accepted working rituals and bounded pose transitions using
authored frames. Preserve each class's gear and persona colour roles. Reuse
the approved shared roster state cues rather than drawing unreadable miniature
acting sequences.

Counsel feedback has its own meaning: seal the parchment only when the
correlated text-and-submit operation is confirmed. Definite rejection,
uncertain delivery and submission failure retain distinct presentations and
the existing recovery actions. Never animate an adventurer resuming because
the user pressed Enter; wait for Herdr's presence update. Do not imply the
agent read, understood or followed the counsel.

Returned-spoils art should be the strongest brief celebratory moment. End it
within the bounded window and retain a calm completed cue. Opening Reviewr is
an inspection action, not proof of review or acceptance. Reconnects, repeated
events and switching rooms must not restart a finished celebration.

Keep architectural changes local to this work:

- Put frame counts and timing beside authored animation definitions, avoiding
  a growing set of class-specific checks in both room renderers.
- Keep `SceneFrame.next_frame_in` authoritative for runtime rendering deadlines;
  reconcile the unused `ScenePlan.cadence` contract and affected tests.
- Share state markers and reserve actor, effect and label space where the new
  art requires it. Include Librarian/prop exclusion in collision review.
- Keep transient presentation outside durable intent. If a new transition
  record is needed, define its event identity, expiry and interruption rules
  explicitly; rendering itself never mutates it.

Primary files: `src/scene/assets/adventurer.rs`, `archetypes.rs`,
`rituals.rs` (replacing the old Barbarian-only route), scene projection/rendering
modules, relevant counsel overlay
code, and `src/storybook/`. Add focused failing behavioural tests before
implementation; asset validity and real rendered differences both matter.

### Card fallback follow-on — 2026-09-05

The Questmancer requested that the cards use the new sprites rather than the
old long-torso portraits. Wizard, Ranger and Barbarian now reuse their exact
personalised `16x24` working master, centred in the existing `24x32` card
canvas. This preserves its aspect ratio, current persona colours, card bounds
and adjacent text. The fallback is static; native illustrations, other class
fallbacks and the revised Librarian keep their existing routes.

The [two-sheet review](../design/reviews/2026-09-05-card-fallbacks/README.md)
shows before/after art and production card buffers in truecolour/Unicode and
ANSI-16/ASCII. A focused regression first failed on the old Wizard portrait;
it now checks exact sprite pixels, transparent margins and two persona
palettes for all three classes. The original pilot card sheet is retained as
historical evidence and superseded by this comparison.

`just verify` passed after this follow-on: **551 Rust tests across 50 test runs,
26 shell tests, formatting, strict Clippy and script syntax**. The 53 focused
asset, card-overlay, Librarian and Storybook tests also pass. The exporter
compiles with warnings denied; both review sheets were visually inspected by
the implementer. The Questmancer visually approved these card fallbacks on
2026-09-05 with “Approved”. Terminal/live acceptance remains separate.
No commit or release was made.

### Terminal resize follow-on — 2026-09-06

The Questmancer approved a real terminal resize pass. Computer Use refused
Ghostty access for safety reasons, so native visual acceptance remains blocked.
An executable PTY check passed all 70 resize cases across ten stories without
accessing Ghostty or Herdr. The owned process exited normally and restored its
terminal modes, visible cursor and primary screen.

The pass exposed a Storybook defect: minimum canvas guards hid production
scene tiers below `80x24`, and card stories below `80x28`. Scene story minima
now permit all positive sizes; sprite galleries retain their own minimum.
The regression failed at Guild Hall `64x20` before the correction and now
passes across all scene fixtures and four small sizes. `just verify` passed
with **552 Rust tests across 50 test runs, 26 shell tests, formatting, strict
Clippy and script syntax**. See the [resize receipt](../design/reviews/2026-09-06-terminal-resize/README.md)
for executable evidence and the remaining native review steps.

The Questmancer approved this reported correction and executable evidence on
2026-09-06 with “Approved”. Native visual acceptance remains open because no
native terminal observation accompanied the approval.

## 4. Campaign identity and small discoveries

Take these as individual follow-on slices, in this order:

1. **Campaign heraldry:** repeat a crest at the appropriate table/landmark and
   on the campaign's contextual parchment. Use a stable existing campaign key
   and redundant shape/colour. A crest must not promise persistent identity
   beyond that key's lifetime. Review multiple campaigns sharing limited bays.
2. **Keepsake details:** an authored illustration and short fixed description
   in the adventurer card. Draw from the existing keepsake assignment, preserve
   saved personas, and keep the current card footprint. Example: “Brass Key —
   Opens nothing anyone has admitted to.”
3. **One cat reaction:** choose one observed aggregate transition, such as the
   party becoming entirely resting, and play a short reaction once. An empty
   party is a separate case. Preserve actor reservations and still-motion
   behaviour; do not add background wakeups or a pet-management interface.

Keep the Librarian's Ledger as the fixed help handbook. Any later Librarian
flourish belongs to presentation and must preserve help access and the
non-agent boundary. Do not turn the Librarian into another conversational agent.

## 5. A guild with history

Start with **Chronicle chapters** built from recorded events and an explicit
time window. Present a short factual recap with names, timestamps and source
entries available underneath. Distinguish “three spoils-return events” from
“three adventurers returned”; repeated events and missing/truncated history
must not turn into invented claims.

Keep initial chapters on demand and local, using authored text templates.
“Since you last looked” needs an explicit reading checkpoint and is a later
choice, not an assumed capability. Campaign closure does not prove successful
delivery of a project, and the event path must actually exist before a chapter
or milestone uses it.

Persistent seals or a trophy shelf require a separate small persistence design
covering source event identity, deduplication, migration, retention and replay.
The Chronicle is bounded; permanent achievements cannot be recomputed from it
without loss. Preserve the existing guild-level standing decision: no
per-adventurer levels, feature unlocks, score for blocking or streak pressure.

Likely boundaries: `src/domain/chronicle.rs`, a pure chapter projection,
Chronicle overlays and fixtures; persistence only when the separately designed
memento slice is promoted.

## 6. Expand the art and record acceptance

After the pilot is approved, propose small class batches sharing useful art
techniques. A sensible next batch is Bard, Artificer and Testmender for
instrument/tool-handling, followed by the remaining classes. Reassess each
batch rather than cloning the pilot poses across every silhouette.

On 2026-09-08 the Questmancer approved the Chronicle chapter screens and the
next storyboard step. The [tool ritual review](../design/reviews/2026-09-08-tool-ritual-storyboard/README.md)
now proposes Bard lute-picking, Artificer device-adjustment and Testmender
needlework. It contains eight authored world moments per class, stable feet,
current/proposed silhouettes, both room placements and a 500 ms working-loop
study. The Questmancer subsequently approved the storyboard and production
implementation. All 24 world frames now use the existing scheduler and persona
substitution, with three new Storybook pose galleries. At that stage, independent
cards and roster families were retained. The [production review](../design/reviews/2026-09-08-tool-ritual-production/README.md)
shows real asset routes and both scene renderers; the Questmancer approved
those production visuals on 2026-09-08.

The implemented batch uses the current scheduling and semantic-state
boundaries; review its actual production frames in both rooms next. Working comes from Herdr; counsel
acknowledgement does not resume it; sealed spoils do not claim verified work.
Do not infer card/roster approval or native/live acceptance from this design pack.
The final full gate passed 593 Rust tests across 54 runs and 28 shell tests;
the release build passed. The subsequent card alignment slice was approved
and implemented: each new ritual class now reuses its personalised `16x24`
working master in the existing `24x32` fallback canvas, with no scaling or
animation. The [card review](../design/reviews/2026-09-08-tool-card-fallbacks/README.md)
was visually approved on 2026-09-08. Native illustrations and roster families
are retained. The next design-only batch was then authorised and prepared:
[Cleric, Paladin and Druid](../design/reviews/2026-09-08-cleric-paladin-druid-storyboard/README.md),
with distinct book, shield and living-staff rituals. Its twenty-four candidate
poses, both room placements and proposed working playback were visually
approved on 2026-09-08. The frames are now integrated through the existing
ritual and persona paths; Storybook has 40 stories. The
[production review](../design/reviews/2026-09-08-cleric-paladin-druid-production/README.md)
was visually approved on 2026-09-08. The subsequent card alignment slice is
implemented: these three classes now reuse their personalised working master
inside the existing `24x32` card canvas without scaling or animation. Their
[card review](../design/reviews/2026-09-08-cleric-paladin-druid-card-fallbacks/README.md)
was visually approved on 2026-09-08. Native illustrations and roster families
are retained. The next design-only
[Rogue, Pathseeker and Runewright storyboard](../design/reviews/2026-09-08-rogue-pathseeker-runewright-storyboard/README.md)
was visually approved on 2026-09-08. Its twenty-four frames are now integrated
through the existing ritual and persona paths; Storybook has 43 stories. The
[production review](../design/reviews/2026-09-08-rogue-pathseeker-runewright-production/README.md)
was visually approved on 2026-09-08. Their subsequent card alignment centres
the same personalised working master in the existing `24x32` canvas without
scaling or animation. The
[card review](../design/reviews/2026-09-08-rogue-pathseeker-runewright-card-fallbacks/README.md)
was visually approved on 2026-09-08. Native illustrations and roster families
are retained. The subsequently authorised design-only
[Mage and Sorcerer storyboard](../design/reviews/2026-09-08-mage-sorcerer-storyboard/README.md)
was visually approved on 2026-09-08. Its sixteen frames are now integrated
through the existing ritual and persona routes; Storybook has 45 stories.
The [production review](../design/reviews/2026-09-08-mage-sorcerer-production/README.md)
was visually approved on 2026-09-08. Their subsequent card alignment now
centres the same personalised world sprites in the existing `24x32` canvas.
The [final card review](../design/reviews/2026-09-08-mage-sorcerer-card-fallbacks/README.md)
was visually approved on 2026-09-08. All fourteen classes now have rituals and
matching card fallbacks. Complete candidate preparation was then authorised;
see the [candidate review](../reviews/2026-09-08-party-candidate/README.md).
Publication remains a separate gate. Later dated approvals supersede the
historical pending statements retained below.

Keep ancestry silhouettes and the unused appearance attributes in the backlog
until their recognition benefit is demonstrated. Retain current persona
generation and saved identity throughout the pilot.

Release work follows acceptance: build the actual release binary, record both
rooms, run guarded Herdr checks with newly discovered IDs, and verify the
intended release/distribution separately. Commit, tag, publish and shared
Herdr-server changes retain their existing explicit approval boundaries.

## Acceptance matrix

Use production Storybook stories and the real terminal for visual review.
RGB exports cover scene pixels. Ratatui buffer exports additionally exercise
labels and colour conversion; neither proves actual terminal presentation,
native image transport or interaction.

| Dimension | Required examples |
| --- | --- |
| Classes | All three pilot classes; a repeated class with different personas; mixed party |
| Presence | Working, blocked, idle, unknown, fresh done, settled done, exited |
| Counsel | Draft, sending, confirmed, rejected, uncertain, submit-only recovery; closed parchment with a late result |
| Worlds | Guild Hall and Delve, with selected and unselected adventurers |
| RGB viewport → terminal cells | `160x90 → 160x45`, `100x60 → 100x30`, `64x40 → 64x20`, `30x30 → 30x15`; one vignette and one status-only size |
| Capacity | One actor, mixed party, full capacity, overflow, multiple blocked adventurers |
| Presentation | Full/reduced/still motion; truecolour/ANSI-16; Unicode/ASCII; dark, neutral and warm backgrounds |
| Lifecycle | Repeated event, newer state interrupts, room switch, reconnect, departure and restart |
| Cards | Native portrait where supported, authored fallback always, readable text and visible actions |

The Questmancer should be able to identify the selected adventurer and who
needs counsel in a brief glance, and distinguish working, resting, completed
and unknown without relying on animation or colour alone. Review class
silhouettes before revealing their labels. Record confusion as a design issue;
pixel inequality alone is not a recognition test.

**Proportion gate:** the three-class static direction was approved on
2026-09-05, as recorded in the review pack. Preserve it when implementing the
pilot; later changes and production presentation still need visual review.
Compare current and proposed masters at 1x, both alone and in each room. The
pilot figures must read as stocky, large-headed adventurers rather than long
rectangular bodies with small heads. Check the head/torso/leg allocation and
preserve class gear recognition; an attractive enlarged image alone does not
pass this gate.

For each behaviour slice, prove a focused failure, make it green, run affected
integration/property tests, then `just verify` and `git diff --check`. Scene
tests are feature-gated: focused commands must include `--all-features` or
`--features storybook` so they actually execute. Useful focused groups:

```bash
cargo test --all-features --test app --test command --test runtime_loop --test interaction
cargo test --all-features --test archetype_assets --test scene_guild_hall --test scene_delve --test scene_runtime --test scene_overlays --test storybook
just verify
git diff --check
```

Run persistence/property gates when their boundaries change, and a release
build before release-binary acceptance. Follow the guarded manual procedure
for live checks. Synthetic Herdr states cannot establish a real done
transition; label fixture completion coverage and live acceptance separately.

Record the revision plus any dirty diff, fixture, viewport, motion/colour mode,
terminal/transport, screenshot or recording, and reviewer decision for each
visual pass. Ask whether the moment is clear, recognisable and enjoyable.

## Historical handoff before the consolidated approval

The following records the earlier sequencing decision. Later approvals below
and the [current assessment](../reviews/2026-09-08-roadmap-assessment.md)
supersede its pending and not-started statements.

2026-09-08 override: the Questmancer moved visual sign-off to the end and asked
to progress engineering. The four-row sidebar is applied and
[campaign heraldry](2026-09-08-campaign-heraldry.md) is implemented. This overrides
the earlier visual-before-heraldry sequencing below, without recording visual
approval. Keepsake expansion, cat reactions and Chronicle chapters are still
separate follow-on work.


The [Herdr 0.9 upgrade](2026-09-08-herdr-090-upgrade.md) now owns current
protocol-22 compatibility and the proposed conditional sidebar trial. Native
visual acceptance remains pending.

The earlier authorised [Herdr 0.8.2 and release-readiness follow-on](2026-09-06-herdr-082-release-readiness.md)
is complete locally against protocol 20, with a verified installed client,
isolated compatibility evidence and release preflight. The separate native
screenshots remain pending; the earlier version baseline and receipts below
are historical.

**Step 1 is complete.** The Questmancer approved the
[six-sheet storyboard](../design/reviews/2026-09-05-party-storyboard/README.md)
and corrected proportions on 2026-09-05. The accepted direction is recorded in
the sprite art guide. Exact animation playback and production terminal
acceptance remain part of the later pilot.

**Steps 2a and 2b are complete.** The Questmancer approved the
[production roster review](../design/reviews/2026-09-05-roster-states/README.md)
on 2026-09-05. `just verify` passed with 542 Rust tests and 26 shell tests.

**Step 2c is complete.** The documentation/workflow contracts and eight
focused Storybook/Librarian tests passed.

**The Librarian and step 3 are implemented and verified.** The historical
2026-09-06 gate passed 555 Rust tests and 28 shell tests after Herdr 0.8.2
compatibility and release workflow corrections. See the 0.9 upgrade for the
current verification receipt.
The cards now use the three new personalised sprites and their visual
approval is recorded on 2026-09-05. The separate Librarian and complete pilot
playback reviews remain open. The Storybook correction and executable resize
results were approved on 2026-09-06. A real terminal resize pass is the next bounded
acceptance step; Ghostty automation is currently blocked by Computer Use.
The executable PTY check does not close that visual gate. Guarded live
acceptance follows the native review. Remaining class expansion,
heraldry and guild-memory work have not started.

## References

- [Repository plan](../../PLAN.md)
- [Agent operating constraints](../../AGENTS.md)
- [Sprite art direction](../design/questmancer-sprite-art-direction.md)
- [Guild Hall art direction](../design/guild-hall-art-direction.md)
- [Guild standing](../design/questmancer-guild-standing.md)
- [Guarded scene acceptance](../manual-test/questmancer-scene-preview.md)
- [Native portrait troubleshooting](../troubleshooting/native-portrait-rendering.md)

## Consolidated approval — 2026-09-08

The Questmancer approved the consolidated visual review after clean local
qualification of 0.1.9 at `c3720a9`. This supersedes earlier pending visual
review entries for the Librarian, three-class pilot, reviewed cards/rooms,
sidebar and heraldry. Native transport observations were not supplied and
remain separate from product approval and local engineering qualification.

The next recommended bounded delight slice is keepsake details: an authored
illustration and fixed description using the existing assignment and card
footprint. Implementation has not been requested. Cat reaction, factual
Chronicle chapters and the next class-art batch follow as separate slices.

## Keepsake details — approved implementation, 2026-09-08

The Questmancer approved the next keepsake slice. The existing six saved
assignments now select a distinct static 8x8 illustration, title and fixed
description. Roomy cards use their lower text section; compact cards use two
text rows and omit artwork. Card dimensions and class portrait bounds stay
unchanged. Live lines in both cards truncate horizontally to preserve all fact rows
and controls at the minimum 60x14 viewport. No new persona, persistence,
inventory, animation or input state is introduced.

[Production review and verification](../design/reviews/2026-09-08-keepsakes/README.md).
This new art needs its own approval. The previous 0.1.9 clean qualification
remains evidence for `c3720a9`, not for these subsequent uncommitted changes.

## Cat reaction — approved implementation, 2026-09-08

The Questmancer approved keepsake visuals and the next single cat reaction.
The canonical Hall's existing cat lifts its head for 400 ms, stretches for
400 ms, then returns to its authored sleeping pose. All frames remain within
the existing 10x5 shelf reservation, without new actor targets.

Runtime compares accepted before/after live party facts. The same non-empty
set of adventurers and panes must move from a known, not-all-resting party
to all Idle. Unknown states, empty parties, additions/removals, stale events
and connection baselines cannot trigger it. Renewed work, changed membership
or a socket boundary cancels it. Both status events and refreshed snapshots
use this path; rendering is pure. Only an ephemeral start timestamp enters
presentation, never SceneSnapshot or persistence.

Full motion uses asset-owned 400/800 ms deadlines; reduced/still motion, Delve
and smaller Hall tiers add no cat wake. Expiration is derived, with no cleanup
timer. [Production review and evidence](../design/reviews/2026-09-08-cat-reaction/README.md).
The reaction awaits visual approval. Factual Chronicle chapters remain a
separate, unstarted slice.

## Chronicle chapters — approved implementation, 2026-09-08

The Questmancer approved the cat reaction and the next factual Chronicle slice.
`c` retains the selected adventurer's existing record list; `Tab` requests a
whole-guild chapter for the last hour, then toggles back to records. The UTC
window is fixed at request time, inclusive at both ends. Only retained entries
inside that window count, deduplicated by EventId with deterministic timestamp
and ID ordering. Fixed templates count events, not distinct adventurers.
Original summaries retain recorded names, with timestamps and source IDs below.
No read checkpoint, generated prose service, stored chapter or achievement is
introduced. Source lines wrap at full chapter width; panes narrower than 80 columns
clip them horizontally.

Source audit found two pre-existing evidence limits: refreshed snapshots do
not append Chronicle entries, and the joined enum also records unknown
whereabouts. Chapters label that category as identity events and cannot fill
unrecorded intervals. Campaign-closure records are counted only if actually
retained; current workspace removal does not emit that event. These limitations
are explicit rather than reconstructed from topology or status.

[Review and verification](../design/reviews/2026-09-08-chronicle-chapters/README.md).
Normal Herdr 0.9 status events already reach the Chronicle through metadata
reconciliation. Snapshot-only capture and clearer identity event taxonomy are
separate correctness follow-ups, preserving reconnect baselines and event
deduplication. The class-art batches that followed are now complete and
visually approved. Snapshot capture and identity taxonomy remain unpromoted.
