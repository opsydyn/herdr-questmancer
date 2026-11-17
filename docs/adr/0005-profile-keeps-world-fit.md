---
status: accepted
date: 2026-11-17
decision-makers: Questmancer owner
---

# Preserve fitted rooms beneath adventurer profiles

The owner reported that opening an adventurer profile loses responsive fitting
in both rooms. ADR-0002 deliberately suspended the world image for every card;
that now causes an unwanted scale jump. This corrects the profile path only.
Other modal parchments retain their existing fallback behavior.

Keep the full room's fit geometry, source pixels and world image lease unchanged
when a profile opens or closes. Keep the existing profile dimensions, native
portrait/fallback, text, keys and authority. Do not shrink the world into a new
side panel or silently revert its rendering mode.

The existing Kitty widget encodes entire rows into their first terminal cell.
Overdrawing a card can suppress its first upload, or leave stale text when the
card closes. Use the installed ratatui-image public SlicedProtocol/SlicedImage
API to paint the visible top, left and bottom bands of the same virtual image.
Reserve the profile's existing right-edge gutter in that mask. No new transport,
protocol encoder, dependency or image identity pool is introduced. Render the
profile after the masked world; its widget cannot overwrite world upload prefixes.

Share one profile rectangle between drawing and input. Only complete unoccluded
actor/interactable regions remain clickable while the card is visible. Consume
mouse input inside the profile so it cannot select/fetch output from a hidden
adventurer. Keyboard navigation still reaches the entire party. A wholly hidden
world retains its bounded lease and requires no animation wake; first exposure
must still transmit that image if it was never drawn. Opening/closing a static
profile does not reencode or delete the world image. Existing shutdown cleanup
still deletes only the world's own two IDs.

Implementation: extract pure card-area projection from scene_overlays; add shared
masked-image/hit-region helpers to ui/world_fit; route runtime input through its
last drawn profile mask. Use the same drawing/interaction behavior in Storybook.
No domain state, persistence, Herdr topology, agent command or portrait routing
change is authorized by this UI fix.

Verify test-first in both rooms: unchanged fit/lease, profile covered clicks,
resize, tiny/full occlusion, open-at-start then dismiss, repeated toggles and
native portrait layering. Use persistent terminal/diff and isolated executable
PTY evidence, plus full verification and release build. Native compositing and
appearance remain human acceptance. Preserve the current owned live resources.

Boundary review: safe with noted terminal-compositing risk. Presentation owns
occlusion, domain/reducer actions retain their authority, and rendering creates
no agent effects. Docking the card would change room scale; a custom Kitty
encoder would duplicate existing transport. Shared row slicing is the smaller
complete change. This supersedes only the profile exception in ADR-0002/0003.
