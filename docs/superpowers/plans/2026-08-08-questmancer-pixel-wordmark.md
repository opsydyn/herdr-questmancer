# Questmancer display type and cursor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the unreadable custom ASCII banner and ship a simple Press Start 2P / DotGothic16 launch page with a small custom SVG cursor.

**Architecture:** Keep the existing semantic Astro hero and copy. Load both Google Fonts from the document head with fallbacks, apply display/body roles in the existing global stylesheet, and let Astro's asset pipeline emit one static SVG as the cursor. No JavaScript, canvas or package is needed.

**Tech Stack:** Astro, plain CSS, Google Fonts, one SVG asset, Node's built-in content contract and Bun scripts.

## Global Constraints

- Keep exactly one semantic `h1`, whose text remains “What is best in code?”.
- Remove the custom `ascii-title` markup/data and all pixel-grid CSS.
- Use Press Start 2P for display headings and DotGothic16 for body/terminal-adjacent copy.
- Keep command blocks on their compact monospace stack.
- Apply the custom cursor with a native fallback; never hide focus outlines or change keyboard behavior.
- Keep the hero artwork, tagline, CTA, screenshot order and contain framing unchanged.
- Add no package, JavaScript, canvas or client directive.
- Prevent horizontal page overflow at desktop, tablet and 375–390px mobile widths.
- Preserve `/herdr-questmancer/` base-path behavior and unrelated worktree changes.

---

## File Map

- Modify: `site/scripts/content-contract.test.mjs` — assert the simple heading/font/cursor contract.
- Modify: `site/src/components/HeroProof.astro` — remove the custom ASCII data and banner only.
- Modify: `site/src/layouts/BaseLayout.astro` — add the two-font stylesheet link.
- Modify: `site/src/styles/global.css` — apply font roles, remove pixel rules and apply the cursor.
- Create: `site/src/assets/cursor.svg` — authored 24px arrow cursor with glow and outline.

---

### Task 1: Replace pixel assertions with the simple content contract

**Files:**

- Modify: `site/scripts/content-contract.test.mjs`

**Interfaces:**

- Consumes: generated `site/dist/index.html` and source `site/src/styles/global.css`.
- Produces: one h1/copy check, an explicit absence check for `.ascii-title`, a two-font link check and a cursor rule check.

- [ ] **Step 1: Write the failing contract assertions.**

Replace the three pixel assertions with:

```js
assert.doesNotMatch(html, /ascii-title/);
assert.match(html, /fonts\.googleapis\.com\/css2\?family=Press\+Start\+2P.*family=DotGothic16/);
const css = await readFile(new URL('../src/styles/global.css', import.meta.url), 'utf8');
assert.match(css, /cursor:\s*url\("\.\.\/assets\/cursor\.svg"\)/);
```

Keep the one-h1 assertion, approved phrase loop, and Cargo-release absence
assertion unchanged.

- [ ] **Step 2: Run the focused contract and confirm the expected failure.**

Run:

```bash
cd site
bun run build
bun run check:content
```

Expected result: the test fails because the current build still contains the
custom `ascii-title` and the source CSS does not yet reference `cursor.svg`.

---

### Task 2: Remove the custom ASCII banner and add the cursor asset

**Files:**

- Modify: `site/src/components/HeroProof.astro`
- Create: `site/src/assets/cursor.svg`

**Interfaces:**

- Consumes: the existing hero copy and image imports.
- Produces: the existing eyebrow followed directly by the unchanged semantic h1, plus a base-root public cursor asset.

- [ ] **Step 1: Remove the bitmap frontmatter and banner markup.**

Delete `glyphs`, `titleLines`, `fadeRows` and the entire `.ascii-title` block
from `HeroProof.astro`. The hero copy should begin:

```astro
<p class="eyebrow"><span aria-hidden="true">//</span> Herdr plugin · Questmancer</p>
<h1 id="hero-title">What is best in code?</h1>
```

Leave the tagline, grounding copy, actions, artwork and screenshot markup
unchanged.

- [ ] **Step 2: Add the authored cursor SVG.**

Create `site/src/assets/cursor.svg` with this 24px arrow:

```svg
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24">
  <defs>
    <filter id="glow" x="-80%" y="-80%" width="260%" height="260%">
      <feGaussianBlur stdDeviation="1.7" />
    </filter>
  </defs>
  <path d="M3 2 20 16H12L8 22Z" fill="#67b5aa" opacity=".75" filter="url(#glow)" />
  <path d="M3 2 20 16H12L8 22Z" fill="#0b0c10" stroke="#f8e8c1" stroke-width="1.4" stroke-linejoin="round" />
</svg>
```

The cursor remains decorative and has no scripting or interaction state.

---

### Task 3: Load the fonts and simplify the global stylesheet

**Files:**

- Modify: `site/src/layouts/BaseLayout.astro`
- Modify: `site/src/styles/global.css`

**Interfaces:**

- Consumes: the existing layout and global tokens.
- Produces: Press Start 2P display headings, DotGothic16 body/terminal-adjacent copy, compact monospace code blocks and a base-safe custom cursor.

- [ ] **Step 1: Load both fonts in the document head.**

Inside `<head>` before `<title>`, add:

```astro
<link rel="preconnect" href="https://fonts.googleapis.com" />
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
<link
  href="https://fonts.googleapis.com/css2?family=Press+Start+2P&family=DotGothic16&display=swap"
  rel="stylesheet"
/>
```

- [ ] **Step 2: Apply the font roles and custom cursor.**

Keep body text on DotGothic16 with a system monospace fallback:

```css
body {
  font-family: "DotGothic16", ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  cursor: url("../assets/cursor.svg") 3 3, auto;
}

a,
button,
[role="button"] {
  cursor: url("../assets/cursor.svg") 3 3, pointer;
}

h1,
h2,
h3,
.wordmark,
.footer-tagline {
  font-family: "Press Start 2P", ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
}
```

Change explicit eyebrow/section-kicker and terminal-bar families to DotGothic16
so they follow body/terminal copy. Keep `pre` code blocks on the existing
compact monospace stack.

- [ ] **Step 3: Delete all pixel-mask CSS and stale narrow-width rules.**

Remove `.ascii-title`, `.ascii-title__wordmark`, `.ascii-title__line`,
`.ascii-title__glyph`, `.ascii-title__pixel*` and the mobile `.ascii-title`
block. Preserve the existing hero, artwork and mobile layout rules.

---

### Task 4: Verify the simple launch page and commit

**Files:**

- Verify: `site/src/components/HeroProof.astro`
- Verify: `site/src/layouts/BaseLayout.astro`
- Verify: `site/src/styles/global.css`
- Verify: `site/src/assets/cursor.svg`
- Verify: `site/scripts/content-contract.test.mjs`

- [ ] **Step 1: Run focused site verification.**

Run:

```bash
cd site
bun run verify
```

Expected: Astro reports zero errors/warnings/hints, the static build succeeds,
the base audit passes and the content contract passes with no ASCII banner.

- [ ] **Step 2: Review desktop and mobile preview behavior.**

Use the local base-prefixed preview at `http://127.0.0.1:4321/herdr-questmancer/`.
Confirm the h1 is Press Start 2P, body/copy is DotGothic16, the hero image and
tagline remain in their approved order, and the cursor asset appears on the
page/links. At a 375px viewport confirm `scrollWidth === clientWidth`.

- [ ] **Step 3: Run hygiene and commit.**

From the repository root:

```bash
git diff --check
git add site/src/components/HeroProof.astro site/src/layouts/BaseLayout.astro site/src/styles/global.css site/scripts/content-contract.test.mjs site/src/assets/cursor.svg
git commit -m "feat: simplify Questmancer launch typography"
```
