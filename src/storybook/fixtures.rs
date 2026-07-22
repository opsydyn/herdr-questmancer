use std::{collections::BTreeMap, path::PathBuf};

use crate::{
    app::{ConnectionState, Model, OutputPreview, View},
    domain::{
        AdventurerClass, AdventurerPersona, Agent, AgentKey, Ancestry, Campaign, Chronicle,
        DomainState, GuildAttention, GuildSummons, PaneId, PersonaKey, Presence, TabId, Timestamp,
        WorkspaceId,
    },
    interaction::reduce_action,
    ui::input::Action,
};

pub const FIXED_NOW: Timestamp = Timestamp::from_millis(121_000);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StoryContext {
    pub now: Timestamp,
}

impl StoryContext {
    pub const fn fixed() -> Self {
        Self { now: FIXED_NOW }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StoryFixture {
    SceneApplication(Box<Model>),
    ArchetypeGallery(ArchetypeGallery),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchetypeGallery {
    WorldMasters,
    BarbarianV2Poses,
    PortraitMasters,
    GoblinEasterEgg,
    Librarian,
}

pub const fn barbarian_v2_pose_fixture() -> StoryFixture {
    StoryFixture::ArchetypeGallery(ArchetypeGallery::BarbarianV2Poses)
}

pub const CORE_ARCHETYPES: [AdventurerClass; 8] = [
    AdventurerClass::Barbarian,
    AdventurerClass::Bard,
    AdventurerClass::Cleric,
    AdventurerClass::Druid,
    AdventurerClass::Paladin,
    AdventurerClass::Ranger,
    AdventurerClass::Rogue,
    AdventurerClass::Wizard,
];

pub fn guild_world_fixture(context: StoryContext) -> Model {
    fixture_model(context, View::Guild)
}

pub fn delve_world_fixture(context: StoryContext) -> Model {
    fixture_model(context, View::Delve)
}

pub fn selected_adventurer_interaction_fixture(context: StoryContext) -> Model {
    let mut model = guild_world_fixture(context);
    let _ = reduce_action(&mut model, Action::Next);
    model
}

pub fn native_barbarian_portrait_fixture(context: StoryContext) -> Model {
    native_portrait_fixture(context, AdventurerClass::Barbarian, Ancestry::Gnome)
}

pub fn native_artificer_portrait_fixture(context: StoryContext) -> Model {
    native_portrait_fixture(context, AdventurerClass::Artificer, Ancestry::Human)
}

pub fn native_bard_portrait_fixture(context: StoryContext) -> Model {
    native_portrait_fixture(context, AdventurerClass::Bard, Ancestry::Halfling)
}

pub fn native_druid_portrait_fixture(context: StoryContext) -> Model {
    native_portrait_fixture(context, AdventurerClass::Druid, Ancestry::Elf)
}

pub fn native_paladin_portrait_fixture(context: StoryContext) -> Model {
    native_portrait_fixture(context, AdventurerClass::Paladin, Ancestry::Dwarf)
}

pub fn native_rogue_portrait_fixture(context: StoryContext) -> Model {
    native_portrait_fixture(context, AdventurerClass::Rogue, Ancestry::Elf)
}

pub fn native_wizard_portrait_fixture(context: StoryContext) -> Model {
    native_portrait_fixture(context, AdventurerClass::Wizard, Ancestry::Human)
}

pub fn native_goblin_portrait_fixture(context: StoryContext) -> Model {
    native_portrait_fixture(context, AdventurerClass::Druid, Ancestry::Goblin)
}

pub fn native_orc_portrait_fixture(context: StoryContext) -> Model {
    native_portrait_fixture(context, AdventurerClass::Paladin, Ancestry::Orc)
}

fn native_portrait_fixture(
    context: StoryContext,
    class: AdventurerClass,
    ancestry: Ancestry,
) -> Model {
    let mut model = guild_world_fixture(context);
    let selected = model
        .selected_agent_key()
        .cloned()
        .expect("fixture selects an adventurer");
    model
        .domain_mut()
        .agents
        .get_mut(&selected)
        .expect("selected fixture adventurer exists")
        .persona
        .class = class;
    model
        .domain_mut()
        .agents
        .get_mut(&selected)
        .expect("selected fixture adventurer exists")
        .persona
        .ancestry = ancestry;
    model.show_adventurer_card();
    model
}

pub fn counsel_interaction_fixture(context: StoryContext) -> Model {
    interaction_fixture(context, Action::Counsel, "Hold at the sealed gate.")
}

pub fn search_interaction_fixture(context: StoryContext) -> Model {
    interaction_fixture(context, Action::Search, "Merrin")
}

pub fn scrying_interaction_fixture(context: StoryContext) -> Model {
    interaction_fixture(context, Action::Refresh, "")
}

pub fn librarian_ledger_fixture(context: StoryContext) -> Model {
    interaction_fixture(context, Action::ToggleLedger, "")
}

pub fn narrow_interaction_fixture(context: StoryContext) -> Model {
    interaction_fixture(context, Action::Counsel, "Wait for the torch signal.")
}

fn interaction_fixture(context: StoryContext, action: Action, text: &str) -> Model {
    let mut model = guild_world_fixture(context);
    let _ = reduce_action(&mut model, action);
    for character in text.chars() {
        let _ = reduce_action(&mut model, Action::TypeCharacter(character));
    }
    model
}

fn fixture_model(context: StoryContext, view: View) -> Model {
    let library = WorkspaceId::new("storybook-library");
    let undercroft = WorkspaceId::new("storybook-undercroft");
    let agents = [
        fixture_agent("Elowen-Typeweaver", &library, Presence::Working, false),
        fixture_agent("Merrin-Ironjaw", &library, Presence::Blocked, true),
        fixture_agent("Arnoldus-Manytools", &undercroft, Presence::Done, false),
        fixture_agent("Pius-Blackquill", &undercroft, Presence::Idle, false),
        fixture_agent("Rowan-Brightward", &undercroft, Presence::Exited, false),
    ];
    let selected = agents[1].key.clone();
    let campaigns = BTreeMap::from([
        (
            library.clone(),
            Campaign {
                workspace_id: library.clone(),
                label: "Forgotten Library".to_owned(),
                cwd: PathBuf::from("/storybook/forgotten-library"),
                party: agents[..2].iter().map(|agent| agent.key.clone()).collect(),
            },
        ),
        (
            undercroft.clone(),
            Campaign {
                workspace_id: undercroft,
                label: "Mossy Undercroft".to_owned(),
                cwd: PathBuf::from("/storybook/mossy-undercroft"),
                party: agents[2..].iter().map(|agent| agent.key.clone()).collect(),
            },
        ),
    ]);
    let agents = agents
        .into_iter()
        .map(|agent| (agent.key.clone(), agent))
        .collect();
    let mut model = Model::new(view);
    model.replace_domain(DomainState {
        campaigns,
        agents,
        selected_agent: Some(selected),
        chronicle: Chronicle::new(16),
    });
    model.set_connection(ConnectionState::Connected);
    model.set_now(context.now);
    let selected = model
        .selected_agent()
        .expect("fixture selects an adventurer");
    model.set_output_preview(Some(OutputPreview {
        pane_id: selected.pane_id.clone(),
        revision: selected.pane_revision,
        text: "The runes resolve into a clean test report.".to_owned(),
        loading: false,
        error: None,
    }));
    model
}

fn fixture_agent(
    name: &str,
    workspace_id: &WorkspaceId,
    presence: Presence,
    needs_counsel: bool,
) -> Agent {
    let key = AgentKey::new(name);
    Agent {
        key: key.clone(),
        pane_id: PaneId::new(format!("storybook:{name}")),
        workspace_id: workspace_id.clone(),
        tab_id: TabId::new("storybook-tab"),
        name: name.replace('-', " "),
        custom_status: Some("Following the Questmancer's commission.".to_owned()),
        presence,
        presence_since: Timestamp::from_millis(101_000),
        attention: if needs_counsel {
            GuildAttention::unread(
                GuildSummons::CounselRequested,
                Timestamp::from_millis(111_000),
            )
        } else {
            GuildAttention::Clear
        },
        focused: false,
        pane_revision: 7,
        persona: AdventurerPersona::for_key(PersonaKey::new(format!("storybook-{name}"))),
    }
}
