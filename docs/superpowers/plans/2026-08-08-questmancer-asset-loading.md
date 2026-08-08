# Questmancer Astro asset loading Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended; not used here because inline execution is approved) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the launch page's screenshot gallery idiomatic Astro and replace generic mapping symbols with base-safe fantasy Pixelarticons.

**Architecture:** Keep raster assets imported from `site/src/assets` and render the evidence gallery through `astro:assets`'s `Image` component. Vendor four raw Pixelarticons SVGs and import them from the existing mapping component. Extend the static build audit to resolve every generated local reference to a real output file.

**Tech Stack:** Astro 7, `astro:assets`, Bun, plain CSS, Node's built-in test runner and vendored Pixelarticons SVGs.

## Global Constraints

- Keep the site static and dependency-light; add no client JavaScript or icon runtime.
- Preserve the `/herdr-questmancer/` GitHub Pages base path.
- Keep all existing approved copy, fonts, cursor, hero framing and responsive layout.
- Use Pixelarticons raw SVGs under the MIT license with `fill="currentColor"`.
- Preserve descriptive screenshot alt text and one semantic `h1`.

## File map

- Modify: `site/src/components/EvidenceGallery.astro` — use `Image` and eager evidence loading.
- Modify: `site/src/components/MappingCards.astro` — map sword/castle/bell/coins imports to the four vocabulary cards.
- Modify: `site/src/assets/icons/{sword,castle,bell,coins}.svg` — vendor the approved Pixelarticons files.
- Remove: `site/src/assets/icons/{home,lock,heart}.svg` — no longer referenced by the mapping cards.
- Modify: `site/scripts/check-base-path.mjs` — resolve generated local references to output files.
- Modify: `site/scripts/check-base-path.test.mjs` — add the missing-reference regression fixture.
- Modify: `site/scripts/content-contract.test.mjs` — assert the Astro image and icon contracts.

### Task 1: Add failing regression contracts

- [ ] Add a base-path test whose HTML references a base-prefixed file that is not present, and expect `validateBuild` to throw a missing-reference error.
- [ ] Add content assertions for direct `astro:assets` `Image` usage, eager evidence images and the `sword`, `castle`, `bell`, `coins` icon stems.
- [ ] Run `cd site && bun run build && bun run check:base && bun run check:content` and confirm the new assertions fail against the current implementation.

### Task 2: Implement idiomatic Astro images and fantasy mappings

- [ ] Import `{ Image }` from `astro:assets` and render each `item.src` directly with its existing alt text, `loading="eager"` and `decoding="async"`.
- [ ] Copy the installed Pixelarticons raw SVGs into the four new icon files and update mapping imports and data without changing card copy.
- [ ] Remove the three unused legacy icon files after the imports are updated.
- [ ] Run the focused checks and confirm the new contracts pass.

### Task 3: Harden the generated-output audit

- [ ] Resolve each local absolute reference against the build directory after removing the configured base path and query/hash fragments.
- [ ] Throw a clear `missing built file for local reference` error when the resolved target is absent, while preserving base-path and expected-stem checks.
- [ ] Run the full site verification and inspect the generated HTML for base-prefixed screenshot and icon URLs.

### Task 4: Browser and repository verification

- [ ] Refresh the local base route and inspect all `document.images` for `complete === true` and `naturalWidth > 0` at desktop and narrow viewport sizes.
- [ ] Run `git diff --check` and the repository's normal `just verify` gate because the change is site-only and should leave Rust behavior untouched.
- [ ] Review the final diff and record any unverified remote deployment state without claiming a push.
