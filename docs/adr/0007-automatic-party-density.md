---
status: accepted
date: 2026-11-17
decision-makers: Questmancer owner
---

# Integrate automatic fitted-room density

The owner approved the Storybook size tiers and automatic switching: retain
current detailed masters for small parties, use smaller authored sprites for
larger parties, and preserve full-size profiles. This promotes ADR-0006's
direction to production after the screenshot-driven saturation review.

The native fitted room uses deterministic neutral seats derived from stable
adventurer identity. Presence changes update pose/cues, not the seating or tier.
Up to eleven visible adventurers use 16x24 masters; twelve through thirty-two
use the approved staggered 8x12 seating. Thirty-three through sixty-four use a
denser 8x12 roster with explicit state cues. The dense tier omits the Librarian
actor, retaining keyboard Ledger access. Exited adventurers do not count.
Above this bounded room capacity retain the existing explicit fallback; never
silently accept a partial fitted party or scale artwork into smaller tokens.

Reuse existing RGB Hall/Delve environment passes and semantic actor animation,
state cues and bounded completion effects. Extract backdrop passes rather than
fabricating empty live snapshots or adding a second production renderer. The
native FitScene projects the fitted composition, then verifies every complete
actor and mouse target. Keep existing raw half-block responsive/crop behavior
for unsupported graphics, missing geometry, other modal parchments and tiny
unaddressable native sizes. Existing profile masking, image ownership and
source-pixel fitting remain shared.

Neutral seats do not assert that an adventurer is working at the pictured
furnishing. Their actual state comes from pose, non-color cue and profile.
Document this presentation change in the Ledger. Herdr still owns live facts;
no domain station, persona, command, persistence, source adapter or configuration
is changed. Production must not import Storybook. Shared authored seat geometry
may be used by the fixed comparison to avoid drift.

Boundary verdict: safe with noted animation and terminal-compositing risks.
All changes belong to presentation; no I/O enters rendering. Use focused failing
production tests first, then threshold/Exited/status transitions, all visible
bounds/clicks, profile/cache, connection lighting, semantic deadlines and dense
limits. Run the full gate, release/Storybook/Rustdoc builds and isolated PTY
evidence. Refresh only the freshly verified owned dogfood UI, retain resources,
work, personas and focus. Native visual acceptance remains separate.

Alternatives: adding station slots alone does not address growing parties;
continuous shrinking loses authored pixels; only reusing the old compact Hall
retains the rejected bottom packing. More than sixty-four visible identities
needs a separate campaign-navigation decision. No commit, publication or shared
server operation is included.
