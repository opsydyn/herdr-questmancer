# Questmancer ASCII hero heading

Status: **Approved direction — written spec review requested**

Date: 2026-08-08

## Decision

Adopt the Varlock-inspired terminal heading treatment for the Questmancer Astro
launch page without changing the approved product copy or adding a second
semantic heading. The hero will open with an authored block-character banner
that spells **QUESTMANCER**, followed by the existing `h1`, “What is best in
code?”, and its Conan-inspired tagline.

The banner is a visual brand mark, not a replacement for the page heading. It
will remain static, dependency-free and base-path agnostic so the GitHub Pages
build continues to emit the same asset contract.

## Goals

- Give the hero the same terminal-authored personality as the supplied Varlock
  reference.
- Spell `QUESTMANCER` in a dense block-letter mask using `█`, `▓`, `▒` and `░`
  characters, with a deliberate lighter fade through the lower rows.
- Preserve exactly one semantic `h1` containing “What is best in code?”.
- Keep the complete hero artwork, tagline, CTA and screenshot order unchanged.
- Fit the banner within desktop, tablet and 375–390px mobile widths without
  horizontal scrolling or client-side resizing logic.
- Keep the site static and dependency-free beyond the already approved Astro
  toolchain.

## Non-goals

- Replacing the approved `What is best in code?` h1 or tagline.
- Adding a second accessible heading, animated canvas, webfont, image asset or
  client-side script.
- Reproducing Varlock’s branding, copy or artwork.
- Changing the hero artwork framing, screenshot gallery, Pages workflow or
  `/herdr-questmancer/` base-path contract.

## Visual design

`HeroProof.astro` adds a decorative banner as the first child of `.hero-copy`:

```html
<pre class="ascii-title" aria-hidden="true">QUESTMANCER</pre>
```

The plain text in this sketch represents a fixed, authored multi-row
block-letter mask. The top rows use the dense `█`/`▓`
characters for a strong light-gray title. Lower rows use `▒`/`░` and a scoped
opacity fade to create the reference’s pixel-drip/glitch tail. The banner will
not contain a second text label or an interactive element.

The existing semantic structure remains:

```html
<pre class="ascii-title" aria-hidden="true">QUESTMANCER</pre>
<h1 id="hero-title">What is best in code?</h1>
```

The h1 retains the current cream typography and responsive sizing. The ASCII
banner uses a monospace stack, `white-space: pre`, a centered max width and a
fluid `font-size` clamp. Its wrapper clips only the decorative banner’s own
overflow; the page itself must retain `scrollWidth === clientWidth` at the
mobile QA viewport.

## Component and style changes

- `site/src/components/HeroProof.astro`
  - Add the fixed `QUESTMANCER` block mask above the existing h1.
  - Keep `aria-hidden="true"` so screen readers announce the single h1 once.
  - Keep all existing hero copy, CTA links, artwork and screenshot markup in
    the same order after the new visual mark.
- `site/src/styles/global.css`
  - Add `.ascii-title` layout, color, line-height and responsive sizing rules.
  - Use the existing `--paper`, `--muted`, `--gold` and `--line` tokens; do not
    introduce a new font request or visual dependency.
  - Add a narrow-width rule that reduces the mask’s character size while
    preserving the complete word and preventing horizontal page overflow.
- `site/scripts/content-contract.test.mjs`
  - Assert the built HTML includes the `ascii-title` marker and
    `QUESTMANCER` while retaining exactly one h1 and all existing approved
    copy assertions.

## Accessibility and responsive behavior

The banner is decorative because the following h1 provides the accessible
heading. `aria-hidden="true"` prevents duplicated speech output. The h1,
tagline, grounding copy, CTA buttons and artwork alt text remain unchanged.

At wide widths the banner is centered above the h1 with generous vertical
breathing room. At 375–390px it scales down through CSS and remains fully
visible; no horizontal page scrolling, clipped product copy or cropped hero
artwork is acceptable.

## Verification

1. Run `cd site && bun run check` and confirm zero Astro diagnostics.
2. Run `cd site && bun run build && bun run check:base && bun run check:content`.
3. Confirm the content contract reports one `h1`, the `QUESTMANCER` banner and
   all approved copy.
4. Use the local base-prefixed preview at
   `/herdr-questmancer/` to review desktop and 375px mobile compositions.
5. Confirm `document.documentElement.scrollWidth` equals
   `document.documentElement.clientWidth` at mobile width and the full hero
   image remains visible inside its existing contain stage.
6. Run `git diff --check` and preserve any unrelated work already present in
   the checkout.

## Alternatives considered

1. **Static ASCII banner — selected.** Closest to the Varlock reference,
   deterministic in a static build and requires no new dependency or asset.
2. **CSS-only block text.** More fluid but loses the authored terminal mask and
   shaded drip character that motivated the request.
3. **SVG pixel title.** Crisp at every scale, but adds an asset and creates a
   more polished logo treatment than the requested terminal heading.
