use std::{collections::BTreeMap, path::PathBuf};

use crate::{
    app::{
        ConnectionState, DisplayPreferences, GuildFocus, Modal, Model, Motion, OutputPreview, View,
    },
    domain::{
        AccentTone, AdventurerPersona, Agent, AgentKey, Campaign, Chronicle, ChronicleEntry,
        ChronicleEvent, DomainState, GuildAttention, GuildSummons, PaneId, PersonaKey, Presence,
        TabId, Timestamp, WorkspaceId,
    },
    scene::{
        assets::{
            IndexedPaletteEntry, adventurer::compact_adventurer_animation_frame, indexed_sprite,
            palette::VOID,
        },
        pixel::Rgb,
        snapshot::{SceneAgent, SceneCampaign, SceneConnection, SceneSnapshot, SceneTransition},
        sprite::SpriteFrame,
        stage::{ScenePose, WorldScene},
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

pub fn connected_delves_fixture(context: &StoryContext) -> Model {
    let mut model = delve_fixture(context);
    for agent in model.domain_mut().agents.values_mut() {
        agent.presence = Presence::Working;
        agent.presence_since = context.now;
        agent.attention = GuildAttention::Clear;
        agent.custom_status = None;
    }
    model.domain_mut().chronicle = Chronicle::new(5);
    model.set_output_preview(None);
    model
}

pub fn guild_empty_fixture(context: &StoryContext) -> Model {
    let mut model = Model::new(View::Guild);
    model.set_connection(ConnectionState::Connected);
    model.set_now(context.now);
    model
}

pub fn guild_populated_fixture(context: &StoryContext) -> Model {
    let mut model = guild_fixture(context);
    for agent in model.domain_mut().agents.values_mut() {
        agent.attention = GuildAttention::Clear;
    }
    model.set_output_preview(None);
    model
}

/// The canonical Great Room review fixture. Its names are authored rather than
/// inferred so visual reviews remain stable across protocol and persona changes.
pub fn great_room_fixture(context: &StoryContext) -> Model {
    let mut model = guild_fixture(context);
    for (campaign, label) in
        model
            .domain_mut()
            .campaigns
            .values_mut()
            .zip(["Ironmere", "Saltwatch", "Moonfen"])
    {
        label.clone_into(&mut campaign.label);
    }
    model
}

pub fn great_room_empty_fixture(context: &StoryContext) -> Model {
    guild_empty_fixture(context)
}

pub fn great_room_one_campaign_fixture(context: &StoryContext) -> Model {
    let mut model = great_room_fixture(context);
    let retained = model.domain().campaigns.keys().next().cloned();
    if let Some(retained) = retained {
        let party = model.domain().campaigns[&retained].party.clone();
        let domain = model.domain_mut();
        domain
            .campaigns
            .retain(|workspace_id, _| workspace_id == &retained);
        domain.agents.retain(|key, _| party.contains(key));
        domain.selected_agent = party.first().cloned();
    }
    model
}

pub fn great_room_reviewr_unavailable_fixture(context: &StoryContext) -> Model {
    let mut model = great_room_fixture(context);
    model.set_reviewr_available(false);
    model.set_reviewr_availability_diagnostic("Reviewr is unavailable.".to_owned());
    model
}

pub fn great_room_scrying_failed_fixture(context: &StoryContext) -> Model {
    let mut model = great_room_fixture(context);
    model.set_guild_focus(GuildFocus::Scrying);
    if let Some(selected) = model.selected_agent().cloned() {
        model.set_output_preview(Some(OutputPreview {
            pane_id: selected.pane_id,
            revision: selected.pane_revision,
            text: String::new(),
            loading: false,
            error: Some("The scrying pool could not read this pane.".to_owned()),
        }));
    }
    model
}

pub fn great_room_focus_fixture(context: &StoryContext, focus: GuildFocus) -> Model {
    let mut model = great_room_fixture(context);
    model.set_guild_focus(focus);
    model
}

pub fn campaign_token_fixture(context: &StoryContext) -> Model {
    truthful_station_fixture(
        *context,
        "Ironmere-Pathfinder",
        Presence::Working,
        GuildAttention::Clear,
        GuildFocus::CampaignTables,
    )
}

pub fn counsel_projection_fixture(context: &StoryContext) -> Model {
    truthful_station_fixture(
        *context,
        "Saltwatch-Counsellor",
        Presence::Blocked,
        GuildAttention::unread(
            GuildSummons::CounselRequested,
            Timestamp::from_millis(31_000),
        ),
        GuildFocus::CounselBell,
    )
}

pub fn hearth_adventurer_fixture(context: &StoryContext) -> Model {
    truthful_station_fixture(
        *context,
        "Moonfen-Restkeeper",
        Presence::Idle,
        GuildAttention::Clear,
        GuildFocus::Hearth,
    )
}

pub fn spoils_adventurer_fixture(context: &StoryContext) -> Model {
    truthful_station_fixture(
        *context,
        "Ironmere-Returnee",
        Presence::Done,
        GuildAttention::unread(GuildSummons::SpoilsReturned, Timestamp::from_millis(61_000)),
        GuildFocus::Spoils,
    )
}

fn truthful_station_fixture(
    context: StoryContext,
    id: &'static str,
    presence: Presence,
    attention: GuildAttention,
    focus: GuildFocus,
) -> Model {
    let workspace_id = WorkspaceId::new("storybook-truthful-stations");
    let agent = agent_fixture(id, workspace_id.clone(), presence, attention, true);
    let agent_key = agent.key.clone();
    let campaign = campaign_fixture(workspace_id.clone(), "Ironmere", vec![agent_key.clone()]);
    let mut model = guild_empty_fixture(&context);
    let domain = model.domain_mut();
    domain.campaigns.insert(workspace_id, campaign);
    domain.agents.insert(agent_key.clone(), agent);
    domain.selected_agent = Some(agent_key);
    model.set_guild_focus(focus);
    model
}

pub fn guild_disconnected_fixture(context: &StoryContext) -> Model {
    let mut model = great_room_fixture(context);
    model.set_connection(ConnectionState::Offline);
    model
}

pub fn guild_reconnecting_fixture(context: &StoryContext) -> Model {
    let mut model = great_room_fixture(context);
    model.set_connection(ConnectionState::Reconnecting { attempt: 3 });
    model
}

pub fn guild_connecting_fixture(context: &StoryContext) -> Model {
    let mut model = great_room_fixture(context);
    model.set_connection(ConnectionState::Connecting);
    model
}

pub fn guild_incompatible_fixture(context: &StoryContext) -> Model {
    let mut model = great_room_fixture(context);
    model.set_connection(ConnectionState::Incompatible {
        expected: 17,
        actual: 16,
    });
    model
}

pub fn library_delve_fixture(context: &StoryContext) -> Model {
    isolated_delve_fixture(*context, &library_id())
}

pub fn undercroft_delve_fixture(context: &StoryContext) -> Model {
    isolated_delve_fixture(*context, &undercroft_id())
}

pub fn watchtower_delve_fixture(context: &StoryContext) -> Model {
    isolated_delve_fixture(*context, &watchtower_id())
}

fn isolated_delve_fixture(context: StoryContext, workspace_id: &WorkspaceId) -> Model {
    let mut model = delve_fixture(&context);
    {
        let domain = model.domain_mut();
        domain
            .campaigns
            .retain(|candidate, _| candidate == workspace_id);
        domain
            .agents
            .retain(|_, agent| &agent.workspace_id == workspace_id);
        domain.selected_agent = domain.agents.keys().next().cloned();
        domain.chronicle = Chronicle::new(5);
    }
    model.set_output_preview(None);
    model
}

pub fn goblin_chest_fixture(context: &StoryContext) -> Model {
    goblin_sighting_fixture(*context, goblin_chest_id(), "Chest-Peeker")
}

pub fn goblin_hand_fixture(context: &StoryContext) -> Model {
    goblin_sighting_fixture(*context, goblin_hand_id(), "Chronicle-Snatcher")
}

pub fn goblin_scroll_fixture(context: &StoryContext) -> Model {
    goblin_sighting_fixture(*context, goblin_scroll_id(), "Rafter-Skulker")
}

pub fn goblin_biscuit_fixture(context: &StoryContext) -> Model {
    goblin_sighting_fixture(*context, goblin_biscuit_id(), "Biscuit-Thief")
}

fn goblin_sighting_fixture(
    context: StoryContext,
    workspace_id: WorkspaceId,
    agent_id: &'static str,
) -> Model {
    let mut model = guild_empty_fixture(&context);
    let agent = agent_fixture(
        agent_id,
        workspace_id.clone(),
        Presence::Working,
        GuildAttention::Clear,
        false,
    );
    let agent_key = agent.key.clone();
    let campaign = campaign_fixture(
        workspace_id.clone(),
        "Goblin Watch",
        vec![agent_key.clone()],
    );
    let domain = model.domain_mut();
    domain.campaigns.insert(workspace_id, campaign);
    domain.agents.insert(agent_key.clone(), agent);
    domain.selected_agent = Some(agent_key);
    model
}

pub fn goblin_outbreak_fixture(context: &StoryContext) -> Model {
    let mut model = guild_fixture(context);
    model.goblins_mut().release(context.now);
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
    let mut model = great_room_fixture(&StoryContext::fixed());
    for agent in model.domain_mut().agents.values_mut() {
        agent.attention = GuildAttention::Clear;
        agent.custom_status = None;
    }
    model.domain_mut().chronicle = Chronicle::new(5);
    model.set_output_preview(None);
    model.set_preferences(preferences);
    model
}

pub fn motion_compatibility_fixture(motion: Motion) -> Model {
    let mut model = compatibility_fixture(DisplayPreferences::default());
    let retained = model
        .domain()
        .agents
        .keys()
        .take(2)
        .cloned()
        .collect::<Vec<_>>();
    {
        let domain = model.domain_mut();
        domain.agents.retain(|key, _agent| retained.contains(key));
        domain.campaigns.retain(|_workspace, campaign| {
            campaign.party.retain(|key| retained.contains(key));
            !campaign.party.is_empty()
        });
        domain.selected_agent = retained.first().cloned();
    }
    let mut agents = model.domain_mut().agents.values_mut();
    let working = agents
        .next()
        .expect("the deterministic motion baseline has a working adventurer");
    working.presence = Presence::Working;
    working.presence_since = Timestamp::from_millis(120_500);
    let idle = agents
        .next()
        .expect("the deterministic motion baseline has an idle adventurer");
    idle.presence = Presence::Idle;
    idle.presence_since = Timestamp::from_millis(118_500);
    model.set_preferences(DisplayPreferences {
        motion,
        ..DisplayPreferences::default()
    });
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
        packed: ratatui::text::Text<'static>,
    },
    RgbSprite {
        frame: SpriteFrame,
        background: Rgb,
    },
    RgbSpriteScaled {
        frame: SpriteFrame,
        background: Rgb,
        scale: u16,
    },
    RgbSpritePair {
        world: SpriteFrame,
        portrait: SpriteFrame,
        background: Rgb,
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
    Application {
        model: Model,
    },
}

impl AtlasContent {
    pub fn pixel(canvas: Canvas, palette: Palette, background: ColorRole) -> Self {
        let packed = crate::ui::pixel::pack(&canvas, &palette, background);
        Self::Pixel {
            canvas,
            palette,
            background,
            packed,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetAtlas {
    pub tiles: Vec<AtlasTile>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PixelSceneFixture {
    pub snapshot: SceneSnapshot,
    pub world_override: Option<WorldScene>,
}

impl PixelSceneFixture {
    #[must_use]
    pub const fn automatic(snapshot: SceneSnapshot) -> Self {
        Self {
            snapshot,
            world_override: None,
        }
    }

    #[must_use]
    pub const fn in_world(snapshot: SceneSnapshot, world: WorldScene) -> Self {
        Self {
            snapshot,
            world_override: Some(world),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(
    clippy::large_enum_variant,
    reason = "the fixture boundary intentionally stores the exact Model payload"
)]
pub enum StoryFixture {
    Application(Model),
    AssetAtlas(AssetAtlas),
    PixelScene(PixelSceneFixture),
}

pub fn calibration_room_scene_fixture(context: &StoryContext) -> PixelSceneFixture {
    PixelSceneFixture::in_world(compact_scene_snapshot(*context), WorldScene::GuildHall)
}

pub fn compact_adventurers_atlas_fixture(_: &StoryContext) -> AssetAtlas {
    let persona = AdventurerPersona::for_key(PersonaKey::new("storybook-compact-scene-atlas"));
    let frames = [
        ("Working", ScenePose::Working, 0),
        ("Seeking counsel", ScenePose::SeekingCounsel, 0),
        ("Returning with spoils", ScenePose::ReturningWithSpoils, 0),
        ("Settled", ScenePose::Settled, 0),
        ("Resting", ScenePose::Resting, 0),
        ("Unknown", ScenePose::Unknown, 0),
        ("Working alternate", ScenePose::Working, 1),
        ("Walking alternate", ScenePose::Working, 2),
    ];
    AssetAtlas {
        tiles: frames
            .into_iter()
            .map(|(label, pose, animation_frame)| AtlasTile {
                label,
                preferred_width: 20,
                preferred_height: 10,
                content: AtlasContent::RgbSprite {
                    frame: compact_adventurer_animation_frame(&persona, pose, animation_frame),
                    background: VOID,
                },
            })
            .collect(),
    }
}

const SPRITE_SILHOUETTE_PALETTE: &[IndexedPaletteEntry] = &[
    IndexedPaletteEntry {
        key: 'o',
        colour: Some(Rgb::new(22, 20, 27)),
    },
    IndexedPaletteEntry {
        key: 'g',
        colour: Some(Rgb::new(152, 191, 94)),
    },
    IndexedPaletteEntry {
        key: 'w',
        colour: Some(Rgb::new(145, 128, 216)),
    },
    IndexedPaletteEntry {
        key: 'b',
        colour: Some(Rgb::new(223, 149, 89)),
    },
    IndexedPaletteEntry {
        key: 'd',
        colour: Some(Rgb::new(111, 63, 42)),
    },
    IndexedPaletteEntry {
        key: 'm',
        colour: Some(Rgb::new(215, 221, 218)),
    },
    IndexedPaletteEntry {
        key: 'e',
        colour: Some(Rgb::new(255, 224, 107)),
    },
];

const GOBLIN_SILHOUETTE: &[&str] = &[
    "................",
    "................",
    "...o........o...",
    "..oggo....oggo..",
    ".oggggooooggggo.",
    ".oggggggggggggo.",
    "..oggggggggggo..",
    "...oggeeggego...",
    "...oggggggggo...",
    "....oggggggo....",
    "....oddddddo....",
    "...oddddddddo..m",
    "...oddoeegddo.mm",
    "....oddddddo..m.",
    "....oo....oo....",
    "...oo......oo...",
    "...oo..mm..oo...",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
];

const WIZARD_SILHOUETTE: &[&str] = &[
    "......oo........",
    ".....owwo.......",
    "....oowwwo......",
    "...owwwwwwwo....",
    "....owwwwwwo....",
    ".....owwwwo.....",
    ".....oweeewo....",
    "..m..owwwwwo....",
    "..m...owwwo.....",
    "..m...owwwo.....",
    "..m...owwwo.....",
    "..m..owwwwwwo...",
    "..m.owwwwwwwwo..",
    "..m.owwwwwwwwo..",
    "..m..owwwwwwo...",
    "..m...owwwo.....",
    "..m...owwwo.....",
    "..m....oo.oo....",
    "..m.............",
    "..e.............",
    "................",
    "................",
    "................",
    "................",
];

const BARBARIAN_SILHOUETTE: &[&str] = &[
    "...oo....oo.....",
    "..obboooobbo....",
    "..obbbbbbbbo....",
    "...obbbbbbo.....",
    "...obeeeebbo....",
    "m..obbbbbbo.....",
    ".m.oobbbbbboo...",
    "mooobbbbbbbboo..",
    "oooobbbbbbbbbo..",
    ".ooobbbebbbboo..",
    "..oobbbbbbo.....",
    "...oobbbboo.....",
    "....obbbbbo.....",
    "....odddddo.....",
    "....odddddo.....",
    "....oo...oo.....",
    "...oo.....oo....",
    "...oo..mm.oo....",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
];

fn silhouette_frame(rows: &[&str]) -> SpriteFrame {
    indexed_sprite(rows, SPRITE_SILHOUETTE_PALETTE)
        .expect("storybook sprite silhouette rows are authored as a valid indexed sprite")
}

pub fn sprite_silhouette_lab_fixture(_: &StoryContext) -> AssetAtlas {
    let tiles = [
        ("Goblin silhouette", GOBLIN_SILHOUETTE),
        ("Wizard silhouette", WIZARD_SILHOUETTE),
        ("Barbarian silhouette", BARBARIAN_SILHOUETTE),
    ]
    .into_iter()
    .map(|(label, rows)| AtlasTile {
        label,
        preferred_width: 30,
        preferred_height: 16,
        content: AtlasContent::RgbSprite {
            frame: silhouette_frame(rows),
            background: VOID,
        },
    })
    .collect();

    AssetAtlas { tiles }
}

const GOBLIN_MATERIAL_PALETTE: &[IndexedPaletteEntry] = &[
    IndexedPaletteEntry {
        key: 'o',
        colour: Some(Rgb::new(24, 19, 28)),
    },
    IndexedPaletteEntry {
        key: 's',
        colour: Some(Rgb::new(54, 104, 58)),
    },
    IndexedPaletteEntry {
        key: 'g',
        colour: Some(Rgb::new(107, 183, 82)),
    },
    IndexedPaletteEntry {
        key: 'h',
        colour: Some(Rgb::new(180, 220, 104)),
    },
    IndexedPaletteEntry {
        key: 'd',
        colour: Some(Rgb::new(94, 49, 37)),
    },
    IndexedPaletteEntry {
        key: 'D',
        colour: Some(Rgb::new(170, 92, 56)),
    },
    IndexedPaletteEntry {
        key: 'a',
        colour: Some(Rgb::new(102, 65, 143)),
    },
    IndexedPaletteEntry {
        key: 'm',
        colour: Some(Rgb::new(191, 199, 205)),
    },
    IndexedPaletteEntry {
        key: 'M',
        colour: Some(Rgb::new(244, 239, 209)),
    },
    IndexedPaletteEntry {
        key: 'e',
        colour: Some(Rgb::new(255, 224, 102)),
    },
];

const WIZARD_MATERIAL_PALETTE: &[IndexedPaletteEntry] = &[
    IndexedPaletteEntry {
        key: 'o',
        colour: Some(Rgb::new(24, 19, 28)),
    },
    IndexedPaletteEntry {
        key: 'k',
        colour: Some(Rgb::new(156, 91, 59)),
    },
    IndexedPaletteEntry {
        key: 'K',
        colour: Some(Rgb::new(239, 173, 117)),
    },
    IndexedPaletteEntry {
        key: 'h',
        colour: Some(Rgb::new(255, 214, 157)),
    },
    IndexedPaletteEntry {
        key: 'r',
        colour: Some(Rgb::new(161, 154, 163)),
    },
    IndexedPaletteEntry {
        key: 'R',
        colour: Some(Rgb::new(238, 230, 216)),
    },
    IndexedPaletteEntry {
        key: 'c',
        colour: Some(Rgb::new(60, 49, 126)),
    },
    IndexedPaletteEntry {
        key: 'C',
        colour: Some(Rgb::new(105, 81, 180)),
    },
    IndexedPaletteEntry {
        key: 'l',
        colour: Some(Rgb::new(237, 181, 77)),
    },
    IndexedPaletteEntry {
        key: 'm',
        colour: Some(Rgb::new(122, 81, 46)),
    },
    IndexedPaletteEntry {
        key: 'M',
        colour: Some(Rgb::new(181, 124, 66)),
    },
    IndexedPaletteEntry {
        key: 'e',
        colour: Some(Rgb::new(112, 220, 255)),
    },
];

const BARBARIAN_MATERIAL_PALETTE: &[IndexedPaletteEntry] = &[
    IndexedPaletteEntry {
        key: 'o',
        colour: Some(Rgb::new(24, 19, 28)),
    },
    IndexedPaletteEntry {
        key: 'k',
        colour: Some(Rgb::new(143, 74, 48)),
    },
    IndexedPaletteEntry {
        key: 'K',
        colour: Some(Rgb::new(222, 137, 84)),
    },
    IndexedPaletteEntry {
        key: 'h',
        colour: Some(Rgb::new(255, 198, 132)),
    },
    IndexedPaletteEntry {
        key: 'r',
        colour: Some(Rgb::new(83, 43, 31)),
    },
    IndexedPaletteEntry {
        key: 'R',
        colour: Some(Rgb::new(149, 75, 42)),
    },
    IndexedPaletteEntry {
        key: 'd',
        colour: Some(Rgb::new(93, 48, 34)),
    },
    IndexedPaletteEntry {
        key: 'D',
        colour: Some(Rgb::new(159, 83, 45)),
    },
    IndexedPaletteEntry {
        key: 'm',
        colour: Some(Rgb::new(128, 140, 145)),
    },
    IndexedPaletteEntry {
        key: 'M',
        colour: Some(Rgb::new(224, 230, 224)),
    },
    IndexedPaletteEntry {
        key: 'a',
        colour: Some(Rgb::new(190, 52, 48)),
    },
    IndexedPaletteEntry {
        key: 'e',
        colour: Some(Rgb::new(255, 226, 126)),
    },
];

const GOBLIN_MATERIAL: &[&str] = &[
    "................",
    "................",
    ".oo..........oo.",
    "osso........osso",
    "osggsooooosggso.",
    ".osggggggggggso.",
    ".osghgggggghgso.",
    ".osgeoggoeggso..",
    "..osggddgggso...",
    ".M.osgggggso....",
    "mM.osodddddoso..",
    ".mgosodDddddDoso",
    ".dgosodDaaaaDoso",
    ".dgosodDdMMdDoso",
    "..osodDDDDdoso..",
    "...odddddddo....",
    "...oddaadddo....",
    "...oddo.oddo....",
    "...odo...odo....",
    "..ooo.....ooo...",
    "................",
    "................",
    "................",
    "................",
];

const WIZARD_MATERIAL: &[&str] = &[
    "................",
    "..........oo....",
    ".......occCCo...",
    "......ocCCCCCo..",
    ".e...ocCCCCCCo..",
    "Me..ocCCCCCCo...",
    ".M..ocCCCCCCco..",
    ".m.oolllllloo...",
    ".m..okKKKKko....",
    ".m..okooooko....",
    ".m..okKKKKko....",
    ".m.orRRRRRRro...",
    ".m..orRRRRro....",
    ".m...orRRro.....",
    ".mKokocCCCCoko..",
    ".m..ocClCCco....",
    ".m..ocCCCCCco...",
    ".m..ocCllCCco...",
    ".m..ocCC.CCco...",
    ".m..ocC...CCo...",
    ".m...oo...oo....",
    ".m..oo.....oo...",
    "................",
    "................",
];

const BARBARIAN_MATERIAL: &[&str] = &[
    "................",
    ".....o.o.o......",
    "....orRrRRo.....",
    "...orRRRRRro....",
    "...orRKKKKRro...",
    "...rokhoohkor...",
    "...rokKhKKkor...",
    "....orRooRro....",
    ".mMMookKKKKoo...",
    "mMMMMKddddKKo...",
    ".mMmoKddddKKo...",
    "...mokDaaDoKKo..",
    "...mokDddDoKKo..",
    "...mokDddddKKo..",
    "...m.oodMMdoo...",
    "...m..oddddo....",
    "...m..oddddo....",
    "...m..od.oddo...",
    "...m..oo..oo....",
    "...m.oo....oo...",
    ".....ooo..ooo...",
    "....ooo....ooo..",
    "................",
    "................",
];

const GOBLIN_PORTRAIT_PALETTE: &[IndexedPaletteEntry] = &[
    IndexedPaletteEntry {
        key: 'o',
        colour: Some(Rgb::new(24, 19, 28)),
    },
    IndexedPaletteEntry {
        key: 's',
        colour: Some(Rgb::new(43, 84, 52)),
    },
    IndexedPaletteEntry {
        key: 'g',
        colour: Some(Rgb::new(76, 151, 63)),
    },
    IndexedPaletteEntry {
        key: 'h',
        colour: Some(Rgb::new(132, 194, 75)),
    },
    IndexedPaletteEntry {
        key: 'd',
        colour: Some(Rgb::new(71, 38, 32)),
    },
    IndexedPaletteEntry {
        key: 'D',
        colour: Some(Rgb::new(128, 67, 43)),
    },
    IndexedPaletteEntry {
        key: 'b',
        colour: Some(Rgb::new(188, 111, 58)),
    },
    IndexedPaletteEntry {
        key: 'a',
        colour: Some(Rgb::new(57, 42, 82)),
    },
    IndexedPaletteEntry {
        key: 'A',
        colour: Some(Rgb::new(100, 67, 145)),
    },
    IndexedPaletteEntry {
        key: 'p',
        colour: Some(Rgb::new(153, 103, 190)),
    },
    IndexedPaletteEntry {
        key: 'm',
        colour: Some(Rgb::new(103, 118, 124)),
    },
    IndexedPaletteEntry {
        key: 'M',
        colour: Some(Rgb::new(185, 199, 200)),
    },
    IndexedPaletteEntry {
        key: 'w',
        colour: Some(Rgb::new(244, 239, 209)),
    },
    IndexedPaletteEntry {
        key: 'e',
        colour: Some(Rgb::new(255, 213, 75)),
    },
];

const WIZARD_PORTRAIT_PALETTE: &[IndexedPaletteEntry] = &[
    IndexedPaletteEntry {
        key: 'o',
        colour: Some(Rgb::new(23, 19, 31)),
    },
    IndexedPaletteEntry {
        key: 'c',
        colour: Some(Rgb::new(48, 39, 91)),
    },
    IndexedPaletteEntry {
        key: 'C',
        colour: Some(Rgb::new(82, 64, 151)),
    },
    IndexedPaletteEntry {
        key: 'v',
        colour: Some(Rgb::new(123, 98, 210)),
    },
    IndexedPaletteEntry {
        key: 'k',
        colour: Some(Rgb::new(139, 79, 54)),
    },
    IndexedPaletteEntry {
        key: 'K',
        colour: Some(Rgb::new(226, 145, 91)),
    },
    IndexedPaletteEntry {
        key: 'h',
        colour: Some(Rgb::new(255, 207, 139)),
    },
    IndexedPaletteEntry {
        key: 'r',
        colour: Some(Rgb::new(151, 143, 151)),
    },
    IndexedPaletteEntry {
        key: 'R',
        colour: Some(Rgb::new(215, 207, 201)),
    },
    IndexedPaletteEntry {
        key: 'w',
        colour: Some(Rgb::new(246, 238, 221)),
    },
    IndexedPaletteEntry {
        key: 'l',
        colour: Some(Rgb::new(177, 114, 39)),
    },
    IndexedPaletteEntry {
        key: 'L',
        colour: Some(Rgb::new(247, 193, 74)),
    },
    IndexedPaletteEntry {
        key: 'm',
        colour: Some(Rgb::new(101, 63, 38)),
    },
    IndexedPaletteEntry {
        key: 'M',
        colour: Some(Rgb::new(177, 122, 63)),
    },
    IndexedPaletteEntry {
        key: 'e',
        colour: Some(Rgb::new(64, 193, 231)),
    },
    IndexedPaletteEntry {
        key: 'E',
        colour: Some(Rgb::new(174, 240, 255)),
    },
];

const BARBARIAN_PORTRAIT_PALETTE: &[IndexedPaletteEntry] = &[
    IndexedPaletteEntry {
        key: 'o',
        colour: Some(Rgb::new(26, 19, 26)),
    },
    IndexedPaletteEntry {
        key: 'k',
        colour: Some(Rgb::new(139, 67, 42)),
    },
    IndexedPaletteEntry {
        key: 'K',
        colour: Some(Rgb::new(215, 116, 65)),
    },
    IndexedPaletteEntry {
        key: 'h',
        colour: Some(Rgb::new(255, 187, 111)),
    },
    IndexedPaletteEntry {
        key: 'r',
        colour: Some(Rgb::new(65, 35, 29)),
    },
    IndexedPaletteEntry {
        key: 'R',
        colour: Some(Rgb::new(126, 61, 35)),
    },
    IndexedPaletteEntry {
        key: 'd',
        colour: Some(Rgb::new(79, 43, 35)),
    },
    IndexedPaletteEntry {
        key: 'D',
        colour: Some(Rgb::new(139, 73, 43)),
    },
    IndexedPaletteEntry {
        key: 'b',
        colour: Some(Rgb::new(192, 107, 55)),
    },
    IndexedPaletteEntry {
        key: 'F',
        colour: Some(Rgb::new(221, 211, 180)),
    },
    IndexedPaletteEntry {
        key: 'm',
        colour: Some(Rgb::new(103, 118, 124)),
    },
    IndexedPaletteEntry {
        key: 'M',
        colour: Some(Rgb::new(180, 194, 195)),
    },
    IndexedPaletteEntry {
        key: 'w',
        colour: Some(Rgb::new(231, 235, 221)),
    },
    IndexedPaletteEntry {
        key: 'a',
        colour: Some(Rgb::new(139, 34, 37)),
    },
    IndexedPaletteEntry {
        key: 'A',
        colour: Some(Rgb::new(210, 55, 49)),
    },
    IndexedPaletteEntry {
        key: 'e',
        colour: Some(Rgb::new(238, 225, 185)),
    },
    IndexedPaletteEntry {
        key: 'E',
        colour: Some(Rgb::new(255, 244, 211)),
    },
];

const GOBLIN_PORTRAIT: &[&str] = &[
    "........................",
    "...oo..............oo...",
    ".osso....oooooo....osso.",
    "osggso.osgggggggsoosggso",
    "osgggsosgghhhggsoosgggso",
    ".osggsosgggggggggsosggso",
    "...osgghhgggggghhggso...",
    "...osgheooggggooehggso..",
    "....osgggoohhoohggggso..",
    ".....osgggddddggggso....",
    ".w....osggddddddggso....",
    ".M.....osggggggggso.....",
    ".mM.ooddddddddddoo......",
    ".mM..odDddddddddDdo.....",
    "odo..odDddaaaaaddDdo....",
    ".od.ogDdaAAAAAadDgo.....",
    "..d.ogDdaApppAadDgo.....",
    "....ogDddaaaaaddDgo.....",
    "..odoodDDbbbbDDdo.......",
    "..odoodddddddddo........",
    "..ooooodddDdddoo........",
    ".......odddodddo........",
    ".......oddo.oddo........",
    ".......oddo.oddo........",
    ".......oddo.oddo........",
    "......oodo.oddoo........",
    ".....ooodo.odooo........",
    ".....oooo...oooo........",
    "......oo.....oo.........",
    "........................",
    "........................",
    "........................",
];

const WIZARD_PORTRAIT: &[&str] = &[
    "........................",
    ".............oo.........",
    "...........ocCo.........",
    "..oo......ocCCo.........",
    ".oEeEo...ocCCvCo........",
    ".oEeeo..ocCCvvCo........",
    "..oeo..ocCCvvvCo........",
    "..omo.ocCCvvvvCo........",
    "..omoocCCCvvvvCo........",
    "..omoocCCCCvvvvCo.......",
    "..omooocCCCCCCCCoo......",
    "..omooolLLLLLloo........",
    "..omo.okKKKKko..........",
    "..omo.okohhoko..........",
    "..omo.okKhhKko..........",
    "..omo.orRRRRro..........",
    "..omoorRRwRRro..........",
    "..omo.orRRRRro..........",
    "..oMoorRRRRRro..........",
    "..omkocCCCCCco..........",
    "..omocCCCLCCCco.........",
    "..omocCvCCCCCCo.........",
    "..omocCvCCvCCCo.........",
    "..omocCvCCvCCCo.........",
    "..omocCvLLvCCCo.........",
    "..omocCCllCCCCo.........",
    "..omocCCC.CCCCo.........",
    "..omocCC...CCCo.........",
    "..omocCo...ocCo.........",
    "...ooCo.....oCoo........",
    ".....oo........oo.......",
    "........................",
];

const BARBARIAN_PORTRAIT: &[&str] = &[
    "........................",
    "...........o..o..o......",
    ".........orRrRRRo.......",
    "........orRRRRRRRo......",
    "........orRKKKKRRo......",
    "........orKhhhKRRo......",
    "........orKohhoRRo......",
    "........orKKhKRrRo......",
    "...oMo.oorRRRRRRroo.....",
    ".oMMMo.ookKKKKKKkoo.....",
    "oMwwMo.okKKddddKKKo.....",
    "oMwwMMo.okKKDbbbbDKKo...",
    ".oMMMMookKDbbDDbbDKo....",
    ".....oDokKDbdDDdbDKo....",
    ".....oDokKDddAAddDKo....",
    ".....oDokKDddaaDdDKo....",
    ".....oDokKDddddddDKo....",
    ".....oDokDDdFFdDDko.....",
    ".....oDoodDFeEFdDoo.....",
    ".....oD.odDFFFFDdo......",
    ".....oD.oddddddddo......",
    ".....oD..oddddddo.......",
    ".....oD..oddo.oddo......",
    ".....oD..oddo.oddo......",
    ".....oD..oddo.oddo......",
    ".....oD..oddo.oddo......",
    ".....oD.oodo.oddoo......",
    ".....oDoooro.ooroo......",
    ".....oDoooo...oooo......",
    ".....ooooo.....ooo......",
    "........................",
    "........................",
];

// The second class batch keeps the same two-tier contract as the first three
// fixtures. These are authored frames, rather than palette swaps or resizes:
// gear must remain legible in the 16x24 world and gain material detail in the
// 24x32 portrait.
const BARD_PALETTE: &[IndexedPaletteEntry] = &[
    IndexedPaletteEntry {
        key: 'o',
        colour: Some(Rgb::new(29, 20, 31)),
    },
    IndexedPaletteEntry {
        key: 'k',
        colour: Some(Rgb::new(133, 73, 48)),
    },
    IndexedPaletteEntry {
        key: 'K',
        colour: Some(Rgb::new(235, 157, 101)),
    },
    IndexedPaletteEntry {
        key: 'h',
        colour: Some(Rgb::new(255, 218, 155)),
    },
    IndexedPaletteEntry {
        key: 'r',
        colour: Some(Rgb::new(71, 38, 36)),
    },
    IndexedPaletteEntry {
        key: 'R',
        colour: Some(Rgb::new(119, 59, 46)),
    },
    IndexedPaletteEntry {
        key: 'c',
        colour: Some(Rgb::new(80, 39, 77)),
    },
    IndexedPaletteEntry {
        key: 'C',
        colour: Some(Rgb::new(147, 66, 126)),
    },
    IndexedPaletteEntry {
        key: 'v',
        colour: Some(Rgb::new(211, 96, 164)),
    },
    IndexedPaletteEntry {
        key: 'd',
        colour: Some(Rgb::new(94, 56, 34)),
    },
    IndexedPaletteEntry {
        key: 'D',
        colour: Some(Rgb::new(171, 111, 55)),
    },
    IndexedPaletteEntry {
        key: 'l',
        colour: Some(Rgb::new(237, 181, 77)),
    },
    IndexedPaletteEntry {
        key: 'L',
        colour: Some(Rgb::new(255, 221, 117)),
    },
    IndexedPaletteEntry {
        key: 'e',
        colour: Some(Rgb::new(117, 221, 233)),
    },
];

const RANGER_PALETTE: &[IndexedPaletteEntry] = &[
    IndexedPaletteEntry {
        key: 'o',
        colour: Some(Rgb::new(21, 25, 26)),
    },
    IndexedPaletteEntry {
        key: 'k',
        colour: Some(Rgb::new(120, 72, 46)),
    },
    IndexedPaletteEntry {
        key: 'K',
        colour: Some(Rgb::new(225, 154, 97)),
    },
    IndexedPaletteEntry {
        key: 'h',
        colour: Some(Rgb::new(255, 220, 151)),
    },
    IndexedPaletteEntry {
        key: 'r',
        colour: Some(Rgb::new(54, 43, 25)),
    },
    IndexedPaletteEntry {
        key: 'R',
        colour: Some(Rgb::new(99, 79, 37)),
    },
    IndexedPaletteEntry {
        key: 'c',
        colour: Some(Rgb::new(42, 76, 39)),
    },
    IndexedPaletteEntry {
        key: 'C',
        colour: Some(Rgb::new(79, 125, 57)),
    },
    IndexedPaletteEntry {
        key: 'v',
        colour: Some(Rgb::new(137, 169, 74)),
    },
    IndexedPaletteEntry {
        key: 'd',
        colour: Some(Rgb::new(92, 57, 34)),
    },
    IndexedPaletteEntry {
        key: 'D',
        colour: Some(Rgb::new(169, 111, 55)),
    },
    IndexedPaletteEntry {
        key: 'l',
        colour: Some(Rgb::new(216, 181, 83)),
    },
    IndexedPaletteEntry {
        key: 'a',
        colour: Some(Rgb::new(94, 143, 69)),
    },
    IndexedPaletteEntry {
        key: 'm',
        colour: Some(Rgb::new(130, 143, 125)),
    },
    IndexedPaletteEntry {
        key: 'M',
        colour: Some(Rgb::new(198, 211, 184)),
    },
    IndexedPaletteEntry {
        key: 'e',
        colour: Some(Rgb::new(238, 199, 95)),
    },
];

const ROGUE_PALETTE: &[IndexedPaletteEntry] = &[
    IndexedPaletteEntry {
        key: 'o',
        colour: Some(Rgb::new(21, 18, 29)),
    },
    IndexedPaletteEntry {
        key: 'k',
        colour: Some(Rgb::new(125, 72, 47)),
    },
    IndexedPaletteEntry {
        key: 'K',
        colour: Some(Rgb::new(225, 151, 97)),
    },
    IndexedPaletteEntry {
        key: 'h',
        colour: Some(Rgb::new(255, 218, 151)),
    },
    IndexedPaletteEntry {
        key: 'r',
        colour: Some(Rgb::new(49, 30, 63)),
    },
    IndexedPaletteEntry {
        key: 'R',
        colour: Some(Rgb::new(82, 51, 110)),
    },
    IndexedPaletteEntry {
        key: 'c',
        colour: Some(Rgb::new(52, 38, 78)),
    },
    IndexedPaletteEntry {
        key: 'C',
        colour: Some(Rgb::new(98, 65, 137)),
    },
    IndexedPaletteEntry {
        key: 'v',
        colour: Some(Rgb::new(157, 111, 204)),
    },
    IndexedPaletteEntry {
        key: 'd',
        colour: Some(Rgb::new(86, 54, 35)),
    },
    IndexedPaletteEntry {
        key: 'D',
        colour: Some(Rgb::new(155, 100, 55)),
    },
    IndexedPaletteEntry {
        key: 'l',
        colour: Some(Rgb::new(219, 184, 87)),
    },
    IndexedPaletteEntry {
        key: 'a',
        colour: Some(Rgb::new(70, 190, 181)),
    },
    IndexedPaletteEntry {
        key: 'm',
        colour: Some(Rgb::new(130, 146, 151)),
    },
    IndexedPaletteEntry {
        key: 'M',
        colour: Some(Rgb::new(223, 232, 224)),
    },
    IndexedPaletteEntry {
        key: 'e',
        colour: Some(Rgb::new(91, 228, 223)),
    },
];

const BARD_WORLD: &[&str] = &[
    "................",
    "................",
    ".....oooo.......",
    "....orRRro......",
    "....orKKhro.....",
    "....orKohro.....",
    "....orKKKro.....",
    ".o..ocCCco..d...",
    ".oo.ocCvcodDd...",
    ".oo.ocCCco.dDd..",
    "....ocClcco..d..",
    "....ocCCCco.....",
    "....ocCcco......",
    "....odddddo.....",
    "....odDddDo.....",
    "....odDllDo.....",
    "....odddddo.....",
    "....oddo.oddo...",
    "....odo...odo...",
    "...ooo.....ooo..",
    "................",
    "................",
    "................",
    "................",
];

const RANGER_WORLD: &[&str] = &[
    "................",
    ".......oo.......",
    ".....ocCCo......",
    "....ocCCCco.....",
    "....ocKKhco..d..",
    "....ocKohco.dDd.",
    "....ocKKKco.dDd.",
    ".m..ocCCCco..d..",
    ".M..ocCvcoddd...",
    ".m..ocCCCco.....",
    "....ocClcco.....",
    "....ocCCCco.....",
    "....ocCcco......",
    "....odddddo.....",
    "....odDddDo.....",
    "....odDaaDo.....",
    "....odddddo.....",
    "....oddo.oddo...",
    "....odo...odo...",
    "...ooo.....ooo..",
    "................",
    "................",
    "................",
    "................",
];

const ROGUE_WORLD: &[&str] = &[
    "................",
    "......oooo......",
    ".....ocCCco.....",
    "....ocCCCCco....",
    "....ocKKhCco....",
    "....ocKooCco....",
    "....ocKKKCco....",
    ".m..ocCCCCco..m.",
    ".M..ocCvCCco..M.",
    ".m..ocCCCCco..m.",
    "....ocCllCco....",
    "....ocCCCCco....",
    "....ocCccCco....",
    "....odddddo.....",
    "....odDaaDo.....",
    "....odDddDo.....",
    "....odddddo.....",
    "....oddo.oddo...",
    "....odo...odo...",
    "...ooo.....ooo..",
    "................",
    "................",
    "................",
    "................",
];

const BARD_PORTRAIT: &[&str] = &[
    "........................",
    ".........oooo...........",
    ".......oorRRroo.........",
    "......orRRRRRRro........",
    "......orRKKhhRro........",
    "......orRKooKRRro.......",
    "......orRKKKKRRro.......",
    "......ooRRRRRRoo........",
    ".....occccccccco....d...",
    "....ocCCvvvCCcco...dDd..",
    "....ocCCvvvCCcco..dDDd..",
    "....ocCCcllCCcco...dDd..",
    "....ocCCCCCccccoo...d...",
    "....ocCCcllCCCCco.......",
    "....ocCCCCCCCCCco.......",
    "....ocCCCllllCCco.......",
    "....ocCCCCCCCCCco.......",
    "....ocCCCCC.CCCco.......",
    "....ocCCCC...CCCco......",
    "....odddddddddddo.......",
    "....odDdddddddDdo.......",
    "....odDddllllDdo........",
    "....odDdddddddDdo.......",
    "....oddddddddddo........",
    ".....odddo.odddo........",
    ".....oddo...oddo........",
    "....oooo.....oooo.......",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
];

const RANGER_PORTRAIT: &[&str] = &[
    "........................",
    "..........oooo..........",
    "........occcccco........",
    ".......ocCCCCCCco.......",
    "......ocCCvvvvCCco......",
    "......ocCCKhhKCCco......",
    "......ocCCKooKCCco......",
    "......ocCCKKKKCCco......",
    "......ocCCCCCCCCco......",
    ".m....ocCCCllCCCco...m..",
    ".M....ocCCCCCCCCco...M..",
    ".m...ocCCccccCCCco...m..",
    ".....ocCCcddddcCCco.....",
    ".....ocCCcDddDcCCco.....",
    ".....ocCCcddddcCCco.....",
    ".....ocCCCCCllCCCco.....",
    ".....ocCCCCCCCCCCco.....",
    ".....ocCCC.CCCC.Cco.....",
    ".....ocCCC..CCC..co.....",
    ".....odddddddddddo......",
    ".....odDdddddddDdo......",
    ".....odDdddeeddDdo......",
    ".....odDdddddddDdo......",
    ".....oddddddddddo.......",
    "......odddo.odddo.......",
    "......oddo...oddo.......",
    ".....oooo.....oooo......",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
];

const ROGUE_PORTRAIT: &[&str] = &[
    "........................",
    "........ooooooo.........",
    "......oocCCCCCcoo.......",
    ".....ocCCCCCCCCCco......",
    ".....ocCCvvvvvCCco......",
    ".....ocCCKhhKCCCco......",
    ".....ocCCKooKCCCco......",
    ".....ocCCKKKKCCCco......",
    ".....ocCCCCCCCCCco......",
    ".m...ocCCccccCCCco...m..",
    ".M...ocCCcddddCCCco...M.",
    ".m...ocCCcDddDCCCco...m.",
    ".....ocCCCccccCCCco.....",
    ".....ocCCCllCCCCCco.....",
    ".....ocCCCCCCCCCCco.....",
    ".....ocCCCCCCCCCcco.....",
    ".....ocCCC.CCCC.Ccco....",
    ".....ocCCC..CCC..cco....",
    ".....ocCC...CCC...co....",
    ".....oddddddddddddo.....",
    ".....odDdddddddddDo.....",
    ".....odDdddeeddddDo.....",
    ".....odDdddddddddDo.....",
    ".....oddddddddddddo.....",
    "......odddo..odddo......",
    "......oddo....oddo......",
    ".....oooo......oooo.....",
    "........................",
    "........................",
    "........................",
    "........................",
    "........................",
];

const DRUID_PALETTE: &[IndexedPaletteEntry] = &[
    IndexedPaletteEntry {
        key: 'o',
        colour: Some(Rgb::new(22, 20, 26)),
    },
    IndexedPaletteEntry {
        key: 'c',
        colour: Some(Rgb::new(35, 68, 38)),
    },
    IndexedPaletteEntry {
        key: 'C',
        colour: Some(Rgb::new(65, 116, 55)),
    },
    IndexedPaletteEntry {
        key: 'v',
        colour: Some(Rgb::new(117, 162, 69)),
    },
    IndexedPaletteEntry {
        key: 'k',
        colour: Some(Rgb::new(134, 77, 51)),
    },
    IndexedPaletteEntry {
        key: 'K',
        colour: Some(Rgb::new(225, 153, 99)),
    },
    IndexedPaletteEntry {
        key: 'h',
        colour: Some(Rgb::new(255, 218, 151)),
    },
    IndexedPaletteEntry {
        key: 'w',
        colour: Some(Rgb::new(125, 118, 99)),
    },
    IndexedPaletteEntry {
        key: 'W',
        colour: Some(Rgb::new(219, 211, 183)),
    },
    IndexedPaletteEntry {
        key: 'd',
        colour: Some(Rgb::new(84, 51, 32)),
    },
    IndexedPaletteEntry {
        key: 'D',
        colour: Some(Rgb::new(158, 100, 53)),
    },
    IndexedPaletteEntry {
        key: 'm',
        colour: Some(Rgb::new(110, 86, 52)),
    },
    IndexedPaletteEntry {
        key: 'M',
        colour: Some(Rgb::new(191, 163, 104)),
    },
    IndexedPaletteEntry {
        key: 'l',
        colour: Some(Rgb::new(217, 179, 78)),
    },
    IndexedPaletteEntry {
        key: 'e',
        colour: Some(Rgb::new(105, 232, 150)),
    },
    IndexedPaletteEntry {
        key: 'a',
        colour: Some(Rgb::new(85, 174, 206)),
    },
];

const DRUID_WORLD: &[&str] = &[
    "................",
    "....m......m....",
    ".....m....m.....",
    "......ocCCo.....",
    ".....ocCCCco....",
    "....ocCvvvCco...",
    "....ocCKhKCCo...",
    "....ocCKoKCCo.d.",
    "....ocCKKKCCo.d.",
    "...owWWWWWWwo.d.",
    "...owWwWWwWwo.d.",
    "...ocCCCCCCco.d.",
    "...ocCCaCCCco.d.",
    "...ocCCllCCCco..",
    "...ocCCCCCCco...",
    "...oddddddddo...",
    "...odDddddDdo...",
    "...odDddddDdo...",
    "...oddddddddo...",
    "...oddo..oddo...",
    "...ooo....ooo...",
    "................",
    "................",
    "................",
];

const DRUID_PORTRAIT: &[&str] = &[
    "........................",
    "....m..............m....",
    ".....m............m.....",
    "......M..........M......",
    ".......occccccco........",
    "......ocCCCCCCCCco......",
    ".....ocCCvvvvvvCCco.....",
    ".....ocCCCKhhKCCCco.....",
    ".....ocCCCKooKCCCco..d..",
    ".....ocCCCKKKKCCCco.dD..",
    ".....ocCCCCCCCCCCco.dD..",
    ".....oowWWWWWWWWwoo.dD..",
    ".....owWwwWWWWwwWwo.dD..",
    ".....owWWWWWWWWWWwo.dD..",
    ".....ocCCCccccCCCco.dD..",
    "....ocCCCCCCaCCCCCco.d..",
    "....ocCCCCClllCCCCCco...",
    "....ocCCCCCCCCCCCCCco...",
    "....ocCCCCC.CCCCCCCco...",
    "....ocCCCCC..CCCCCco....",
    "....ocCCCCC...CCCCco....",
    "....oddddddddddddddo....",
    "....odDdddddddddddDo....",
    "....odDddddeedddddDo....",
    "....odDdddddddddddDo....",
    "....oddddddddddddddo....",
    ".....oddddo..oddddo.....",
    ".....odddo....odddo.....",
    "....ooooo......ooooo....",
    "........................",
    "........................",
    "........................",
];

const REVIEW_DELVE: Rgb = Rgb::new(12, 17, 31);
const REVIEW_NEUTRAL: Rgb = Rgb::new(28, 27, 35);
const REVIEW_TORCH: Rgb = Rgb::new(38, 24, 24);

fn material_frame(rows: &[&str], palette: &[IndexedPaletteEntry]) -> SpriteFrame {
    indexed_sprite(rows, palette)
        .expect("storybook material sprite rows are authored as a valid indexed sprite")
}

pub fn sprite_material_and_face_lab_fixture(_: &StoryContext) -> AssetAtlas {
    AssetAtlas {
        tiles: material_review_tiles(1),
    }
}

pub fn sprite_material_inspection_fixture(_: &StoryContext) -> AssetAtlas {
    AssetAtlas {
        tiles: material_review_tiles(2),
    }
}

pub fn sprite_portrait_masters_fixture(_: &StoryContext) -> AssetAtlas {
    let tiles = [
        (
            "Goblin portrait master",
            GOBLIN_MATERIAL,
            GOBLIN_MATERIAL_PALETTE,
            GOBLIN_PORTRAIT,
            GOBLIN_PORTRAIT_PALETTE,
            REVIEW_DELVE,
        ),
        (
            "Wizard portrait master",
            WIZARD_MATERIAL,
            WIZARD_MATERIAL_PALETTE,
            WIZARD_PORTRAIT,
            WIZARD_PORTRAIT_PALETTE,
            REVIEW_NEUTRAL,
        ),
        (
            "Barbarian portrait master",
            BARBARIAN_MATERIAL,
            BARBARIAN_MATERIAL_PALETTE,
            BARBARIAN_PORTRAIT,
            BARBARIAN_PORTRAIT_PALETTE,
            REVIEW_TORCH,
        ),
    ]
    .into_iter()
    .map(
        |(label, world_rows, world_palette, portrait_rows, portrait_palette, background)| {
            AtlasTile {
                label,
                preferred_width: 50,
                preferred_height: 18,
                content: AtlasContent::RgbSpritePair {
                    world: material_frame(world_rows, world_palette),
                    portrait: material_frame(portrait_rows, portrait_palette),
                    background,
                },
            }
        },
    )
    .collect();

    AssetAtlas { tiles }
}

pub fn sprite_scout_and_shadow_masters_fixture(_: &StoryContext) -> AssetAtlas {
    let tiles = [
        (
            "Bard portrait master",
            BARD_WORLD,
            BARD_PALETTE,
            BARD_PORTRAIT,
            BARD_PALETTE,
            REVIEW_TORCH,
        ),
        (
            "Ranger portrait master",
            RANGER_WORLD,
            RANGER_PALETTE,
            RANGER_PORTRAIT,
            RANGER_PALETTE,
            REVIEW_DELVE,
        ),
        (
            "Rogue portrait master",
            ROGUE_WORLD,
            ROGUE_PALETTE,
            ROGUE_PORTRAIT,
            ROGUE_PALETTE,
            REVIEW_NEUTRAL,
        ),
    ]
    .into_iter()
    .map(
        |(label, world_rows, world_palette, portrait_rows, portrait_palette, background)| {
            AtlasTile {
                label,
                preferred_width: 50,
                preferred_height: 18,
                content: AtlasContent::RgbSpritePair {
                    world: material_frame(world_rows, world_palette),
                    portrait: material_frame(portrait_rows, portrait_palette),
                    background,
                },
            }
        },
    )
    .collect();

    AssetAtlas { tiles }
}

pub fn sprite_scout_and_shadow_world_fixture(_: &StoryContext) -> AssetAtlas {
    let tiles = [
        ("Bard world sprite", BARD_WORLD, BARD_PALETTE, REVIEW_TORCH),
        (
            "Ranger world sprite",
            RANGER_WORLD,
            RANGER_PALETTE,
            REVIEW_DELVE,
        ),
        (
            "Rogue world sprite",
            ROGUE_WORLD,
            ROGUE_PALETTE,
            REVIEW_NEUTRAL,
        ),
    ]
    .into_iter()
    .map(|(label, rows, palette, background)| AtlasTile {
        label,
        preferred_width: 30,
        preferred_height: 16,
        content: AtlasContent::RgbSprite {
            frame: material_frame(rows, palette),
            background,
        },
    })
    .collect();

    AssetAtlas { tiles }
}

pub fn sprite_grovekeeper_world_fixture(_: &StoryContext) -> AssetAtlas {
    AssetAtlas {
        tiles: vec![AtlasTile {
            label: "Druid world sprite",
            preferred_width: 30,
            preferred_height: 16,
            content: AtlasContent::RgbSprite {
                frame: material_frame(DRUID_WORLD, DRUID_PALETTE),
                background: REVIEW_DELVE,
            },
        }],
    }
}

pub fn sprite_grovekeeper_masters_fixture(_: &StoryContext) -> AssetAtlas {
    AssetAtlas {
        tiles: vec![AtlasTile {
            label: "Druid portrait master",
            preferred_width: 50,
            preferred_height: 18,
            content: AtlasContent::RgbSpritePair {
                world: material_frame(DRUID_WORLD, DRUID_PALETTE),
                portrait: material_frame(DRUID_PORTRAIT, DRUID_PALETTE),
                background: REVIEW_DELVE,
            },
        }],
    }
}

fn material_review_tiles(scale: u16) -> Vec<AtlasTile> {
    [
        (
            "Goblin material pass",
            GOBLIN_MATERIAL,
            GOBLIN_MATERIAL_PALETTE,
            REVIEW_DELVE,
        ),
        (
            "Wizard material pass",
            WIZARD_MATERIAL,
            WIZARD_MATERIAL_PALETTE,
            REVIEW_NEUTRAL,
        ),
        (
            "Barbarian material pass",
            BARBARIAN_MATERIAL,
            BARBARIAN_MATERIAL_PALETTE,
            REVIEW_TORCH,
        ),
    ]
    .into_iter()
    .map(|(label, rows, palette, background)| AtlasTile {
        label,
        preferred_width: if scale == 1 { 30 } else { 38 },
        preferred_height: if scale == 1 { 16 } else { 28 },
        content: if scale == 1 {
            AtlasContent::RgbSprite {
                frame: material_frame(rows, palette),
                background,
            }
        } else {
            AtlasContent::RgbSpriteScaled {
                frame: material_frame(rows, palette),
                background,
                scale,
            }
        },
    })
    .collect()
}

fn compact_scene_snapshot(context: StoryContext) -> SceneSnapshot {
    let mut snapshot = SceneSnapshot::from_model(&guild_fixture(&context));
    snapshot.agents.truncate(2);
    snapshot
}

pub fn guild_hall_empty_scene_fixture(context: &StoryContext) -> PixelSceneFixture {
    PixelSceneFixture::in_world(
        SceneSnapshot {
            connection: SceneConnection::Connected,
            campaigns: Vec::new(),
            agents: Vec::new(),
            motion: Motion::None,
            now: context.now,
        },
        WorldScene::GuildHall,
    )
}

pub fn guild_hall_mixed_party_scene_fixture(context: &StoryContext) -> PixelSceneFixture {
    PixelSceneFixture::in_world(guild_hall_mixed_snapshot(*context), WorldScene::GuildHall)
}

pub fn guild_hall_counsel_requested_scene_fixture(context: &StoryContext) -> PixelSceneFixture {
    let campaigns = guild_hall_campaigns();
    let mut blocked = guild_hall_agent(
        "Mara-Sealkeeper",
        &campaigns[0].workspace_id,
        Presence::Blocked,
        AccentTone::Magenta,
        context.now,
    );
    blocked.transition = Some(SceneTransition {
        summons: GuildSummons::CounselRequested,
        since: Timestamp::from_millis(context.now.as_millis().saturating_sub(8_000)),
    });
    PixelSceneFixture::in_world(
        SceneSnapshot {
            connection: SceneConnection::Connected,
            campaigns,
            agents: vec![blocked],
            motion: Motion::None,
            now: context.now,
        },
        WorldScene::GuildHall,
    )
}

pub fn guild_hall_spoils_returned_scene_fixture(context: &StoryContext) -> PixelSceneFixture {
    let campaigns = guild_hall_campaigns();
    let mut returned = guild_hall_agent(
        "Ivo-Runeporter",
        &campaigns[1].workspace_id,
        Presence::Done,
        AccentTone::Amber,
        context.now,
    );
    returned.transition = Some(SceneTransition {
        summons: GuildSummons::SpoilsReturned,
        since: Timestamp::from_millis(context.now.as_millis().saturating_sub(2_000)),
    });
    PixelSceneFixture::in_world(
        SceneSnapshot {
            connection: SceneConnection::Connected,
            campaigns,
            agents: vec![returned],
            motion: Motion::Full,
            now: context.now,
        },
        WorldScene::GuildHall,
    )
}

pub fn guild_hall_reconnecting_scene_fixture(context: &StoryContext) -> PixelSceneFixture {
    let mut snapshot = guild_hall_mixed_snapshot(*context);
    snapshot.connection = SceneConnection::Reconnecting { attempt: 3 };
    PixelSceneFixture::in_world(snapshot, WorldScene::GuildHall)
}

pub fn guild_hall_minimum_viewport_scene_fixture(context: &StoryContext) -> PixelSceneFixture {
    let campaigns = guild_hall_campaigns();
    PixelSceneFixture::in_world(
        SceneSnapshot {
            connection: SceneConnection::Connected,
            agents: vec![guild_hall_agent(
                "Nia-Bellward",
                &campaigns[0].workspace_id,
                Presence::Blocked,
                AccentTone::Cyan,
                context.now,
            )],
            campaigns,
            motion: Motion::None,
            now: context.now,
        },
        WorldScene::GuildHall,
    )
}

fn guild_hall_mixed_snapshot(context: StoryContext) -> SceneSnapshot {
    let campaigns = guild_hall_campaigns();
    let agents = vec![
        guild_hall_agent(
            "Elowen-Typeweaver",
            &campaigns[0].workspace_id,
            Presence::Working,
            AccentTone::Cyan,
            context.now,
        ),
        guild_hall_agent(
            "Bram-Pathfinder",
            &campaigns[0].workspace_id,
            Presence::Unknown,
            AccentTone::Lime,
            context.now,
        ),
        guild_hall_agent(
            "Sable-Watch",
            &campaigns[1].workspace_id,
            Presence::Idle,
            AccentTone::Teal,
            context.now,
        ),
        guild_hall_agent(
            "Orin-Caskwright",
            &campaigns[1].workspace_id,
            Presence::Done,
            AccentTone::Blue,
            context.now,
        ),
    ];
    SceneSnapshot {
        connection: SceneConnection::Connected,
        campaigns,
        agents,
        motion: Motion::None,
        now: context.now,
    }
}

fn guild_hall_campaigns() -> Vec<SceneCampaign> {
    vec![
        SceneCampaign {
            workspace_id: WorkspaceId::new("scene-guild-library"),
            label: "Amber Library".to_owned(),
            variant_seed: 0x45a7_011d,
        },
        SceneCampaign {
            workspace_id: WorkspaceId::new("scene-guild-undercroft"),
            label: "Mossy Undercroft".to_owned(),
            variant_seed: 0x9b02_c471,
        },
    ]
}

fn guild_hall_agent(
    key: &str,
    workspace_id: &WorkspaceId,
    presence: Presence,
    accent: AccentTone,
    now: Timestamp,
) -> SceneAgent {
    let mut persona = AdventurerPersona::for_key(PersonaKey::new(format!("scene-guild-{key}")));
    persona.appearance.accent = accent;
    SceneAgent {
        key: AgentKey::new(key),
        workspace_id: workspace_id.clone(),
        name: key.replace('-', " "),
        custom_status: None,
        presence,
        presence_since: Timestamp::from_millis(now.as_millis().saturating_sub(20_000)),
        transition: None,
        focused: false,
        persona,
    }
}

pub fn delve_active_party_scene_fixture(context: &StoryContext) -> PixelSceneFixture {
    PixelSceneFixture::in_world(delve_active_snapshot(*context), WorldScene::Delve)
}

pub fn delve_mixed_states_scene_fixture(context: &StoryContext) -> PixelSceneFixture {
    let campaigns = delve_campaigns();
    PixelSceneFixture::in_world(
        SceneSnapshot {
            connection: SceneConnection::Connected,
            agents: vec![
                delve_agent(
                    "Tala-Pathfinder",
                    &campaigns[0].workspace_id,
                    Presence::Working,
                    AccentTone::Cyan,
                    context.now,
                ),
                delve_agent(
                    "Mara-Sealkeeper",
                    &campaigns[0].workspace_id,
                    Presence::Blocked,
                    AccentTone::Magenta,
                    context.now,
                ),
                delve_agent(
                    "Ivo-Runeporter",
                    &campaigns[1].workspace_id,
                    Presence::Done,
                    AccentTone::Amber,
                    context.now,
                ),
                delve_agent(
                    "Sable-Campward",
                    &campaigns[1].workspace_id,
                    Presence::Idle,
                    AccentTone::Lime,
                    context.now,
                ),
                delve_agent(
                    "Orin-Unlit",
                    &campaigns[0].workspace_id,
                    Presence::Unknown,
                    AccentTone::Violet,
                    context.now,
                ),
                delve_agent(
                    "Bram-Departed",
                    &campaigns[1].workspace_id,
                    Presence::Exited,
                    AccentTone::Red,
                    context.now,
                ),
            ],
            campaigns,
            motion: Motion::None,
            now: context.now,
        },
        WorldScene::Delve,
    )
}

pub fn delve_sealed_gate_scene_fixture(context: &StoryContext) -> PixelSceneFixture {
    let campaigns = delve_campaigns();
    let mut blocked = delve_agent(
        "Mara-Sealkeeper",
        &campaigns[0].workspace_id,
        Presence::Blocked,
        AccentTone::Magenta,
        context.now,
    );
    blocked.transition = Some(SceneTransition {
        summons: GuildSummons::CounselRequested,
        since: Timestamp::from_millis(context.now.as_millis().saturating_sub(6_000)),
    });
    PixelSceneFixture::in_world(
        SceneSnapshot {
            connection: SceneConnection::Connected,
            campaigns,
            agents: vec![blocked],
            motion: Motion::None,
            now: context.now,
        },
        WorldScene::Delve,
    )
}

pub fn delve_reconnecting_scene_fixture(context: &StoryContext) -> PixelSceneFixture {
    let mut snapshot = delve_active_snapshot(*context);
    snapshot.connection = SceneConnection::Reconnecting { attempt: 3 };
    PixelSceneFixture::in_world(snapshot, WorldScene::Delve)
}

pub fn delve_minimum_viewport_scene_fixture(context: &StoryContext) -> PixelSceneFixture {
    let campaigns = delve_campaigns();
    let mut focused = delve_agent(
        "Nia-Deepwalker",
        &campaigns[0].workspace_id,
        Presence::Working,
        AccentTone::Teal,
        context.now,
    );
    focused.focused = true;
    PixelSceneFixture::in_world(
        SceneSnapshot {
            connection: SceneConnection::Connected,
            campaigns,
            agents: vec![focused],
            motion: Motion::None,
            now: context.now,
        },
        WorldScene::Delve,
    )
}

pub fn scene_first_motion_full_fixture(context: &StoryContext) -> PixelSceneFixture {
    let mut snapshot = delve_active_snapshot(*context);
    snapshot.motion = Motion::Full;
    PixelSceneFixture::in_world(snapshot, WorldScene::Delve)
}

pub fn scene_first_motion_reduced_fixture(context: &StoryContext) -> PixelSceneFixture {
    let mut snapshot = guild_hall_mixed_snapshot(*context);
    snapshot.motion = Motion::Reduced;
    PixelSceneFixture::in_world(snapshot, WorldScene::GuildHall)
}

pub fn scene_first_motion_none_fixture(context: &StoryContext) -> PixelSceneFixture {
    let mut snapshot = guild_hall_mixed_snapshot(*context);
    snapshot.motion = Motion::None;
    PixelSceneFixture::in_world(snapshot, WorldScene::GuildHall)
}

pub fn scene_first_minimum_viewport_fixture(context: &StoryContext) -> PixelSceneFixture {
    delve_minimum_viewport_scene_fixture(context)
}

fn delve_active_snapshot(context: StoryContext) -> SceneSnapshot {
    let campaigns = delve_campaigns();
    SceneSnapshot {
        connection: SceneConnection::Connected,
        agents: vec![
            delve_agent(
                "Tala-Pathfinder",
                &campaigns[0].workspace_id,
                Presence::Working,
                AccentTone::Cyan,
                context.now,
            ),
            delve_agent(
                "Elowen-Runesight",
                &campaigns[0].workspace_id,
                Presence::Working,
                AccentTone::Lime,
                context.now,
            ),
            delve_agent(
                "Ivo-Deepward",
                &campaigns[1].workspace_id,
                Presence::Working,
                AccentTone::Amber,
                context.now,
            ),
        ],
        campaigns,
        motion: Motion::None,
        now: context.now,
    }
}

fn delve_campaigns() -> Vec<SceneCampaign> {
    vec![
        SceneCampaign {
            workspace_id: WorkspaceId::new("scene-delve-moss-vault"),
            label: "Moss Vault".to_owned(),
            variant_seed: 0x47a1_0ee5,
        },
        SceneCampaign {
            workspace_id: WorkspaceId::new("scene-delve-rune-road"),
            label: "Rune Road".to_owned(),
            variant_seed: 0xb20f_91c3,
        },
    ]
}

fn delve_agent(
    key: &str,
    workspace_id: &WorkspaceId,
    presence: Presence,
    accent: AccentTone,
    now: Timestamp,
) -> SceneAgent {
    let mut persona = AdventurerPersona::for_key(PersonaKey::new(format!("scene-delve-{key}")));
    persona.appearance.accent = accent;
    SceneAgent {
        key: AgentKey::new(key),
        workspace_id: workspace_id.clone(),
        name: key.replace('-', " "),
        custom_status: None,
        presence,
        presence_since: Timestamp::from_millis(now.as_millis().saturating_sub(20_000)),
        transition: None,
        focused: false,
        persona,
    }
}
