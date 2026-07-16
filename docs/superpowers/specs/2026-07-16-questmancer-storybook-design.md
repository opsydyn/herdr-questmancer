# Questmancer Storybook design

**Date:** 2026-07-16

**Status:** Approved

**Scope:** Developer-only static TUI asset and scene catalogue

## Objective

Add a developer-only Questmancer Storybook: a static Ratatui application for
reviewing every authored visual asset and representative complete UI scene
without starting Herdr or running coding agents.

The Storybook is a visual workshop, not a third product view. It exists to make
sprite, palette, widget, accessibility and responsive-layout review fast and
repeatable while preserving one production rendering implementation.

## Decisions

- Ship a feature-gated second binary named `questmancer-storybook`.
- Expose it through the developer command `just storybook`.
- Use fixed, deterministic stories rather than mutable fixture controls.
- Give every authored visual asset exactly one canonical review location.
- Render full application stories through production `ui::render`.
- Render atlas tiles through their registered production renderers.
- Keep the Storybook independent of Herdr, persistence and plugin lifecycle.
- Include Storybook compilation, tests and linting in all-features CI.
- Do not package the Storybook binary as a plugin action or release asset.

## Non-goals

The first version does not provide:

- a live Herdr connection;
- editable fixture data;
- runtime knobs or a control panel;
- automatic screenshot export;
- an alternate theme or rendering engine;
- persisted Storybook selection;
- a public fixture SDK;
- a user-facing plugin action;
- exhaustive Cartesian combinations of every persona trait.

## User experience

The Storybook is a three-pane TUI.

```text
+-- QUESTMANCER STORYBOOK ------------------------------------------+
| STORIES          | PRODUCTION CANVAS              | STORY         |
|                  |                                |               |
| Asset Atlas      | fixed story rendered with      | description   |
|   Classes        | production assets and widgets  | viewport      |
|   Ancestries     |                                | compatibility |
|   Keepsakes      |                                | owns/shows    |
|   Poses          |                                | coverage      |
|                  |                                |               |
| Widgets          |                                |               |
| Full Scenes      |                                |               |
| Compatibility    |                                |               |
+-------------------------------------------------------------------+
| [j/k] story [h/l] category [enter] inspect [?] help [q] quit      |
+-------------------------------------------------------------------+
```

The left pane is a fixed catalogue. The centre pane is the production canvas.
The right pane explains the selected story and shows its asset-coverage
evidence. Full-screen inspection temporarily gives the entire usable terminal
area to the production canvas.

### Navigation

```text
j / down     next story
k / up       previous story
h / left     previous category
l / right    next category
enter        inspect the selected canvas full-screen
esc          return from inspection; quit from the catalogue
?            toggle Storybook help
q            quit
```

Story movement clamps at the first and last story in a category. Category
movement clamps at the first and last category and selects that category's
first story. The footer always describes actions valid for the active
Storybook state.

### Responsive behaviour

- The Storybook shell reflows with the real terminal.
- Each story declares and displays its reference viewport.
- The normal view preserves catalogue, canvas and evidence when space permits.
- Inspection gives the canvas all available space.
- A terminal too small for meaningful output shows required and available
  dimensions instead of attempting a broken render.
- Zero-sized regions are legal and must never panic.

Fixed stories mean fixed data, clock and animation frame. They do not require a
fixed physical terminal size.

## Catalogue

The catalogue is deliberately finite. Atlas stories show every member of an
asset family together; full scenes demonstrate composition and product meaning.

### Asset Atlas

- all 11 adventurer classes and their associated gear;
- all 7 ancestries, including Goblin;
- body proportions and head shapes;
- skin tones;
- hair shapes and hair tones;
- face details;
- garb, legwear and footwear;
- all 6 keepsakes;
- all 8 accent tones;
- all 7 theatre poses;
- all four ambient goblin sightings and the temporary outbreak.

### Widgets

- adventurer card, full and compact;
- chamber, full and compact, across every theatre state;
- Quest Board;
- Party;
- Summons;
- Chronicle;
- adventurer profile;
- Scrying;
- Spoils;
- Counsel composer;
- Search composer;
- Help overlay.

### Full Scenes

- empty Guild Hall;
- populated Guild Hall;
- mixed-attention Guild Hall;
- disconnected and reconnecting Guild Hall;
- Forgotten Library Delve;
- Mossy Undercroft Delve;
- Old Watchtower Delve;
- connected multi-campaign Delves;
- mixed-state party in a Delve;
- narrow-terminal Guild Hall and Delve fallbacks.

### Compatibility

- Unicode with Xterm-256 colours;
- Unicode with ANSI-16 colours;
- ASCII with ANSI-16 colours;
- full motion at a fixed animation instant;
- reduced motion;
- no motion.

Compatibility stories may reuse assets owned elsewhere. Their purpose is to
make degradation visible without becoming the canonical owner of those assets.

## Architecture

The Storybook is compiled inside the Questmancer library boundary and launched
by a minimal second binary.

```text
src/bin/questmancer_storybook.rs
        |
        v
#[cfg(feature = "storybook")]
questmancer::storybook::run()
        |
        +-- catalogue and navigation
        +-- deterministic fixtures
        +-- coverage inventory
        +-- production ui::render
        `-- production widget and sprite renderers
```

Cargo declares the binary only when the feature is enabled:

```toml
[features]
default = []
storybook = []

[[bin]]
name = "questmancer-storybook"
path = "src/bin/questmancer_storybook.rs"
required-features = ["storybook"]
```

The library exposes only the feature-gated `storybook::run` entrypoint needed
by the binary. Catalogue, fixture and coverage types remain implementation
details unless tests require crate-visible access.

### Story model

```rust
struct Story {
    id: StoryId,
    title: &'static str,
    category: Category,
    description: &'static str,
    viewport: Viewport,
    build: fn(&StoryContext) -> StoryFixture,
    owns: &'static [AssetId],
    shows: &'static [AssetId],
}

enum StoryFixture {
    Application(Model),
    AssetAtlas(AssetAtlas),
}
```

`Application` fixtures pass directly to production `ui::render`. `AssetAtlas`
owns only grid composition and review labels. Each tile invokes a registered
production sprite, palette, pose or widget renderer. It must not contain a
simplified or copied rendering implementation.

`StoryId` and `AssetId` are explicit stable identifiers. Catalogue ordering is
explicit rather than derived from filesystem or map iteration order.

Closed production enums such as class, ancestry, pose and Delve variant use
exhaustive mappings into the asset inventory. Adding a production enum variant
therefore creates a compile error until its `AssetId` and canonical story are
chosen. Authored widgets and scenes that are not represented by a closed enum
are listed explicitly in the inventory and protected by focused registry tests.

### Canonical ownership and reuse

Each authored visual asset has exactly one canonical review story:

- `owns` declares that canonical location;
- `shows` documents legitimate reuse elsewhere;
- an asset may be shown in many composed scenes;
- an asset may be owned by only one story.

This distinction makes the inventory exhaustive without treating normal scene
composition as duplicate coverage.

The coverage inspector reports:

- assets canonically owned by the selected story;
- other assets visible in the selected story;
- total owned assets by family;
- missing ownership;
- duplicate ownership;
- unknown identifiers.

Missing, duplicate or unknown ownership is a visible Storybook defect and an
automated test failure.

## Deterministic fixtures

Fixtures use:

- fixed timestamps;
- fixed animation instants;
- stable agent, pane, tab and workspace identities;
- production persona structures and generation rules;
- explicit presence and attention states;
- bounded, authored output and chronicle text.

No fixture uses the operating-system clock, environment, persistence or random
number generation.

Delve and goblin fixtures preserve production determinism. Fixed workspace IDs
are selected and tested to hash to:

- Forgotten Library;
- Mossy Undercroft;
- Old Watchtower;
- each ambient goblin sighting needed by the catalogue.

The Storybook must not add a variant override to production code. If the
production identity-to-variant function changes, the fixture test explains the
resulting intentional or accidental visual change.

## Runtime boundary

`just storybook` runs the equivalent of:

```bash
cargo run --features storybook --bin questmancer-storybook
```

The process:

- does not require Herdr to be installed or running;
- does not inspect `HERDR_SOCKET_PATH` or plugin environment variables;
- does not start the Herdr supervisor or protocol clients;
- does not load configuration or persisted runtime state;
- does not write files;
- stores selection only in memory;
- reuses Questmancer's production terminal guard and panic restoration path.

Normal exit, `Ctrl-C`, errors and panics restore terminal mode, cursor state and
screen state. Fixture or catalogue validation failures produce a readable error
screen where possible and a non-zero exit status.

The release workflow continues packaging only the production Questmancer
binary. The Storybook is neither installed by Herdr nor registered in the
plugin manifest.

## Testing

### Catalogue invariants

- every `AssetId` has exactly one canonical owner;
- no story owns an unknown asset;
- every declared shown asset exists;
- story IDs are unique;
- catalogue order is stable;
- every category is reachable.

### Fixture invariants

- building the same story twice produces equal fixture data;
- fixtures do not depend on wall-clock time;
- known workspace identities produce expected Delve variants;
- known workspace identities produce expected goblin sightings;
- application stories contain only bounded test data.

### Rendering tests

- every story renders with `ratatui::backend::TestBackend` at its reference
  viewport;
- application stories execute production `ui::render`;
- atlas tiles execute registered production renderers;
- representative labels and asset evidence are present;
- the catalogue, inspection view, help and too-small screen render safely.

Golden screenshots are not required for the first version. The Storybook is the
human visual-review surface; automated tests protect reachability, ownership,
determinism and render safety without making every intentional art adjustment
rewrite opaque buffer snapshots.

### Property tests

Use `proptest` to exercise every story over supported and pathological terminal
dimensions, including zero-sized areas. Properties include:

- rendering does not panic;
- every produced buffer remains within its declared bounds;
- selection remains valid after navigation and resize;
- inspection can always return to the catalogue;
- coverage results are independent of catalogue traversal.

### Quality gates

The existing gates extend to include Storybook:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

## Acceptance criteria

1. `just storybook` starts with no Herdr server or agents.
2. Every authored visual asset has one discoverable canonical story.
3. Every fixed story is reachable with the keyboard.
4. Full application scenes use production `ui::render`.
5. Atlas tiles use production asset renderers.
6. Reference viewport and compatibility mode remain visible in catalogue mode.
7. Unicode, ASCII, Xterm-256, ANSI-16 and motion modes are reviewable.
8. Every deterministic Delve and goblin variant is represented.
9. Resizing, tiny terminals and zero-sized regions do not panic.
10. Normal and abnormal exits restore the terminal.
11. Storybook reads no Herdr environment and writes no runtime state.
12. The normal production build, manifest and release package are unchanged
    unless the `storybook` feature is explicitly enabled.
13. Formatting, Clippy, unit tests, rendering tests and property tests pass with
    all features.

## Design principle

The Storybook may add catalogue chrome, fixture construction and coverage
metadata. It may not become an alternate Questmancer renderer. If an asset
cannot be reviewed through production code, the production asset boundary
should be made reusable rather than copied into Storybook.
