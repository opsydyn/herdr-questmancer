# Questmancer sprite art direction

Status: existing art system integrated; the revised three-class proportion
direction below was visually approved on 2026-09-05 and is implemented in the
three-class pilot. Production art uses authored `16x24` world masters and
`24x32` card canvases. Wizard, Ranger and Barbarian reuse their new world
sprites inside that canvas; the other classes retain independent portraits.
No class routes to another's silhouette: a borrowed master makes two different adventurers
indistinguishable in the world, which is the same defect as a borrowed card.
`no_two_classes_share_a_world_or_portrait_master` keeps it that way.

## Decision

Questmancer characters should read as cute, stocky, 16-bit adventure figures:
large-headed, short-legged, outlined, and readable at production size against a
dark world. The design goal is not realism or a literal copy of any existing
game. It is an original shared vocabulary for the Guild Hall, Delve and profile
cards.

The current profile card uses a native transparent PNG when an approved Kitty,
Sixel or iTerm2 protocol is detected. Every production class now owns a
distinct card: Artificer, Barbarian, Bard, Cleric, Druid, Paladin, Pathseeker,
Ranger, Rogue, Runewright, Testmender, Wizard, Mage and Sorcerer. Cards are `384x512` with a
transparent background, and no two classes may share one — a class without its
own card silently borrows a sibling's, which is how two different adventurers
end up wearing the same face. Class is the primary visual identity
for both ordinary world sprites and cards: an Orc Ranger reads as a Ranger and
an Orc Wizard reads as a Wizard. Goblin and Orc illustration is reserved for
future event/NPC storytelling rather than automatic persona routing. The
registered authored class sprite in a `24x32` canvas is the unconditional fallback. The RGB scene
renderer and its half-block adapter remain the correct foundation for the world
itself.

Detection describes the complete pane transport, not merely the outer terminal.
For Herdr-managed panes, native Kitty graphics require
`experimental.kitty_graphics = true` in Herdr configuration. Questmancer must
not infer transport support from `TERM_PROGRAM`: a false positive suppresses
the authored sprite while an intermediary discards the native image sequence,
leaving an empty card.

```text
transparent authored sprite
        -> RGB scene buffer
        -> upper/lower half-block conversion
        -> Ratatui buffer
```

Native terminal graphics are confined to expanded cards and the Librarian's
Ledger illustration. A
terminal's half-block image fallback is not used because Questmancer's authored
RGB sprite in its `24x32` canvas is the canonical non-native result. No full-block scaling trick,
Braille renderer or world-renderer rewrite is required.

## Shared character grammar

Every character should have:

- a mostly continuous one-logical-pixel near-black *tinted* outline;
- a head occupying roughly 40–50% of visible body height;
- no visible neck or only a one-pixel suggestion of one;
- a torso as wide as it is tall, with short two- or three-pixel feet;
- one or two deliberate facial pixels, not a generic skin rectangle;
- connected shadow/base/highlight clusters rather than checkerboard noise;
- a gear silhouette separated from the body by negative space; and
- one bright, meaningful focal accent: gem, blade edge, buckle, eye or rune.

Avoid thin one-pixel limbs, long rectangular torsos, large unbroken primary
colour fields, pure-black interiors, anti-aliasing, smooth gradients and pillow
shading.

## Librarian status — 2026-09-05

The Librarian remains a static help NPC, visible and clickable in canonical
and compact Halls. The 2026-09-05 refresh retains orange fur, low gold
spectacles, the purple robe and carried books while broadening the head and
shortening the body. Its world master stays `16x24`, with feet on row 21.
The Ledger now has an independently authored `24x32` fallback with a larger
face and distinct book spines. The native illustration at
`src/assets/librarian.png` is unchanged.

The [two-sheet production review](reviews/2026-09-05-librarian/README.md)
includes before/after art, literal size, ANSI-16, both Hall compositions and
actual Ledger overlay buffers. Implementation was authorised by “Yes librarian
then move on”; visual approval of the resulting art is pending. The three-class
storyboard approval does not substitute for this review.

## Accepted three-class proportion revision — 2026-09-05

The Questmancer approved the
[Wizard, Ranger and Barbarian storyboard](reviews/2026-09-05-party-storyboard/README.md)
with “Way better! Approved”. Its corrected static silhouettes are the reference
for the implemented pilot. Production playback and presentation await visual
approval; the [review pack](reviews/2026-09-05-party-pilot/README.md) records
the current frames, rooms, cards and counsel outcomes.

- Keep the `16x24` canvas and foot row 21, counted from zero. Preserve each
  class's horizontal foot anchor across poses; redraw body allocation without
  shrinking the whole sprite.
- Enlarge the head/face, shorten and broaden the torso, minimise the neck and
  retain short, grounded feet. Evaluate the visible anatomy separately from
  hats, weapons and other projecting gear.
- Wizard: face/beard allocation grows from six to eight rows; the body
  shortens from six to five. Retain the crooked hat, staff and book.
- Ranger: head/hood allocation grows from seven to nine rows; the body
  shortens from nine to six. Retain the separated bow, quiver and map.
- Barbarian: head/hair allocation grows from seven to ten rows; the body
  shortens from ten to six. Retain broad shoulders, copper hair and the axe.
- Keep `8x12` roster art independently authored, with foot row 10. The three
  roster studies establish the pilot's direction. The shared state cues are
  now implemented and their [production roster review](reviews/2026-09-05-roster-states/README.md)
  was visually approved on 2026-09-05.

Those row allocations describe authored anatomical bands, not universal test
thresholds. The existing material roles and persona identity remain the
production contract. The pilot uses the existing `k`/`K`/`h` skin, `r`/`R`
hair, `l` garb and `a` accent roles; saved personas remain unchanged.

Use the approved key poses as the starting point for two working frames per
class, a short counsel gesture followed by stillness, and returned-spoils
theatre that settles within three seconds. Rest, unknown and settled completion
remain static. A later Herdr working event resumes work; counsel delivery
acknowledgement alone does not. These pilot decisions supersede the older
idle-frame targets below for these three classes. Exact timing and
intermediate frames require production playback review. The implementation
uses two 500 ms working frames, a 600 ms raised-hand gesture, placement of
spoils at 1000 ms and quiet completion by 3000 ms. All feet remain fixed.
Frame timing lives beside the authored assets in `rituals.rs`; both rooms
consume the same sample and next visible pixel-change deadline.

The pilot originally retained its older card fallbacks. The Questmancer then
requested that cards use the new sprites, replacing those long-bodied images.
Wizard, Ranger and Barbarian now use the exact personalised `16x24` working
master, centred in the existing `24x32` portrait canvas with a four-pixel
margin. No scaling, stretching or separate copy of the art is involved. This
static identity portrait does not animate or claim a change in presence.

The [card fallback review](reviews/2026-09-05-card-fallbacks/README.md) shows
before/after art and actual truecolour/ANSI card buffers. The other eleven
classes retain their current fallbacks; native illustrations and the revised
independent Librarian fallback keep their existing routes. The Questmancer
visually approved the card fallbacks on 2026-09-05. Actual terminal
presentation remains a separate acceptance check.

## Two authored sizes, one visual language

| Presentation | Logical size | Terminal footprint | Purpose |
|---|---:|---:|---|
| Profile portrait canvas | 24x32 | 24 columns x 16 rows | Current pilot world sprite centred at native size; independent portrait masters for other classes. |
| World sprite | 16x24 | 16 columns x 12 rows | Readable Guild Hall and Delve actor with the same class identity. |
| Roster master | 8x12 | 8 columns x 6 rows | Whole-party read in a narrow pane, authored per silhouette family. |

Roster masters follow two rules the larger sizes can afford to relax. Their
outline is *tinted* per family rather than near-black: at eight pixels wide a
one-pixel border is roughly half the sprite, and a black one turns the party
into a row of rectangles against the Hall floor. Their silhouette must taper —
a head narrower than the shoulders and legs parted by negative space — because
without it the outline closes into a box whatever colour it is.

The presentations share silhouette rules, material roles and class gear.
Independent portrait masters must not become mechanical world downscales.
The current pilot deliberately reuses the same native world sprite on cards
so its newly approved proportions and persona colours stay consistent.

World masters are personalised at render time: the persona's skin (`k`/`K`/`h`),
hair (`r`/`R`), garb (`l`, the trim band) and accent (`a`) role clusters take
the adventurer's palette, while cloth, metal, gear and focal colours stay
authored so class identity never changes.

Garb deliberately lands on the trim rather than the cloth mass. Tinting the
cloth itself was tried and abandoned: at every tint strength the blend
collided with a world material somewhere different — a Bard in Vestments on
Hall stone, a Cleric in Armour on dungeon floor, a Ranger in Armour on dungeon
moss. Garb colours are already proven against both worlds on their own, so
using them directly on the trim is safe where a blend was not, and the class's
body mass stays the class's. All three pilot classes use the standard role
grammar. Tests prove that role colours stay unique within each master palette,
that substitution never alters a transparency mask, and that same-class
personas render distinctly. The pilot also registers its approved authored
roster bodies; other classes keep their existing family masters. Optional 2x display in a development
view uses nearest-neighbour expansion in the RGB buffer only.

The standing anchor is horizontally centred on the bottom opaque row. Hats,
ears, staffs and weapons may use outer frame space, but feet remain stable
across idle and walking frames.

## Palette roles

Frame data uses named tokens rather than scattered RGB values. The baseline
roles are:

```text
.  transparent
o  tinted outline
k/K  skin shadow / base
h  skin highlight
r/R  hair shadow / base
c/C  cloth shadow / base
l  cloth highlight or trim
m/M  metal shadow / light
a  accent
e  eye, gem or rune focal point
d  leather or wood
```

Not every sprite uses every role. A visible character normally uses eight to
ten colours plus transparency, spending shades only where a cluster has room.
The palette must remain readable over the dark Delve, a neutral debug field and
Guild Hall torch light.

## Art-direction fixtures

The following three figures are review fixtures, not a replacement for
Questmancer's ancestry/class domain model.

| Fixture | Silhouette proof | Material and focal proof |
|---|---|---|
| Goblin | upward uneven ears, square head, tiny boots, separated dagger | green skin clusters, yellow eyes, leather and purple sash |
| Wizard | bent hat, staff, broad triangular beard, bell robe | blue/purple robe clusters, gold trim, bright blue gem |
| Barbarian | broad shoulders, spiked hair/beard, narrow waist, distinct axe | warm skin, fur/leather, steel axe head and bone/buckle accent |

They establish reusable rules:

- **ancestry** changes head outline, ears, proportions and face read;
- **class and gear** supply the strongest external silhouette; and
- **appearance** changes palette, hair, face detail and clothing without
  erasing class recognition.

The existing seven ancestries and fourteen classes remain authoritative. Goblin,
Wizard and Barbarian are the first useful extremes because they expose the
silhouette system most clearly. Bard, Ranger and Rogue are the second approved
review batch: their readable primary gear is, respectively, lute, bow and
quiver, and thieves' tools with paired daggers as a secondary silhouette. They
do not add or alter a domain class. Druid is now a persisted domain class with
a Living Staff, and has a dedicated world/portrait master review slice.

## Pose and animation contract

The application’s semantic poses remain the source of truth. They map to
authored frame groups rather than a whole unchanged sprite being moved around.

| Semantic state | Art group | Minimum visible change |
|---|---|---|
| Working | Walk / task | leg, arm, tool or clothing change |
| Seeking Counsel | Idle / signal | raised hand, shifted ears, staff or expression |
| Returning with Spoils | Signature | item, weapon or celebratory gesture changes shape |
| Resting | Idle | beard, ear, chest or hat movement within one pixel |
| Unknown | Static concern | subdued but still readable silhouette |

Initial target frame groups: idle 2, walk 4, signature 4, hurt/concern 2.
Feet may not drift; a body bob is at most one logical pixel and never substitutes
for an unchanged animation frame.

Two routes currently supply poses. Wizard, Ranger and Barbarian author complete
frames for working, counsel, spoils, completion, rest and unknown. The other
eleven classes keep one authored master and take an authored *pose decoration*
over it: a carried chest for Returning
with Spoils, a wider seated stance for Resting. Decorations resolve their
glyphs against the class's own palette, so the pose changes without the class
changing, and they occupy the torso and leg zones because class gear lives on
the left and right edges of every master. For those eleven classes, Seeking Counsel has no
decoration: that state already carries the authored counsel marker, and a
second signal would only compete with it.

## Storybook sprite lab

Extend the existing development-only Storybook instead of introducing a second
tool. The lab should show, for each fixture:

- actual 1x production presentation;
- optional 2x nearest-neighbour inspection;
- dark Delve, neutral debug and warm torch backgrounds;
- palette swatches and current frame identity; and
- portrait and scene-spritlet side by side.

The lab is for review only. It has no agent prompt, Herdr command, persistence
mutation or sprite-editor ambition.

The current Storybook has thirty-four stories, including these asset galleries:

- **Core World Masters** shows Barbarian, Bard, Cleric, Druid, Paladin, Ranger,
  Rogue and Wizard at their native 16x24 production scale.
- **Core Portrait Masters** shows the production card fallbacks: centred native
  world sprites for the three pilot classes, independent portraits for the
  other classes. All occupy a `24x32` canvas without stretching.
- **Goblin Easter Egg** shows the authored Goblin ancestry callback at both
  production scales.
- **Roster Silhouette Families** shows the five authored 8x12 masters a narrow
  pane recomposes the party into.
- **Persona Palette Family** shows one shared class master across the persona
  skin, hair and accent range.
- **Custom Class Masters** shows Artificer, Runewright, Testmender and Pathseeker
  world/portrait pairs. **Barbarian Poses**, **Wizard Poses** and **Ranger Poses**
  each show eight production moments, including the still counsel wait.
- **Librarian** shows the revised world sprite and independent Ledger fallback.
- Fourteen native class-card stories cover all classes, including Mage and
  Sorcerer; Goblin and Orc have separate reserved event-art stories.

The Guild Hall and Delve stories render these same registered production
assets in context at fixed fixture time. Review GIFs sample production room
frames at explicit times for animation playback; the Storybook itself remains
event-driven. It does not retain the retired 8x14 generator.

## Four review passes

1. **Silhouette:** outline plus one flat light fill. Goblin, Wizard and
   Barbarian must be recognisable before detail.
2. **Palette and face:** add material clusters, eyes and the single focal accent;
   review at 1x against all three backgrounds.
3. **World and portrait:** approve recognisable 16x24 world sprites beside
   richer 24x32 masters before either enters production.
4. **Animation and card:** add authored frames, then review the real profile
   card beside its text. Art never earns space by making the card larger.

## The goblin outbreak

Typing `release the goblins` into the search parchment (`/`) and submitting it
opens a three-second window in which goblins raid both worlds: the doorway and
the shelves in the Guild Hall, the entrance-left and the centre-bottom of the
Delve. The parchment answers `The goblins deny any involvement.` either way.

Two rules govern it, and both are load-bearing:

- **It is presentation, never truth.** The flag rides on `ScenePresentation`,
  not `SceneSnapshot`. Releasing goblins must not alter anything Questmancer
  reports back to Herdr, and nothing about the outbreak is persisted — reopening
  the app never restores it. `snapshot_ignores_legacy_ui_persistence_and_goblin_state`
  holds this line.
- **It is a sighting, not a takeover.** Goblins occupy authored dens that no
  adventurer station uses. Actor regions are byte-identical whether or not
  goblins are loose, proven by `goblins_never_stand_where_an_adventurer_stands`.

The painting renderer returns its own 200ms frame deadline, because the
snapshot-driven cadence knows nothing about presentation state; without it the
window would not visibly close until unrelated Herdr traffic arrived.

This egg shipped inert for a long time. The trigger set the state and
`GoblinState::is_visible` had no callers, so the feature existed in the release
notes and nowhere on screen. It is the clearest instance of this codebase's
recurring failure mode — **a test asserting that state changed while nothing
asserts it reaches a pixel** — and the reason the goblin tests above assert
against rendered buffers rather than against the flag.

## Automated safety checks

Tests should prove frame dimensions, row widths, known palette tokens,
non-empty frames, valid foot anchors, stable dimensions within an animation,
deterministic output for fixed persona/time, and safe clipping in small cards.
They do not substitute for the three visible review passes.

## Implementation order

1. Add the Storybook fixtures and fixed world/portrait contracts.
2. Author and approve silhouette-only frames.
3. Add named palette roles and material/face detail.
4. Replace the old profile-card rectangle composer with the portrait path.
5. Add pose-specific frames and only the small layout adjustment needed to keep
   portrait text readable.

## Non-goals

- No change to Herdr truth, action handling or persistence.
- No literal Zelda or reference-sheet sprite reproduction.
- No procedural sprite generator, ECS, image protocol or full sprite editor.
