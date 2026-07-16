use std::{collections::BTreeMap, path::PathBuf};

use crate::{
    app::{ConnectionState, DisplayPreferences, Modal, Model, OutputPreview, View},
    domain::{
        AdventurerPersona, Agent, AgentKey, Campaign, Chronicle, ChronicleEntry, ChronicleEvent,
        DomainState, GuildAttention, GuildSummons, PaneId, PersonaKey, Presence, TabId, Timestamp,
        WorkspaceId,
    },
    ui::{
        pixel::{Canvas, ColorRole, Palette},
        theatre::TheatreFrame,
    },
};

pub const FIXED_NOW: Timestamp = Timestamp::from_millis(121_000);

pub fn library_id() -> WorkspaceId {
    WorkspaceId::new("workspace-0")
}

pub fn undercroft_id() -> WorkspaceId {
    WorkspaceId::new("workspace-2")
}

pub fn watchtower_id() -> WorkspaceId {
    WorkspaceId::new("workspace-4")
}

pub fn goblin_chest_id() -> WorkspaceId {
    WorkspaceId::new("goblin-fixture-32")
}

pub fn goblin_hand_id() -> WorkspaceId {
    WorkspaceId::new("goblin-fixture-2901")
}

pub fn goblin_scroll_id() -> WorkspaceId {
    WorkspaceId::new("goblin-fixture-330")
}

pub fn goblin_biscuit_id() -> WorkspaceId {
    WorkspaceId::new("goblin-fixture-801")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StoryContext {
    pub now: Timestamp,
}

impl StoryContext {
    pub const fn fixed() -> Self {
        Self { now: FIXED_NOW }
    }
}

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

pub fn campaign_fixture(
    workspace_id: WorkspaceId,
    label: impl Into<String>,
    party: Vec<AgentKey>,
) -> Campaign {
    let cwd = PathBuf::from(format!("/storybook/{workspace_id}"));
    Campaign {
        workspace_id,
        label: label.into(),
        cwd,
        party,
    }
}

pub fn guild_fixture(context: &StoryContext) -> Model {
    let (domain, blocked) = fixed_domain();
    let mut model = Model::new(View::Guild);
    model.replace_domain(domain);
    model.set_connection(ConnectionState::Connected);
    model.set_output_preview(Some(OutputPreview {
        pane_id: blocked.pane_id,
        revision: blocked.pane_revision,
        text: "Checking the local schema...\nAwaiting counsel at the sealed gate.".to_owned(),
        loading: false,
        error: None,
    }));
    model.set_now(context.now);
    model
}

fn fixed_domain() -> (DomainState, Agent) {
    let [working, blocked, done, idle, exited] = fixed_agents();
    let campaigns = fixed_campaigns(&working, &blocked, &done, &idle, &exited);
    let preview_agent = blocked.clone();
    let agents: BTreeMap<AgentKey, Agent> = [working, blocked, done, idle, exited]
        .into_iter()
        .map(|agent| (agent.key.clone(), agent))
        .collect();
    let chronicle = fixed_chronicle(&agents);
    let selected_agent = Some(preview_agent.key.clone());
    (
        DomainState {
            campaigns,
            agents,
            selected_agent,
            chronicle,
        },
        preview_agent,
    )
}

fn fixed_agents() -> [Agent; 5] {
    let library = library_id();
    let undercroft = undercroft_id();
    let watchtower = watchtower_id();
    let working = agent_fixture(
        "Elowen-Typeweaver",
        library.clone(),
        Presence::Working,
        GuildAttention::Clear,
        false,
    );
    let blocked = agent_fixture(
        "Merrin-Ironjaw",
        library.clone(),
        Presence::Blocked,
        GuildAttention::unread(
            GuildSummons::CounselRequested,
            Timestamp::from_millis(31_000),
        ),
        true,
    );
    let done = agent_fixture(
        "Arnoldus-Manytools",
        undercroft.clone(),
        Presence::Done,
        GuildAttention::unread(GuildSummons::SpoilsReturned, Timestamp::from_millis(61_000)),
        false,
    );
    let idle = agent_fixture(
        "Pius-Blackquill",
        undercroft.clone(),
        Presence::Idle,
        GuildAttention::Clear,
        false,
    );
    let exited = agent_fixture(
        "Rowan-Brightward",
        watchtower.clone(),
        Presence::Exited,
        GuildAttention::Read {
            summons: GuildSummons::AdventurerDeparted,
            since: Timestamp::from_millis(91_000),
        },
        false,
    );
    [working, blocked, done, idle, exited]
}

fn fixed_campaigns(
    working: &Agent,
    blocked: &Agent,
    done: &Agent,
    idle: &Agent,
    exited: &Agent,
) -> BTreeMap<WorkspaceId, Campaign> {
    BTreeMap::from([
        (
            working.workspace_id.clone(),
            campaign_fixture(
                working.workspace_id.clone(),
                "Forgotten Library",
                vec![working.key.clone(), blocked.key.clone()],
            ),
        ),
        (
            done.workspace_id.clone(),
            campaign_fixture(
                done.workspace_id.clone(),
                "Mossy Undercroft",
                vec![done.key.clone(), idle.key.clone()],
            ),
        ),
        (
            exited.workspace_id.clone(),
            campaign_fixture(
                exited.workspace_id.clone(),
                "Old Watchtower",
                vec![exited.key.clone()],
            ),
        ),
    ])
}

fn fixed_chronicle(agents: &BTreeMap<AgentKey, Agent>) -> Chronicle {
    let mut chronicle = Chronicle::new(5);
    for entry in [
        chronicle_entry(
            Timestamp::from_millis(101_000),
            agents.get(&AgentKey::new("Elowen-Typeweaver")),
            ChronicleEvent::DelveBegan,
            "Elowen entered the Forgotten Library.",
        ),
        chronicle_entry(
            Timestamp::from_millis(106_000),
            agents.get(&AgentKey::new("Merrin-Ironjaw")),
            ChronicleEvent::CounselRequested,
            "Merrin requested counsel at a sealed gate.",
        ),
        chronicle_entry(
            Timestamp::from_millis(111_000),
            agents.get(&AgentKey::new("Arnoldus-Manytools")),
            ChronicleEvent::SpoilsReturned,
            "Arnoldus returned with unopened spoils.",
        ),
        chronicle_entry(
            Timestamp::from_millis(116_000),
            agents.get(&AgentKey::new("Pius-Blackquill")),
            ChronicleEvent::AdventurerRested,
            "Pius is resting by the hearth.",
        ),
        chronicle_entry(
            Timestamp::from_millis(120_000),
            agents.get(&AgentKey::new("Rowan-Brightward")),
            ChronicleEvent::AdventurerDeparted,
            "Rowan departed the Old Watchtower.",
        ),
    ] {
        chronicle.append(entry);
    }
    chronicle
}

pub fn delve_fixture(context: &StoryContext) -> Model {
    let mut model = guild_fixture(context);
    model.switch_to(View::Delve);
    model
}

pub fn modal_fixture(modal: Modal) -> Model {
    let mut application = guild_fixture(&StoryContext::fixed());
    match modal {
        Modal::None => {}
        Modal::Help => application.toggle_help(),
        Modal::Counsel { draft } => {
            drop(draft);
            application.open_counsel();
            for character in "Use the local schema".chars() {
                application.push_counsel_character(character);
            }
        }
        Modal::Search { query } => {
            drop(query);
            application.open_search();
            for character in "Elowen".chars() {
                application.push_modal_character(character);
            }
        }
    }
    application
}

pub fn compatibility_fixture(preferences: DisplayPreferences) -> Model {
    let mut model = delve_fixture(&StoryContext::fixed());
    model.set_preferences(preferences);
    model
}

fn chronicle_entry(
    occurred_at: Timestamp,
    agent: Option<&Agent>,
    event: ChronicleEvent,
    summary: &'static str,
) -> ChronicleEntry {
    ChronicleEntry::new(
        occurred_at,
        agent.map(|agent| agent.key.clone()),
        agent.map(|agent| agent.workspace_id.clone()),
        agent.map(|agent| agent.pane_id.clone()),
        agent.map_or(0, |agent| agent.pane_revision),
        event,
        summary,
    )
}

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
#[allow(
    clippy::large_enum_variant,
    reason = "the fixture boundary intentionally stores the exact Model payload"
)]
pub enum StoryFixture {
    Application(Model),
    AssetAtlas(AssetAtlas),
}
