# Questmancer Astro launch page

Status: **Revised — pending user review**

Date: 2026-08-08

## Decision

Add a single-page Astro marketing site in `site/` for the planned GitHub Pages
address:

`https://opsydyn.github.io/herdr-questmancer/`

The page is a static launch surface for Questmancer. It introduces the product,
shows the supplied Conan-inspired hero and current product screenshots, gives a
source-install path, and sends visitors to GitHub. It does not become a second
Questmancer renderer or attempt to connect to Herdr.

The visual direction is **Proof of the Delve**: a large hero mosaic establishes
the world, the exact “What is best in code?” copy follows, and a Varlock-like
terminal rhythm explains how to get the source. The page closes with concise
operational mappings and a GitHub CTA.

## Goals

- Make Questmancer legible within one scroll: a Herdr plugin that presents
  coding-agent state as a living adventurers’ guild.
- Lead with the user-provided 1536x1024 pixel-art hero.
- Use the supplied Guild Hall, Delve, Librarian’s Ledger and adventurer
  screenshots as product evidence.
- Preserve the exact approved hero copy:

  > What is best in code?
  >
  > To crush your bugs.
  > See your agents driven before you.
  > Hear the lamentations of your token budget.

- Make **View on GitHub** the primary and repeated call to action.
- Explain source installation accurately while no Cargo package is published.
- Use Pixelarticons for a small set of crisp interface accents.
- Keep dependencies and client-side JavaScript to a minimum.
- Deploy correctly beneath the repository path `/herdr-questmancer/`.

## Non-goals

- A documentation site or multi-route content system.
- A Cargo installation command or package-release promise.
- Live Herdr data, socket connections, authentication, forms, telemetry or
  client-side state.
- A second production renderer, dashboard, terminal emulator or interactive
  product demo.
- Reworking the existing Rust plugin or its uncommitted changes.

## Page design

### 1. Site chrome

A compact dark header identifies Questmancer and links to the page anchors plus
the GitHub repository. The header remains readable on narrow screens and does
not compete with the hero. The GitHub URL is the absolute repository URL:

`https://github.com/opsydyn/herdr-questmancer`

### 2. Proof of the Delve hero

The first section uses the supplied barbarian throne image as a wide, framed
pixel-art stage. A small “Proof of the Delve” label establishes the hook. The
hero image has one heading only: the large “What is best in code?” heading in
the following copy block. The image itself carries no duplicate heading or
marketing text. A two-image mosaic immediately below shows the Guild Hall and
Delve screenshots, proving that the product has two distinct but related rooms.

The hero is intentionally image-led. The full image must remain visible: the
stage uses a black background, responsive padding and contain-style fitting so
the artwork has breathing room on every side. It is not cropped, altered,
smoothed or re-rendered. The frame may become shorter or stack on narrow
screens, but it must preserve the complete source composition.

### 3. Hero words and CTA

Below the mosaic, a single large `h1` reads “What is best in code?” and sets
the three approved Conan-inspired lines exactly as written. There is no second
heading for the same phrase elsewhere in the hero. A short paragraph
grounds the metaphor in Herdr’s facts: campaigns are workspaces, agents are
adventurers, and the scene remains honest about state. A filled **View on
GitHub** button is the primary CTA. A secondary anchor may scroll to the
installation/evidence sections, but it must not imply a download or Cargo
release.

### 4. Source-install workbench

The first post-hero section adopts Varlock’s docs-like code rhythm: a small
section label, a clear heading and a framed terminal block. The commands are
source installation commands already supported by the repository:

```text
git clone https://github.com/opsydyn/herdr-questmancer
cd herdr-questmancer
cargo build --release
herdr plugin link .
herdr plugin action invoke opsydyn.questmancer.open
```

The copy explicitly says “Source install · no Cargo release yet”. It must not
present `bun install` as the Questmancer runtime install; Bun is only the site’s
development/build tool.

### 5. Operational evidence

The screenshot gallery uses normalized copies of the root assets and concise
captions:

| Site asset | Source file | Meaning |
| --- | --- | --- |
| `hero.png` | `/Users/alancurrie/Downloads/ChatGPT Image Aug 8, 2026, 09_07_45 AM.png` | Conan-inspired Questmancer hero |
| `guild-hall.png` | `Screenshot 2026-08-08 at 09.17.35.png` | Guild Hall whole-party scene |
| `delve.png` | `Screenshot 2026-08-08 at 09.18.07.png` | Delve active-work scene |
| `compact-party.png` | `Screenshot 2026-08-08 at 09.18.24.png` | Compact party composition |
| `compact-party-warm.png` | `Screenshot 2026-08-08 at 09.18.36.png` | Warm compact composition |
| `ledger.png` | `Screenshot 2026-08-08 at 09.18.52.png` | Librarian’s Ledger overlay |
| `adventurer.png` | `Screenshot 2026-08-08 at 09.19.19.png` | Adventurer profile overlay |

The first gallery view prioritizes `guild-hall.png`, `delve.png` and
`ledger.png`; compact and profile images may appear in a responsive evidence
strip if the layout remains readable. Captions describe actual rooms and
actions; they do not claim live connectivity or unobserved transitions.

### 6. Mapping and footer

Four small feature cards connect the fantasy vocabulary to product facts:

- Guild Hall — whole-party operational home;
- Delve — active work in connected chambers;
- Summons — blocked agents call for counsel; and
- Spoils — completed work returns for inspection.

Each card uses one selected Pixelarticons SVG accent. The footer repeats the
Questmancer tagline, identifies Herdr and provides the GitHub link.

## Visual system

- Palette: near-black and plum night surfaces, parchment cream, torch gold,
  ember red and restrained teal from the product screenshots.
- Typography: system sans for readable copy and a system monospace stack for
  terminal content; no remote font request.
- Pixel treatment: crisp borders, squared corners, small framed panels and
  deliberate 24px-aligned icon sizes. No filters, smoothing or effects may
  blur the supplied pixel art; any tonal gradient is a separate overlay layer
  behind text, not a treatment applied to the image pixels.
- Hero framing: the main artwork uses `object-fit: contain` (or an equivalent
  imported-image layout) inside a padded black stage. `object-fit: cover` is
  reserved for secondary screenshot thumbnails where a captioned crop is
  acceptable.
- Responsive behaviour: two-column hero and evidence layout at wide sizes;
  stacked hero, screenshots and install block below the narrow breakpoint;
  compact feature cards become one or two columns without horizontal scrolling.
- Accessibility: semantic landmarks, one `h1`, logical heading order,
  descriptive image alt text, visible keyboard focus, usable contrast and
  reduced-motion-safe CSS.

## Astro architecture

The site is a self-contained Astro project:

```text
site/
  astro.config.mjs
  package.json
  bun.lock
  src/
    assets/
      hero.png
      screenshots/*.png
      icons/*.svg
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
  scripts/
    check-base-path.mjs
  THIRD_PARTY_NOTICES
```

`BaseLayout.astro` owns document metadata, skip link, global tokens and the
canonical `<main>`. Components own their markup and scoped styles; the small
`global.css` contains only reset, tokens, typography and shared focus styles.
`index.astro` composes the single page from those components and supplies the
static copy and image imports.

`package.json` exposes `dev`, `build`, `check`, `check:base` and `preview`
scripts. `check:base` runs `scripts/check-base-path.mjs` against the generated
`dist/` directory after the build.

Pixelarticons is used as an asset source, not a UI framework. The implementation
will vendor only the selected MIT-licensed SVGs into `src/assets/icons/`, retain
the upstream attribution in `site/THIRD_PARTY_NOTICES`, and render each icon as
an imported/static SVG with `currentColor`. No icon runtime or client bundle is
required.

## GitHub Pages base-path contract

`site/astro.config.mjs` must define:

```js
export default defineConfig({
  site: 'https://opsydyn.github.io',
  base: '/herdr-questmancer',
})
```

This is a hard correctness boundary. The implementation must not use root-
absolute asset paths such as `/assets/hero.png`. Images and icons are imported
from `src/assets` so Astro emits and rewrites them for the configured base.
Internal navigation uses Astro’s base-aware URL handling; external GitHub links
remain absolute. If any public asset is needed, its URL is formed from
`import.meta.env.BASE_URL`, never a handwritten root path.

`scripts/check-base-path.mjs` runs after `astro build` and fails when generated
HTML/CSS contains a local root-relative `/assets/`, `/icons/` or page link that
would bypass `/herdr-questmancer/`, or when a referenced built asset is absent.
The same script checks that the built `index.html` exists and that the expected
hero, screenshot and icon outputs are reachable under the base path.

## Deployment

Add a dedicated workflow at `.github/workflows/deploy-site.yml` using the
official Astro GitHub Action. It runs on pushes to `main` that touch `site/**`
or the workflow itself, and supports manual dispatch. The action uses `path:
site`, installs from the committed Bun lockfile, builds the static site, and
deploys with `actions/deploy-pages`. It grants only `contents: read`,
`pages: write` and `id-token: write`, and exposes the deployment URL through the
`github-pages` environment.

The repository’s existing Rust CI and release workflows remain unchanged. A
site build failure must fail the site workflow; it must not be hidden behind a
prebuilt artifact.

## Failure handling and privacy

- Missing image/icon imports fail the Astro build.
- `check-base-path.mjs` fails the build for path or missing-output errors.
- External GitHub links are static and have no fallback that claims a release
  exists.
- There are no forms, analytics, cookies, remote fonts, API requests or Herdr
  connections.
- The page remains meaningful if JavaScript is disabled because all content is
  server-rendered static HTML.

## Verification and acceptance

Focused checks:

```bash
cd site
bun install --frozen-lockfile
bun run check
bun run build
bun run check:base
```

Review the built site at wide desktop, tablet and narrow mobile sizes. Verify
that the hero, screenshots, text, CTA and terminal remain legible; no image is
stretched into an unreadable crop; focus styles are visible; and the browser
requests assets beneath `/herdr-questmancer/` rather than `/`.

The slice is accepted when:

1. the `site/` project builds from its committed lockfile;
2. the base-path audit passes and no local asset returns a 404 in the built
   preview;
3. the exact hero copy and GitHub CTA are present;
4. the supplied hero and product screenshots render with useful alt text;
5. the page has no Cargo-release claim and no live-data claim;
6. the GitHub Pages workflow is scoped to the site and succeeds; and
7. `git diff --check` is clean for the site slice.
