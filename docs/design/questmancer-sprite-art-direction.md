# Questmancer sprite art direction

Status: approved direction; implementation has not begun.

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
| Profile portrait | 16x24 | 16 columns x 12 rows | The full Adventurer Card and an art-review fixture. |
| Scene spritlet | 8x14 to 10x16 | compact | Guild Hall and Delve actor placement. |

Both sizes share the same silhouette rules, material roles, class gear and foot
anchor. They are independently authored; the scene spritlet must not be a
mechanical downscale of the portrait. Optional 2x display in a development view
uses nearest-neighbour expansion in the RGB buffer only.

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

The existing seven ancestries and eleven classes remain authoritative. Goblin,
Wizard and Barbarian are the first useful extremes because they expose the
silhouette system most clearly.

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

## Three review passes

1. **Silhouette:** outline plus one flat light fill. Goblin, Wizard and
   Barbarian must be recognisable before detail.
2. **Palette and face:** add material clusters, eyes and the single focal accent;
   review at 1x against all three backgrounds.
3. **Animation and card:** add authored frames, then review the real profile
   card beside its text. Art never earns space by making the card larger.

## Automated safety checks

Tests should prove frame dimensions, row widths, known palette tokens,
non-empty frames, valid foot anchors, stable dimensions within an animation,
deterministic output for fixed persona/time, and safe clipping in small cards.
They do not substitute for the three visible review passes.

## Implementation order

1. Add the three Storybook fixtures and the fixed portrait/spritlet contracts.
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
