# Questmancer sprite art direction

Status: approved direction; silhouette, material and two-tier portrait Storybook
passes are available for visual review. Production sprite integration has not
begun.

## Decision

Questmancer characters should read as cute, stocky, 16-bit adventure figures:
large-headed, short-legged, outlined, and readable at production size against a
dark world. The design goal is not realism or a literal copy of any existing
game. It is an original shared vocabulary for the Guild Hall, Delve and profile
cards.

The current profile-card figure is a separate 10x12 terminal-cell rectangle
composer. It is the source of the tall, flat, diagram-like look. The RGB scene
renderer and its half-block adapter remain the correct technical foundation.

```text
transparent authored sprite
        -> RGB scene buffer
        -> upper/lower half-block conversion
        -> Ratatui buffer
```

No terminal image protocol, full-block scaling trick, Braille renderer or
renderer rewrite is required.

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

Both sizes share the same silhouette rules, material roles, class gear and foot
anchor. They are independently authored; the world sprite must not be a
mechanical downscale of the portrait. The existing compact production spritlets
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

The current Storybook sequence is intentionally split into eight stable views:

- **Sprite Silhouette Lab** preserves the outline-and-flat-fill baseline.
- **Sprite Material & Face Lab** presents the three material passes at native
  scale against Delve-dark, neutral and torch-warm backgrounds.
- **Sprite Material Inspection 2x** reuses those exact assets through a
  nearest-neighbour scene blit. Use `enter` in Storybook to give all three
  inspection cards their full-width review surface.
- **Sprite World & Portrait Masters** places each 16x24 world sprite beside its
  independently authored 24x32 portrait. Neither tier is scaled to manufacture
  the other.
- **Sprite Scout & Shadow World** is the native-size Bard, Ranger and Rogue
  review surface.
- **Sprite Scout & Shadow Masters** compares that batch's 16x24 world frames
  with their independent 24x32 portrait masters. It is the review gate before
  any old sprite is deprecated or any production card is changed.
- **Sprite Grovekeeper World** is the native-size Druid review surface.
- **Sprite Grovekeeper Masters** compares its leafy Living Staff, antlered hood
  and beard materials at 16x24 and 24x32 before any production portrait
  cutover.

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
5. Re-author compact scene spritlets in the same vocabulary.
6. Add pose-specific frames and only the small layout adjustment needed to keep
   portrait text readable.

## Non-goals

- No production scene cutover.
- No change to Herdr truth, action handling or persistence.
- No literal Zelda or reference-sheet sprite reproduction.
- No procedural sprite generator, ECS, image protocol or full sprite editor.
