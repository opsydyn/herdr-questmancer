# Questmancer Astro asset loading and mapping icons

Status: **Approved**

Date: 2026-08-08

## Decision

Use Astro's built-in `Image` component for the launch page's local raster
screenshots. The evidence gallery will import its files from `src/assets` and
pass those imports directly to `<Image />`, with eager loading because these
screenshots are the page's operational proof and are expected to be visible in
the launch experience.

Keep the site base-path safe by retaining all local assets under `src/assets`
and hardening the build audit so every generated local absolute reference maps
to an emitted file. Replace the generic vocabulary symbols with four vendored
Pixelarticons SVGs: `sword`, `castle`, `bell` and `coins`.

## Scope

- Update `EvidenceGallery.astro` to use `Image` from `astro:assets`.
- Replace the four mapping icon files and imports with the approved fantasy
  set.
- Make the base-path audit validate referenced output files, not only expected
  asset stems.
- Extend the content and audit tests to protect these contracts.
- Leave the hero artwork, copy, layout, fonts, cursor and deployment workflow
  unchanged.

## Constraints

- Keep raster screenshots in `site/src/assets/screenshots` so Astro can process
  and bundle them for GitHub Pages.
- Keep Pixelarticons as raw MIT-licensed SVG assets with `currentColor`; do not
  add a runtime or icon framework.
- Keep `/herdr-questmancer/` as the configured base and reject root-bypassing
  local references.
- Preserve descriptive alt text, one semantic `h1`, no client-side JavaScript
  and the current responsive composition.

## Acceptance evidence

- `site/scripts/content-contract.test.mjs` requires direct `<Image />` usage,
  eager evidence images and the new icon stems.
- `site/scripts/check-base-path.test.mjs` fails on a generated reference whose
  target file is absent.
- `cd site && bun run verify` passes and emits base-prefixed screenshot and
  icon URLs.
- Browser inspection reports non-zero `naturalWidth` for every image at the
  base route at desktop and mobile widths.
