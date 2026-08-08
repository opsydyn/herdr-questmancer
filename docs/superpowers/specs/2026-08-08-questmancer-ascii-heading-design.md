# Questmancer display type and cursor

Status: **Approved direction — implementation requested**

Date: 2026-08-08

## Decision

Keep the Questmancer launch page visually simple. Remove the custom
`QUESTMANCER` ASCII/bitmap banner and use the existing semantic `h1`, “What is
best in code?”, as the hero heading. Load Press Start 2P for headings and
DotGothic16 for body and terminal-adjacent copy through one Google Fonts
stylesheet link, with system fallbacks. Add a small authored SVG cursor with a
dark arrow, light outline and blue glow.

The page remains static, dependency-light and base-path safe. The cursor is a
decorative interaction cue, not a replacement for native focus indicators or
keyboard interaction.

## Goals

- Make the hero readable without a font-dependent ASCII art block.
- Use Press Start 2P for `h1`, section headings, the site wordmark and footer
  display copy.
- Use DotGothic16 for body copy, navigation, labels, tagline, buttons and
  captions while leaving command blocks on a compact monospace stack.
- Add a pixel-era cursor that works on the base-prefixed GitHub Pages route.
- Preserve exactly one semantic `h1` containing “What is best in code?”.
- Keep the hero artwork, tagline, CTA, screenshots and artwork framing unchanged.
- Keep the site free of JavaScript and package dependencies for the font/cursor
  treatment.

## Non-goals

- Reintroducing a second visual heading, ASCII mask, canvas or client script.
- Replacing the approved Conan-inspired copy or the hero artwork.
- Hiding the native cursor, removing focus outlines or changing keyboard UX.
- Changing the Pages workflow or `/herdr-questmancer/` asset contract.

## Visual design

The hero structure remains:

```html
<p class="eyebrow">// Herdr plugin · Questmancer</p>
<h1 id="hero-title">What is best in code?</h1>
<p class="hero-tagline">To crush your bugs.<br />...</p>
```

Press Start 2P gives the heading and display labels a chunky arcade rhythm.
DotGothic16 gives supporting copy a compact dotted-terminal texture without
forcing long paragraphs into the display face. `display=swap` keeps fallback
text visible while the hosted fonts load.

The cursor is `site/src/assets/cursor.svg`: a 24px arrow with a dark fill,
cream outline and restrained teal-blue glow. CSS applies it to the document and
interactive links/buttons with a `3 3` hotspot and falls back to the platform
cursor when custom cursors are unsupported. Astro's asset pipeline emits the
hashed, base-prefixed URL.

## Component and style changes

- `site/src/components/HeroProof.astro`
  - Remove the ASCII/bitmap frontmatter data and decorative banner.
  - Leave the existing eyebrow, one h1, tagline, grounding copy, actions and
    proof markup in their current order.
- `site/src/layouts/BaseLayout.astro`
  - Add preconnect hints and one Google Fonts stylesheet link for Press Start 2P
    and DotGothic16.
- `site/src/styles/global.css`
  - Remove the `.ascii-title` and pixel-cell rules.
  - Set DotGothic16 as the body/terminal-adjacent family and Press Start 2P on
    display headings.
  - Apply the asset-pipeline SVG cursor with a native fallback.
- `site/src/assets/cursor.svg`
  - Add the small authored cursor asset.
- `site/scripts/content-contract.test.mjs`
  - Assert the built page has no ASCII banner, retains one h1 and includes the
    two-font stylesheet link; assert the source CSS applies the cursor asset.

## Accessibility and responsive behavior

The semantic h1 remains the only heading required by the hero. The cursor does
not affect focus, hit targets or keyboard operation. At desktop and 375–390px
mobile widths the h1 and body copy must remain contained with
`scrollWidth === clientWidth`; if either hosted font is unavailable, fallback
metrics must preserve that contract.

## Verification

1. Run `cd site && bun run verify` and confirm zero Astro diagnostics, a passing
   base audit and a passing content contract.
2. Use the local base-prefixed preview at `/herdr-questmancer/` to inspect the
   heading and cursor at desktop and 375px mobile widths.
3. Confirm there is no `.ascii-title` in generated HTML, exactly one h1, the
   font stylesheet link and the cursor source rule.
4. Confirm `document.documentElement.scrollWidth` equals
   `document.documentElement.clientWidth` at mobile width.
5. Run `git diff --check` and preserve unrelated Rust/release work.

## Alternatives considered

1. **Plain semantic heading plus two hosted pixel fonts and SVG cursor —
   selected.** Smallest readable implementation and closest to the requested
   simplified direction.
2. **Custom ASCII/bitmap wordmark.** More authored, but the supplied review
   showed it became too large and difficult to read across breakpoints.
3. **CSS-only cursor data URI.** Avoids a file, but is harder to inspect and
   tune than the tiny authored SVG asset.
