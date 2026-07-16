# Questmancer Storybook Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a developer-only, feature-gated Ratatui Storybook that renders every authored Questmancer asset and representative fixed scenes without Herdr or persisted runtime state.

**Architecture:** A second binary calls a feature-gated `questmancer::storybook` module. The module owns deterministic fixtures, catalogue navigation, coverage metadata and review chrome; application stories render into an off-screen `TestBackend` through production `ui::render`, while atlas stories compose and pack the existing production pixel canvases.

**Tech Stack:** Rust 2024, Rust 1.90, Tokio, Crossterm 0.29, Ratatui 0.30, Proptest 1.11, Cargo features, Just.

## Global Constraints

- The binary is named `questmancer-storybook` and requires the Cargo feature `storybook`.
- The developer entrypoint is exactly `just storybook`.
- Story fixtures are fixed and deterministic; they do not read wall-clock time, environment, Herdr, configuration or persistence.
- Full application stories must call production `ui::render`.
- Atlas tiles must call production persona composers and `pixel::pack`; no copied sprite renderer is permitted.
- Every authored asset has exactly one canonical owner story; composed scenes may show it again.
- Storybook state remains in memory and the process writes no files.
- Unicode/Xterm-256, Unicode/ANSI-16, ASCII/ANSI-16, full, reduced and no-motion stories are reviewable.
- The normal plugin manifest, action surface, installer and release assets remain unchanged.
- No unsafe code and no new third-party dependencies.

---

## File map

### New production files

- `src/bin/questmancer_storybook.rs` — minimal Tokio entrypoint and panic-hook installation.
- `src/storybook/mod.rs` — feature entrypoint and public exports used by integration tests.
- `src/storybook/app.rs` — Storybook selection, inspection and help state reducer.
- `src/storybook/assets.rs` — exhaustive asset identifiers, labels and inventory.
- `src/storybook/catalogue.rs` — categories, story metadata, builders and coverage validation.
- `src/storybook/fixtures.rs` — fixed clock, agent/campaign builders and application fixtures.
- `src/storybook/atlas.rs` — production-composed atlas tile fixtures.
- `src/storybook/input.rs` — Crossterm-to-Storybook action mapping.
- `src/storybook/runtime.rs` — event loop with no Herdr or persistence dependencies.
- `src/storybook/ui.rs` — three-pane shell, atlas rendering, production-buffer embedding and inspection view.

### Modified production files

- `Cargo.toml` — declare `storybook` and its required-feature binary.
- `justfile` — add `storybook` and `storybook-test` developer recipes.
- `src/lib.rs` — expose `storybook` only when the feature is enabled.
- `src/terminal.rs` — make the existing terminal guard reusable inside the crate and test its drop restoration hook.
- `src/ui/delve_scene.rs` — derive `Hash` for `DelveVariant`.
- `src/ui/goblins.rs` — derive `Hash` for `GoblinSighting`.
- `src/ui/theatre.rs` — derive `Hash` for `TheatrePose`.
- `README.md` — document the developer-only Storybook workflow.

### New tests

- `tests/storybook_catalogue.rs` — unique IDs, exhaustive ownership and deterministic variant identities.
- `tests/storybook_fixtures.rs` — stable fixture data and production model construction.
- `tests/storybook_navigation.rs` — clamped category/story navigation and inspection transitions.
- `tests/storybook_rendering.rs` — every story, shell, inspection and compatibility rendering.
- `tests/storybook_properties.rs` — terminal-dimension and navigation properties.

Existing `.github/workflows/ci.yml` already runs `cargo clippy --all-targets --all-features` and `cargo test --all-targets --all-features`; do not add a separate CI job.

---

### Task 1: Feature-gated developer binary

**Files:**
- Modify: `Cargo.toml`
- Modify: `justfile`
- Modify: `src/lib.rs:1-15`
- Create: `src/bin/questmancer_storybook.rs`
- Create: `src/storybook/mod.rs`

**Interfaces:**
- Consumes: `questmancer::terminal::install_panic_hook()`.
- Produces: `pub async fn storybook::run() -> anyhow::Result<()>`; Cargo binary `questmancer-storybook`; Just recipes `storybook` and `storybook-test`.

- [ ] **Step 1: Verify the gated binary does not exist yet**

Run:

```bash
cargo check --features storybook --bin questmancer-storybook
```

Expected: FAIL because Cargo has neither the feature nor the binary target.

- [ ] **Step 2: Declare the feature, target and library module**

Add to `Cargo.toml` before dependencies:

```toml
[features]
default = []
storybook = []

[[bin]]
name = "questmancer-storybook"
path = "src/bin/questmancer_storybook.rs"
required-features = ["storybook"]
```

Add to `src/lib.rs`:

```rust
#[cfg(feature = "storybook")]
pub mod storybook;
```

Create `src/storybook/mod.rs` with a compiling entrypoint that will be replaced
by the real runtime in Task 7:

```rust
use anyhow::Result;

pub async fn run() -> Result<()> {
    Ok(())
}
```

Create `src/bin/questmancer_storybook.rs`:

```rust
use anyhow::Result;
use questmancer::{storybook, terminal};

#[tokio::main]
async fn main() -> Result<()> {
    terminal::install_panic_hook();
    storybook::run().await
}
```

- [ ] **Step 3: Add the exact developer commands**

Append to `justfile`:

```make
storybook:
    cargo run --features storybook --bin questmancer-storybook

storybook-test:
    cargo test --all-targets --features storybook
```

- [ ] **Step 4: Verify both normal and gated builds**

Run:

```bash
cargo check --bin questmancer
cargo check --features storybook --bin questmancer-storybook
cargo metadata --no-deps --format-version 1 | jq -e '.packages[0].targets[] | select(.name == "questmancer-storybook")'
```

Expected: all three commands exit 0; metadata reports `required-features: ["storybook"]` for the second binary.

- [ ] **Step 5: Commit the executable boundary**

```bash
git add Cargo.toml justfile src/lib.rs src/bin/questmancer_storybook.rs src/storybook/mod.rs
git commit -m "feat: add gated Questmancer Storybook binary"
```

---

### Task 2: Asset inventory and coverage invariants

**Files:**
- Create: `src/storybook/assets.rs`
- Create: `src/storybook/catalogue.rs`
- Modify: `src/storybook/mod.rs`
- Modify: `src/ui/delve_scene.rs:7-11`
- Modify: `src/ui/goblins.rs:18-25`
- Modify: `src/ui/theatre.rs:10-20`
- Create: `tests/storybook_catalogue.rs`

**Interfaces:**
- Consumes: production enum variants from `domain::persona`, `ui::theatre`, `ui::delve_scene` and `ui::goblins`.
- Produces: `AssetId`, `WidgetAsset`, `SceneAsset`, `CompatibilityAsset`, `StoryId`, `Category`, `Viewport`, `Story`, `CoverageReport`, `validate_coverage(inventory, stories)` and `asset_inventory()`.

- [ ] **Step 1: Write failing coverage tests**

Create `tests/storybook_catalogue.rs`:

```rust
#![cfg(feature = "storybook")]

use questmancer::storybook::catalogue::{
    Category, Story, StoryId, Viewport, validate_coverage,
};
use questmancer::{
    app::{Model, View},
    storybook::{
        AssetId, WidgetAsset, asset_inventory,
        fixtures::{StoryContext, StoryFixture},
    },
};

fn build(_: &StoryContext) -> StoryFixture {
    StoryFixture::Application(Model::new(View::Guild))
}

fn story(id: &'static str, owns: &'static [AssetId]) -> Story {
    Story::new(
        StoryId::new(id),
        id,
        Category::Widgets,
        "coverage fixture",
        Viewport::new(80, 24, 40, 12),
        build,
        owns,
        &[],
    )
}

#[test]
fn coverage_accepts_exactly_one_owner_per_asset() {
    const BOARD: AssetId = AssetId::Widget(WidgetAsset::QuestBoard);
    let report = validate_coverage(&[BOARD], &[story("board", &[BOARD])]).unwrap();
    assert_eq!(report.owned(), 1);
    assert!(report.missing().is_empty());
    assert!(report.duplicates().is_empty());
}

#[test]
fn coverage_rejects_missing_duplicate_and_unknown_ownership() {
    const BOARD: AssetId = AssetId::Widget(WidgetAsset::QuestBoard);
    const PARTY: AssetId = AssetId::Widget(WidgetAsset::Party);
    let error = validate_coverage(
        &[BOARD],
        &[story("one", &[BOARD, PARTY]), story("two", &[BOARD])],
    )
    .unwrap_err();
    assert_eq!(error.duplicates(), &[BOARD]);
    assert_eq!(error.unknown(), &[PARTY]);
}

#[test]
fn authored_inventory_contains_no_duplicate_identifiers() {
    let inventory = asset_inventory();
    let unique = inventory.iter().copied().collect::<std::collections::HashSet<_>>();
    assert_eq!(inventory.len(), unique.len());
}
```

- [ ] **Step 2: Run the tests to verify the missing module failure**

Run:

```bash
cargo test --features storybook --test storybook_catalogue
```

Expected: FAIL because `storybook::catalogue`, `AssetId` and `WidgetAsset` do not exist.

- [ ] **Step 3: Define exhaustive, typed asset identifiers**

In `src/ui/delve_scene.rs`, `src/ui/goblins.rs` and `src/ui/theatre.rs`, add
`Hash` to the existing derives. Then create `src/storybook/assets.rs` with these
public types:

```rust
use crate::{
    domain::{
        AccentTone, AdventurerClass, AdventuringGear, Ancestry, BodyProportions,
        FaceDetail, Footwear, Garb, HairShape, HairTone, HeadShape, Keepsake,
        Legwear, SkinTone,
    },
    ui::{
        delve_scene::DelveVariant,
        goblins::GoblinSighting,
        pixel::ColorRole,
        theatre::TheatrePose,
    },
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AssetId {
    Class(AdventurerClass),
    Gear(AdventuringGear),
    Ancestry(Ancestry),
    BodyProportions(BodyProportions),
    HeadShape(HeadShape),
    SkinTone(SkinTone),
    HairShape(HairShape),
    HairTone(HairTone),
    FaceDetail(FaceDetail),
    Garb(Garb),
    Legwear(Legwear),
    Footwear(Footwear),
    Keepsake(Keepsake),
    AccentTone(AccentTone),
    ColorRole(ColorRole),
    Pose(TheatrePose),
    DelveVariant(DelveVariant),
    GoblinSighting(GoblinSighting),
    GoblinOutbreak,
    Widget(WidgetAsset),
    Scene(SceneAsset),
    Compatibility(CompatibilityAsset),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WidgetAsset {
    AdventurerCardFull,
    AdventurerCardCompact,
    ChamberFull,
    ChamberCompact,
    QuestBoard,
    Party,
    Summons,
    Chronicle,
    AdventurerProfile,
    Scrying,
    Spoils,
    Counsel,
    Search,
    Help,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SceneAsset {
    GuildEmpty,
    GuildPopulated,
    GuildMixedAttention,
    GuildDisconnected,
    GuildReconnecting,
    ConnectedDelves,
    MixedStateDelve,
    NarrowGuild,
    NarrowDelve,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CompatibilityAsset {
    UnicodeXterm256,
    UnicodeAnsi16,
    AsciiAnsi16,
    MotionFull,
    MotionReduced,
    MotionNone,
}
```

Define one `const` complete slice for every production enum family, including
all 11 `AdventuringGear` variants and all 22 `ColorRole` variants. Add an
exhaustive `AssetId::label` match covering every `AssetId` variant and every
nested production enum variant. This match is the compile-time gate: adding a
production variant cannot compile until its Storybook label is chosen.

Build `asset_inventory()` by chaining the complete slices, then appending all
widget, scene and compatibility assets. Deduplicate the resulting vector with
a debug assertion before returning it.

- [ ] **Step 4: Implement story metadata and coverage validation**

Create `src/storybook/catalogue.rs` around these interfaces:

```rust
use std::collections::{HashMap, HashSet};

use super::AssetId;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StoryId(&'static str);

impl StoryId {
    pub const fn new(value: &'static str) -> Self { Self(value) }
    pub const fn as_str(self) -> &'static str { self.0 }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Category { AssetAtlas, Widgets, FullScenes, Compatibility }

impl Category {
    pub const ALL: [Self; 4] = [
        Self::AssetAtlas,
        Self::Widgets,
        Self::FullScenes,
        Self::Compatibility,
    ];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Viewport {
    pub reference_width: u16,
    pub reference_height: u16,
    pub minimum_width: u16,
    pub minimum_height: u16,
}

impl Viewport {
    pub const fn new(
        reference_width: u16,
        reference_height: u16,
        minimum_width: u16,
        minimum_height: u16,
    ) -> Self {
        Self { reference_width, reference_height, minimum_width, minimum_height }
    }
}

pub type StoryBuilder = fn(&super::fixtures::StoryContext) -> super::fixtures::StoryFixture;

#[derive(Clone, Debug)]
pub struct Story {
    pub id: StoryId,
    pub title: &'static str,
    pub category: Category,
    pub description: &'static str,
    pub viewport: Viewport,
    pub build: StoryBuilder,
    pub owns: &'static [AssetId],
    pub shows: &'static [AssetId],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoverageReport {
    owned: usize,
    missing: Vec<AssetId>,
    duplicates: Vec<AssetId>,
    unknown: Vec<AssetId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoverageError {
    missing: Vec<AssetId>,
    duplicates: Vec<AssetId>,
    unknown: Vec<AssetId>,
}
```

`Story::new` stores every field exactly as supplied. Focused coverage tests use
the local deterministic builder shown above; production catalogue entries use
their specific fixture builder.

`validate_coverage` must use `HashSet<AssetId>` for the inventory and a
`HashMap<AssetId, Vec<StoryId>>` for owners, sort issues by `AssetId::label()`,
and return a `CoverageReport` only when missing, duplicate and unknown lists are
empty. Both result types expose `owned`, `missing`, `duplicates` and `unknown`
accessors where the field applies. Implement `Display` and `std::error::Error`
for `CoverageError`; its message lists all three issue groups using stable asset
labels so runtime validation failures are actionable.

Export `assets`, `catalogue`, `AssetId`, `WidgetAsset`, `SceneAsset` and
`CompatibilityAsset` from `src/storybook/mod.rs`. Add a minimal
`src/storybook/fixtures.rs` containing `StoryContext`, atlas payloads and
`StoryFixture` so the builder type compiles:

```rust
use crate::{
    app::{DisplayPreferences, Model},
    domain::Agent,
    ui::{
        pixel::{Canvas, ColorRole, Palette},
        theatre::TheatreFrame,
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StoryContext;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AtlasTile {
    pub label: &'static str,
    pub preferred_width: u16,
    pub preferred_height: u16,
    pub content: AtlasContent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AtlasContent {
    Pixel {
        canvas: Canvas,
        palette: Palette,
        background: ColorRole,
    },
    AdventurerCard {
        agent: Agent,
        theatre: TheatreFrame,
        preferences: DisplayPreferences,
    },
    Chamber {
        agent: Agent,
        theatre: TheatreFrame,
        selected: bool,
        preferences: DisplayPreferences,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetAtlas {
    pub tiles: Vec<AtlasTile>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StoryFixture {
    Application(Model),
    AssetAtlas(AssetAtlas),
}
```

- [ ] **Step 5: Verify the coverage unit and integration tests**

Run:

```bash
cargo test --features storybook --test storybook_catalogue
cargo clippy --all-targets --features storybook -- -D warnings
```

Expected: all commands exit 0; the coverage integration target reports 3 passed.

- [ ] **Step 6: Commit the inventory boundary**

```bash
git add src/storybook src/ui/delve_scene.rs src/ui/goblins.rs src/ui/theatre.rs tests/storybook_catalogue.rs
git commit -m "feat: define Storybook asset coverage"
```

---

### Task 3: Deterministic application fixtures

**Files:**
- Modify: `src/storybook/fixtures.rs`
- Create: `tests/storybook_fixtures.rs`

**Interfaces:**
- Consumes: `Model`, `DomainState`, `Agent`, `Campaign`, `ChronicleEntry`, `OutputPreview`, `variant_for_campaign` and `sighting_for_campaign`.
- Produces: `StoryContext::fixed()`, `agent_fixture`, `campaign_fixture`, `guild_fixture`, `delve_fixture`, `modal_fixture`, `compatibility_fixture` and fixed identity constants.

- [ ] **Step 1: Write failing fixture determinism tests**

Create `tests/storybook_fixtures.rs`:

```rust
#![cfg(feature = "storybook")]

use questmancer::{
    app::{Modal, View},
    storybook::fixtures::{
        StoryContext, delve_fixture, goblin_biscuit_id, goblin_chest_id,
        goblin_hand_id, goblin_scroll_id, guild_fixture, library_id,
        undercroft_id, watchtower_id,
    },
    ui::{
        delve_scene::{DelveVariant, variant_for_campaign},
        goblins::{GoblinSighting, sighting_for_campaign},
    },
};

#[test]
fn fixtures_are_value_deterministic() {
    let context = StoryContext::fixed();
    assert_eq!(guild_fixture(&context), guild_fixture(&context));
    assert_eq!(delve_fixture(&context), delve_fixture(&context));
    assert_eq!(guild_fixture(&context).view(), View::Guild);
    assert_eq!(delve_fixture(&context).view(), View::Delve);
    assert_eq!(guild_fixture(&context).modal(), &Modal::None);
}

#[test]
fn fixed_workspace_ids_lock_authored_variants() {
    assert_eq!(variant_for_campaign(&library_id()), DelveVariant::ForgottenLibrary);
    assert_eq!(variant_for_campaign(&undercroft_id()), DelveVariant::MossyUndercroft);
    assert_eq!(variant_for_campaign(&watchtower_id()), DelveVariant::OldWatchtower);
    assert_eq!(sighting_for_campaign(&goblin_chest_id()), Some(GoblinSighting::ChestEyes));
    assert_eq!(sighting_for_campaign(&goblin_hand_id()), Some(GoblinSighting::ChronicleHand));
    assert_eq!(sighting_for_campaign(&goblin_scroll_id()), Some(GoblinSighting::RaftersScroll));
    assert_eq!(sighting_for_campaign(&goblin_biscuit_id()), Some(GoblinSighting::StolenBiscuit));
}
```

- [ ] **Step 2: Run the fixture tests and observe the missing exports**

Run:

```bash
cargo test --features storybook --test storybook_fixtures
```

Expected: FAIL because the fixed context and fixture builders do not exist.

- [ ] **Step 3: Add fixed identities and the fixture context**

Use exact production-stable identifiers in `src/storybook/fixtures.rs`:

```rust
pub const FIXED_NOW: Timestamp = Timestamp::from_millis(121_000);

pub fn library_id() -> WorkspaceId { WorkspaceId::new("workspace-0") }
pub fn undercroft_id() -> WorkspaceId { WorkspaceId::new("workspace-2") }
pub fn watchtower_id() -> WorkspaceId { WorkspaceId::new("workspace-4") }
pub fn goblin_chest_id() -> WorkspaceId { WorkspaceId::new("goblin-fixture-32") }
pub fn goblin_hand_id() -> WorkspaceId { WorkspaceId::new("goblin-fixture-2901") }
pub fn goblin_scroll_id() -> WorkspaceId { WorkspaceId::new("goblin-fixture-330") }
pub fn goblin_biscuit_id() -> WorkspaceId { WorkspaceId::new("goblin-fixture-801") }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StoryContext {
    pub now: Timestamp,
}

impl StoryContext {
    pub const fn fixed() -> Self { Self { now: FIXED_NOW } }
}
```

- [ ] **Step 4: Add complete model builders**

Implement `agent_fixture` by constructing the production `Agent` directly:

```rust
pub fn agent_fixture(
    id: &'static str,
    workspace_id: WorkspaceId,
    presence: Presence,
    attention: GuildAttention,
    focused: bool,
) -> Agent {
    let key = AgentKey::new(id);
    let persona_key = PersonaKey::new(format!("storybook-{id}"));
    Agent {
        key,
        pane_id: PaneId::new(format!("storybook:{id}")),
        workspace_id,
        tab_id: TabId::new("storybook-tab"),
        name: id.replace('-', " "),
        custom_status: Some("Following the Questmancer's commission.".to_owned()),
        presence,
        presence_since: Timestamp::from_millis(1_000),
        attention,
        focused,
        pane_revision: 7,
        persona: AdventurerPersona::for_key(persona_key),
    }
}
```

`campaign_fixture` sets `/storybook/<workspace-id>` as its cwd and preserves
the supplied lexical party order. `guild_fixture` must contain one working,
one blocked, one done-unread, one idle and one exited adventurer, three campaign
records, five bounded chronicle entries, a selected blocked adventurer, a
connected state, fixed output preview text and `FIXED_NOW`.

`delve_fixture` starts from the same domain, switches to `View::Delve`, and uses
the three fixed Delve workspace IDs. `modal_fixture(Modal)` applies Help,
Counsel text `"Use the local schema"`, or Search text `"Elowen"` through public
`Model` methods. `compatibility_fixture(DisplayPreferences)` applies the exact
preferences to a cloned Delve model.

- [ ] **Step 5: Verify deterministic data and production variant selection**

Run:

```bash
cargo test --features storybook --test storybook_fixtures
cargo test --features storybook --test delve_scene --test goblins
```

Expected: all tests pass; the fixed-ID test proves all three Delves and four goblin sightings.

- [ ] **Step 6: Commit deterministic fixtures**

```bash
git add src/storybook/fixtures.rs tests/storybook_fixtures.rs
git commit -m "feat: add deterministic Storybook fixtures"
```

---

### Task 4: Production-composed asset atlas

**Files:**
- Create: `src/storybook/atlas.rs`
- Modify: `src/storybook/fixtures.rs`
- Modify: `src/storybook/catalogue.rs`
- Modify: `src/storybook/mod.rs`
- Create: `tests/storybook_rendering.rs`

**Interfaces:**
- Consumes: `compose_profile_adventurer`, `compose_chamber_adventurer_for_palette`, `pixel::pack`, exhaustive asset slices and `StoryContext`.
- Produces: `AtlasTile`, `AtlasContent`, asset-family story builders and `catalogue()` entries for all atlas stories.

- [ ] **Step 1: Write failing atlas production tests**

Start `tests/storybook_rendering.rs` with:

```rust
#![cfg(feature = "storybook")]

use questmancer::{
    storybook::{
        catalogue::catalogue,
        fixtures::{AtlasContent, StoryContext, StoryFixture},
    },
};

#[test]
fn class_atlas_uses_production_profile_canvases() {
    let story = catalogue().iter().find(|story| story.id.as_str() == "atlas.classes").unwrap();
    let StoryFixture::AssetAtlas(atlas) = (story.build)(&StoryContext::fixed()) else {
        panic!("class atlas must be an asset atlas");
    };
    assert_eq!(atlas.tiles.len(), 11);
    for tile in &atlas.tiles {
        let AtlasContent::Pixel { canvas, .. } = &tile.content else {
            panic!("class tiles must contain production pixel canvases");
        };
        assert_eq!((canvas.width(), canvas.height()), (16, 32));
        assert!(canvas.pixels().iter().any(Option::is_some));
    }
}

#[test]
fn pose_atlas_uses_all_seven_production_theatre_poses() {
    let story = catalogue().iter().find(|story| story.id.as_str() == "atlas.poses").unwrap();
    let StoryFixture::AssetAtlas(atlas) = (story.build)(&StoryContext::fixed()) else {
        panic!("pose atlas must be an asset atlas");
    };
    assert_eq!(atlas.tiles.len(), 7);
}
```

- [ ] **Step 2: Run the atlas tests to verify the missing catalogue failure**

Run:

```bash
cargo test --features storybook --test storybook_rendering
```

Expected: FAIL because the real catalogue and atlas builders do not exist.

- [ ] **Step 3: Build atlas tiles only through production composers**

Create `src/storybook/atlas.rs` with:

```rust
pub fn profile_tile(
    label: &'static str,
    mutate: impl FnOnce(&mut AdventurerPersona),
) -> AtlasTile {
    let mut persona = AdventurerPersona::for_key(PersonaKey::new("storybook-atlas"));
    mutate(&mut persona);
    AtlasTile {
        label,
        preferred_width: 18,
        preferred_height: 18,
        content: AtlasContent::Pixel {
            canvas: compose_profile_adventurer(&persona),
            palette: Palette::Xterm256,
            background: ColorRole::DarkStone,
        },
    }
}

pub fn chamber_tile(
    label: &'static str,
    pose: TheatrePose,
    animation_frame: u8,
) -> AtlasTile {
    let persona = AdventurerPersona::for_key(PersonaKey::new("storybook-pose-atlas"));
    AtlasTile {
        label,
        preferred_width: 14,
        preferred_height: 8,
        content: AtlasContent::Pixel {
            canvas: compose_chamber_adventurer_for_palette(
                &persona,
                TheatreFrame { pose, animation_frame, focused: false, label },
                Palette::Xterm256,
            ),
            palette: Palette::Xterm256,
            background: ColorRole::DarkStone,
        },
    }
}
```

Add family builders for classes, ancestries, body proportions, head shapes, skin
tones, hair shapes, hair tones, face details, garb, legwear, footwear,
keepsakes, accent tones, colour roles and poses. Each builder iterates the corresponding
complete asset slice from `assets.rs` and mutates only the trait under review.
Classes and appearance families use profile canvases; poses use chamber
canvases. Each class tile canonically covers both its class and the gear returned
by `AdventurerClass::gear()`. The done-unread pose uses animation frame `4`;
every other pose uses frame `0`.

- [ ] **Step 4: Register the canonical atlas stories**

Add fixed catalogue entries with IDs:

```text
atlas.classes
atlas.ancestries
atlas.body-proportions
atlas.head-shapes
atlas.skin-tones
atlas.hair-shapes
atlas.hair-tones
atlas.face-details
atlas.garb
atlas.legwear
atlas.footwear
atlas.keepsakes
atlas.accent-tones
atlas.palette-roles
atlas.poses
```

Each entry owns the complete matching `AssetId` slice and shows no additional
assets. `atlas.classes` owns both its class and derived gear slices.
`atlas.palette-roles` renders one production `ColorRole` swatch per tile with
`Palette::Xterm256`. Use reference viewport `120x36`, minimum `60x18`.

- [ ] **Step 5: Verify production canvas dimensions and atlas ownership**

Run:

```bash
cargo test --features storybook --test storybook_rendering --test storybook_catalogue
cargo test --features storybook --test persona_art --test pixel
```

Expected: all tests pass and existing production art tests remain unchanged.

- [ ] **Step 6: Commit the atlas catalogue**

```bash
git add src/storybook tests/storybook_catalogue.rs tests/storybook_rendering.rs
git commit -m "feat: add production-composed Storybook atlases"
```

---

### Task 5: Storybook navigation state

**Files:**
- Create: `src/storybook/app.rs`
- Create: `src/storybook/input.rs`
- Modify: `src/storybook/mod.rs`
- Create: `tests/storybook_navigation.rs`

**Interfaces:**
- Consumes: ordered `&[Story]` and Crossterm `Event`.
- Produces: `StorybookApp`, `Mode`, `Action`, `reduce(app, action, catalogue) -> Exit`, `action_for_event`.

- [ ] **Step 1: Write clamped-navigation and inspection tests**

Create `tests/storybook_navigation.rs`:

```rust
#![cfg(feature = "storybook")]

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use questmancer::{
    app::{Model, View},
    storybook::{
        app::{Action, Exit, Mode, StorybookApp, reduce},
        catalogue::{Category, Story, StoryId, Viewport},
        fixtures::{StoryContext, StoryFixture},
        input::action_for_event,
    },
};

fn build(_: &StoryContext) -> StoryFixture {
    StoryFixture::Application(Model::new(View::Guild))
}

fn navigation_catalogue() -> Vec<Story> {
    Category::ALL.into_iter().enumerate().map(|(index, category)| {
        Story::new(
            StoryId::new(["atlas", "widgets", "scenes", "compat"][index]),
            "Navigation fixture",
            category,
            "Navigation fixture",
            Viewport::new(80, 24, 40, 12),
            build,
            &[],
            &[],
        )
    }).collect()
}

#[test]
fn story_and_category_navigation_clamp() {
    let stories = navigation_catalogue();
    let mut app = StorybookApp::new(&stories);
    assert_eq!(reduce(&mut app, Action::PreviousStory, &stories), Exit::Continue);
    assert_eq!(app.selected_index(), 0);
    reduce(&mut app, Action::NextCategory, &stories);
    assert_eq!(app.selected_story(&stories).category, Category::Widgets);
    assert_eq!(app.index_within_category(&stories), 0);
}

#[test]
fn inspect_and_escape_return_to_the_catalogue_before_quitting() {
    let stories = navigation_catalogue();
    let mut app = StorybookApp::new(&stories);
    reduce(&mut app, Action::Inspect, &stories);
    assert_eq!(app.mode(), Mode::Inspect);
    assert_eq!(reduce(&mut app, Action::Escape, &stories), Exit::Continue);
    assert_eq!(app.mode(), Mode::Catalogue);
    assert_eq!(reduce(&mut app, Action::Escape, &stories), Exit::Quit);
}

#[test]
fn keys_map_without_leaking_production_actions() {
    let key = |code| Event::Key(KeyEvent::new(code, KeyModifiers::NONE));
    assert_eq!(action_for_event(&key(KeyCode::Char('j'))), Action::NextStory);
    assert_eq!(action_for_event(&key(KeyCode::Enter)), Action::Inspect);
    assert_eq!(action_for_event(&key(KeyCode::Char('?'))), Action::ToggleHelp);
    let ctrl_c = Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
    assert_eq!(action_for_event(&ctrl_c), Action::Quit);
}
```

- [ ] **Step 2: Run the navigation tests to confirm missing state**

Run:

```bash
cargo test --features storybook --test storybook_navigation
```

Expected: FAIL because `storybook::app` and `storybook::input` do not exist.

- [ ] **Step 3: Implement pure state transitions**

Create these exact enums in `src/storybook/app.rs`:

```rust
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Mode { #[default] Catalogue, Inspect }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    NextStory,
    PreviousStory,
    NextCategory,
    PreviousCategory,
    Inspect,
    ToggleHelp,
    Escape,
    Quit,
    Ignore,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Exit { Continue, Quit }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorybookApp {
    selected: usize,
    mode: Mode,
    help_visible: bool,
}
```

`StorybookApp::new` selects index zero and asserts the catalogue is non-empty.
Expose `selected_index()`, `selected_story(stories)`,
`index_within_category(stories)`, `mode()`, `help_visible()` and
`select(index, stories)`. `select` clamps to `stories.len() - 1` and is used by
table-driven rendering and property tests.
Story movement searches only indices with the selected category and clamps.
Category movement uses `Category::ALL = [AssetAtlas, Widgets, FullScenes,
Compatibility]`, clamps, and selects the first story in the destination
category. `Escape` hides help first, returns from inspection second, and quits
from the base catalogue third. `Quit` always exits.

- [ ] **Step 4: Map only Storybook input**

Create `src/storybook/input.rs` with this match:

```rust
use crossterm::event::{Event, KeyCode, KeyModifiers};

use super::app::Action;

pub fn action_for_event(event: &Event) -> Action {
    let Event::Key(key) = event else { return Action::Ignore };
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Action::Quit;
    }
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Action::NextStory,
        KeyCode::Char('k') | KeyCode::Up => Action::PreviousStory,
        KeyCode::Char('l') | KeyCode::Right => Action::NextCategory,
        KeyCode::Char('h') | KeyCode::Left => Action::PreviousCategory,
        KeyCode::Enter => Action::Inspect,
        KeyCode::Char('?') => Action::ToggleHelp,
        KeyCode::Esc => Action::Escape,
        KeyCode::Char('q') => Action::Quit,
        _ => Action::Ignore,
    }
}
```

- [ ] **Step 5: Verify navigation tests**

Run:

```bash
cargo test --features storybook --test storybook_navigation
```

Expected: all navigation tests pass.

- [ ] **Step 6: Commit pure Storybook interaction**

```bash
git add src/storybook/app.rs src/storybook/input.rs src/storybook/mod.rs tests/storybook_navigation.rs
git commit -m "feat: add Storybook navigation state"
```

---

### Task 6: Three-pane shell and production rendering bridge

**Files:**
- Create: `src/storybook/ui.rs`
- Modify: `src/storybook/mod.rs`
- Modify: `tests/storybook_rendering.rs`

**Interfaces:**
- Consumes: `StorybookApp`, selected `Story`, `StoryFixture`, production `ui::render`, production `pixel::pack`.
- Produces: `render(frame, app, catalogue, context)`, `render_application_buffer(model, width, height)` and bounds-safe `blit`.

- [ ] **Step 1: Add failing shell and inspection rendering tests**

Append to `tests/storybook_rendering.rs`:

```rust
use questmancer::storybook::{app::{Action, StorybookApp, reduce}, ui as storybook_ui};
use ratatui::{Terminal, backend::TestBackend};

fn render_storybook(app: &StorybookApp, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|frame| {
        storybook_ui::render(frame, app, catalogue(), &StoryContext::fixed());
    }).unwrap();
    terminal.backend().buffer().content().iter()
        .map(|cell| cell.symbol())
        .collect::<String>()
}

#[test]
fn wide_shell_shows_catalogue_canvas_and_coverage() {
    let app = StorybookApp::new(catalogue());
    let screen = render_storybook(&app, 140, 40);
    assert!(screen.contains("QUESTMANCER STORYBOOK"));
    assert!(screen.contains("STORIES"));
    assert!(screen.contains("PRODUCTION CANVAS"));
    assert!(screen.contains("COVERAGE"));
    assert!(screen.contains("offline fixture realm"));
}

#[test]
fn inspection_hides_catalogue_chrome() {
    let stories = catalogue();
    let mut app = StorybookApp::new(stories);
    reduce(&mut app, Action::Inspect, stories);
    let screen = render_storybook(&app, 120, 36);
    assert!(!screen.contains("STORIES"));
    assert!(screen.contains("[esc] catalogue"));
}
```

- [ ] **Step 2: Run the tests and confirm the UI module is absent**

Run:

```bash
cargo test --features storybook --test storybook_rendering
```

Expected: FAIL because `storybook::ui` does not exist.

- [ ] **Step 3: Render application stories through an off-screen production terminal**

In `src/storybook/ui.rs`, implement:

```rust
fn render_application_buffer(model: &Model, width: u16, height: u16) -> Buffer {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("in-memory terminal is valid");
    terminal
        .draw(|frame| crate::ui::render(frame, model))
        .expect("in-memory render is infallible");
    terminal.backend().buffer().clone()
}

fn blit(source: &Buffer, target: &mut Buffer, area: Rect) {
    for y in 0..area.height.min(source.area.height) {
        for x in 0..area.width.min(source.area.width) {
            let Some(cell) = source.cell((x, y)).cloned() else { continue };
            if let Some(target_cell) = target.cell_mut((area.x + x, area.y + y)) {
                *target_cell = cell;
            }
        }
    }
}
```

This is the only application-scene bridge. It must call `crate::ui::render`,
not individual Guild Hall or Delve modules.

- [ ] **Step 4: Render atlas tiles through production packing**

For each visible tile, create a bordered rectangle using its preferred width
and height and render its label on the first row. Match `tile.content`:

```rust
match &tile.content {
    AtlasContent::Pixel { canvas, palette, background } => {
        let pixels = pixel::pack(canvas, palette, *background);
        frame.render_widget(Paragraph::new(pixels), content_area);
    }
    AtlasContent::AdventurerCard { agent, theatre, preferences } => {
        render_adventurer_card(frame, content_area, agent, *theatre, preferences);
    }
    AtlasContent::Chamber { agent, theatre, selected, preferences } => {
        render_chamber(frame, content_area, agent, *theatre, *selected, preferences);
    }
}
```

Compute columns as `max(1, area.width / maximum_preferred_width)` and rows by
ceiling division; clip tiles whose top is outside the canvas. Do not scale
logical pixels. This is how the full and compact widget stories exercise the
production widget functions rather than pixel-only substitutes.

Use a shared `render_fixture` match:

```rust
match (story.build)(context) {
    StoryFixture::Application(model) => {
        let source = render_application_buffer(&model, area.width, area.height);
        blit(&source, frame.buffer_mut(), area);
    }
    StoryFixture::AssetAtlas(atlas) => render_atlas(frame, area, &atlas.tiles),
}
```

- [ ] **Step 5: Build the responsive shell**

Use `Layout` with exact wide percentages `22/56/22` after reserving one header
and one footer row. At widths `80..=119`, use `30/70` and place story evidence
below the catalogue. Below 80 columns, show the canvas plus a one-line story
selector; `Tab` is not added because fixed keyboard navigation already changes
stories.

The header contains `QUESTMANCER STORYBOOK`, `offline fixture realm`, selected
reference viewport, character set, colour mode and motion. The right evidence
panel lists description, `owns`, `shows`, total owned count and validation
status. When the canvas is below the story minimum, render:

```text
This story needs at least {minimum_width}x{minimum_height}.
Canvas available: {actual_width}x{actual_height}.
```

Help is a centred overlay listing every key. Inspection renders only the
selected fixture and footer `[esc] catalogue  [?] help  [q] quit`.

- [ ] **Step 6: Verify wide, narrow, tiny and inspection output**

Run:

```bash
cargo test --features storybook --test storybook_rendering
cargo test --features storybook --test guild_hall_rendering --test delve_rendering
```

Expected: Storybook tests and all production rendering regressions pass.

- [ ] **Step 7: Commit the visual shell**

```bash
git add src/storybook/ui.rs src/storybook/mod.rs tests/storybook_rendering.rs
git commit -m "feat: render the Questmancer Storybook shell"
```

---

### Task 7: Complete fixed catalogue and coverage

**Files:**
- Modify: `src/storybook/catalogue.rs`
- Modify: `src/storybook/fixtures.rs`
- Modify: `src/storybook/atlas.rs`
- Modify: `tests/storybook_catalogue.rs`
- Modify: `tests/storybook_rendering.rs`

**Interfaces:**
- Consumes: all fixture and atlas builders from Tasks 3 and 4.
- Produces: complete ordered `catalogue() -> &'static [Story]` and zero-error `validate_catalogue()`.

- [ ] **Step 1: Add failing complete-catalogue assertions**

Append to `tests/storybook_catalogue.rs`:

```rust
use questmancer::storybook::{asset_inventory, catalogue::{catalogue, validate_catalogue}};

#[test]
fn production_catalogue_owns_every_authored_asset_once() {
    let report = validate_catalogue().unwrap();
    assert_eq!(report.owned(), asset_inventory().len());
    assert!(report.missing().is_empty());
    assert!(report.duplicates().is_empty());
    assert!(report.unknown().is_empty());
}

#[test]
fn story_ids_and_order_are_stable() {
    let ids = catalogue().iter().map(|story| story.id.as_str()).collect::<Vec<_>>();
    assert_eq!(ids.first(), Some(&"atlas.classes"));
    assert_eq!(ids.last(), Some(&"compat.motion-none"));
    assert_eq!(ids.len(), ids.iter().collect::<std::collections::HashSet<_>>().len());
}
```

- [ ] **Step 2: Run the complete coverage test and see missing assets**

Run:

```bash
cargo test --features storybook --test storybook_catalogue production_catalogue_owns_every_authored_asset_once -- --exact
```

Expected: FAIL with the coverage report naming widget, scene, Delve, goblin and compatibility assets not yet owned.

- [ ] **Step 3: Register every remaining fixed story**

After the 15 atlas stories, add entries in this exact order:

```text
widgets.adventurer-cards
widgets.chambers
widgets.guild-regions
widgets.counsel
widgets.search
widgets.help
scenes.guild-empty
scenes.guild-populated
scenes.guild-mixed-attention
scenes.guild-disconnected
scenes.guild-reconnecting
scenes.delve-library
scenes.delve-undercroft
scenes.delve-watchtower
scenes.connected-delves
scenes.mixed-state-delve
scenes.narrow-guild
scenes.narrow-delve
goblins.chest-eyes
goblins.chronicle-hand
goblins.rafters-scroll
goblins.stolen-biscuit
goblins.outbreak
compat.unicode-xterm256
compat.unicode-ansi16
compat.ascii-ansi16
compat.motion-full
compat.motion-reduced
compat.motion-none
```

`widgets.adventurer-cards`, `widgets.chambers` and `widgets.guild-regions` are
atlas fixtures built from the production card/chamber render inputs. Modal,
scene, goblin and compatibility stories are `Application(Model)` fixtures and
therefore render through `ui::render`.

Use tile dimensions `36x21` for the full adventurer card, `30x12` for its
compact path, `30x12` for a full chamber and `26x9` for its compact path. After
the atlas border is removed these dimensions cross the production widgets'
existing `34x19` and `28x10` thresholds exactly as intended.

Use these canonical ownership rules:

- the three named Delve scene stories own their `DelveVariant` assets;
- each goblin story owns its matching sighting or outbreak asset;
- grouped widget stories own every widget named by their title;
- each scene story owns its corresponding `SceneAsset`;
- each compatibility story owns its corresponding `CompatibilityAsset`;
- `shows` includes every reused class, ancestry, pose, widget or scene visible
  in that fixed fixture.

- [ ] **Step 4: Make validation part of catalogue construction**

Implement:

```rust
pub fn validate_catalogue() -> Result<CoverageReport, CoverageError> {
    validate_coverage(&asset_inventory(), catalogue())
}
```

Store the catalogue in `std::sync::OnceLock<Vec<Story>>` so fixed builder
registration occurs once without adding a dependency:

```rust
pub fn catalogue() -> &'static [Story] {
    static CATALOGUE: OnceLock<Vec<Story>> = OnceLock::new();
    CATALOGUE.get_or_init(build_catalogue).as_slice()
}
```

- [ ] **Step 5: Render every completed story at its reference viewport**

Add a table-driven test that loops over `catalogue()`, creates a
`TestBackend(story.viewport.reference_width, story.viewport.reference_height)`,
renders the full Storybook with that story selected, and asserts the resulting
buffer contains at least one non-space cell. Include the story ID in every
assertion message.

Run:

```bash
cargo test --features storybook --test storybook_catalogue --test storybook_rendering
```

Expected: all catalogue and rendering tests pass; coverage reports no issues.

- [ ] **Step 6: Commit the complete authored catalogue**

```bash
git add src/storybook tests/storybook_catalogue.rs tests/storybook_rendering.rs
git commit -m "feat: complete the Storybook catalogue"
```

---

### Task 8: Runtime loop and shared terminal restoration

**Files:**
- Modify: `src/terminal.rs:37-38,177-204,514-522`
- Create: `src/storybook/runtime.rs`
- Modify: `src/storybook/mod.rs`
- Modify: `src/bin/questmancer_storybook.rs`
- Modify: `tests/storybook_navigation.rs`

**Interfaces:**
- Consumes: shared `terminal::TerminalGuard::enter`, `EventStream`, `storybook::input`, `storybook::ui` and `validate_catalogue`.
- Produces: live `storybook::run`, clean `q`/Escape/Ctrl-C exits and terminal restoration on every return path.

- [ ] **Step 1: Add a failing terminal-guard drop test**

Inside `src/terminal.rs` unit tests, add:

```rust
#[test]
fn terminal_guard_runs_its_restore_action_on_drop() {
    use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
    let restored = Arc::new(AtomicBool::new(false));
    let guard = TerminalGuard::for_test(Arc::clone(&restored));
    drop(guard);
    assert!(restored.load(Ordering::SeqCst));
}
```

Run:

```bash
cargo test terminal_guard_runs_its_restore_action_on_drop --lib
```

Expected: FAIL because `TerminalGuard::for_test` does not exist.

- [ ] **Step 2: Make the existing guard reusable and observable in unit tests**

Change the terminal aliases and guard to crate-visible:

```rust
pub(crate) type Tui = Terminal<CrosstermBackend<Stdout>>;

#[derive(Debug)]
enum RestoreAction {
    Crossterm,
    #[cfg(test)]
    Probe(std::sync::Arc<std::sync::atomic::AtomicBool>),
}

#[derive(Debug)]
pub(crate) struct TerminalGuard {
    restore: RestoreAction,
}

impl TerminalGuard {
    pub(crate) fn enter() -> Result<(Self, Tui)> {
        enable_raw_mode()?;
        let guard = Self { restore: RestoreAction::Crossterm };
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture, Hide)?;
        Ok((guard, Terminal::new(CrosstermBackend::new(stdout))?))
    }

    #[cfg(test)]
    fn for_test(restored: std::sync::Arc<std::sync::atomic::AtomicBool>) -> Self {
        Self { restore: RestoreAction::Probe(restored) }
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        match &self.restore {
            RestoreAction::Crossterm => restore(),
            #[cfg(test)]
            RestoreAction::Probe(restored) => {
                restored.store(true, std::sync::atomic::Ordering::SeqCst);
            }
        }
    }
}
```

Keep the existing production `restore()` and panic hook unchanged.

- [ ] **Step 3: Implement the Herdr-free event loop**

Create `src/storybook/runtime.rs`:

```rust
pub async fn run() -> anyhow::Result<()> {
    validate_catalogue().map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let stories = catalogue();
    let context = StoryContext::fixed();
    let mut app = StorybookApp::new(stories);
    let (_guard, mut terminal) = TerminalGuard::enter()?;
    let mut events = EventStream::new();

    loop {
        terminal.draw(|frame| ui::render(frame, &app, stories, &context))?;
        tokio::select! {
            event = events.next() => {
                let Some(event) = event else { break };
                let action = input::action_for_event(&event.context("read Storybook input")?);
                if reduce(&mut app, action, stories) == Exit::Quit { break; }
            }
            result = tokio::signal::ctrl_c() => {
                result.context("install Storybook Ctrl-C handler")?;
                break;
            }
        }
    }
    Ok(())
}
```

Import `futures_util::StreamExt` and `anyhow::Context`. Replace the temporary
entrypoint in `storybook/mod.rs` with `pub use runtime::run`.

- [ ] **Step 4: Verify restoration and runtime compilation**

Run:

```bash
cargo test terminal_guard_runs_its_restore_action_on_drop --lib
cargo test --features storybook --test storybook_navigation
cargo check --features storybook --bin questmancer-storybook
```

Expected: all commands pass. The binary links no direct Herdr or persistence call from `src/storybook`.

- [ ] **Step 5: Commit the standalone runtime**

```bash
git add src/terminal.rs src/storybook/runtime.rs src/storybook/mod.rs src/bin/questmancer_storybook.rs tests/storybook_navigation.rs
git commit -m "feat: run Storybook without Herdr"
```

---

### Task 9: Property tests, documentation and release boundary

**Files:**
- Create: `tests/storybook_properties.rs`
- Modify: `README.md`
- Modify: `tests/scripts.sh`

**Interfaces:**
- Consumes: completed `catalogue`, `StorybookApp`, reducer and renderer.
- Produces: terminal-size safety properties, user-facing developer instructions and a shell assertion that Storybook is absent from the plugin manifest.

- [ ] **Step 1: Write dimension and navigation properties**

Create `tests/storybook_properties.rs`:

```rust
#![cfg(feature = "storybook")]

use proptest::prelude::*;
use questmancer::storybook::{
    app::{Action, StorybookApp, reduce},
    catalogue::catalogue,
    fixtures::StoryContext,
    ui,
};
use ratatui::{Terminal, backend::TestBackend};

proptest! {
    #[test]
    fn every_story_renders_for_any_terminal_size(
        width in 1_u16..180,
        height in 1_u16..60,
        story_index in any::<usize>(),
    ) {
        let stories = catalogue();
        let mut app = StorybookApp::new(stories);
        app.select(story_index % stories.len(), stories);
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| ui::render(frame, &app, stories, &StoryContext::fixed())).unwrap();
    }

    #[test]
    fn arbitrary_navigation_keeps_selection_valid(actions in prop::collection::vec(0_u8..8, 0..200)) {
        let stories = catalogue();
        let mut app = StorybookApp::new(stories);
        for value in actions {
            let action = match value {
                0 => Action::NextStory,
                1 => Action::PreviousStory,
                2 => Action::NextCategory,
                3 => Action::PreviousCategory,
                4 => Action::Inspect,
                5 => Action::ToggleHelp,
                6 => Action::Escape,
                _ => Action::Ignore,
            };
            let _ = reduce(&mut app, action, stories);
            prop_assert!(app.selected_index() < stories.len());
        }
    }
}
```

- [ ] **Step 2: Run properties at the normal and elevated case counts**

Run:

```bash
cargo test --features storybook --test storybook_properties
PROPTEST_CASES=1024 cargo test --features storybook --test storybook_properties
```

Expected: both commands pass without a regression seed file being created.

- [ ] **Step 3: Document the developer workflow**

Add a `Developer Storybook` section to `README.md` with this prose and the two
commands as ordinary fenced Bash blocks:

```text
Developer Storybook

Review Questmancer's sprites, widgets, fixed Guild Hall scenes, Delve variants
and compatibility modes without starting Herdr by running `just storybook`.

The Storybook is a developer-only Cargo feature. It reads no Herdr environment,
connects to no socket and writes no plugin state.

Use j/k to move between stories, h/l to change categories, Enter to inspect the
production canvas, Esc to return, ? for help and q to quit.

Run its focused automated checks with `just storybook-test`.
```

Render `just storybook` and `just storybook-test` as separate `bash` code fences
immediately after the sentences that introduce them.

- [ ] **Step 4: Protect the plugin release surface**

Append assertions to `tests/scripts.sh` after the manifest checks:

```bash
if grep -R -E -q 'questmancer-storybook|storybook' herdr-plugin.toml herdr; then
  echo "developer Storybook leaked into the plugin release surface" >&2
  exit 1
fi
```

Do not modify `herdr-plugin.toml`, `herdr/install.sh`, `herdr/run.sh`,
`herdr/control.sh` or `.github/workflows/release.yml`.

- [ ] **Step 5: Run the full verification matrix**

Run fresh, in this order:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
bash tests/scripts.sh
cargo build --release
git diff --check
git status --short --branch
```

Expected:

- formatting and Clippy exit 0;
- every unit, integration and property test passes;
- shell release-surface tests pass;
- the normal release build produces `target/release/questmancer` without requiring the Storybook feature;
- `git diff --check` reports no whitespace errors;
- status lists only the intended Task 9 files before commit.

- [ ] **Step 6: Manually launch the fixed Storybook**

Run in an interactive terminal with no Herdr-specific variables required:

```bash
env -u HERDR_SOCKET_PATH -u HERDR_PLUGIN_STATE_DIR -u HERDR_PLUGIN_CONFIG_DIR just storybook
```

Verify every catalogue category is reachable, Enter opens inspection, Esc
returns, resizing remains readable, and `q` returns to a normal visible shell
prompt with the cursor restored.

- [ ] **Step 7: Commit the completed developer workflow**

```bash
git add README.md tests/storybook_properties.rs tests/scripts.sh
git commit -m "test: verify Questmancer Storybook assets"
```

---

## Final review checkpoint

After Task 9, invoke `superpowers:requesting-code-review`. The reviewer must
compare the implementation against
`docs/superpowers/specs/2026-07-16-questmancer-storybook-design.md`, inspect the
complete asset ownership report, and verify that no Storybook symbol appears in
the Herdr manifest, install scripts or release assets. Address findings through
`superpowers:receiving-code-review`, rerun the complete verification matrix,
and only then use `superpowers:finishing-a-development-branch` to integrate the
work locally.
