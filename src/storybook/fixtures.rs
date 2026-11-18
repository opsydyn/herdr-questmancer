use std::{collections::BTreeMap, path::PathBuf};

use crate::{
    app::{ConnectionState, Model, OutputPreview, View},
    command::{AgentCommand, CommandResult},
    domain::{
        AdventurerClass, AdventurerPersona, Agent, AgentKey, Ancestry, Campaign, Chronicle,
        DomainState, GuildAttention, GuildSummons, PaneId, PersonaKey, Presence, TabId, Timestamp,
        WorkspaceId,
    },
    interaction::reduce_action,
    runtime_loop::apply_command_result,
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
    WorldFit(Box<Model>),
    ArchetypeGallery(ArchetypeGallery),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FitParty {
    Five,
    Eleven,
    Crowded,
}

/// Fixed dungeon party using the same source facts as the Hall studies.
pub fn delve_fit_fixture(context: StoryContext, party: FitParty) -> Model {
    let mut model = hall_fit_fixture(context, party);
    model.switch_to(View::Delve);
    model
}

/// Reproduces the reported crowded-resting screenshot without live agents.
pub fn delve_resting_fit_fixture(context: StoryContext) -> Model {
    let mut model = delve_fit_fixture(context, FitParty::Five);
    for (index, agent) in model.domain_mut().agents.values_mut().enumerate() {
        agent.presence = if index == 3 {
            Presence::Unknown
        } else {
            Presence::Idle
        };
        agent.attention = GuildAttention::Clear;
    }
    model
}

/// No Herdr bindings or commands are executed by these local preview fixtures.
pub fn hall_fit_fixture(context: StoryContext, party: FitParty) -> Model {
    let left = WorkspaceId::new("fit-library");
    let right = WorkspaceId::new("fit-workshop");
    let count = if party == FitParty::Five { 5 } else { 11 };
    let classes = [
        AdventurerClass::Druid,
        AdventurerClass::Bard,
        AdventurerClass::Cleric,
        AdventurerClass::Sorcerer,
        AdventurerClass::Sorcerer,
    ];
    let mut agents = BTreeMap::new();
    for i in 0..count {
        let presence = if party == FitParty::Crowded {
            Presence::Working
        } else if party == FitParty::Five {
            [
                Presence::Working,
                Presence::Blocked,
                Presence::Working,
                Presence::Idle,
                Presence::Unknown,
            ][i]
        } else {
            match i {
                4 => Presence::Blocked,
                5 => Presence::Idle,
                6 | 7 => Presence::Done,
                _ => Presence::Working,
            }
        };
        let campaign = if party == FitParty::Crowded || !(2..4).contains(&i) {
            &left
        } else {
            &right
        };
        let mut a = fixture_agent(
            &format!("fit-{i:02}"),
            campaign,
            presence,
            presence == Presence::Blocked,
        );
        a.persona.class = classes[i % classes.len()];
        a.custom_status = None;
        agents.insert(a.key.clone(), a);
    }
    let campaigns = [&left, &right]
        .into_iter()
        .map(|id| {
            (
                id.clone(),
                Campaign {
                    workspace_id: id.clone(),
                    label: id.as_str().replace('-', " "),
                    cwd: PathBuf::from("/storybook/fit"),
                    party: agents
                        .values()
                        .filter(|a| &a.workspace_id == id)
                        .map(|a| a.key.clone())
                        .collect(),
                },
            )
        })
        .collect();
    let mut model = Model::new(View::Guild);
    model.replace_domain(DomainState {
        capture: crate::domain::CaptureClock::default(),
        campaigns,
        agents,
        selected_agent: Some(AgentKey::new("fit-00")),
        chronicle: Chronicle::new(16),
    });
    model.set_connection(ConnectionState::Connected);
    model.set_now(context.now);
    let mut preferences = *model.preferences();
    preferences.motion = crate::app::Motion::None;
    model.set_preferences(preferences);
    model
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchetypeGallery {
    WorldMasters,
    RitualPoses(AdventurerClass),
    PersonaPalettes,
    RosterFamilies,
    CustomClassMasters,
    PortraitMasters,
    GoblinEasterEgg,
    Librarian,
    RatRoyalty,
}

pub const fn ritual_pose_fixture(class: AdventurerClass) -> StoryFixture {
    StoryFixture::ArchetypeGallery(ArchetypeGallery::RitualPoses(class))
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

pub fn native_cleric_portrait_fixture(context: StoryContext) -> Model {
    native_portrait_fixture(context, AdventurerClass::Cleric, Ancestry::Human)
}

pub fn native_druid_portrait_fixture(context: StoryContext) -> Model {
    native_portrait_fixture(context, AdventurerClass::Druid, Ancestry::Elf)
}

pub fn native_paladin_portrait_fixture(context: StoryContext) -> Model {
    native_portrait_fixture(context, AdventurerClass::Paladin, Ancestry::Dwarf)
}

pub fn native_ranger_portrait_fixture(context: StoryContext) -> Model {
    native_portrait_fixture(context, AdventurerClass::Ranger, Ancestry::Halfling)
}

pub fn native_rogue_portrait_fixture(context: StoryContext) -> Model {
    native_portrait_fixture(context, AdventurerClass::Rogue, Ancestry::Elf)
}

pub fn native_testmender_portrait_fixture(context: StoryContext) -> Model {
    native_portrait_fixture(context, AdventurerClass::Testmender, Ancestry::Gnome)
}

pub fn native_wizard_portrait_fixture(context: StoryContext) -> Model {
    native_portrait_fixture(context, AdventurerClass::Wizard, Ancestry::Human)
}

pub fn native_runewright_portrait_fixture(context: StoryContext) -> Model {
    native_portrait_fixture(context, AdventurerClass::Runewright, Ancestry::Human)
}

pub fn native_pathseeker_portrait_fixture(context: StoryContext) -> Model {
    native_portrait_fixture(context, AdventurerClass::Pathseeker, Ancestry::Human)
}

pub fn native_mage_portrait_fixture(context: StoryContext) -> Model {
    native_portrait_fixture(context, AdventurerClass::Mage, Ancestry::Human)
}

pub fn native_sorcerer_portrait_fixture(context: StoryContext) -> Model {
    native_portrait_fixture(context, AdventurerClass::Sorcerer, Ancestry::Human)
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
    let mut model = guild_world_fixture(context);
    let preview = model
        .output_preview()
        .cloned()
        .expect("authored scrying output");
    let effects = reduce_action(&mut model, Action::Refresh);
    let [
        AgentCommand::LoadOutput {
            pane_id, request, ..
        },
    ] = effects.commands.as_slice()
    else {
        unreachable!("the connected fixture selects a live adventurer");
    };
    // Settle the authored response through the same request boundary as live
    // scrying; Storybook never executes the socket command.
    apply_command_result(
        &mut model,
        CommandResult::OutputLoaded {
            pane_id: pane_id.clone(),
            request: *request,
            revision: preview.revision,
            text: preview.text,
            truncated: false,
        },
        context.now,
    );
    model
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
        capture: crate::domain::CaptureClock::default(),
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
        capture_identity: crate::domain::CaptureIdentity::default(),
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

#[derive(Clone, Copy, Debug)]
pub enum WorkBoardState {
    Populated,
    Empty,
    Offline,
    Deferred,
    Resolved,
    Narrow,
    WorkerExited,
}
pub fn quest_board_fixture(context: StoryContext, state: WorkBoardState) -> Model {
    use crate::work::{
        self, LiveBinding, Message, ObligationId, QuestId, SourceId, WorkSelection,
        fixture::FixtureSource,
    };
    let mut model = guild_world_fixture(context);
    if matches!(state, WorkBoardState::Empty) {
        model.open_quest_board();
        return model;
    }
    let key = model
        .domain()
        .agents
        .keys()
        .next()
        .expect("fixture party")
        .clone();
    let agent = model
        .domain_mut()
        .agents
        .get_mut(&key)
        .expect("fixture agent");
    agent.name = "Linus".into();
    agent.presence = Presence::Working;
    agent.attention = GuildAttention::Clear;
    agent.capture_identity = crate::domain::CaptureIdentity::from_parts(
        "fixture-terminal",
        Some(&crate::herdr::protocol::AgentSessionInfo {
            source: "storybook".into(),
            agent: "Linus".into(),
            kind: "fixture".into(),
            value: "auth-1".into(),
        }),
    );
    let binding = LiveBinding {
        agent: key.clone(),
        pane: agent.pane_id.clone(),
        identity: agent
            .capture_identity
            .fingerprint()
            .expect("qualified fixture"),
        counsel_since: None,
    };
    let source = SourceId::new("fixture-auth");
    let quest = QuestId::new(&source, "web");
    let id = ObligationId::new(&source, "compatibility");
    let mut projection = work::Projection::default();
    for mut event in FixtureSource::authentication(context.now) {
        if let Message::Baseline { bindings, .. } = &mut event {
            bindings.insert(quest.clone(), binding.clone());
        }
        projection = work::reduce(projection, event).expect("authored source");
    }
    match state {
        WorkBoardState::Offline => {
            projection = work::reduce(projection, Message::Disconnected(source.clone()))
                .expect("disconnect");
        }
        WorkBoardState::Deferred => {
            projection.acknowledge(&id, Some(context.now.plus(Model::SNOOZE)));
        }
        WorkBoardState::Resolved => {
            projection = work::reduce(projection, FixtureSource::resolution(context.now))
                .expect("resolution");
        }
        WorkBoardState::WorkerExited => {
            model
                .domain_mut()
                .agents
                .get_mut(&key)
                .expect("fixture agent")
                .presence = Presence::Exited;
            projection = work::reduce(projection, Message::BindingRemoved { source, quest })
                .expect("binding removal");
        }
        WorkBoardState::Populated | WorkBoardState::Empty | WorkBoardState::Narrow => {}
    }
    model.replace_work(projection);
    model.select_work(WorkSelection::Obligation(id));
    model
}

#[derive(Clone, Copy, Debug)]
pub enum WarTableState {
    Populated,
    Narrow,
    Empty,
    Offline,
    Cyclic,
    Large,
}
pub fn war_table_fixture(context: StoryContext, state: WarTableState) -> Model {
    use crate::work::{self, Change, Message, QuestId, SourceId, WorkSelection};
    let board = match state {
        WarTableState::Empty => WorkBoardState::Empty,
        WarTableState::Offline => WorkBoardState::Offline,
        _ => WorkBoardState::Populated,
    };
    let mut model = quest_board_fixture(context, board);
    let source = SourceId::new("fixture-auth");
    let mut changes = Vec::new();
    if matches!(state, WarTableState::Cyclic | WarTableState::Large) {
        let seed = model
            .work()
            .quest(&QuestId::new(&source, "api"))
            .expect("auth fixture")
            .clone();
        if matches!(state, WarTableState::Cyclic) {
            let mut q = seed;
            q.revision += 1;
            q.dependencies = vec![QuestId::new(&source, "tests")];
            changes.push(Change::Quest(q));
        } else {
            for i in 0..96 {
                let mut q = seed.clone();
                q.id = QuestId::new(&source, &format!("extra-{i:03}"));
                q.revision = 1;
                q.assignment = None;
                q.dependencies.clear();
                q.title = format!("Synthetic campaign work {i:03}");
                changes.push(Change::Quest(q));
            }
        }
        let projection = work::reduce(
            model.work().clone(),
            Message::Update {
                source: source.clone(),
                generation: 1,
                occurred_at: context.now,
                cursor: 2,
                event: "table-scenario".into(),
                changes,
            },
        )
        .expect("authored table scenario");
        model.replace_work(projection);
    }
    if matches!(state, WarTableState::Cyclic) {
        model.select_work(WorkSelection::Quest(QuestId::new(&source, "api")));
    }
    model.open_war_table();
    model
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwnedAuthoringState {
    Actions,
    EmptyCampaign,
    Answer,
    Narrow,
    ViewPending,
    Indeterminate,
    Answered,
    Small,
}
/// Fixed owned facts and production forms; no files, runtime owner or random identities.
#[allow(clippy::too_many_lines)] // One fixed fixture composes facts and all form/recovery variants.
pub fn owned_authoring_fixture(context: StoryContext, variant: OwnedAuthoringState) -> Model {
    use crate::{
        owned_work::{
            Action, Answer, Command, CommandId, OwnedState, StoreSelection,
            authoring::{FormKind, Metadata, Phase, SelectedDetail, Surface},
            transition,
        },
        work::{CampaignId, ObligationId, SourceId},
    };
    let source = SourceId::new("owned:storybook");
    let campaign = CampaignId::new(&source, "authentication");
    let decision = ObligationId::new(&source, "compatibility");
    let mut state = OwnedState::new(source.clone()).expect("fixed source");
    let prompt = "Should the old token endpoint remain for backwards compatibility during the authentication migration?";
    let actions = vec![
        Action::CreateCampaign {
            id: campaign.clone(),
            name: "Replace authentication layer".into(),
            objective: Some("Keep existing clients working during migration.".into()),
        },
        Action::RequestDecision {
            id: decision.clone(),
            campaign: campaign.clone(),
            quest: None,
            incarnation: "request-1".into(),
            prompt: prompt.into(),
            context: Some(
                "The web migration needs an explicit policy before it can proceed.".into(),
            ),
        },
    ];
    for (i, action) in actions.into_iter().enumerate() {
        state = transition(
            &state,
            &Command {
                id: CommandId::new(&format!("fixed-{i}")),
                source: source.clone(),
                expected_revision: state.revision(),
                actor: "Questmancer".into(),
                at: context.now,
                action,
            },
        )
        .expect("fixed transition")
        .state;
    }
    let answer=Answer {text:"Keep the old endpoint for one release and document its removal date.\nThe new endpoint becomes the documented default.".into(),actor:"Questmancer".into(),at:context.now};
    if variant == OwnedAuthoringState::Answered {
        state = transition(
            &state,
            &Command {
                id: CommandId::new("fixed-answer"),
                source: source.clone(),
                expected_revision: state.revision(),
                actor: answer.actor.clone(),
                at: answer.at,
                action: Action::AnswerDecision {
                    id: decision.clone(),
                    incarnation: "request-1".into(),
                    answer: answer.text.clone(),
                },
            },
        )
        .expect("fixed answer")
        .state;
    }
    let mut model = fixture_model(context, View::Guild);
    let projection = crate::sources::local_work::LocalWorkSource::new(&state, model.work())
        .expect("fixed source")
        .baseline(&state, context.now)
        .expect("fixed baseline");
    model.replace_work(
        crate::work::reduce(model.work().clone(), projection.message).expect("fixed projection"),
    );
    let a = model.authoring_mut();
    a.available = true;
    a.phase = Phase::Ready;
    a.campaign = Some(campaign);
    a.selection = Some(
        StoreSelection::new("/synthetic/owned-work.json".into(), source.clone())
            .expect("fixed pin"),
    );
    a.metadata = Some(Metadata {
        source,
        revision: state.revision(),
        inventory: state.inventory().clone(),
    });
    a.detail = Some(SelectedDetail {
        id: decision,
        incarnation: "request-1".into(),
        revision: state.revision(),
        prompt: prompt.into(),
        context: Some("The web migration needs an explicit policy before it can proceed.".into()),
        requested_by: "Questmancer".into(),
        answer: if variant == OwnedAuthoringState::Answered {
            Some(answer.clone())
        } else {
            None
        },
    });
    match variant {
        OwnedAuthoringState::Actions => a.menu(),
        OwnedAuthoringState::EmptyCampaign => a.surface = Surface::Campaigns,
        OwnedAuthoringState::Answer | OwnedAuthoringState::Narrow | OwnedAuthoringState::Small => {
            a.form(FormKind::Answer);
            a.fields[0].text = answer.text;
        }
        OwnedAuthoringState::ViewPending => {
            a.phase = Phase::ViewPending;
            a.surface = Surface::Store;
            a.feedback="Answer saved. The Board has not updated yet. Retry view update; this does not submit another answer.".into();
        }
        OwnedAuthoringState::Indeterminate => {
            a.phase = Phase::Indeterminate;
            a.surface = Surface::Store;
            a.feedback =
                "The answer may have saved. Check its outcome before submitting another command."
                    .into();
        }
        OwnedAuthoringState::Answered => a.surface = Surface::Detail,
    }
    model.open_owned_authoring();
    model
}
