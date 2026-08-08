# Questmancer ASCII hero heading Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended; not used here because inline execution is approved) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a responsive Varlock-inspired ASCII `QUESTMANCER` visual banner above the existing Questmancer hero heading.

**Architecture:** Keep the banner as static Astro markup in `HeroProof.astro`,
with no image, font, script or runtime dependency. Render the semantic
“What is best in code?” `h1` immediately below it, and use scoped CSS rows for
the dense-to-faded block-character treatment.

**Tech Stack:** Astro, TypeScript checking, plain CSS, Node’s built-in test
runner and the existing Bun scripts under `site/`.

## Global Constraints

- Spell the visual banner `QUESTMANCER` using the fixed seven-row mask below.
- Keep exactly one semantic `h1`, whose text remains “What is best in code?”.
- Mark the visual banner `aria-hidden="true"`; do not add a second accessible heading.
- Use only the existing `--paper`, `--muted`, `--gold` and `--line` CSS tokens.
- Add no package, image, font, JavaScript, canvas or client directive.
- Keep the existing hero artwork, tagline, CTA, screenshot order and contain framing unchanged.
- Prevent horizontal page overflow at desktop, tablet and 375–390px mobile widths.
- Preserve `/herdr-questmancer/` base-path behavior and unrelated worktree changes.

---

## File Map

Modify these existing site files only:

- `site/src/components/HeroProof.astro` — add the decorative ASCII rows before the existing h1.
- `site/src/styles/global.css` — add banner typography, tone rows, fade and responsive rules.
- `site/scripts/content-contract.test.mjs` — assert the built banner marker while preserving the existing copy contract.

## Task 1: Extend the content contract with a failing banner assertion

**Files:**

- Modify: `site/scripts/content-contract.test.mjs`

**Interfaces:**

- Consumes: generated `site/dist/index.html`.
- Produces: a test that requires the `ascii-title` marker, `data-title="QUESTMANCER"`, `aria-hidden="true"`, exactly one h1 and all existing approved phrases.

- [ ] **Step 1: Add the banner assertions.**

After the existing one-h1 assertion, add:

```js
assert.match(html, /class="ascii-title"[^>]*aria-hidden="true"/);
assert.match(html, /data-title="QUESTMANCER"/);
```

Keep the existing phrase loop and Cargo-release absence assertion unchanged.

- [ ] **Step 2: Run the focused contract and confirm the expected failure.**

Run:

```bash
cd site
bun run build
bun run check:content
```

Expected result: the test fails because the current page has no `.ascii-title`
or `data-title="QUESTMANCER"` banner yet.

## Task 2: Implement the static ASCII banner and responsive CSS

**Files:**

- Modify: `site/src/components/HeroProof.astro`
- Modify: `site/src/styles/global.css`

**Interfaces:**

- Consumes: the approved hero copy and current `.hero-copy` layout.
- Produces: a decorative `<pre class="ascii-title" data-title="QUESTMANCER" aria-hidden="true">` immediately before the existing h1.

- [ ] **Step 1: Add the exact seven-row mask in HeroProof.astro.**

Define this frontmatter constant before the component markup:

```js
const asciiRows = [
  ' ███  █   █ █████  ████ █████ █   █  ███  █   █  ████ █████ ████ ',
  '▓   ▓ ▓   ▓ ▓     ▓       ▓   ▓▓ ▓▓ ▓   ▓ ▓▓  ▓ ▓     ▓     ▓   ▓',
  '▓   ▓ ▓   ▓ ▓     ▓       ▓   ▓ ▓ ▓ ▓   ▓ ▓ ▓ ▓ ▓     ▓     ▓   ▓',
  '▓   ▓ ▓   ▓ ▓▓▓▓   ▓██    ▓   ▓ ▓ ▓ █████ ▓  ██ ▓     ▓███  ▓███ ',
  '▒ ▒ ▒ ▒   ▒ ▒         ▒   ▒   ▒   ▒ ▒   ▒ ▒   ▒ ▒     ▒     ▒ ▒  ',
  '▒  ▒  ▒   ▒ ▒         ▒   ▒   ▒   ▒ ▒   ▒ ▒   ▒ ▒     ▒     ▒  ▒ ',
  '░░ ░  ░░░  ░░░░░ ░░░░    ░   ░   ░ ░   ░ ░   ░  ░░░░ ░░░░░ ░   ░ ',
];
```

Render it before the existing h1 without changing the h1 text:

```astro
<pre class="ascii-title" data-title="QUESTMANCER" aria-hidden="true">
  {asciiRows.map((row, index) => <span class:list={["ascii-title__row", `ascii-title__row--${index}`]}>{row}</span>)}
</pre>
<h1 id="hero-title">What is best in code?</h1>
```

The banner remains decorative and has no link, event handler or second heading.

- [ ] **Step 2: Add the desktop banner styles.**

Place these rules before the existing `h1` rule in `global.css`:

```css
.ascii-title {
  max-width: 100%;
  margin: 0 auto 1.2rem;
  overflow: hidden;
  color: var(--paper);
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: clamp(0.25rem, 1.1vw, 0.75rem);
  line-height: 0.96;
  letter-spacing: 0.02em;
  text-align: center;
  white-space: pre;
}

.ascii-title__row {
  display: block;
  min-height: 1em;
}

.ascii-title__row--4,
.ascii-title__row--5 {
  color: var(--muted);
  opacity: 0.72;
}

.ascii-title__row--6 {
  color: var(--line);
  opacity: 0.62;
}
```

The `overflow: hidden` applies only to the decorative pre; it must not create
horizontal overflow on `html`, `body` or `.hero`.

- [ ] **Step 3: Add the mobile sizing rule.**

Inside the existing `@media (max-width: 640px)` block, add:

```css
.ascii-title {
  margin-bottom: 0.9rem;
  font-size: clamp(0.25rem, 1.1vw, 0.42rem);
  letter-spacing: 0;
}
```

The seven rows must remain visible at 375px; do not use a horizontal scroll
container or CSS transform that changes the document width.

## Task 3: Verify visual behavior and commit the implementation

**Files:**

- Verify: `site/src/components/HeroProof.astro`
- Verify: `site/src/styles/global.css`
- Verify: `site/scripts/content-contract.test.mjs`

**Interfaces:**

- Consumes: the static banner and existing Astro site verification scripts.
- Produces: a passing content contract, base audit and responsive preview.

- [ ] **Step 1: Run the focused site checks.**

Run:

```bash
cd site
bun run check
bun run build
bun run check:base
bun run check:content
```

Expected: Astro reports zero diagnostics; the base audit passes; the content
contract reports one h1 and finds the banner marker plus all approved copy.

- [ ] **Step 2: Review desktop and mobile preview behavior.**

Start the existing production preview with `bun run preview --host 127.0.0.1`
and inspect `/herdr-questmancer/`. At the default desktop viewport, confirm the
light-gray `QUESTMANCER` mask sits above the h1 and its lower rows fade without
covering the tagline. At a 375px viewport, confirm the complete word remains
visible, `scrollWidth === clientWidth`, the h1/tagline/CTA remain readable, and
the hero artwork remains fully contained.

- [ ] **Step 3: Run final hygiene checks.**

From the repository root, run:

```bash
git diff --check
git status --short --branch
```

Confirm only the intended site files and this feature’s documentation are
changed; preserve unrelated release-plz or Rust work.

- [ ] **Step 4: Commit the implementation.**

```bash
git add site/src/components/HeroProof.astro site/src/styles/global.css site/scripts/content-contract.test.mjs
git commit -m "feat: add Questmancer ASCII hero heading"
```
