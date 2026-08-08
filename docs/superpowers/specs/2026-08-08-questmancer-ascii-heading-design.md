# Questmancer pixel hero wordmark

Status: **Approved direction — written spec review requested**

Date: 2026-08-08

## Decision

Adopt the Varlock-inspired terminal heading treatment for the Questmancer Astro
launch page without changing the approved product copy or adding a second
semantic heading. The hero will open with an authored pixel wordmark that spells
**QUESTMANCER**, followed by the existing `h1`, “What is best in code?”, and its
Conan-inspired tagline. The wordmark is the hero's largest display element so
the product name establishes the visual hierarchy before the slogan. Press Start
2P is used for display headings, while DotGothic16 carries body and
terminal-adjacent copy, each with a local fallback if the hosted font is
unavailable.

The banner is a visual brand mark, not a replacement for the page heading. It
will remain static, package-free and base-path agnostic so the GitHub Pages
build continues to emit the same asset contract. DotGothic16 is a progressive
hosted display enhancement, not a build-time dependency.

## Goals

- Give the hero the same terminal-authored personality as the supplied Varlock
  reference.
- Spell `QUESTMANCER` in a readable 5x7 bitmap-letter mask with a solid cream
  face and discrete pixel falloff rows beneath it.
- Make the wordmark visibly larger than the semantic h1 on desktop while
  preserving a contained, readable two-line treatment at narrow mobile widths.
- Use Press Start 2P for display headings and DotGothic16 for body/terminal
  copy without changing the copy or adding a package dependency.
- Preserve exactly one semantic `h1` containing “What is best in code?”.
- Keep the complete hero artwork, tagline, CTA and screenshot order unchanged.
- Fit the banner within desktop, tablet and 375–390px mobile widths without
  horizontal scrolling or client-side resizing logic.
- Keep the site static and dependency-free beyond the already approved Astro
  toolchain.

## Non-goals

- Replacing the approved `What is best in code?` h1 or tagline.
- Adding a second accessible heading, animated canvas, bundled font asset or
  client-side script.
- Reproducing Varlock’s branding, copy or artwork.
- Changing the hero artwork framing, screenshot gallery, Pages workflow or
  `/herdr-questmancer/` base-path contract.

## Visual design

`HeroProof.astro` adds a decorative banner as the first child of `.hero-copy`:

```html
<div class="ascii-title" aria-hidden="true">QUESTMANCER</div>
```

The plain text in this sketch represents a fixed, authored bitmap wordmark. The
implementation renders each letter as a 5x7 grid of CSS pixels, with three
deterministic shadow rows made from sparse pixels below the main face. The top
face is high-contrast cream; the shadow rows use the existing muted/line tokens
and stepped opacity so the fade reads as intentional pixel falloff rather than
blur or noise. The banner will not contain a second text label or an interactive
element.

The existing semantic structure remains:

```html
<div class="ascii-title" aria-hidden="true">QUESTMANCER</div>
<h1 id="hero-title">What is best in code?</h1>
```

The h1 retains the current cream typography and responsive sizing. The pixel
wordmark uses a centered flex layout with a fluid pixel size and a max width
that is intentionally wider/taller than the h1 at desktop. Its wrapper clips
only the decorative banner's own overflow; the page itself must retain
`scrollWidth === clientWidth` at the mobile QA viewport. At narrow widths the
letters recompose into `QUEST` and `MANCER` rows so each pixel remains large
enough to read. Press Start 2P and DotGothic16 are loaded through the Google
Fonts stylesheet with `display=swap`; selectors retain system fallbacks so the
layout remains usable without the network fonts.

## Component and style changes

- `site/src/components/HeroProof.astro`
  - Add the fixed `QUESTMANCER` 5x7 bitmap mask and deterministic fade rows
    above the existing h1.
  - Keep `aria-hidden="true"` so screen readers announce the single h1 once.
  - Keep all existing hero copy, CTA links, artwork and screenshot markup in
    the same order after the new visual mark.
- `site/src/layouts/BaseLayout.astro`
  - Add the Google Fonts preconnect and Press Start 2P/DotGothic16 stylesheet
    link in the document head.
- `site/src/styles/global.css`
  - Add `.ascii-title` pixel-grid layout, color, fade and responsive sizing
    rules.
  - Apply `"Press Start 2P", ...` to display headings and
    `"DotGothic16", ...` to body/terminal copy while preserving code blocks.
  - Use the existing `--paper`, `--muted`, `--gold` and `--line` tokens.
  - Add narrow-width rules that recompose the two wordmark rows while
    preserving the complete mark and preventing horizontal page overflow.
- `site/scripts/content-contract.test.mjs`
  - Assert the built HTML includes the `ascii-title` marker and
    `QUESTMANCER` while retaining exactly one h1 and all existing approved
    copy assertions.

## Accessibility and responsive behavior

The banner is decorative because the following h1 provides the accessible
heading. `aria-hidden="true"` prevents duplicated speech output. The h1,
tagline, grounding copy, CTA buttons and artwork alt text remain unchanged.

At wide widths the banner is centered above the h1 with generous vertical
breathing room and a larger display footprint than the h1. At 375–390px it
recomposes to two contained wordmark rows and remains fully visible; no
horizontal page scrolling, clipped product copy or cropped hero artwork is
acceptable. If the hosted font has not loaded, fallback metrics must preserve
the same containment contract.

## Verification

1. Run `cd site && bun run check` and confirm zero Astro diagnostics.
2. Run `cd site && bun run build && bun run check:base && bun run check:content`.
3. Confirm the content contract reports one `h1`, the `QUESTMANCER` banner, the
   Press Start 2P/DotGothic16 stylesheet link and all approved copy.
4. Use the local base-prefixed preview at
   `/herdr-questmancer/` to review desktop and 375px mobile compositions.
5. Confirm `document.documentElement.scrollWidth` equals
   `document.documentElement.clientWidth` at mobile width and the full hero
   image remains visible inside its existing contain stage.
6. Run `git diff --check` and preserve any unrelated work already present in
   the checkout.

## Alternatives considered

1. **Authored CSS pixel grid plus hosted display/body fonts — selected.** Keeps
   the terminal-authored spirit, makes each letter readable at every breakpoint
   and gives the fade explicit pixel control without a package or bundled asset.
2. **Authored CSS pixel grid with a bundled font.** More deterministic offline,
   but adds a binary asset and license-notice maintenance to a launch page.
3. **Larger ASCII mask.** Faster, but remains font-dependent and cannot keep a
   strong visual hierarchy without becoming too wide on mobile.
4. **SVG pixel title.** Crisp at every scale, but adds a more polished logo
   treatment than the requested terminal heading and moves the art out of the
   HTML/CSS contract.
