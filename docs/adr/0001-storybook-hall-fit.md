---
status: accepted for Storybook experiment only
date: 2026-11-17
review-date: 2026-11-24
decision-makers: Questmancer owner
---

# Qualify proportional Hall fitting in Storybook

## Context

The owner approved the right-hand fit proposal in the Hall responsiveness
review and explicitly requested Storybook first. The current compact Hall keeps
16x24 actors at native cell scale and loses room depth. The current canonical
station allocator can also omit actors in crowded state distributions.

## Decision

Keep all new fitting, geometry, transport and interaction state behind the
existing `storybook` feature. Render the existing RGB Hall at 160x90. Present
its pixels using integer physical-pixel scaling, with native Kitty placeholders
through the existing ratatui-image dependency. Negotiate support and geometry;
otherwise show the existing half-block renderer with an explicit diagnostic.
Refuse native canonical fitting if its actor set is incomplete.

Use fixed fixture models and local selection/card/Ledger actions only. Render
readable selection/attention text outside the picture. Map mouse cell centres
through the same fitted pixel transform. Use bounded image identities and delete
only those identities on replacement, story exit and shutdown. Suspend the
fitted world while parchment overlays are open; closing them restores the fit.

The production renderer, runtime, state schema, Herdr commands and native-world
policy remain unchanged. This decision is not approval for production native
world output. No dependency, agent launch or live-party operation is added.

## Implementation and boundaries

- `src/storybook/fit.rs`: pure fit transform and checked projection from the
  existing scene snapshot/renderer. No domain mutation, transport or persistence.
- `src/storybook/fit_preview.rs`: bounded native image cache, isolated fixture
  interaction and preview widgets. Cleanup commands are returned to the runtime.
- Storybook catalogue/fixtures/UI/runtime: new explicit stories, shared startup
  capability query, routing and cleanup. Existing stories retain their renderer.
- `tests/storybook_fit.rs` and local tests: geometry, capacity, source-pixel
  preservation, hit mapping, fallback, selection and cache lifecycle.

Boundary review: **safe with noted risk**. Storybook depends on production
rendering/domain projections and existing image transport; production imports
no preview module. Native stacking and delivery remain terminal acceptance risks.

## Alternatives

- Earlier 8x12 roster: retains broad compatibility but loses source detail and
  full-room composition; keep it as a fallback rather than claiming equivalence.
- Downsample masters: rejected because it discards approved source pixels.
- Production native adapter now: deferred until this experiment is reviewed.
- New renderer or Herdr App port: outside this approved scope.

## Verification

- [x] At 120x34, 96x28 and 80x24 with 8x16 cells, every source pixel is preserved.
- [x] Five- and eleven-person distributed fixtures retain all actors/Librarian.
- [x] Crowded canonical placement is diagnosed rather than silently accepted.
- [x] Clicks, keyboard selection, letterboxing and resize use one transform.
- [x] Unsupported graphics/geometry and overlay transitions show a usable view.
- [x] Image identities remain bounded and cleanup never targets unrelated images.
- [x] Full repository verification passes; native appearance is recorded separately.

## Consequences and revisit

The prototype adds a second output adapter in Storybook, not a second world
renderer. Integer scaling may letterbox and cannot fit at arbitrarily tiny
sizes. Cards/Ledger use the existing half-block backdrop during the experiment.
Fixed Storybook time supplies no animation acceptance. Native visual/resize
review and any animation qualification precede a production decision. Revisit
after the owner's native review or on the review date above.

See `docs/design/reviews/2026-11-17-hall-responsiveness/README.md` and
`docs/reviews/2026-11-17-storybook-hall-fit/README.md`.

## Qualification result — 2026-11-17

The automated criteria above passed: 807 Rust tests / 65 runs and 28 shell
tests, native/graphics-off synthetic PTY scenarios, and a byte-exact native
image payload comparison. These checkboxes describe fixture/producer evidence,
not terminal display acceptance. See the linked review receipt. Production
integration remains unapproved; native visual acceptance and dynamic animation
remain unqualified.
