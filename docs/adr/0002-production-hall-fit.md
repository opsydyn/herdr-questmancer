---
status: accepted
date: 2026-11-17
decision-makers: Questmancer owner
---

# Integrate proportional Guild Hall fitting

The owner approved the Storybook direction and explicitly promoted production
integration with “yes integrate the hall”. This extends ADR-0001's experiment;
its historical qualification remains separate from production acceptance.

Use the existing canonical 160x90 RGB renderer and approved sprites, expanded
at integer physical-pixel scale through the existing negotiated Kitty transport.
Share the pure transform with Storybook. Never import Storybook from production.
Keep the existing responsive half-block renderer for missing capability or
geometry, tmux, insufficient hit resolution, crowded stations and parchments.
The Delve retains its existing renderer and camera contract.

Reserve one header and two footer rows outside the image: standing, selected
identity/condition and counsel count, then existing hints/notices. Image
placeholders must never be overwritten by text. Existing cards, Ledger,
counsel, search, Chronicle, Board and War Table suspend the native world and
resume it on dismissal. Their existing actions and authority remain unchanged.

Use one startup capability query shared with portraits. Resize uses current
kernel cell pixels when available; missing kernel pixel geometry retains the
negotiated startup geometry, so font zoom may require reopening the pane.
No Herdr graphics geometry workaround or server/configuration change is added.

The adapter owns at most two random image identities. Replace/delete only its
own identities after frame flush and release them on fallback, view change,
errors and normal shutdown. Cache identical RGB/geometry. Reuse the existing
semantic frame deadlines; no independent timer, domain state or persistence.
The upload canvas remains bounded to four million pixels and integer scale16.

Canonical station allocation must fit the actual party, not merely its total
count; otherwise choose the existing complete compact/roster composition.
Never report a partial canonical party as a successful native fit. Mouse cells
and visible pixels must use the same transform, including letterboxing.

Implementation: promote `storybook/fit` to `scene/fit`; add a production output
adapter in `ui/hall_fit`; share startup Picker in `terminal`; retain existing
input/reducer semantics using projected cell hit regions. Test source pixels,
mouse targets, overflow, overlay/fallback/Delve transitions, resize, cached
uploads, own-image cleanup and semantic deadlines. Run affected suites, full
`just verify`, release build and isolated PTY producer checks. Native visibility,
mouse, animation and resize acceptance remain human gates. Leave the synthetic
party open; do not commit, publish, restart the server or change agent facts.

Alternatives: earlier roster switching loses source detail; downsampling discards
approved pixels; a new renderer/platform would exceed this scope. Revisit the
upload budget if measured native animation cost is excessive, and revisit font
zoom when the transport exposes fresh geometry without concurrent stdin queries.
