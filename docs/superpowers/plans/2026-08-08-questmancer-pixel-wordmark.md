# Questmancer pixel hero wordmark Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the unreadable font-dependent `QUESTMANCER` mask with a large, readable CSS bitmap wordmark whose lower pixels fade like a deliberate terminal shadow.

**Architecture:** Keep the wordmark as static Astro markup generated from authored 5x7 glyph data. Each glyph renders a 5-column pixel grid for its solid face plus three deterministic sparse rows for the fade. CSS controls the pixel size, tone and desktop/mobile composition; the existing semantic h1 and all product copy remain unchanged.

**Tech Stack:** Astro, plain CSS, Google Fonts (Press Start 2P + DotGothic16), Node's built-in content contract, Bun scripts. No new package, image, client directive or runtime code.

## Global Constraints

- Keep exactly one semantic `h1`, whose text remains “What is best in code?”.
- Mark the visual wordmark `aria-hidden="true"`; do not add a second accessible heading.
- Use the existing `--paper`, `--muted`, `--gold` and `--line` CSS tokens.
- Keep the wordmark larger than the h1 at desktop widths and recompose it as `QUEST` / `MANCER` rows on narrow mobile widths.
- Keep the complete hero artwork, tagline, CTA, screenshot order and contain framing unchanged.
- Add no package, bundled font asset, image, JavaScript, canvas or client directive; load the two approved display fonts through one Google Fonts stylesheet link with fallbacks.
- Prevent horizontal page overflow at desktop, tablet and 375–390px mobile widths.
- Preserve `/herdr-questmancer/` base-path behavior and unrelated worktree changes.

---

## File Map

Modify these existing files only:

- `site/scripts/content-contract.test.mjs` — require the decorative pixel-grid markers in the built page while retaining the existing copy contract.
- `site/src/components/HeroProof.astro` — replace the current seven-row text mask with authored glyph data and static pixel spans.
- `site/src/layouts/BaseLayout.astro` — load Press Start 2P and DotGothic16 with preconnect hints.
- `site/src/styles/global.css` — style the pixel grid, stepped fade, responsive wordmark and font roles.

---

### Task 1: Extend the content contract with pixel-grid assertions

**Files:**

- Modify: `site/scripts/content-contract.test.mjs`

**Interfaces:**

- Consumes: generated `site/dist/index.html`.
- Produces: assertions for `data-title="QUESTMANCER"`, `aria-hidden="true"`, an on-pixel class and a fade-row class, alongside the existing one-h1 and approved-copy checks.

- [ ] **Step 1: Add assertions that describe the new visual contract.**

Immediately after the existing one-h1 assertion, add:

```js
assert.match(html, /class="ascii-title"[^>]*data-title="QUESTMANCER"[^>]*aria-hidden="true"/);
assert.match(html, /class="ascii-title__pixel ascii-title__pixel--on"/);
assert.match(html, /class="ascii-title__pixel-row ascii-title__pixel-row--fade/);
assert.match(html, /fonts\.googleapis\.com\/css2\?family=Press\+Start\+2P.*family=DotGothic16/);
```

Keep the existing phrase loop and Cargo-release absence assertion unchanged.

- [ ] **Step 2: Run the focused contract and confirm the expected failure.**

Run:

```bash
cd site
bun run build
bun run check:content
```

Expected result: the contract fails because the current page has the old text
mask and no pixel-cell or fade-row classes.

---

### Task 2: Render an authored 5x7 bitmap wordmark in Astro

**Files:**

- Modify: `site/src/components/HeroProof.astro`

**Interfaces:**

- Consumes: the existing hero copy and `githubUrl`/image imports.
- Produces: a decorative `.ascii-title[data-title="QUESTMANCER"]` containing two `.ascii-title__line` groups, 5x7 glyph face pixels and three generated fade rows before the unchanged h1.

- [ ] **Step 1: Replace the old `asciiRows` constant with the glyph map and fade helper.**

Use this authored 5-column glyph map in the frontmatter:

```js
const glyphs = {
  Q: ['01110', '10001', '10001', '10001', '10101', '10010', '01101'],
  U: ['10001', '10001', '10001', '10001', '10001', '10001', '01110'],
  E: ['11111', '10000', '10000', '11110', '10000', '10000', '11111'],
  S: ['01111', '10000', '10000', '01110', '00001', '00001', '11110'],
  T: ['11111', '00100', '00100', '00100', '00100', '00100', '00100'],
  M: ['10001', '11011', '10101', '10101', '10001', '10001', '10001'],
  A: ['01110', '10001', '10001', '11111', '10001', '10001', '10001'],
  N: ['10001', '11001', '10101', '10011', '10001', '10001', '10001'],
  C: ['01111', '10000', '10000', '10000', '10000', '10000', '01111'],
  R: ['11110', '10001', '10001', '11110', '10100', '10010', '10001'],
};

const titleLines = ['QUEST', 'MANCER'];

const fadeRows = (rows, glyphIndex) =>
  [0, 1, 2].map((depth) => {
    const source = rows[rows.length - 1 - (depth % 3)];
    const divisor = depth + 2;
    return [...source]
      .map((pixel, column) =>
        pixel === '1' && (column + glyphIndex * 2 + depth) % divisor === 0 ? '1' : '0',
      )
      .join('');
  });
```

The helper is deterministic and only uses the authored bottom rows, so the
fade stays stable between builds.

- [ ] **Step 2: Replace the old `<pre>` rows with static pixel spans.**

Immediately before the existing h1, render:

```astro
<div class="ascii-title" data-title="QUESTMANCER" aria-hidden="true">
  <div class="ascii-title__wordmark">
    {titleLines.map((line) => (
      <div class="ascii-title__line">
        {[...line].map((letter, glyphIndex) => {
          const rows = glyphs[letter];
          const rowsWithFade = [...rows, ...fadeRows(rows, glyphIndex)];

          return (
            <span class="ascii-title__glyph" data-letter={letter}>
              {rowsWithFade.map((row, rowIndex) => (
                <span class:list={[
                  'ascii-title__pixel-row',
                  rowIndex >= rows.length && `ascii-title__pixel-row--fade-${rowIndex - rows.length}`,
                ]}>
                  {[...row].map((pixel) => (
                    <span class:list={['ascii-title__pixel', pixel === '1' && 'ascii-title__pixel--on']} />
                  ))}
                </span>
              ))}
            </span>
          );
        })}
      </div>
    ))}
  </div>
</div>
<h1 id="hero-title">What is best in code?</h1>
```

Keep the h1 text, tagline, grounding copy, actions and proof markup byte-for-
byte unchanged after this insertion. The parent `aria-hidden` makes all pixel
spans decorative.

---

### Task 3: Style the large solid face and stepped pixel fade

**Files:**

- Modify: `site/src/layouts/BaseLayout.astro`
- Modify: `site/src/styles/global.css`

**Interfaces:**

- Consumes: `.ascii-title`, `.ascii-title__line`, `.ascii-title__glyph`, `.ascii-title__pixel-row--fade-*` and `.ascii-title__pixel--on` emitted by `HeroProof.astro`.
- Produces: a centered wordmark that is larger than the h1 at desktop, recomposes into two readable rows on mobile and uses Press Start 2P for display text with DotGothic16 for body/terminal copy.

- [ ] **Step 1: Load the approved display and body fonts.**

In `site/src/layouts/BaseLayout.astro`, add these two preconnect hints and the
stylesheet link inside `<head>` before the page title:

```astro
<link rel="preconnect" href="https://fonts.googleapis.com" />
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
<link
  href="https://fonts.googleapis.com/css2?family=Press+Start+2P&family=DotGothic16&display=swap"
  rel="stylesheet"
/>
```

The Google Fonts request is progressive enhancement; every selector below
retains a system fallback.

- [ ] **Step 2: Replace the old text-mask rules with pixel-grid rules.**

Place this block before the existing `h1` rule:

```css
.ascii-title {
  --pixel-size: clamp(0.34rem, 1.25vw, 1.08rem);
  --pixel-gap: clamp(1px, 0.18vw, 3px);
  width: 100%;
  max-width: 1160px;
  margin: 0 auto 1.05rem;
  overflow: hidden;
}

.ascii-title__wordmark {
  display: flex;
  justify-content: center;
  gap: clamp(0.65rem, 1.45vw, 1.25rem);
}

.ascii-title__line {
  display: contents;
}

.ascii-title__glyph {
  display: grid;
  grid-template-rows: repeat(10, var(--pixel-size));
  gap: var(--pixel-gap);
}

.ascii-title__pixel-row {
  display: grid;
  grid-template-columns: repeat(5, var(--pixel-size));
  gap: var(--pixel-gap);
}

.ascii-title__pixel {
  width: var(--pixel-size);
  height: var(--pixel-size);
}

.ascii-title__pixel--on {
  background: var(--paper);
}

.ascii-title__pixel-row--fade-0 .ascii-title__pixel--on {
  background: var(--muted);
  opacity: 0.85;
}

.ascii-title__pixel-row--fade-1 .ascii-title__pixel--on {
  background: var(--line);
  opacity: 0.58;
}

.ascii-title__pixel-row--fade-2 .ascii-title__pixel--on {
  background: var(--line);
  opacity: 0.3;
}
```

Remove the old monospace/pre rules and row-specific opacity selectors so no
font-dependent text styling competes with the pixel cells.

- [ ] **Step 3: Add the display/body font roles and narrow-width two-line composition.**

Inside the existing `@media (max-width: 640px)` block, add:

```css
.ascii-title {
  --pixel-size: clamp(0.52rem, 3.1vw, 0.7rem);
  --pixel-gap: 1px;
  margin-bottom: 0.85rem;
}

.ascii-title__wordmark {
  flex-direction: column;
  align-items: center;
  gap: 0.6rem;
}

.ascii-title__line {
  display: flex;
  gap: 0.28rem;
}
```

Add these font-role selectors alongside the existing heading rules:

```css
body {
  font-family: "DotGothic16", ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
}

h1,
h2,
h3,
.wordmark,
.hero-tagline,
.button,
.footer-tagline {
  font-family: "Press Start 2P", ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
}
```

Keep `pre`, terminal bars and other code surfaces on their existing monospace
stack so command text remains compact and legible.

Do not add a page-level overflow rule or horizontal scrolling container.

---

### Task 4: Verify visual behavior and commit the implementation

**Files:**

- Verify: `site/src/components/HeroProof.astro`
- Verify: `site/src/styles/global.css`
- Verify: `site/scripts/content-contract.test.mjs`

- [ ] **Step 1: Run the focused site checks.**

Run:

```bash
cd site
bun run verify
```

Expected: Astro reports zero diagnostics, the page builds, the base audit
passes and the content contract reports one h1 plus the pixel/fade markers.

- [ ] **Step 2: Review desktop and mobile preview behavior.**

Use the local base-prefixed preview at `http://127.0.0.1:4321/herdr-questmancer/`.
At desktop, confirm the cream `QUESTMANCER` wordmark is visibly larger than
“What is best in code?”, every letter is readable and the lower three rows fade
as discrete pixels. At a 375px viewport, confirm `QUEST` and `MANCER` remain
contained and readable, `scrollWidth === clientWidth`, the h1/tagline/actions
remain readable and the hero art remains fully contained.

- [ ] **Step 3: Run final hygiene checks.**

From the repository root, run:

```bash
git diff --check
git status --short --branch
```

Preserve unrelated release-plz or Rust work.

- [ ] **Step 4: Commit the implementation.**

```bash
git add site/src/components/HeroProof.astro site/src/styles/global.css site/scripts/content-contract.test.mjs
git commit -m "feat: make Questmancer wordmark pixel readable"
```
