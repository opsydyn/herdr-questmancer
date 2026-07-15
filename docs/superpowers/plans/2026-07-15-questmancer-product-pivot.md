# Questmancer Product Pivot Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Transform the existing, tested `webmaster` plugin into Questmancer: a cozy adventurers' guild with an operational Guild Hall, a connected dungeon-crawler Delve, stable fantasy adventurers, and contained goblin Easter eggs.

**Architecture:** Perform a hard product-identity cutover while retaining the proven Herdr socket client, event adapter, reducer, command executor, async runtime, persistence worker, and accessibility infrastructure. Keep host facts neutral (`Agent`, `WorkspaceId`, `PaneId`, `Presence`); rename only product projections and user-facing effects (`Campaign`, `GuildAttention`, `Chronicle`, `AdventurerPersona`, `Guild Hall`, `Delve`). The new plugin ID creates a clean persistence namespace, so Questmancer starts with state schema version 1 and does not read or mutate Webmaster state.

**Tech Stack:** Rust 1.90 / edition 2024, Ratatui 0.30, Crossterm 0.29, Tokio, Serde, Blake3, Proptest, shell lifecycle scripts, Herdr 0.7.4 / protocol 16.

## Global Constraints

- The approved creative canon is `docs/superpowers/specs/2026-07-15-questmancer-creative-direction.md`.
- Product name: `Questmancer`; repository: `herdr-questmancer`; plugin ID: `opsydyn.questmancer`; binary: `questmancer`; pane entrypoint: `guild-hall`.
- Primary views are `guild` and `delve`; remove `desk` and `cafe` aliases rather than maintaining two product vocabularies.
- Target Herdr 0.7.4 and protocol 16; verify request and event shapes against the installed schema before changing protocol code.
- Preserve separate presence and user-attention domains.
- Preserve the pure reducer/effect boundary, two logical socket connections, lazy selected-output reads, reconnect/resnapshot behavior, managed-pane exclusion, and debounced local persistence.
- Do not infer progress, enemies, combat, collaboration, or task semantics from terminal output.
- Do not add a separate Questmancer mascot, narrator agent, theme framework, image protocol, sound, telemetry, cloud service, database, or Git analysis.
- Keep Herdr 0.7.4 customizable-sidebar publishing out of this pivot; rebase the existing sidebar plan onto Questmancer after product names and persona metadata stabilize.
- A pane exit maps to `Departed`; do not implement `Downed` until Herdr exposes a distinct recoverable failure fact.
- Classic fantasy classes are canonical; custom Questmancer classes may be added from curated fixed lists.
- Goblins remain rare, original, non-semantic and non-obstructive.
- Unicode is canonical; ASCII, ANSI-16, reduced-motion, no-motion, 80x24, tiny and zero-sized surfaces remain supported.
- No unsafe Rust.

---

## File and boundary map

The pivot deliberately leaves these host/runtime boundaries structurally intact:

- `src/herdr/**`: protocol, request, subscription and reconnect behavior.
- `src/runtime_loop.rs`, `src/terminal.rs`: async orchestration and animation scheduling.
- `src/update/reducer.rs`: pure semantic transitions.
- `src/persistence/worker.rs`, `src/persistence/atomic_json.rs`: durable write mechanics.
- `src/ui/pixel/{canvas,pack,palette}.rs`: terminal pixel primitives.

Rename or replace the product-facing boundaries:

```text
src/domain/site.rs                    -> src/domain/campaign.rs
src/domain/guestbook.rs               -> src/domain/chronicle.rs
src/persistence/guestbook_jsonl.rs    -> src/persistence/chronicle_jsonl.rs
src/ui/cafe_scene.rs                  -> src/ui/delve_scene.rs
src/ui/views/desk.rs                  -> src/ui/views/guild_hall.rs
src/ui/views/cafe.rs                  -> src/ui/views/delve.rs
src/ui/widgets/agent_crt.rs           -> src/ui/widgets/chamber.rs
src/ui/widgets/profile_card.rs        -> src/ui/widgets/adventurer_card.rs
tests/guestbook.rs                     -> tests/chronicle.rs
tests/guestbook_persistence.rs         -> tests/chronicle_persistence.rs
tests/desk_rendering.rs                -> tests/guild_hall_rendering.rs
tests/cafe_scene.rs                    -> tests/delve_scene.rs
tests/cafe_widgets.rs                  -> tests/delve_widgets.rs
tests/cafe_rendering.rs                -> tests/delve_rendering.rs
```

Add focused modules instead of scattering copy and Easter-egg state:

```text
src/ui/copy.rs          approved operational and diagnostic wording
src/ui/goblins.rs       deterministic sightings and one-shot outbreak projection
tests/goblins.rs        containment, determinism and scheduler coverage
```

---

### Task 1: Cut over package, plugin and lifecycle identity

**Files:**
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `herdr-plugin.toml`
- Modify: `src/cli.rs`
- Modify: `src/app.rs`
- Modify: `src/ui/input.rs`
- Modify: `src/ui/mod.rs`
- Modify: `src/ui/theatre.rs`
- Modify: `herdr/run.sh`
- Modify: `herdr/control.sh`
- Modify: `herdr/install.sh`
- Modify: `tests/cli.rs`
- Modify: `tests/scripts.sh`
- Modify: every Rust integration-test import from `herdr_webmaster` to `questmancer`
- Create: `.gitignore`

**Interfaces:**
- Consumes: existing `Model`, runtime, shell singleton protocol and plugin manifest structure.
- Produces: crate `questmancer`, binary `questmancer`, `View::{Guild, Delve}`, plugin actions `open|close|toggle|guild|delve`, and environment variable `QUESTMANCER_INITIAL_VIEW`.

- [ ] **Step 1: Change CLI and shell tests first**

Replace `tests/cli.rs` with the new public contract:

```rust
use clap::Parser;
use questmancer::{
    app::View,
    cli::{Cli, Command},
};

#[test]
fn omits_the_initial_view_by_default() {
    let cli = Cli::try_parse_from(["questmancer", "ui"]).expect("valid CLI");
    assert_eq!(cli.command, Command::Ui { view: None });
}

#[test]
fn accepts_guild_and_delve_as_initial_views() {
    for (value, expected) in [("guild", View::Guild), ("delve", View::Delve)] {
        let cli = Cli::try_parse_from(["questmancer", "ui", "--view", value])
            .expect("valid CLI");
        assert_eq!(cli.command, Command::Ui { view: Some(expected) });
    }
}
```

Update `tests/scripts.sh` expectations to use:

```text
bin/questmancer
target/release/questmancer
target/debug/questmancer
QUESTMANCER_INITIAL_VIEW=guild|delve
opsydyn.questmancer
--entrypoint guild-hall
control.sh open|close|toggle|guild|delve
```

- [ ] **Step 2: Run the focused tests and observe the identity failures**

Run:

```bash
cargo test --test cli
bash tests/scripts.sh
```

Expected: FAIL because the crate, binary, views, actions and script paths still use Webmaster names.

- [ ] **Step 3: Apply the package and application identity**

Set the package metadata in `Cargo.toml`:

```toml
[package]
name = "questmancer"
version = "0.1.0"
edition = "2024"
rust-version = "1.90"
description = "A cozy fantasy guild and dungeon control room for coding agents."
license = "MIT"
repository = "https://github.com/opsydyn/herdr-questmancer"
readme = "README.md"
keywords = ["herdr", "ratatui", "agents", "tui", "fantasy"]
categories = ["command-line-utilities", "development-tools"]
```

Replace the view enum in `src/app.rs`:

```rust
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum View {
    #[default]
    Guild,
    Delve,
}
```

Change view switches in `src/ui/input.rs` and `src/ui/mod.rs`:

```rust
KeyCode::Char('1') | KeyCode::F(1) => Action::Switch(View::Guild),
KeyCode::Char('2') | KeyCode::F(2) => Action::Switch(View::Delve),
```

```rust
match model.view() {
    View::Guild => views::desk::render(frame, model),
    View::Delve => views::cafe::render(frame, model),
}
```

Keep the old module filenames temporarily in this task so the crate remains
buildable. Tasks 4 and 5 rename the modules when their projections are replaced.

Update `src/cli.rs`:

```rust
#[derive(Clone, Debug, Eq, Parser, PartialEq)]
#[command(name = "questmancer", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Debug, Eq, PartialEq, Subcommand)]
pub enum Command {
    /// Open the interactive Questmancer interface.
    Ui {
        /// Initial view to display.
        #[arg(long, value_enum)]
        view: Option<View>,
    },
}
```

- [ ] **Step 4: Apply the manifest and lifecycle identity**

Replace `herdr-plugin.toml` with:

```toml
id = "opsydyn.questmancer"
name = "questmancer"
version = "0.1.0"
min_herdr_version = "0.7.4"
platforms = ["macos", "linux"]
description = "A cozy fantasy guild and dungeon control room for coding agents."

[[build]]
command = ["bash", "herdr/install.sh"]

[[panes]]
id = "guild-hall"
title = "Questmancer"
placement = "tab"
command = ["bash", "herdr/run.sh", "ui"]

[[actions]]
id = "toggle"
title = "Questmancer: toggle"
contexts = ["pane", "workspace"]
command = ["bash", "herdr/control.sh", "toggle"]

[[actions]]
id = "open"
title = "Questmancer: open"
contexts = ["pane", "workspace"]
command = ["bash", "herdr/control.sh", "open"]

[[actions]]
id = "close"
title = "Questmancer: close"
contexts = ["pane", "workspace"]
command = ["bash", "herdr/control.sh", "close"]

[[actions]]
id = "guild"
title = "Questmancer: enter the Guild Hall"
contexts = ["pane", "workspace"]
command = ["bash", "herdr/control.sh", "guild"]

[[actions]]
id = "delve"
title = "Questmancer: enter the Delve"
contexts = ["pane", "workspace"]
command = ["bash", "herdr/control.sh", "delve"]
```

In the shell scripts, use these exact constants and paths:

```bash
PLUGIN_ID=${HERDR_PLUGIN_ID:-opsydyn.questmancer}
ENTRYPOINT=guild-hall
INITIAL_VIEW_ENV=QUESTMANCER_INITIAL_VIEW
REPOSITORY=${QUESTMANCER_REPOSITORY:-opsydyn/herdr-questmancer}
archive="questmancer-v$VERSION-$target.tar.gz"
```

`herdr/run.sh` must search `bin/questmancer`, `target/release/questmancer`, then `target/debug/questmancer`. `herdr/control.sh` must send key `1` for `guild`, key `2` for `delve`, pass `--env QUESTMANCER_INITIAL_VIEW=guild` for a newly opened `guild` action, and pass `--env QUESTMANCER_INITIAL_VIEW=delve` for a newly opened `delve` action.

- [ ] **Step 5: Update crate imports and lockfile mechanically**

Run:

```bash
rg -l -0 'herdr_webmaster' tests src | xargs -0 perl -pi -e 's/herdr_webmaster/questmancer/g'
cargo check --all-targets
```

Review every changed import; do not use a broad replacement for user-facing copy in this task.

Create `.gitignore`:

```gitignore
/target/
/bin/
/.superpowers/
```

- [ ] **Step 6: Verify the executable shell**

Run:

```bash
cargo test --test cli --test app --test input
bash tests/scripts.sh
bash -n herdr/install.sh herdr/run.sh herdr/control.sh
cargo run -- ui --view guild
cargo run -- ui --view delve
```

Expected: tests pass; both views start in offline mode; no `desk` or `cafe` value is accepted.

- [ ] **Step 7: Commit**

```bash
git add .gitignore Cargo.toml Cargo.lock herdr-plugin.toml herdr src tests
git commit -m "refactor: cut over Questmancer product identity"
```

---

### Task 2: Establish campaign, summons and Chronicle vocabulary

**Files:**
- Rename: `src/domain/site.rs` -> `src/domain/campaign.rs`
- Rename: `src/domain/guestbook.rs` -> `src/domain/chronicle.rs`
- Modify: `src/domain/{mod,state,attention,agent}.rs`
- Rename: `src/persistence/guestbook_jsonl.rs` -> `src/persistence/chronicle_jsonl.rs`
- Modify: `src/persistence/{mod,startup,state,worker}.rs`
- Modify: `src/update/{event,reducer}.rs`
- Modify: `src/command.rs`
- Modify: `src/config.rs`
- Modify: `src/interaction.rs`
- Modify: `src/app.rs`
- Modify: `src/runtime_loop.rs`
- Modify: `src/terminal.rs`
- Modify: `src/ui/cafe_scene.rs`
- Modify: `src/ui/theatre.rs`
- Modify: `src/ui/views/{cafe,desk}.rs`
- Modify: `src/ui/widgets/profile_card.rs`
- Rename: `tests/guestbook.rs` -> `tests/chronicle.rs`
- Rename: `tests/guestbook_persistence.rs` -> `tests/chronicle_persistence.rs`
- Modify: `tests/{app,cafe_rendering,cafe_scene,cafe_widgets,command,config,desk_rendering,domain_types,interaction,normalization,persisted_state,persistence_worker,property_domain,reducer,runtime_loop,startup,theatre}.rs`
- Modify: `tests/support/{mod,strategies}.rs`

**Interfaces:**
- Consumes: neutral `Agent`, `AgentKey`, `WorkspaceId`, `Presence`, `PaneId`, `Timestamp` and reducer boundaries.
- Produces: `Campaign`, `CampaignStatus`, `GuildAttention`, `GuildSummons`, `Chronicle`, `ChronicleEntry`, `ChronicleEvent`, `AgentCommand::SendCounsel`, and `chronicle.jsonl`.

- [ ] **Step 1: Rename the tests and express the new domain contract**

Use `git mv` for the four files, then change the focused assertions to require:

```rust
let campaign = Campaign {
    workspace_id: WorkspaceId::new("w1"),
    label: "Schema Runes".to_owned(),
    cwd: "/tmp/schema-runes".into(),
    party: agents.keys().cloned().collect(),
};
assert_eq!(campaign.status(&agents), CampaignStatus::CounselRequired);
assert!(matches!(
    adventurer.attention,
    GuildAttention::Unread {
        summons: GuildSummons::CounselRequested,
        ..
    }
));
assert_eq!(chronicle.entries().back().unwrap().event, ChronicleEvent::CounselRequested);
```

Update persistence path assertions:

```rust
assert_eq!(
    paths.chronicle_path().unwrap(),
    Path::new("/tmp/state/chronicle.jsonl")
);
```

- [ ] **Step 2: Run the focused domain and persistence tests**

Run:

```bash
cargo test --test domain_types --test normalization --test reducer --test chronicle --test chronicle_persistence --test persisted_state --test startup
```

Expected: FAIL because the new types and path do not exist.

- [ ] **Step 3: Introduce the exact product projections**

Define `Campaign` and `CampaignStatus` in `src/domain/campaign.rs`:

```rust
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Campaign {
    pub workspace_id: WorkspaceId,
    pub label: String,
    pub cwd: PathBuf,
    pub party: Vec<AgentKey>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CampaignStatus {
    CounselRequired,
    SpoilsAwaitingInspection,
    ExpeditionActive,
    PartyAtRest,
    Abandoned,
}
```

Derive status in this priority order:

```rust
CounselRequired
SpoilsAwaitingInspection
ExpeditionActive
PartyAtRest
Abandoned
```

Replace attention types in `src/domain/attention.rs`:

```rust
pub enum GuildAttention {
    Clear,
    Unread { summons: GuildSummons, since: Timestamp },
    Read { summons: GuildSummons, since: Timestamp },
    Deferred { summons: GuildSummons, since: Timestamp, until: Timestamp },
}

pub enum GuildSummons {
    CounselRequested,
    SpoilsReturned,
    AdventurerDeparted,
}
```

Map Herdr states without changing `Presence`:

```text
Blocked -> CounselRequested
Done    -> SpoilsReturned
Exited  -> AdventurerDeparted
```

Define Chronicle types in `src/domain/chronicle.rs`:

```rust
pub struct ChronicleEntry {
    pub id: EventId,
    pub occurred_at: Timestamp,
    pub adventurer: Option<AgentKey>,
    pub campaign: Option<WorkspaceId>,
    pub pane: Option<PaneId>,
    pub pane_revision: u64,
    pub event: ChronicleEvent,
    pub summary: String,
}

pub enum ChronicleEvent {
    AdventurerJoined,
    DelveBegan,
    CounselRequested,
    SpoilsReturned,
    AdventurerRested,
    AdventurerDeparted,
    CampaignClosed,
}
```

Keep the existing event identity algorithm unchanged except for the serialized event names.

- [ ] **Step 4: Rename state and command ownership**

Update `DomainState`:

```rust
pub struct DomainState {
    pub campaigns: BTreeMap<WorkspaceId, Campaign>,
    pub agents: BTreeMap<AgentKey, Agent>,
    pub selected_agent: Option<AgentKey>,
    pub chronicle: Chronicle,
}
```

Rename shared external commands in `src/command.rs`:

```rust
pub enum AgentCommand {
    FocusPane(PaneId),
    SendCounsel { pane_id: PaneId, text: String },
    LoadOutput { pane_id: PaneId, lines: u32 },
    RefreshSnapshot,
    DiscoverReviewr { qualified_id: String },
    InspectSpoils { pane_id: PaneId, qualified_id: String },
}
```

Rename results `ReplySent` to `CounselSent` and `ReviewrOpened` to `SpoilsOpened`. Keep wire operations and error details unchanged.

- [ ] **Step 5: Cut persistence over to Questmancer names**

Rename configuration and paths:

```rust
pub struct QuestmancerConfig {
    pub default_view: View,
    pub preferences: DisplayPreferences,
    pub output_preview_lines: u32,
    pub chronicle_max_entries: usize,
    pub reviewr_action: String,
    pub show_elapsed_time: bool,
}
```

`PersistencePaths::chronicle_path()` must return `chronicle.jsonl`. Keep `state.json`, `runtime.json`, schema version 1, atomic replacement, invalid-state protection, debounce and shutdown flush unchanged. Rename internal worker commands and diagnostics from append/replay guestbook to append/replay Chronicle.

- [ ] **Step 6: Verify invariants and property tests**

Run:

```bash
cargo test --all-targets --all-features
PROPTEST_CASES=1024 cargo test --test property_domain
```

Expected: PASS; all temporary desk/café projections compile against the new product-facing types while their visual rewrite remains deferred, and campaign status priority, attention idempotence, Chronicle deduplication, selection safety and managed-pane exclusion remain proven.

- [ ] **Step 7: Commit**

```bash
git add src/app.rs src/command.rs src/config.rs src/domain src/interaction.rs src/persistence src/runtime_loop.rs src/terminal.rs src/ui src/update tests
git commit -m "refactor: project Herdr state as a Questmancer guild"
```

---

### Task 3: Generate stable fantasy adventurers

**Files:**
- Modify: `src/domain/persona.rs`
- Modify: `src/domain/mod.rs`
- Modify: `src/persistence/state.rs`
- Modify: `src/ui/persona/appearance.rs`
- Modify: `src/ui/persona/cafe_sprite.rs`
- Modify: `src/ui/persona/profile.rs`
- Modify: `src/ui/persona/mod.rs`
- Modify: `src/ui/views/desk.rs`
- Modify: `src/ui/widgets/{agent_crt,profile_card}.rs`
- Modify: `src/app.rs`
- Modify: `src/interaction.rs`
- Modify: `src/update/reducer.rs`
- Modify: `tests/persona.rs`
- Modify: `tests/persona_art.rs`
- Modify: `tests/{cafe_widgets,interaction,normalization,persistence_worker,reducer,runtime_loop,startup}.rs`
- Modify: `tests/persisted_state.rs`
- Modify: `tests/property_domain.rs`
- Modify: `tests/support/strategies.rs`

**Interfaces:**
- Consumes: existing `PersonaKey::for_agent` identity precedence and Blake3 labelled hashing.
- Produces: `AdventurerPersona { key, name, ancestry, class, epithet, appearance }`, classic and curated custom classes, deterministic equipment, and rare Goblin ancestry.

- [ ] **Step 1: Write persona behavior tests**

Add tests for the public persona contract:

```rust
#[test]
fn persona_generation_is_stable_and_independent_of_pane_moves() {
    let original = fixture_agent();
    let mut moved_agent = original.clone();
    moved_agent.pane_id = "w1:p9".to_owned();
    let first = AdventurerPersona::for_agent(&original, Some("/repo"));
    let moved = AdventurerPersona::for_agent(&moved_agent, Some("/repo"));
    assert_eq!(first, moved);
    assert!(!first.name.trim().is_empty());
    assert!(!first.epithet.as_str().trim().is_empty());
}

#[test]
fn classic_and_questmancer_classes_are_reachable() {
    let classes = (0..4096)
        .map(|index| AdventurerPersona::for_key(
            PersonaKey::new(format!("persona-{index}")),
        ).class)
        .collect::<BTreeSet<_>>();
    assert!(classes.contains(&AdventurerClass::Wizard));
    assert!(classes.contains(&AdventurerClass::Rogue));
    assert!(classes.contains(&AdventurerClass::Cleric));
    assert!(classes.contains(&AdventurerClass::Runewright));
    assert!(classes.contains(&AdventurerClass::Testmender));
}

#[test]
fn goblins_are_possible_but_rare() {
    let goblins = (0..16_384)
        .filter(|index| AdventurerPersona::for_key(
            PersonaKey::new(format!("persona-{index}")),
        ).ancestry == Ancestry::Goblin)
        .count();
    assert!(goblins > 0);
    assert!(goblins < 256);
}
```

- [ ] **Step 2: Run persona tests and observe the missing fantasy model**

Run:

```bash
cargo test --test persona --test persona_art --test persisted_state --test property_domain
```

Expected: FAIL because `AdventurerPersona`, ancestry, classes and epithets are undefined.

- [ ] **Step 3: Define the semantic persona types**

Replace `AgentPersona` with:

```rust
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdventurerPersona {
    pub key: PersonaKey,
    pub name: String,
    pub ancestry: Ancestry,
    pub class: AdventurerClass,
    pub epithet: Epithet,
    pub appearance: PersonaAppearance,
}

impl AdventurerPersona {
    pub fn for_key(key: PersonaKey) -> Self {
        const COMMON_ANCESTRIES: [Ancestry; 6] = [
            Ancestry::Human, Ancestry::Dwarf, Ancestry::Elf,
            Ancestry::Halfling, Ancestry::Orc, Ancestry::Gnome,
        ];
        const CLASSES: [AdventurerClass; 11] = [
            AdventurerClass::Barbarian, AdventurerClass::Bard,
            AdventurerClass::Cleric, AdventurerClass::Paladin,
            AdventurerClass::Ranger, AdventurerClass::Rogue,
            AdventurerClass::Wizard, AdventurerClass::Artificer,
            AdventurerClass::Runewright, AdventurerClass::Testmender,
            AdventurerClass::Pathseeker,
        ];
        const FIRST_NAMES: [&str; 12] = [
            "Elowen", "Merrin", "Arnoldus", "Pius", "Rowan", "Tamsin",
            "Brindle", "Nessa", "Orin", "Sabine", "Alder", "Lyra",
        ];
        const BYNAMES: [&str; 12] = [
            "Typeweaver", "Ironjaw", "Manytools", "Blackquill",
            "Brightward", "Mossfoot", "Runehand", "Mapkeeper",
            "Copperkettle", "Longpath", "Softstep", "Embercloak",
        ];
        const EPITHETS: [&str; 8] = [
            "Keeper of Schemas", "Mender of Tests", "Walker of Worktrees",
            "Delver of Forgotten Modules", "Breaker of Builds",
            "Reader of Runes", "Warden of Boundaries",
            "Cartographer of Call Stacks",
        ];
        let digest = labelled_hash(key.as_str(), "adventurer");
        let ancestry = if digest[0] == 0 {
            Ancestry::Goblin
        } else {
            COMMON_ANCESTRIES[usize::from(digest[0] - 1) % COMMON_ANCESTRIES.len()]
        };
        Self {
            name: format!(
                "{} {}",
                FIRST_NAMES[usize::from(digest[1]) % FIRST_NAMES.len()],
                BYNAMES[usize::from(digest[2]) % BYNAMES.len()],
            ),
            ancestry,
            class: CLASSES[usize::from(digest[3]) % CLASSES.len()],
            epithet: Epithet(
                EPITHETS[usize::from(digest[4]) % EPITHETS.len()].to_owned()
            ),
            appearance: Self::appearance_for_key(&key),
            key,
        }
    }

    pub fn for_agent(agent: &AgentInfo, workspace_root: Option<&str>) -> Self {
        Self::for_key(PersonaKey::for_agent(agent, workspace_root))
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Ancestry {
    Human,
    Dwarf,
    Elf,
    Halfling,
    Orc,
    Gnome,
    Goblin,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdventurerClass {
    Barbarian,
    Bard,
    Cleric,
    Paladin,
    Ranger,
    Rogue,
    Wizard,
    Artificer,
    Runewright,
    Testmender,
    Pathseeker,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Epithet(String);

impl Epithet {
    pub fn new(value: impl Into<String>) -> Self { Self(value.into()) }
    pub fn as_str(&self) -> &str { &self.0 }
}

fn labelled_hash(key: &str, label: &str) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(label.as_bytes());
    hasher.update(&[0]);
    hasher.update(key.as_bytes());
    *hasher.finalize().as_bytes()
}
```

Move the existing `appearance_for_key` method onto `AdventurerPersona` and update its selected traits in Step 4. Use labelled digest bytes for independent choices. Assign Goblin only when the ancestry byte equals zero; use modulo selection across the six common ancestries otherwise. Generate names and epithets from curated arrays, never from agent output and never from wall-clock or insertion order.

- [ ] **Step 4: Replace 90s appearance traits with fantasy equipment traits**

Keep geometry-bearing traits such as proportions, head shape, skin and hair. Replace the five 90s wardrobe fields with four deterministic appearance fields, and derive primary gear from class:

```rust
trait_enum!(Garb { Armour, Cloak, Doublet, Leathers, Robes, Vestments, WorkApron });
trait_enum!(Legwear { BootsAndBreeches, Greaves, RobeHem, TravelingSkirt });
trait_enum!(Footwear { Boots, Sabatons, Sandals, SoftShoes });
trait_enum!(Keepsake { Feather, LuckyCoin, Mug, PressedLeaf, Ribbon, TinyFamiliar });
trait_enum!(AdventuringGear {
    Axe,
    BowAndQuiver,
    HolySymbol,
    Lute,
    MapAndCompass,
    RuneChisel,
    Shield,
    SpellbookAndStaff,
    TestKit,
    ThievesTools,
    Toolkit,
});

impl AdventurerClass {
    pub const fn gear(self) -> AdventuringGear {
        match self {
            Self::Barbarian => AdventuringGear::Axe,
            Self::Bard => AdventuringGear::Lute,
            Self::Cleric => AdventuringGear::HolySymbol,
            Self::Paladin => AdventuringGear::Shield,
            Self::Ranger => AdventuringGear::BowAndQuiver,
            Self::Rogue => AdventuringGear::ThievesTools,
            Self::Wizard => AdventuringGear::SpellbookAndStaff,
            Self::Artificer => AdventuringGear::Toolkit,
            Self::Runewright => AdventuringGear::RuneChisel,
            Self::Testmender => AdventuringGear::TestKit,
            Self::Pathseeker => AdventuringGear::MapAndCompass,
        }
    }
}
```

`PersonaAppearance` must expose `garb`, `legwear`, `footwear` and `keepsake` in place of `top`, `bottom`, `shoes`, `accessory` and `desk_prop`; no independently random gear field remains. Class controls the primary gear silhouette through `AdventurerClass::gear`; the persona digest controls colour and non-semantic keepsakes. Update the existing sprite composers with exhaustive temporary mappings for every new enum so this commit remains green; Task 6 replaces those transitional shapes with the approved fantasy art. Do not claim that class or gear describes measured ability.

- [ ] **Step 5: Update durable intent and property strategies**

Persist the complete `AdventurerPersona` against the unchanged `PersonaKey`. Update test strategies to generate every enum without creating mismatched keys. Keep validation rules for selected personas and seen summons unchanged.

- [ ] **Step 6: Verify deterministic identity**

Run:

```bash
cargo test --all-targets --all-features
PROPTEST_CASES=2048 cargo test --test property_domain persona_key_and_appearance_are_deterministic
```

Expected: PASS; generated combinations are deterministic, non-empty, bounded and stable under pane moves.

- [ ] **Step 7: Commit**

```bash
git add src/app.rs src/domain src/interaction.rs src/persistence/state.rs src/ui src/update/reducer.rs tests
git commit -m "feat: generate stable fantasy adventurers"
```

---

### Task 4: Build the operational Guild Hall

**Files:**
- Rename: `src/ui/views/desk.rs` -> `src/ui/views/guild_hall.rs`
- Rename: `src/ui/widgets/profile_card.rs` -> `src/ui/widgets/adventurer_card.rs`
- Modify: `src/ui/views/mod.rs`
- Modify: `src/ui/widgets/mod.rs`
- Modify: `src/app.rs`
- Create: `src/ui/copy.rs`
- Modify: `src/ui/mod.rs`
- Rename: `tests/desk_rendering.rs` -> `tests/guild_hall_rendering.rs`
- Modify: `tests/{rendering,reply,interaction,runtime_loop}.rs`

**Interfaces:**
- Consumes: campaigns, adventurer personas, summons, Chronicle, selected output and existing typed actions.
- Produces: responsive Guild Hall with Quest Board, Party Roster, Calls for Counsel, Scrying Table, Spoils Desk and Chronicle.

- [ ] **Step 1: Rename the rendering test and specify approved copy**

Add assertions for the wide view:

```rust
assert!(screen.contains("QUESTMANCER'S GUILD HALL"));
assert!(screen.contains("QUEST BOARD"));
assert!(screen.contains("PARTY ROSTER"));
assert!(screen.contains("CALLS FOR COUNSEL"));
assert!(screen.contains("SCRYING TABLE"));
assert!(screen.contains("CHRONICLE"));
assert!(screen.contains("Elowen"));
assert!(screen.contains("requests counsel"));
```

Add empty, reconnecting and unavailable-Reviewr assertions:

```rust
assert!(empty.contains("The hearth is warm. The guild awaits its next commission."));
assert!(reconnecting.contains("The scrying pool has clouded. Reconnecting"));
assert!(unavailable.contains("The spoils cannot be inspected here"));
```

- [ ] **Step 2: Run the Guild Hall tests and observe the old world**

Run:

```bash
cargo test --test guild_hall_rendering --test rendering --test reply --test interaction
```

Expected: FAIL because the view still renders sites, webmaster mail and guestbook copy.

- [ ] **Step 3: Centralize approved user-facing copy**

Create `src/ui/copy.rs` with pure functions:

```rust
pub const EMPTY_GUILD: &str = "The hearth is warm. The guild awaits its next commission.";
pub const SCRYING_CLOUDED: &str = "The scrying pool has clouded. Reconnecting...";
pub const SCRYING_STILL: &str = "The scrying pool is still.";
pub const COUNSEL_ISSUED: &str = "Counsel issued.";
pub const SUMMONS_ACKNOWLEDGED: &str = "Summons acknowledged.";

pub fn counsel_requested(name: &str) -> String {
    format!("{name} requests counsel at a sealed gate.")
}

pub fn spoils_returned(name: &str) -> String {
    format!("{name} has returned with unopened spoils.")
}

pub fn no_match(query: &str) -> String {
    format!("No adventurer or campaign answers {query:?}.")
}
```

Operational diagnostics must append the real cause after the atmospheric sentence.

- [ ] **Step 4: Render the responsive Guild Hall**

Use these layouts:

```text
>=120 columns: Quest Board | Party, Summons, Chronicle | Scrying Table
80-119 columns: Quest Board and Party | selected adventurer and output
<80 columns: one focused region with Tab cycling
```

Rename `Region` variants to:

```rust
pub enum Region {
    QuestBoard,
    Party,
    Summons,
    Chronicle,
    Adventurer,
}
```

Keep selection and output loading behavior unchanged. Render persona name, ancestry, class and epithet in the selected-adventurer card. Use `Observe`, `Issue counsel`, `Acknowledge summons`, `Inspect spoils` and `Open Chronicle` in contextual footers.

- [ ] **Step 5: Rename reply semantics without changing send behavior**

Rename `Modal::Reply` to `Modal::Counsel`, `Action::Reply` to `Action::Counsel`, and composer title to `ISSUE COUNSEL`. The submit path must still send the exact draft bytes to the selected pane and reject whitespace-only drafts.

- [ ] **Step 6: Verify all Guild Hall interaction paths**

Run:

```bash
cargo test --test guild_hall_rendering --test rendering --test reply --test input --test interaction --test command --test runtime_loop
```

Expected: PASS for empty, active, blocked, completed, exited, narrow, tiny, reconnecting, counsel, search, focus, refresh and Reviewr states.

- [ ] **Step 7: Commit**

```bash
git add src/app.rs src/interaction.rs src/ui tests
git commit -m "feat: open the operational Guild Hall"
```

---

### Task 5: Turn connected café bays into connected Delves

**Files:**
- Rename: `src/ui/cafe_scene.rs` -> `src/ui/delve_scene.rs`
- Rename: `src/ui/views/cafe.rs` -> `src/ui/views/delve.rs`
- Rename: `src/ui/widgets/agent_crt.rs` -> `src/ui/widgets/chamber.rs`
- Modify: `src/ui/{mod,theatre}.rs`
- Modify: `src/ui/widgets/mod.rs`
- Rename: `tests/cafe_scene.rs` -> `tests/delve_scene.rs`
- Rename: `tests/cafe_widgets.rs` -> `tests/delve_widgets.rs`
- Rename: `tests/cafe_rendering.rs` -> `tests/delve_rendering.rs`
- Modify: `tests/{theatre,rendering,interaction,runtime_loop,property_domain}.rs`

**Interfaces:**
- Consumes: stable campaign ordering, selected adventurer, display preferences and existing connected-bay invariants.
- Produces: deterministic connected campaign Delves, chamber anchors and semantic dungeon theatre.

- [ ] **Step 1: Rename tests and specify the topology**

Require these public types:

```rust
pub enum DelveVariant {
    ForgottenLibrary,
    MossyUndercroft,
    OldWatchtower,
}

pub struct CampaignDelve {
    pub workspace_id: WorkspaceId,
    pub variant: DelveVariant,
    pub rect: Rect,
    pub chambers: Vec<ChamberAnchor>,
    pub adventurers: Vec<AgentKey>,
}
```

Carry forward the existing topology tests with concrete ownership, ordering and overlap assertions:

```rust
let assigned = delves
    .iter()
    .flat_map(|delve| delve.adventurers.iter().cloned())
    .collect::<Vec<_>>();
assert_eq!(assigned.len(), agents.len());
assert_eq!(
    assigned
        .iter()
        .collect::<std::collections::BTreeSet<_>>()
        .len(),
    agents.len()
);

assert_eq!(
    delves
        .iter()
        .map(|delve| &delve.workspace_id)
        .collect::<Vec<_>>(),
    [&WorkspaceId::new("alpha"), &WorkspaceId::new("zeta")]
);
for (index, left) in delves.iter().enumerate() {
    for right in delves.iter().skip(index + 1) {
        let overlaps = left.rect.x < right.rect.right()
            && right.rect.x < left.rect.right()
            && left.rect.y < right.rect.bottom()
            && right.rect.y < left.rect.bottom();
        assert!(!overlaps, "Delves must not overlap: {left:?} and {right:?}");
    }
}

assert!(
    screen.contains("Overflow 4"),
    "selected overflow adventurer must remain visible:\n{screen}"
);
let selected_offset = screen.find("Overflow 4").unwrap();
let overflow_marker = screen
    .find("[more chambers]")
    .expect("overflow marker must be rendered");
assert!(selected_offset < overflow_marker);
```

- [ ] **Step 2: Run the focused tests and observe missing Delve types**

Run:

```bash
cargo test --test delve_scene --test delve_widgets --test delve_rendering --test theatre
```

Expected: FAIL because café bays, workstations and CRT state language remain.

- [ ] **Step 3: Port geometry without changing invariants**

Use `git mv`, then rename:

```text
BayVariant       -> DelveVariant
CafeBay          -> CampaignDelve
SeatAnchor       -> ChamberAnchor
layout_bays      -> layout_delves
authored_seats   -> authored_chambers
variant_for_workspace -> variant_for_campaign
```

Change the stable hash label from `cafe-variant\0` to `questmancer-delve-variant\0`. Retain lexical workspace ordering, bounded capacity, overflow splitting, selected-overflow targeting, zero-sized safety and absolute terminal coordinates.

- [ ] **Step 4: Replace café architecture with dungeon architecture**

Each variant must own real room geometry:

```text
ForgottenLibrary: shelves, reading alcove, rune table, connecting arch
MossyUndercroft: stone wall, root break, camp junction, descending passage
OldWatchtower: stair, map wall, narrow landing, signal brazier
```

Render only architecture, furniture, adventurer identity, semantic state theatre and navigation. Remove CRTs, desks, modem lights, cable labels and cybercafé signage.

- [ ] **Step 5: Project approved state language**

Rename theatre poses and labels:

```rust
pub enum TheatrePose {
    Delving,
    SeekingCounsel,
    SpoilsUnopened,
    VictoryRecorded,
    Resting,
    Departed,
    Unknown,
}
```

Use labels:

```text
DELVING
COUNSEL REQUESTED
SPOILS RETURNED
VICTORY RECORDED
RESTING
DEPARTED
UNKNOWN
```

Keep the existing animation rates and one-second completion boundary unless a rendering test proves a different cadence is needed. Rename the effects: typing becomes tool/rune movement, raised hand becomes signal lantern or sealed door, confetti becomes a brief chest sparkle, screensaver becomes campfire, broken CRT becomes an empty chamber.

- [ ] **Step 6: Verify Delve accessibility and scheduling**

Run:

```bash
cargo test --test delve_scene --test delve_widgets --test delve_rendering --test theatre --test runtime_loop --test interaction
PROPTEST_CASES=1024 cargo test --test property_domain every_generated_agent_is_owned_by_exactly_one_visible_delve
```

Expected: PASS at 160, 120, 80, 60, tiny and zero dimensions; ASCII has no block glyphs; ANSI-16 has no indexed/RGB colours; reduced/no-motion geometry is stable.

- [ ] **Step 7: Commit**

```bash
git add src/ui tests
git commit -m "feat: lead parties through connected Delves"
```

---

### Task 6: Re-author pixel art as fantasy adventurers

**Files:**
- Modify: `src/ui/persona/{appearance,cafe_sprite,profile,state_pose}.rs`
- Rename: `src/ui/persona/cafe_sprite.rs` -> `src/ui/persona/chamber_sprite.rs`
- Modify: `src/ui/pixel/palette.rs`
- Modify: `src/ui/widgets/{chamber,adventurer_card}.rs`
- Modify: `tests/{persona_art,pixel,delve_widgets,delve_rendering}.rs`

**Interfaces:**
- Consumes: `AdventurerPersona`, fantasy appearance traits, `TheatreFrame`, palette and packer.
- Produces: original seated/chamber and full-body fantasy sprites whose class, ancestry, gear and state remain recognizable.

- [ ] **Step 1: Rewrite the art tests around recognition anchors**

Require fixed dimensions and semantic anchors:

```rust
fn fixed_persona(key: &str) -> AdventurerPersona {
    AdventurerPersona::for_key(PersonaKey::new(key))
}

let persona = fixed_persona("art-fixture");
let chamber = compose_chamber_adventurer(
    &persona,
    frame(TheatrePose::Delving, 0),
);
assert_eq!((chamber.width(), chamber.height()), (10, 12));
let profile = compose_profile_adventurer(&persona);
assert_eq!((profile.width(), profile.height()), (16, 32));

let mut wizard = fixed_persona("class-fixture");
wizard.class = AdventurerClass::Wizard;
let mut ranger = wizard.clone();
ranger.class = AdventurerClass::Ranger;
assert_ne!(
    silhouette(&compose_profile_adventurer(&wizard)),
    silhouette(&compose_profile_adventurer(&ranger)),
    "wizard spellbook/staff and ranger bow/quiver must use distinct logical pixels"
);

let mut dwarf = fixed_persona("ancestry-fixture");
dwarf.ancestry = Ancestry::Dwarf;
let mut human = dwarf.clone();
human.ancestry = Ancestry::Human;
assert_ne!(
    silhouette(&compose_profile_adventurer(&dwarf)),
    silhouette(&compose_profile_adventurer(&human)),
    "dwarf must retain a compact, bearded silhouette"
);

let counsel = compose_chamber_adventurer(
    &persona,
    frame(TheatrePose::SeekingCounsel, 0),
);
let delving = compose_chamber_adventurer(
    &persona,
    frame(TheatrePose::Delving, 0),
);
assert_ne!(silhouette(&counsel), silhouette(&delving));
```

Keep tests for palette collision safety, ASCII-only output and top/bottom profile coverage.

- [ ] **Step 2: Run art tests and verify the 90s wardrobe fails them**

Run:

```bash
cargo test --test persona_art --test pixel --test delve_widgets
```

Expected: FAIL because the sprites still use band tees, headphones, lanyards, office chairs and desk props.

- [ ] **Step 3: Compose class and ancestry silhouettes**

Split composition into three explicit layers:

```rust
fn compose_body(appearance: &PersonaAppearance, pose: BodyPose, palette: Palette) -> Canvas;
fn overlay_class_gear(canvas: &mut Canvas, class: AdventurerClass, gear: AdventuringGear);
fn overlay_state_prop(canvas: &mut Canvas, pose: TheatrePose, frame: u8);
```

The base body owns proportions, head, hair, ancestry anchors and garb. Class gear owns the staff, spellbook, shield, bow, lute, toolkit or runes. State props own signal lantern, sealed-door cue, spoils bundle, campfire or empty-chamber composition.

- [ ] **Step 4: Author the approved palette**

Retain xterm-256 and ANSI-16 projections. Replace neon café accents with semantic roles:

```rust
Stone, DarkStone, Timber, Parchment, Ink, Hearth, Moss,
RuneGlow, Counsel, Spoils, Selection, Fog, Goblin,
SkinLight, SkinMedium, SkinDark, HairDark, HairLight,
Leather, Steel, ClothWarm, ClothCool
```

Every adjacent semantic role must remain distinguishable under both palettes. Colour is supplementary to labels and silhouettes.

- [ ] **Step 5: Implement ASCII fantasy silhouettes**

Provide six-row ASCII poses for delving, counsel, spoils, victory, resting, departed and unknown. They must contain explicit markers (`[>]`, `[!]`, `[+]`, `[~]`, `[x]`, `[?]`) and no Unicode glyphs.

- [ ] **Step 6: Verify art and rendering**

Run:

```bash
cargo test --test persona_art --test pixel --test delve_widgets --test delve_rendering
```

Expected: PASS; fixed personas remain visually distinct, class/ancestry anchors survive both palettes, and state never relies on colour alone.

- [ ] **Step 7: Commit**

```bash
git add src/ui/persona src/ui/pixel src/ui/widgets tests
git commit -m "feat: dress the guild as fantasy adventurers"
```

---

### Task 7: Contain and release the goblins

**Files:**
- Create: `src/ui/goblins.rs`
- Modify: `src/ui/mod.rs`
- Modify: `src/app.rs`
- Modify: `src/interaction.rs`
- Modify: `src/ui/views/guild_hall.rs`
- Modify: `src/ui/theatre.rs`
- Create: `tests/goblins.rs`
- Modify: `tests/{interaction,runtime_loop,guild_hall_rendering}.rs`

**Interfaces:**
- Consumes: workspace identity, search submission, injected model time and existing adaptive scheduler.
- Produces: deterministic rare sightings and a non-persistent three-second `release the goblins` outbreak with transient `CREATURES DETECTED` Chronicle marginalia.

- [ ] **Step 1: Write goblin containment tests**

```rust
#[test]
fn sightings_are_deterministic_and_non_semantic() {
    let first = sighting_for_campaign(&WorkspaceId::new("w1"));
    let second = sighting_for_campaign(&WorkspaceId::new("w1"));
    assert_eq!(first, second);
}

#[test]
fn exact_incantation_releases_goblins_without_selecting_an_agent() {
    let mut model = Model::new(View::Guild);
    model.set_now(Timestamp::from_millis(1_000));
    model.open_search();
    for character in "release the goblins".chars() {
        model.push_modal_character(character);
    }
    let reduction = reduce_action(&mut model, Action::Submit);
    assert!(model.goblins().is_visible(model.now()));
    assert!(reduction.commands.is_empty());
    assert!(reduction.persistence.is_empty());
}

#[test]
fn outbreak_ends_after_three_seconds() {
    let mut model = Model::new(View::Guild);
    model.set_now(Timestamp::from_millis(1_000));
    let released_at = model.now();
    model.goblins_mut().release(released_at);
    model.set_now(Timestamp::from_millis(3_999));
    assert!(model.goblins().is_visible(model.now()));
    model.set_now(Timestamp::from_millis(4_000));
    assert!(!model.goblins().is_visible(model.now()));
}

#[test]
fn outbreak_notice_is_visible_but_never_enters_factual_history() {
    let mut model = live_model();
    let chronicle_len = model.domain().chronicle.entries().len();
    let released_at = model.now();
    model.goblins_mut().release(released_at);

    let active = render(&model, 120, 30);
    assert!(active.contains("CREATURES DETECTED"));
    assert_eq!(model.domain().chronicle.entries().len(), chronicle_len);

    model.set_now(Timestamp::from_millis(
        released_at.as_millis() + 3_000,
    ));
    let settled = render(&model, 120, 30);
    assert!(!settled.contains("CREATURES DETECTED"));
    assert_eq!(model.domain().chronicle.entries().len(), chronicle_len);
}
```

- [ ] **Step 2: Run tests and observe the missing Easter egg**

Run:

```bash
cargo test --test goblins --test interaction --test runtime_loop
```

Expected: FAIL because goblin projection state is undefined.

- [ ] **Step 3: Implement deterministic sightings and outbreak state**

Create:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoblinSighting {
    ChestEyes,
    ChronicleHand,
    RaftersScroll,
    StolenBiscuit,
}

pub fn sighting_for_campaign(workspace_id: &WorkspaceId) -> Option<GoblinSighting> {
    let digest = labelled_campaign_hash("questmancer-goblin-sighting", workspace_id);
    (digest[0] == 0).then(|| match digest[1] % 4 {
        0 => GoblinSighting::ChestEyes,
        1 => GoblinSighting::ChronicleHand,
        2 => GoblinSighting::RaftersScroll,
        _ => GoblinSighting::StolenBiscuit,
    })
}

fn labelled_campaign_hash(label: &str, workspace_id: &WorkspaceId) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(label.as_bytes());
    hasher.update(&[0]);
    hasher.update(workspace_id.as_str().as_bytes());
    *hasher.finalize().as_bytes()
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GoblinState {
    released_at: Option<Timestamp>,
}

impl GoblinState {
    pub const OUTBREAK_DURATION: Duration = Duration::from_secs(3);
    pub fn release(&mut self, now: Timestamp) { self.released_at = Some(now); }
    pub fn is_visible(self, now: Timestamp) -> bool {
        self.released_at
            .is_some_and(|start| start.elapsed_until(now) < Self::OUTBREAK_DURATION)
    }
}
```

Add `goblins: GoblinState` to `Model`, initialize it with `Default`, and expose
`goblins(&self) -> &GoblinState` plus `goblins_mut(&mut self) -> &mut GoblinState`.

Return a background sighting only when `blake3("questmancer-goblin-sighting\0" + workspace_id)[0] == 0`; use the next byte to choose the sighting. Sightings never affect campaign status, selection, output, attention, factual Chronicle entries or persistence.

- [ ] **Step 4: Wire the exact secret incantation**

In search submission, intercept only the case-insensitive exact trimmed phrase `release the goblins`. Release the outbreak, dismiss search, set status `The goblins deny any involvement.`, and emit no host command or persistence effect. While active, project `CREATURES DETECTED` as decorative Chronicle marginalia without appending a `ChronicleEntry`. All other searches retain their existing behavior.

- [ ] **Step 5: Render and schedule the contained outbreak**

Render small original goblin silhouettes in unoccupied Guild Hall architecture only. Never overlay text, selected adventurers, summons or footers. During the three-second outbreak, schedule at most 4 fps; after the exact terminal boundary, return the Guild Hall to event-driven rendering. Reduced motion renders one static group; no-motion renders the status line without moving sprites.

- [ ] **Step 6: Verify containment**

Run:

```bash
cargo test --test goblins --test guild_hall_rendering --test interaction --test runtime_loop --test theatre
```

Expected: PASS; the incantation is exact, effects are temporary, state is not persisted, idle Guild Hall remains event-driven, and goblins never obscure actionable regions.

- [ ] **Step 7: Commit**

```bash
git add src/app.rs src/interaction.rs src/ui tests
git commit -m "feat: contain the Guild Hall goblins"
```

---

### Task 8: Rewrite documentation, recipes and release packaging

**Files:**
- Modify: `README.md`
- Modify: `PLAN.md`
- Modify: `CHANGELOG.md`
- Modify: `justfile`
- Modify: `.github/workflows/ci.yml`
- Create: `.github/workflows/release.yml`
- Modify: `herdr/install.sh`
- Modify: `tests/scripts.sh`
- Modify: `docs/superpowers/specs/2026-07-15-questmancer-creative-direction.md` only if implementation exposes an approved contradiction

**Interfaces:**
- Consumes: completed Questmancer binary, manifest, actions, config, persistence and view behavior.
- Produces: user-first installation and operation guide, local migration instructions, release artifacts and CI gates.

- [ ] **Step 1: Add packaging assertions before changing release files**

Extend `tests/scripts.sh` to assert:

```text
questmancer-v0.1.0-x86_64-unknown-linux-gnu.tar.gz
questmancer-v0.1.0-aarch64-unknown-linux-gnu.tar.gz
questmancer-v0.1.0-x86_64-apple-darwin.tar.gz
questmancer-v0.1.0-aarch64-apple-darwin.tar.gz
QUESTMANCER_REPOSITORY
bin/questmancer
opsydyn.questmancer.open
opsydyn.questmancer.guild
opsydyn.questmancer.delve
```

Add a check that `rg -n 'opsydyn\.webmaster|herdr-webmaster|WEBMASTER_INITIAL_VIEW' Cargo.toml herdr herdr-plugin.toml .github README.md justfile` returns no matches.

- [ ] **Step 2: Run packaging tests and observe old release names**

Run:

```bash
bash tests/scripts.sh
rg -n 'opsydyn\.webmaster|herdr-webmaster|WEBMASTER_INITIAL_VIEW' Cargo.toml herdr herdr-plugin.toml .github README.md justfile
```

Expected: shell tests fail or the search reports remaining Webmaster packaging names.

- [ ] **Step 3: Rewrite the README in the approved voice**

Open with:

```markdown
# Questmancer

> Your agents have entered the dungeon.
>
> You are the Questmancer.

Questmancer turns a Herdr session into a living adventurers' guild. Working
agents delve through chambers of code. Blocked agents call for counsel.
Completed work returns as spoils awaiting inspection.
```

Document:

```text
cargo build
herdr plugin link .
herdr plugin action invoke opsydyn.questmancer.open
```

Include Guild Hall/Delve explanations, keys, configuration, persistence ownership, local-only privacy, fake-agent test procedure, current Herdr 0.7.4 limitations, release targets and cleanup. Use `chronicle_max_entries` and `default_view = "guild"` in config examples.

Document the development cutover explicitly:

```bash
herdr plugin action invoke opsydyn.webmaster.close 2>/dev/null || true
herdr plugin unlink opsydyn.webmaster
cargo build --release
herdr plugin link .
herdr plugin action invoke opsydyn.questmancer.open
```

Verify `plugin unlink` syntax against `herdr plugin --help` before publishing; if the command differs, use the installed CLI's exact form.

- [ ] **Step 4: Update recipes and CI**

Rename recipes:

```text
desk-test  -> guild-test
cafe-test  -> delve-test
run view="guild"
install-local copies target/release/questmancer to bin/questmancer
```

CI must run:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
bash tests/scripts.sh
bash -n herdr/install.sh herdr/run.sh herdr/control.sh
cargo build --release
git diff --check
```

- [ ] **Step 5: Add release workflow**

Build and package:

```text
x86_64-unknown-linux-gnu
aarch64-unknown-linux-gnu
x86_64-apple-darwin
aarch64-apple-darwin
```

Publish:

```text
questmancer-v<version>-x86_64-unknown-linux-gnu.tar.gz
questmancer-v<version>-aarch64-unknown-linux-gnu.tar.gz
questmancer-v<version>-x86_64-apple-darwin.tar.gz
questmancer-v<version>-aarch64-apple-darwin.tar.gz
SHA256SUMS
```

Each archive contains an executable named `questmancer`. Use GitHub Actions matrix builds and create SHA-256 checksums after downloading all matrix artifacts into the release job.

- [ ] **Step 6: Verify documentation and packaging**

Run:

```bash
bash tests/scripts.sh
bash -n herdr/install.sh herdr/run.sh herdr/control.sh
cargo build --release
test -x target/release/questmancer
rg -n 'opsydyn\.webmaster|herdr-webmaster|WEBMASTER_INITIAL_VIEW' Cargo.toml herdr herdr-plugin.toml .github README.md justfile && exit 1 || true
git diff --check
```

Expected: PASS and no legacy product identity remains in current user/release surfaces. Historical design documents may retain Webmaster references.

- [ ] **Step 7: Commit**

```bash
git add README.md PLAN.md CHANGELOG.md justfile .github herdr tests/scripts.sh
git commit -m "docs: prepare Questmancer for release"
```

---

### Task 9: Run live Herdr 0.7.4 acceptance and close the pivot

**Files:**
- Modify: `README.md` manual-test evidence section
- Modify: `CHANGELOG.md`
- Create: `docs/manual-test/questmancer-0.1.0.md`

**Interfaces:**
- Consumes: linked Questmancer plugin, Herdr 0.7.4 / protocol 16 and a dedicated plain test pane.
- Produces: reproducible pass/fail/blocked evidence without mutating another agent pane or stopping the pre-existing Herdr server.

- [ ] **Step 1: Run the complete automated gate**

Run:

```bash
just verify
just release-check
PROPTEST_CASES=4096 cargo test --test property_domain --test persisted_state
```

Expected: all tests, formatting, clippy, shell syntax, release build and diff checks pass.

- [ ] **Step 2: Inspect the live environment without changing it**

Run:

```bash
herdr version
herdr api ping
herdr plugin list
herdr plugin action list --plugin opsydyn.questmancer
```

Record exact client/server version, protocol, plugin source, enabled state and the five required actions.

- [ ] **Step 3: Link and exercise singleton views**

After following the documented old-plugin cutover:

```bash
cargo build --release
herdr plugin link .
herdr plugin action invoke opsydyn.questmancer.open
herdr plugin action invoke opsydyn.questmancer.open
herdr plugin action invoke opsydyn.questmancer.guild
herdr plugin action invoke opsydyn.questmancer.delve
```

Confirm exactly one Questmancer pane exists and keys `1`/`2` switch between Guild Hall and Delve.

- [ ] **Step 4: Exercise a dedicated synthetic adventurer**

Create a dedicated plain tab and pane; never report over Codex, Claude or Questmancer's managed pane:

```bash
WORKSPACE_ID="$(herdr workspace list | jq -r '.result.workspaces[] | select(.focused) | .workspace_id' | head -n 1)"
test -n "$WORKSPACE_ID"
herdr tab create --workspace "$WORKSPACE_ID" --cwd "$PWD" --label questmancer-smoke --focus
PANE_ID="$(herdr pane current | jq -r '.result.pane.pane_id // .result.pane_id // .pane_id')"
SOURCE_ID="questmancer-manual-$(date +%s)"

herdr pane report-agent "$PANE_ID" \
  --source "$SOURCE_ID" \
  --agent questmancer-smoke \
  --state working

herdr pane report-agent "$PANE_ID" \
  --source "$SOURCE_ID" \
  --agent questmancer-smoke \
  --state blocked \
  --message "Counsel requested at the sealed gate"

herdr pane report-agent "$PANE_ID" \
  --source "$SOURCE_ID" \
  --agent questmancer-smoke \
  --state idle
```

Herdr 0.7.4's synthetic reporter supports `idle`, `working`, `blocked` and `unknown`; it does not support `done`. Exercise completion with a real agent event or mark the spoils path blocked. Confirm:

```text
working -> DELVING
blocked -> unread call for counsel + sealed-door/signal pose
done -> unopened spoils + one-shot sparkle
idle -> resting/campfire
```

If 0.7.4 still cannot synthesize `done`, mark it blocked rather than claiming it passed.

- [ ] **Step 5: Exercise real actions and persistence**

Against only the dedicated test pane:

```text
j/k selection
/ search
Enter observe
r issue exact counsel
o refresh scrying output
Space acknowledge summons
v inspect spoils when Reviewr exists
release the goblins
close/reopen Questmancer
```

Confirm persona, selected adventurer, view and acknowledged summons survive restart. Confirm the goblin outbreak does not persist.

- [ ] **Step 6: Restore the environment**

Release the synthetic source and close the dedicated test pane:

```bash
herdr pane report-agent "$PANE_ID" \
  --source "$SOURCE_ID" \
  --agent questmancer-smoke \
  --state working
herdr pane release-agent "$PANE_ID" \
  --source "$SOURCE_ID" \
  --agent questmancer-smoke
herdr pane close "$PANE_ID"
herdr plugin action invoke opsydyn.questmancer.close
```

Retain the linked plugin unless the user explicitly requests unlinking, and never stop the pre-existing Herdr server. Run `git status --short --branch` and record remaining expected files only.

- [ ] **Step 7: Record evidence and commit**

Write `docs/manual-test/questmancer-0.1.0.md` with a concise pass/fail/blocked table, exact commands, screenshots and cleanup result. Update README/CHANGELOG only with claims supported by that run.

```bash
git add README.md CHANGELOG.md docs/manual-test/questmancer-0.1.0.md
git commit -m "docs: record Questmancer live acceptance"
```

---

## Final verification

Run from a clean checkout:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
bash tests/scripts.sh
bash -n herdr/install.sh herdr/run.sh herdr/control.sh
cargo build --release
test -x target/release/questmancer
git diff --check
git status --short --branch
```

Completion requires automated green status, recorded Herdr 0.7.4 evidence, one managed pane, working Guild Hall and Delve actions, stable fantasy personas, honest state theatre, accessible fallbacks, and no unreviewed legacy product vocabulary in current surfaces.
