# Questmancer sprite art direction

Status: approved and integrated. All twelve production classes now use
authored 16x24 world masters and independent 24x32 portraits. No class routes
to another's silhouette: a borrowed master makes two different adventurers
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
Ranger, Rogue, Runewright, Testmender and Wizard. Cards are `384x512` with a
transparent background, and no two classes may share one — a class without its
own card silently borrows a sibling's, which is how two different adventurers
end up wearing the same face. Class is the primary visual identity
for both ordinary world sprites and cards: an Orc Ranger reads as a Ranger and
an Orc Wizard reads as a Wizard. Goblin and Orc illustration is reserved for
future event/NPC storytelling rather than automatic persona routing. The
registered 24x32 class master is the unconditional fallback. The RGB scene
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

Native terminal graphics are deliberately confined to the expanded card. A
terminal's half-block image fallback is not used because Questmancer's authored
24x32 sprite is the canonical non-native result. No full-block scaling trick,
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

## Two authored sizes, one visual language

| Presentation | Logical size | Terminal footprint | Purpose |
|---|---:|---:|---|
| Profile portrait | 24x32 | 24 columns x 16 rows | Face, gear and material detail in the full Adventurer Card. |
| World sprite | 16x24 | 16 columns x 12 rows | Readable Guild Hall and Delve actor with the same class identity. |
| Roster master | 8x12 | 8 columns x 6 rows | Whole-party read in a narrow pane, authored per silhouette family. |

Roster masters follow two rules the larger sizes can afford to relax. Their
outline is *tinted* per family rather than near-black: at eight pixels wide a
one-pixel border is roughly half the sprite, and a black one turns the party
into a row of rectangles against the Hall floor. Their silhouette must taper —
a head narrower than the shoulders and legs parted by negative space — because
without it the outline closes into a box whatever colour it is.

Both sizes share the same silhouette rules, material roles, class gear and foot
anchor. They are independently authored; the world sprite must not be a
mechanical downscale of the portrait.

World masters are personalised at render time: the persona's skin (`k`/`K`/`h`),
hair (`r`/`R`) and accent (`a`) role clusters take the adventurer's palette,
while cloth, metal, gear and focal colours stay authored so class identity
never changes. The Barbarian v2 masters use their own key grammar (`S` skin,
`H`/`h` hair) and personalise through the same substitution. Tests prove that
role colours stay unique within each master palette, that substitution never
alters a transparency mask, and that same-class personas render distinctly. The existing compact production spritlets
remain unchanged during this review slice. Optional 2x display in a development
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

The existing seven ancestries and twelve classes remain authoritative. Goblin,
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

Two routes satisfy this contract. The Barbarian authors a complete frame per
pose and is the reference. Every other class keeps one authored master and
takes an authored *pose decoration* over it: a carried chest for Returning
with Spoils, a wider seated stance for Resting. Decorations resolve their
glyphs against the class's own palette, so the pose changes without the class
changing, and they occupy the torso and leg zones because class gear lives on
the left and right edges of every master. Seeking Counsel deliberately has no
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

The current Storybook owns three stable asset-review views:

- **Core World Masters** shows Barbarian, Bard, Cleric, Druid, Paladin, Ranger,
  Rogue and Wizard at their native 16x24 production scale.
- **Core Portrait Masters** shows the same class identities as independently
  authored 24x32 portraits. These are not scaled world sprites.
- **Goblin Easter Egg** shows the authored Goblin ancestry callback at both
  production scales.
- **Roster Silhouette Families** shows the five authored 8x12 masters a narrow
  pane recomposes the party into.
- **Persona Palette Family** shows one shared class master across the persona
  skin, hair and accent range.

The Guild Hall and Delve stories render these same registered production
assets in context. Storybook does not retain the retired 8x14 generator.

## Four review passes

1. **Silhouette:** outline plus one flat light fill. Goblin, Wizard and
   Barbarian must be recognisable before detail.
2. **Palette and face:** add material clusters, eyes and the single focal accent;
   review at 1x against all three backgrounds.
3. **World and portrait:** approve recognisable 16x24 world sprites beside
   richer 24x32 masters before either enters production.
4. **Animation and card:** add authored frames, then review the real profile
   card beside its text. Art never earns space by making the card larger.

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
