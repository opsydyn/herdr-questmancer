# Questmancer Astro launch page Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox syntax for tracking.

**Goal:** Build and deploy a single-page Astro launch site for Questmancer at
https://opsydyn.github.io/herdr-questmancer/ with a proof-first pixel-art hero,
source-install workbench, screenshot evidence and base-path-safe assets.

**Architecture:** Keep the Astro project self-contained under site/ and leave
the Rust plugin root unchanged. Compose one static page from focused Astro
components, import all images from site/src/assets, vendor a small set of
Pixelarticons SVGs, and run a Node build-output audit before Pages deployment.
GitHub Actions builds site/ with the official Astro action and publishes the
static dist/ artifact.

**Tech Stack:** Astro, TypeScript for Astro checking, Bun, plain CSS, vendored
Pixelarticons SVG assets, GitHub Pages, withastro/action, and Node’s built-in
test runner.

## Global Constraints

- Work in /Users/alancurrie/Projects/herdr-web-master; preserve unrelated Rust changes and root screenshots.
- Put all website source under site/; do not modify the Rust renderer, Herdr protocol, or runtime behavior.
- Target https://opsydyn.github.io/herdr-questmancer/ with site: 'https://opsydyn.github.io' and base: '/herdr-questmancer'.
- Use one static Astro route, plain Astro components and CSS; add no UI framework, remote font, telemetry, form, Herdr connection or client state.
- Use the exact approved hero copy: “What is best in code?”, “To crush your bugs.”, “See your agents driven before you.”, and “Hear the lamentations of your token budget.”
- Make View on GitHub point to https://github.com/opsydyn/herdr-questmancer and do not claim a Cargo release.
- Keep one heading for “What is best in code?”; the image has only the “Proof of the Delve” section label and no duplicate marketing text.
- Fit the full hero artwork inside a padded black stage with contain-style fitting; never crop, smooth or filter the main image.
- Treat object-fit: cover as acceptable only for secondary screenshot thumbnails where a captioned crop remains legible.
- Use the supplied hero and root screenshots as committed site assets; the build must not depend on files remaining in Downloads or temporary directories.
- Use Pixelarticons only as a small set of raw SVG assets with upstream MIT attribution; do not ship an icon runtime.
- Every local asset and internal URL must resolve beneath /herdr-questmancer/; no handwritten root-absolute /assets/ or /icons/ paths.
- Run focused site checks before the repository-wide gate and report unrelated Rust failures separately.

---

## File map

Create the following site-owned files:

~~~
site/
  astro.config.mjs                         # site/base and static Astro config
  package.json                             # scripts and minimal dependencies
  bun.lock                                 # committed dependency resolution
  THIRD_PARTY_NOTICES                      # Pixelarticons MIT attribution
  scripts/
    check-base-path.mjs                    # built-output URL and asset audit
    check-base-path.test.mjs               # audit pass/fail fixtures
    content-contract.test.mjs              # exact copy, heading and CTA checks
  src/
    assets/
      hero.png
      screenshots/
        guild-hall.png
        delve.png
        compact-party.png
        compact-party-warm.png
        ledger.png
        adventurer.png
      icons/
        home.svg
        bell.svg
        heart.svg
        lock.svg
    components/
      SiteHeader.astro
      HeroProof.astro
      InstallWorkbench.astro
      EvidenceGallery.astro
      MappingCards.astro
      SiteFooter.astro
    layouts/
      BaseLayout.astro
    pages/
      index.astro
    styles/
      global.css
.github/workflows/deploy-site.yml          # Pages workflow; Rust workflows unchanged
~~~

The two test files are plain Node ESM tests. check-base-path.mjs exports
validateBuild(buildDir, basePath, expectedAssetStems) for the tests and runs
the same function from its CLI entry point against site/dist/.

---

### Task 1: Scaffold Astro and make the base-path audit executable

**Files:**
- Modify: .gitignore
- Create: site/package.json
- Create: site/astro.config.mjs
- Create: site/src/pages/index.astro
- Create: site/src/layouts/BaseLayout.astro
- Create: site/scripts/check-base-path.mjs
- Create: site/scripts/check-base-path.test.mjs

**Interfaces:**
- Produces validateBuild(buildDir: string, basePath: string, expectedAssetStems: string[]): void.
- Produces package scripts build, check, check:base, check:content, verify, dev and preview for later tasks and CI.

- [ ] **Step 1: Write the failing base-path tests.**

Create site/scripts/check-base-path.test.mjs with Node’s test runner. The
fixtures cover one valid build, one root-absolute asset reference and one
missing expected asset:

~~~
import { mkdtemp, mkdir, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import assert from 'node:assert/strict';
import test from 'node:test';
import { validateBuild } from './check-base-path.mjs';

test('accepts base-prefixed local references and expected outputs', async () => {
  const root = await mkdtemp(join(tmpdir(), 'questmancer-base-'));
  await mkdir(join(root, '_astro'), { recursive: true });
  await writeFile(join(root, 'index.html'), '<img src="/herdr-questmancer/_astro/hero.abc.png">');
  await writeFile(join(root, '_astro/hero.abc.png'), 'fixture');
  assert.doesNotThrow(() => validateBuild(root, '/herdr-questmancer', ['hero']));
});

test('rejects a local root-relative asset path', async () => {
  const root = await mkdtemp(join(tmpdir(), 'questmancer-base-'));
  await writeFile(join(root, 'index.html'), '<img src="/assets/hero.png">');
  assert.throws(() => validateBuild(root, '/herdr-questmancer', ['hero']), /base path/);
});

test('rejects a missing expected output', async () => {
  const root = await mkdtemp(join(tmpdir(), 'questmancer-base-'));
  await writeFile(join(root, 'index.html'), '<main>ok</main>');
  assert.throws(() => validateBuild(root, '/herdr-questmancer', ['hero']), /missing/);
});
~~~

- [ ] **Step 2: Run the tests and confirm the expected failure.**

Run:

~~~
cd site
bun test scripts/check-base-path.test.mjs
~~~

Expected: FAIL because check-base-path.mjs does not exist yet.

- [ ] **Step 3: Add the minimal Astro package and config.**

Run:

~~~
mkdir -p site
cd site
bun init -y
bun add --dev astro @astrojs/check typescript pixelarticons
~~~

Set site/package.json to type: module and these scripts:

~~~
{
  "scripts": {
    "dev": "astro dev",
    "build": "astro build",
    "check": "astro check",
    "check:base": "node scripts/check-base-path.mjs",
    "check:content": "node --test scripts/content-contract.test.mjs",
    "verify": "bun run check && bun run build && bun run check:base && bun run check:content",
    "preview": "astro preview"
  }
}
~~~

Add these generated directories to the repository root .gitignore:

~~~
/site/node_modules/
/site/dist/
~~~

Create site/astro.config.mjs with:

~~~
import { defineConfig } from 'astro/config';

export default defineConfig({
  site: 'https://opsydyn.github.io',
  base: '/herdr-questmancer',
});
~~~

Create a minimal BaseLayout.astro and index.astro that render one main with a
temporary Questmancer heading. The temporary page exists only to make the build
pipeline testable before component work; replace its visible content in Task 3.

- [ ] **Step 4: Implement validateBuild.**

Implement check-base-path.mjs with these exact rules:

1. Throw Error('build directory is missing: ' + buildDir) when buildDir is absent.
2. Read every .html, .css and .js file below buildDir.
3. Extract local absolute references from src, href, action and CSS url(value)
   values. Ignore http:, https:, mailto:, data:, # and relative references.
4. Throw Error('local reference bypasses base path: ' + reference) when a local
   absolute reference does not equal basePath or begin with basePath + '/'.
5. Throw Error('missing expected built asset: ' + expectedStem) when no file
   basename begins with an entry in expectedAssetStems.

The CLI entry calls validateBuild('dist', '/herdr-questmancer', [ 'hero',
'guild-hall', 'delve', 'ledger', 'home', 'bell', 'heart', 'lock' ]) from site/.

- [ ] **Step 5: Run the audit tests and minimal build.**

Run:

~~~
cd site
bun test scripts/check-base-path.test.mjs
bun run check
bun run build
~~~

Expected: the three audit tests pass, Astro produces site/dist/index.html, and
the temporary page builds beneath the configured base.

- [ ] **Step 6: Commit the scaffold.**

~~~
git add site/package.json site/bun.lock site/astro.config.mjs site/src site/scripts
git commit -m "build: scaffold Questmancer Astro site"
~~~

---

### Task 2: Import the hero, screenshots and Pixelarticons assets

**Files:**
- Create: site/src/assets/hero.png
- Create: site/src/assets/screenshots/guild-hall.png
- Create: site/src/assets/screenshots/delve.png
- Create: site/src/assets/screenshots/compact-party.png
- Create: site/src/assets/screenshots/compact-party-warm.png
- Create: site/src/assets/screenshots/ledger.png
- Create: site/src/assets/screenshots/adventurer.png
- Create: site/src/assets/icons/home.svg
- Create: site/src/assets/icons/bell.svg
- Create: site/src/assets/icons/heart.svg
- Create: site/src/assets/icons/lock.svg
- Create: site/THIRD_PARTY_NOTICES

**Interfaces:**
- Produces stable source imports for HeroProof.astro, EvidenceGallery.astro and MappingCards.astro.
- Produces expected basenames consumed by check-base-path.mjs.

- [ ] **Step 1: Copy the approved image assets into site/src/assets.**

Copy the Downloads hero and the seven root screenshots into the normalized paths
from the file map. Do not delete or rename the root screenshots; they are
user-owned evidence and remain available for the Rust project’s release notes.
After copying, verify each file is non-empty and that sips -g pixelWidth
-g pixelHeight reports the hero at 1536x1024.

- [ ] **Step 2: Obtain and vendor four known Pixelarticons SVGs.**

Use the installed pixelarticons package as the source and copy its raw 24x24-grid
SVG files home.svg, bell.svg, heart.svg and lock.svg into site/src/assets/icons/.
Confirm each file contains an SVG viewBox="0 0 24 24" and fill="currentColor";
do not use React components or a webfont.

- [ ] **Step 3: Record attribution.**

Create site/THIRD_PARTY_NOTICES containing:

~~~
Pixelarticons
https://github.com/halfmage/pixelarticons
Copyright Gerrit Halfmann
License: MIT

This site vendors four raw SVG icons from the Pixelarticons free set:
home.svg, bell.svg, heart.svg, lock.svg.
~~~

- [ ] **Step 4: Run the asset-aware build audit.**

Run:

~~~
cd site
bun run build
bun run check:base
~~~

Expected: the audit finds the hero, three primary screenshots and four icon
basenames in dist/ and reports no root-relative local asset path.

- [ ] **Step 5: Commit the assets.**

~~~
git add site/src/assets site/THIRD_PARTY_NOTICES
git commit -m "feat: add Questmancer launch assets"
~~~

---

### Task 3: Build the static page components and content contract

**Files:**
- Create: site/src/components/SiteHeader.astro
- Create: site/src/components/HeroProof.astro
- Create: site/src/components/InstallWorkbench.astro
- Create: site/src/components/EvidenceGallery.astro
- Create: site/src/components/MappingCards.astro
- Create: site/src/components/SiteFooter.astro
- Modify: site/src/layouts/BaseLayout.astro
- Modify: site/src/pages/index.astro
- Create: site/src/styles/global.css
- Create: site/scripts/content-contract.test.mjs

**Interfaces:**
- BaseLayout.astro accepts title and description props and renders the document shell.
- HeroProof.astro accepts imported hero and primary screenshot assets; it owns the only page h1.
- InstallWorkbench.astro renders the five source-install commands as static code.
- EvidenceGallery.astro accepts screenshot records { src, alt, caption, className }.
- MappingCards.astro accepts records { icon, title, body } and renders one SVG icon per card.
- SiteFooter.astro renders the absolute GitHub CTA and no release/download claim.

- [ ] **Step 1: Write the failing content-contract test.**

Create site/scripts/content-contract.test.mjs that reads dist/index.html and
asserts the exact approved copy, exactly one h1, the absolute GitHub URL, the
source-install commands and the absence of a Cargo release claim:

~~~
import { readFile } from 'node:fs/promises';
import assert from 'node:assert/strict';
import test from 'node:test';

test('launch page preserves the approved content contract', async () => {
  const html = await readFile(new URL('../dist/index.html', import.meta.url), 'utf8');
  assert.equal((html.match(/<h1\b/g) ?? []).length, 1);
  for (const phrase of [
    'What is best in code?',
    'To crush your bugs.',
    'See your agents driven before you.',
    'Hear the lamentations of your token budget.',
    'https://github.com/opsydyn/herdr-questmancer',
    'herdr plugin action invoke opsydyn.questmancer.open',
  ]) assert.ok(html.includes(phrase), 'missing approved phrase: ' + phrase);
  assert.doesNotMatch(html, /cargo install questmancer/i);
});
~~~

- [ ] **Step 2: Run it and confirm the expected failure.**

Run cd site && bun run build && bun run check:content.

Expected: FAIL because the temporary page does not yet contain the approved
copy and component structure.

- [ ] **Step 3: Implement the layout shell and global styles.**

Make BaseLayout.astro render doctype, language metadata, a skip link,
global.css, a description meta tag and main id="main-content". Use CSS
variables for the night, paper, muted, gold, ember and line colors. Keep
component-local rules in each component’s style block.

Use the base-aware imported-image pattern:

~~~
---
import hero from '../assets/hero.png';
---

<img src={hero.src} width={hero.width} height={hero.height} alt="Pixel-art barbarian seated on a throne between red guild banners" />
~~~

Do not write /assets/ or /icons/ URLs. Astro’s imported asset URL must carry the
configured /herdr-questmancer/ base into generated HTML.

- [ ] **Step 4: Implement SiteHeader and HeroProof.**

SiteHeader contains the Questmancer wordmark, #guild, #screens anchors and the
GitHub link. HeroProof renders the Proof of the Delve kicker, a padded black
stage with the entire hero image using object-fit: contain, and the two
screenshot mosaic images. It does not render a marketing heading over the
image.

HeroProof’s following copy block owns the only h1:

~~~
<h1>What is best in code?</h1>
<p>To crush your bugs.<br />See your agents driven before you.<br />Hear the lamentations of your token budget.</p>
~~~

Place View on GitHub beside the grounding paragraph and keep the source-install
anchor as a secondary action.

- [ ] **Step 5: Implement InstallWorkbench and EvidenceGallery.**

Render the exact source commands in a framed pre/code block with a “Source
install · no Cargo release yet” label. EvidenceGallery renders the wide Guild
Hall and Delve screenshots first, then Ledger, Adventurer and the two compact
compositions with descriptive captions. Keep screenshots as static img
elements with intrinsic dimensions; use CSS containment and captioned
secondary crops only where the full screenshot cannot fit.

- [ ] **Step 6: Implement MappingCards and SiteFooter.**

Map the four product facts to icon imports:

~~~
const mappings = [
  { icon: home, title: 'Guild Hall', body: 'Whole-party operational home.' },
  { icon: lock, title: 'Delve', body: 'Active work in connected chambers.' },
  { icon: bell, title: 'Summons', body: 'Blocked agents call for counsel.' },
  { icon: heart, title: 'Spoils', body: 'Completed work returns for inspection.' },
];
~~~

Render icon.src as a decorative image with a visible text label beside it; the
card must remain understandable if the SVG is unavailable. The footer repeats
the tagline and GitHub URL.

- [ ] **Step 7: Compose index.astro in the approved order.**

Compose:

~~~
<BaseLayout title="Questmancer — What is best in code?" description="A pixel-art adventurers' guild for coordinating coding agents with Herdr.">
  <SiteHeader />
  <HeroProof />
  <InstallWorkbench />
  <EvidenceGallery />
  <MappingCards />
  <SiteFooter />
</BaseLayout>
~~~

Use semantic header, nav, section, figure, figcaption and footer landmarks.
Give each section a stable id only where the header links to it. Keep the page
free of client directives and inline event handlers.

- [ ] **Step 8: Run content and Astro checks.**

Run:

~~~
cd site
bun run check
bun run build
bun run check:base
bun run check:content
~~~

Expected: all checks pass, exactly one h1 is present, all approved phrases are
present, and built asset URLs carry the repository base.

- [ ] **Step 9: Commit the page slice.**

~~~
git add site/src site/scripts/content-contract.test.mjs
git commit -m "feat: build Questmancer launch page"
~~~

---

### Task 4: Add the GitHub Pages workflow with a fail-closed build

**Files:**
- Create: .github/workflows/deploy-site.yml
- Modify: site/package.json

**Interfaces:**
- The workflow consumes site/package.json script verify and the configured site/astro.config.mjs base.
- The deploy job publishes the artifact created by withastro/action@v6 to the github-pages environment.

- [ ] **Step 1: Confirm the verify command.**

Ensure the script remains:

~~~
"verify": "bun run check && bun run build && bun run check:base && bun run check:content"
~~~

- [ ] **Step 2: Write the workflow.**

Create .github/workflows/deploy-site.yml:

~~~
name: Deploy Questmancer site

on:
  push:
    branches: [main]
    paths:
      - 'site/**'
      - '.github/workflows/deploy-site.yml'
  workflow_dispatch:

permissions:
  contents: read
  pages: write
  id-token: write

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v7
      - name: Build Astro site
        uses: withastro/action@v6
        with:
          path: site
          package-manager: bun@latest
          build-cmd: bun run verify
  deploy:
    needs: build
    runs-on: ubuntu-latest
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - name: Deploy to GitHub Pages
        id: deployment
        uses: actions/deploy-pages@v5
~~~

Do not edit .github/workflows/ci.yml or release.yml. The site workflow is the
only workflow that owns the Pages deployment.

- [ ] **Step 3: Validate workflow syntax and local verification.**

Run:

~~~
cd site
bun run verify
cd ..
git diff --check
~~~

Inspect the workflow with sed -n '1,220p'
.github/workflows/deploy-site.yml and verify that its trigger paths include the
workflow and all site-owned files. Expected: local verification passes and the
workflow contains no secret, upload or production fallback.

- [ ] **Step 4: Commit the workflow.**

~~~
git add .github/workflows/deploy-site.yml site/package.json
git commit -m "ci: deploy Questmancer Astro site to Pages"
~~~

---

### Task 5: Perform responsive, accessibility and base-path visual QA

**Files:**
- Modify: site/src/components/*.astro or site/src/styles/global.css only when a QA finding requires it.
- Modify: site/scripts/content-contract.test.mjs only when a corrected content contract is explicitly approved.

**Interfaces:**
- Consumes the built site/dist/ and the local Astro preview.
- Produces fresh evidence for desktop, tablet, mobile, keyboard focus and base-prefixed asset requests.

- [ ] **Step 1: Start a production preview under the configured base.**

Run:

~~~
cd site
bun run build
bun run preview --host 127.0.0.1
~~~

Open the preview at the base-prefixed route /herdr-questmancer/ and confirm the
root route is not used as the acceptance URL.

- [ ] **Step 2: Review wide and narrow compositions.**

At desktop width, verify the full hero has visible black breathing room, the
hero artwork is not cropped, the screenshot mosaic is legible, and the terminal
block follows the hero copy. At tablet and 390px mobile widths, verify the hero
stacks without horizontal scrolling, the full artwork remains visible, code
lines scroll or wrap without breaking the viewport, and the four mapping cards
remain readable.

- [ ] **Step 3: Review accessibility and links.**

Keyboard-tab through the skip link, header anchors, GitHub buttons and footer
link; confirm a visible focus ring and logical order. Inspect the rendered DOM
for one h1, ordered headings, meaningful alt text, pre/code for the install
block and semantic landmarks. Click the GitHub CTA and confirm the target is
https://github.com/opsydyn/herdr-questmancer.

- [ ] **Step 4: Verify base-prefixed requests.**

Use browser network/DOM inspection to confirm image and icon requests begin with
/herdr-questmancer/ or are external GitHub URLs. A request to /assets/, /icons/
or another local root path is a failure. Re-run:

~~~
cd site
bun run check:base
bun run check:content
~~~

- [ ] **Step 5: Run repository-level hygiene checks.**

From the repository root, run:

~~~
git diff --check
git status --short --branch
~~~

Confirm only intended site/spec/workflow changes are present and unrelated Rust
edits remain preserved. Record visual or live Pages checks as unverified until
fresh evidence exists from the deployed commit.

- [ ] **Step 6: Commit approved QA fixes.**

~~~
git add site
git commit -m "fix: polish Questmancer launch page QA"
~~~

Create this commit only when the preceding checks identify a real, scoped site
issue; do not reformat or rewrite unrelated Rust files.

---

## Final handoff gate

Before claiming completion, run:

~~~
cd site
bun install --frozen-lockfile
bun run verify
cd ..
git diff --check
git status --short --branch
~~~

Report separately:

1. local Astro build, check, base and content results;
2. responsive and accessibility visual evidence;
3. GitHub Actions/Pages deployment evidence, if a current run exists; and
4. remaining unverified live-domain or release-state checks.
