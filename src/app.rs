use std::{collections::BTreeMap, time::Duration};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::{
    chronicle_chapter::{ChapterRequest, ChapterWindow},
    config::OutputPreviewLines,
    domain::{
        AdventurerClass, Agent, AgentKey, DomainState, PaneId, PersonaKey, Presence, Timestamp,
    },
    ledger::LedgerPageId,
    persistence::DurableIntent,
    update::{AppEvent, update},
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GoblinState {
    released_at: Option<Timestamp>,
}

impl GoblinState {
    pub const OUTBREAK_DURATION: Duration = Duration::from_secs(3);

    pub const fn release(&mut self, now: Timestamp) {
        self.released_at = Some(now);
    }

    #[must_use]
    pub fn is_visible(self, now: Timestamp) -> bool {
        self.released_at
            .is_some_and(|start| now >= start && start.elapsed_until(now) < Self::OUTBREAK_DURATION)
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum View {
    #[default]
    Guild,
    Delve,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Motion {
    #[default]
    Full,
    Reduced,
    None,
}

impl Motion {
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Full => Self::Reduced,
            Self::Reduced => Self::None,
            Self::None => Self::Full,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Reduced => "reduced",
            Self::None => "still",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CharacterSet {
    #[default]
    Unicode,
    Ascii,
}

impl CharacterSet {
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Unicode => Self::Ascii,
            Self::Ascii => Self::Unicode,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Unicode => "Unicode",
            Self::Ascii => "ASCII",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorMode {
    #[default]
    Xterm256,
    Ansi16,
}

impl ColorMode {
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Xterm256 => Self::Ansi16,
            Self::Ansi16 => Self::Xterm256,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Xterm256 => "truecolour",
            Self::Ansi16 => "16 colours",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DisplayPreferences {
    pub motion: Motion,
    pub character_set: CharacterSet,
    pub color_mode: ColorMode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeSettings {
    pub output_preview_lines: OutputPreviewLines,
    pub reviewr_action: String,
    pub show_elapsed_time: bool,
    pub sidebar_urgency_order: bool,
}

impl Default for RuntimeSettings {
    fn default() -> Self {
        Self {
            output_preview_lines: OutputPreviewLines::default(),
            reviewr_action: "persiyanov.reviewr.open".to_owned(),
            show_elapsed_time: true,
            sidebar_urgency_order: false,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum ConnectionState {
    #[default]
    Offline,
    Connecting,
    Connected,
    Reconnecting {
        attempt: u32,
    },
    Incompatible {
        expected: u32,
        actual: u32,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectionStateKind {
    Offline,
    Connecting,
    Connected,
    Reconnecting,
    Incompatible,
}

impl ConnectionStateKind {
    pub const ALL: &'static [Self] = &[
        Self::Offline,
        Self::Connecting,
        Self::Connected,
        Self::Reconnecting,
        Self::Incompatible,
    ];
}

impl ConnectionState {
    #[must_use]
    pub const fn kind(&self) -> ConnectionStateKind {
        match self {
            Self::Offline => ConnectionStateKind::Offline,
            Self::Connecting => ConnectionStateKind::Connecting,
            Self::Connected => ConnectionStateKind::Connected,
            Self::Reconnecting { .. } => ConnectionStateKind::Reconnecting,
            Self::Incompatible { .. } => ConnectionStateKind::Incompatible,
        }
    }
}

/// How long an action confirmation stays on screen.
///
/// Longer than the command ribbon's three seconds: the ribbon is a reminder
/// you can re-summon by moving, whereas this may be the only report that an
/// action succeeded or was clipped, and it must survive being read.
pub const ACTION_FEEDBACK_TTL: Duration = Duration::from_millis(6_000);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Notice {
    ConnectionDiagnostic(String),
    ActionFeedback(String),
    /// A correlated counsel operation confirmed both text and submission.
    CounselIssued(String),
    PersistenceDiagnostic(String),
    ReviewrAvailabilityDiagnostic(String),
    IntegrationDiagnostic(String),
}

impl Notice {
    pub fn message(&self) -> &str {
        match self {
            Self::ConnectionDiagnostic(message)
            | Self::ActionFeedback(message)
            | Self::CounselIssued(message)
            | Self::PersistenceDiagnostic(message)
            | Self::ReviewrAvailabilityDiagnostic(message)
            | Self::IntegrationDiagnostic(message) => message,
        }
    }

    pub const fn is_connection_diagnostic(&self) -> bool {
        matches!(self, Self::ConnectionDiagnostic(_))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Notices {
    connection: Option<Notice>,
    action: Option<Notice>,
    persistence: Option<Notice>,
    reviewr: Option<Notice>,
    integration: Option<Notice>,
}

impl Notices {
    fn primary(&self) -> Option<&Notice> {
        self.action
            .as_ref()
            .or(self.persistence.as_ref())
            .or(self.reviewr.as_ref())
            .or(self.integration.as_ref())
            .or(self.connection.as_ref())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum Modal {
    #[default]
    None,
    LibrarianLedger {
        page: LedgerPageId,
    },
    Counsel {
        draft: String,
        phase: CounselPhase,
    },
    Search {
        query: String,
    },
    Scrying,
    Chronicle,
    QuestBoard,
    QuestInbox {
        detail: bool,
    },
    OwnedAuthoring,
    WarTable,
    CharacterPicker {
        adventurer: AgentKey,
        persona: PersonaKey,
        choice: usize,
        return_page: Option<LedgerPageId>,
    },
    PartyName {
        draft: String,
        return_page: LedgerPageId,
    },
    StatusExplanation(Box<StatusExplanation>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusRequest {
    pub serial: u64,
    pub pane: PaneId,
    pub identity: crate::domain::CaptureIdentity,
    pub presence: Presence,
    pub presence_since: Timestamp,
}

impl StatusRequest {
    pub(crate) fn matches(&self, agent: &Agent) -> bool {
        agent.pane_id == self.pane
            && agent.capture_identity == self.identity
            && agent.presence == self.presence
            && agent.presence_since == self.presence_since
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusExplanation {
    pub request: StatusRequest,
    pub outcome: Option<Result<crate::herdr::protocol::AgentExplanation, String>>,
}

/// Where an open counsel parchment is in its send.
///
/// The parchment used to close the instant `Enter` was pressed, which left the
/// result of the send with nowhere to land: the success notice arrived after
/// the surface that should have shown it was gone, and a failure was
/// indistinguishable from a success. "In flight" has to be a state you can
/// store before it can be a state you can draw.
///
/// `Sending` carries the request it is waiting on. Results are correlated
/// against it and anything else is ignored, so counsel sent to one adventurer
/// cannot resolve a parchment since reopened for another.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum CounselPhase {
    #[default]
    Drafting,
    Sending {
        request: CounselRequest,
        pane_id: PaneId,
    },
    /// The text was not accepted, so retrying the complete write is safe.
    TextFailed { message: String },
    /// The text request was sent but its acknowledgement was lost or invalid.
    /// The pane must be inspected because rewriting could duplicate counsel.
    TextUncertain {
        request: CounselRequest,
        pane_id: PaneId,
        message: String,
    },
    /// The text was accepted but its Enter submission was not. A retry may
    /// submit the existing prompt, but must never write the text again.
    SubmissionFailed {
        request: CounselRequest,
        pane_id: PaneId,
        message: String,
    },
}

/// Identifies one counsel send, so a late result cannot resolve a later draft.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct CounselRequest(pub u64);

/// A counsel operation whose Herdr result has not settled yet.
///
/// This deliberately lives on the model rather than in [`Modal`]. The user
/// may dismiss a parchment while its I/O is still running; presentation
/// lifetime must not erase the operation identity needed to settle that I/O.
#[derive(Clone, Debug, Eq, PartialEq)]
struct CounselAttempt {
    agent_key: AgentKey,
    pane_id: PaneId,
    draft: String,
    stage: CounselAttemptStage,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum CounselAttemptStage {
    Sending,
    TextUncertain { message: String },
    SubmissionFailed { message: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputPreview {
    pub pane_id: PaneId,
    pub revision: u64,
    pub text: String,
    pub loading: bool,
    pub error: Option<String>,
}

/// Identifies one selected-output read for this model's lifetime. It is never
/// persisted or reused after a selection or connection boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OutputRequest(pub u64);

#[derive(Clone, Debug, Eq, PartialEq)]
struct OutputContext {
    identity: crate::domain::CaptureIdentity,
    agent_key: AgentKey,
    pane_id: PaneId,
    pending: Option<OutputRequest>,
}

/// A transient comparison of live party facts, never saved in the model.
#[derive(Debug)]
pub(crate) struct PartyActivity {
    members: Vec<(AgentKey, PaneId, crate::domain::CaptureIdentity)>,
    all_resting: bool,
    all_known: bool,
}

impl PartyActivity {
    pub(crate) fn from_domain(domain: &DomainState) -> Self {
        let agents = domain
            .agents
            .values()
            .filter(|agent| agent.presence != Presence::Exited);
        let members = agents
            .clone()
            .map(|agent| {
                (
                    agent.key.clone(),
                    agent.pane_id.clone(),
                    agent.capture_identity.clone(),
                )
            })
            .collect::<Vec<_>>();
        Self {
            all_resting: !members.is_empty()
                && agents.clone().all(|agent| agent.presence == Presence::Idle),
            all_known: agents
                .clone()
                .all(|agent| agent.presence != Presence::Unknown),
            members,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Model {
    pub(crate) snapshot_refresh: crate::snapshot_refresh::SnapshotRefresh,
    view: View,
    authoring: crate::owned_work::authoring::Authoring,
    work: crate::work::Projection,
    work_source_notice: Option<crate::work::source::SourceNotice>,
    pending_work: Option<crate::work::Projection>,
    work_selection: Option<crate::work::WorkSelection>,
    inbox_adventurer: Option<crate::work::inbox::AdventurerLink>,
    board_viewport_width: u16,
    work_viewport_height: u16,
    war_campaign: Option<crate::work::CampaignId>,
    domain: DomainState,
    connection: ConnectionState,
    scene_transition_floor: Option<Timestamp>,
    party_rest_since: Option<Timestamp>,
    modal: Modal,
    inbox_is_home: bool,
    adventurer_card_visible: bool,
    output_preview: Option<OutputPreview>,
    output_context: Option<OutputContext>,
    next_output_request: u64,
    next_status_request: u64,
    notices: Box<Notices>,
    reviewr_available: bool,
    now: Timestamp,
    preferences: DisplayPreferences,
    settings: RuntimeSettings,
    durable_intent: DurableIntent,
    managed_pane_id: Option<PaneId>,
    goblins: GoblinState,
    last_interaction_at: Option<Timestamp>,
    action_feedback_at: Option<Timestamp>,
    search: SearchResults,
    reading_scroll: u16,
    chronicle_chapter: Option<ChapterRequest>,
    counsel_drafts: BTreeMap<AgentKey, String>,
    active_counsel: BTreeMap<CounselRequest, CounselAttempt>,
    next_counsel_request: u64,
}

/// The party a search matched, kept so the matches after the first are
/// reachable.
///
/// Search used to `find_map` the first hit and drop the rest on the floor: a
/// query matching three adventurers silently picked one and never said the
/// others existed.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct SearchResults {
    query: String,
    matched: Vec<AgentKey>,
}

impl Model {
    pub fn new(view: View) -> Self {
        Self {
            snapshot_refresh: crate::snapshot_refresh::SnapshotRefresh::default(),
            view,
            authoring: crate::owned_work::authoring::Authoring::default(),
            inbox_adventurer: None,
            work: crate::work::Projection::default(),
            work_source_notice: None,
            pending_work: None,
            work_selection: None,
            board_viewport_width: 80,
            work_viewport_height: 40,
            war_campaign: None,
            domain: DomainState::default(),
            connection: ConnectionState::Offline,
            scene_transition_floor: None,
            party_rest_since: None,
            modal: Modal::None,
            inbox_is_home: false,
            adventurer_card_visible: false,
            output_preview: None,
            output_context: None,
            next_output_request: 0,
            next_status_request: 0,
            notices: Box::default(),
            reviewr_available: false,
            now: Timestamp::from_millis(0),
            preferences: DisplayPreferences::default(),
            settings: RuntimeSettings::default(),
            durable_intent: DurableIntent::default(),
            managed_pane_id: None,
            goblins: GoblinState::default(),
            last_interaction_at: None,
            action_feedback_at: None,
            search: SearchResults::default(),
            reading_scroll: 0,
            chronicle_chapter: None,
            counsel_drafts: BTreeMap::new(),
            active_counsel: BTreeMap::new(),
            next_counsel_request: 0,
        }
    }

    pub fn owned_form_fits(&self) -> bool {
        self.board_viewport_width >= 28 && self.work_viewport_height >= 12
    }
    pub fn authoring(&self) -> &crate::owned_work::authoring::Authoring {
        &self.authoring
    }
    pub fn authoring_mut(&mut self) -> &mut crate::owned_work::authoring::Authoring {
        &mut self.authoring
    }
    pub fn open_owned_authoring(&mut self) {
        self.keep_counsel_draft();
        self.modal = Modal::OwnedAuthoring;
    }

    pub fn work(&self) -> &crate::work::Projection {
        &self.work
    }
    pub fn work_source_notice(&self) -> Option<&crate::work::source::SourceNotice> {
        self.work_source_notice.as_ref()
    }
    pub fn set_work_source_notice(&mut self, notice: Option<crate::work::source::SourceNotice>) {
        self.work_source_notice = notice;
    }

    pub fn replace_work(&mut self, work: crate::work::Projection) {
        self.work = work;
        if self
            .work_selection
            .as_ref()
            .is_some_and(|s| !crate::work::view::items(self).contains(s))
        {
            self.work_selection = crate::work::view::items(self).into_iter().next();
            self.reading_scroll = 0;
        }
        if self.modal == Modal::WarTable {
            self.normalise_war_selection();
        }
        if matches!(self.modal, Modal::QuestInbox { .. }) {
            self.normalise_inbox_selection();
        }
    }
    pub fn open_war_table(&mut self) {
        self.keep_counsel_draft();
        self.modal = Modal::WarTable;
        self.reading_scroll = 0;
        // Opening from a Board obligation follows its explicit quest/campaign.
        self.war_campaign = crate::work::view::selected_quest(self)
            .and_then(|id| self.work.quest(id))
            .map(|q| q.campaign.clone())
            .or_else(|| match &self.work_selection {
                Some(crate::work::WorkSelection::Obligation(id)) => {
                    self.work.obligation(id).map(|o| o.campaign.clone())
                }
                _ => crate::work::graph::campaign_id(self),
            });
        self.normalise_war_selection();
    }
    fn normalise_war_selection(&mut self) {
        let campaign = crate::work::graph::campaign_id(self);
        self.war_campaign.clone_from(&campaign);
        let selected = crate::work::view::selected_quest(self).cloned();
        let next = selected
            .filter(|id| {
                self.work
                    .quest(id)
                    .is_some_and(|q| Some(&q.campaign) == campaign.as_ref())
            })
            .or_else(|| {
                self.work
                    .sources
                    .values()
                    .flat_map(|s| s.inventory.quests.values())
                    .find(|q| Some(&q.campaign) == campaign.as_ref())
                    .map(|q| q.id.clone())
            })
            .map(crate::work::WorkSelection::Quest);
        if self.work_selection != next {
            self.work_selection = next;
            self.reading_scroll = 0;
        }
    }
    pub fn cycle_war_quest(&mut self, forward: bool) {
        let Some(graph) = crate::work::graph::CampaignGraph::from_model(self) else {
            return;
        };
        let ids = graph.nodes.keys().cloned().collect::<Vec<_>>();
        if ids.is_empty() {
            return;
        }
        let index = graph
            .selected
            .as_ref()
            .and_then(|id| ids.iter().position(|q| q == id))
            .unwrap_or(0);
        let next = if forward {
            (index + 1) % ids.len()
        } else {
            (index + ids.len() - 1) % ids.len()
        };
        self.work_selection = Some(crate::work::WorkSelection::Quest(ids[next].clone()));
        self.reading_scroll = 0;
    }
    pub fn cycle_war_campaign(&mut self, forward: bool) {
        let ids = crate::work::graph::campaign_ids(self);
        if ids.is_empty() {
            return;
        }
        let current = crate::work::graph::campaign_id(self);
        let index = current
            .as_ref()
            .and_then(|id| ids.iter().position(|q| q == id))
            .unwrap_or(0);
        let next = if forward {
            (index + 1) % ids.len()
        } else {
            (index + ids.len() - 1) % ids.len()
        };
        self.war_campaign = Some(ids[next].clone());
        self.normalise_war_selection();
        self.reading_scroll = 0;
    }
    pub fn war_campaign(&self) -> Option<&crate::work::CampaignId> {
        self.war_campaign.as_ref()
    }
    pub fn work_selection(&self) -> Option<&crate::work::WorkSelection> {
        self.work_selection.as_ref()
    }
    pub fn open_quest_board(&mut self) {
        self.keep_counsel_draft();
        self.inbox_is_home = false;
        self.modal = Modal::QuestBoard;
        self.reading_scroll = 0;
        if self.work_selection.is_none() {
            self.work_selection = crate::work::view::items(self).into_iter().next();
        }
    }
    pub fn select_work(&mut self, selection: crate::work::WorkSelection) {
        self.open_quest_board();
        self.work_selection = Some(selection);
        self.reading_scroll = 0;
    }
    pub(crate) fn select_inbox_item(&mut self, id: crate::work::ObligationId) {
        self.work_selection = Some(crate::work::WorkSelection::Obligation(id));
        self.reading_scroll = 0;
    }
    pub(crate) fn link_inbox_adventurer(&mut self) {
        let linked = crate::work::view::selected_quest(self)
            .and_then(|quest| crate::work::inbox::AdventurerLink::new(self, quest));
        let Some(link) = linked else {
            self.set_action_feedback(
                "Select a quest and a live adventurer with verified session identity.".into(),
            );
            return;
        };
        self.inbox_adventurer = Some(link);
        self.set_action_feedback("Linked selected adventurer for this review session.".into());
    }
    pub(crate) fn inbox_adventurer(&self, quest: &crate::work::QuestId) -> Option<&Agent> {
        self.inbox_adventurer
            .as_ref()
            .filter(|link| &link.quest == quest)?
            .resolve(self)
    }
    pub(crate) fn open_inbox(&mut self, detail: bool) {
        self.modal = Modal::QuestInbox { detail };
        self.reading_scroll = 0;
        self.normalise_inbox_selection();
    }
    pub(crate) fn open_startup_inbox(&mut self) {
        self.inbox_is_home = true;
        self.open_inbox(false);
    }

    pub(crate) const fn inbox_is_home(&self) -> bool {
        self.inbox_is_home
    }
    fn normalise_inbox_selection(&mut self) {
        use crate::work::{WorkSelection, inbox::rows};
        let rows = rows(&self.work, self.now);
        let current = self.work_selection.as_ref();
        let row = rows
            .iter()
            .find(|row| current.is_some_and(|s| row.selected(s)))
            .or(rows.first());
        self.work_selection = row.map(|row| {
            current
                .filter(|s| matches!(s, WorkSelection::Obligation(_)) && row.selected(s))
                .cloned()
                .unwrap_or_else(|| WorkSelection::Obligation(row.items[0].clone()))
        });
    }
    pub(crate) fn cycle_inbox(&mut self, forward: bool, within_quest: bool) {
        use crate::work::{WorkSelection, inbox::rows};
        let rows = rows(&self.work, self.now);
        let Some(index) = self
            .work_selection
            .as_ref()
            .and_then(|s| rows.iter().position(|row| row.selected(s)))
        else {
            self.normalise_inbox_selection();
            return;
        };
        let step = |index, len| {
            if forward {
                (index + 1) % len
            } else {
                (index + len - 1) % len
            }
        };
        let id = if within_quest {
            let items = &rows[index].items;
            let item = items
                .iter()
                .position(|id| self.work_selection == Some(WorkSelection::Obligation(id.clone())))
                .unwrap_or(0);
            &items[step(item, items.len())]
        } else {
            &rows[step(index, rows.len())].items[0]
        };
        self.select_inbox_item(id.clone());
    }
    pub fn cycle_work(&mut self, forward: bool) {
        let items = crate::work::view::items(self);
        if items.is_empty() {
            return;
        }
        let index = self
            .work_selection
            .as_ref()
            .and_then(|s| items.iter().position(|i| i == s));
        let next = index.map_or(0, |i| {
            if forward {
                (i + 1) % items.len()
            } else {
                (i + items.len() - 1) % items.len()
            }
        });
        self.work_selection = Some(items[next].clone());
        self.reading_scroll = 0;
    }
    pub fn request_work_intent(&mut self, defer: bool) {
        self.update_work_intent(defer, false);
    }
    pub(crate) fn request_inbox_intent(&mut self, defer: bool) {
        self.update_work_intent(defer, true);
    }
    fn update_work_intent(&mut self, defer: bool, restore: bool) {
        let Some(crate::work::WorkSelection::Obligation(id)) = &self.work_selection else {
            self.set_action_feedback("Select an open summons to mark seen or defer.".into());
            return;
        };
        let Some(obligation) = self
            .work
            .obligation(id)
            .filter(|o| o.state == crate::work::ObligationState::Open)
        else {
            self.set_action_feedback("This source obligation is already closed.".into());
            return;
        };
        let mut candidate = self.work.clone();
        if defer
            && restore
            && crate::work::inbox::deferred_until(&self.work, obligation, self.now).is_some()
        {
            if let Some(intent) = candidate.intent.get_mut(id) {
                intent.deferred_until = None;
            }
        } else {
            candidate.acknowledge(id, defer.then(|| self.now.plus(Self::SNOOZE)));
        }
        self.pending_work = Some(candidate);
    }
    pub fn set_work_viewport_size(&mut self, width: u16, height: u16) {
        self.board_viewport_width = width;
        self.work_viewport_height = height;
    }
    pub fn set_board_viewport_width(&mut self, width: u16) {
        self.board_viewport_width = width;
    }
    pub fn clear_work_selection(&mut self) {
        self.work_selection = None;
    }

    pub fn pending_work(&self) -> Option<&crate::work::Projection> {
        self.pending_work.as_ref()
    }
    pub fn settle_work_intent(&mut self, work: Option<crate::work::Projection>) {
        self.pending_work = None;
        if let Some(work) = work {
            self.work = work;
            self.set_action_feedback("Saved locally; source obligation remains open.".into());
        } else {
            self.set_action_feedback(
                "Local intent was not saved. The obligation remains open.".into(),
            );
        }
    }

    pub const fn view(&self) -> View {
        self.view
    }

    pub const fn switch_to(&mut self, view: View) {
        self.view = view;
    }

    pub const fn connection(&self) -> &ConnectionState {
        &self.connection
    }

    pub fn set_connection(&mut self, connection: ConnectionState) {
        self.set_connection_at(connection, self.now);
    }

    pub fn set_connection_at(&mut self, connection: ConnectionState, observed_at: Timestamp) {
        self.inbox_adventurer = None;
        self.snapshot_refresh
            .connection_changed(connection == ConnectionState::Connected);
        self.domain.capture.connection_changed(
            self.snapshot_refresh.epoch(),
            connection == ConnectionState::Connected,
        );
        self.party_rest_since = None;
        // Even Connected -> Connected starts a fresh snapshot lifetime.
        self.invalidate_output();
        if let Modal::StatusExplanation(status) = &mut self.modal {
            status.outcome = Some(Err("Herdr connection changed. Reopen to refresh.".into()));
        }
        // Retained summons remain truthful history, but cannot replay theatre
        // after a socket boundary. This presentation cutoff is never persisted.
        self.scene_transition_floor = Some(
            self.scene_transition_floor
                .map_or(observed_at, |previous| previous.max(observed_at)),
        );
        self.connection = connection;
    }

    pub const fn scene_transition_floor(&self) -> Option<Timestamp> {
        self.scene_transition_floor
    }

    pub const fn party_rest_since(&self) -> Option<Timestamp> {
        self.party_rest_since
    }

    pub(crate) fn observe_party_activity(
        &mut self,
        previous: &PartyActivity,
        observed_at: Option<Timestamp>,
    ) {
        let current = PartyActivity::from_domain(&self.domain);
        let same_party = current.members == previous.members;
        if !current.all_resting || !same_party {
            self.party_rest_since = None;
        }
        if same_party
            && previous.all_known
            && !previous.all_resting
            && current.all_resting
            && self.connection == ConnectionState::Connected
            && self.preferences.motion == Motion::Full
            && let Some(at) =
                observed_at.filter(|at| self.scene_transition_floor.is_none_or(|floor| *at > floor))
        {
            self.party_rest_since = Some(at);
        }
    }

    pub const fn domain(&self) -> &DomainState {
        &self.domain
    }

    pub fn domain_mut(&mut self) -> &mut DomainState {
        &mut self.domain
    }

    pub(crate) fn take_domain(&mut self) -> DomainState {
        self.remember_current_selection();
        std::mem::take(&mut self.domain)
    }

    pub fn replace_domain(&mut self, mut domain: DomainState) {
        self.remember_current_selection();
        if self
            .domain
            .selected_agent
            .as_ref()
            .is_some_and(|key| domain.agents.contains_key(key))
        {
            domain
                .selected_agent
                .clone_from(&self.domain.selected_agent);
        } else if domain
            .selected_agent
            .as_ref()
            .is_none_or(|key| !domain.agents.contains_key(key))
        {
            domain.selected_agent = domain.agents.keys().next().cloned();
        }
        self.durable_intent.overlay(&mut domain);
        self.domain = domain;
        if self
            .inbox_adventurer
            .as_ref()
            .is_some_and(|link| link.resolve(self).is_none())
        {
            self.inbox_adventurer = None;
        }
        self.reconcile_output();
    }

    fn remember_current_selection(&mut self) {
        let selected_persona = self.selected_agent().map(|agent| agent.persona.key.clone());
        if selected_persona.is_some() {
            self.durable_intent.remember_selected(selected_persona);
        }
    }

    /// How long `s` sets a summons aside for.
    ///
    /// Long enough to finish what you are doing, short enough that a snooze is
    /// not a quiet dismissal. Deferring is deliberately not persisted across a
    /// restart: a summons still genuinely needs answering, and reopening
    /// Questmancer is a reasonable moment to be reminded of it.
    pub const SNOOZE: Duration = Duration::from_secs(15 * 60);

    /// Sets the selected adventurer's summons aside. Returns false when there
    /// is nothing to set aside, so the caller can say so.
    pub fn defer_selected_summons(&mut self) -> bool {
        let Some(agent) = self.selected_agent() else {
            return false;
        };
        if agent.attention.summons().is_none() {
            return false;
        }
        let agent_key = agent.key.clone();
        let until = self.now.plus(Self::SNOOZE);
        let domain = self.take_domain();
        let (domain, _commands) = update(domain, AppEvent::DeferSummons { agent_key, until });
        self.replace_domain(domain);
        true
    }

    pub fn mark_selected_attention_read(&mut self) {
        let Some(agent_key) = self.selected_agent_key().cloned() else {
            return;
        };
        let domain = self.take_domain();
        let (domain, _commands) = update(domain, AppEvent::MarkRead(agent_key));
        self.replace_domain(domain);
    }

    pub fn selected_agent(&self) -> Option<&Agent> {
        self.domain
            .selected_agent
            .as_ref()
            .and_then(|key| self.domain.agents.get(key))
    }

    pub fn select_next_agent(&mut self) {
        self.move_agent_selection(1);
    }

    pub fn select_previous_agent(&mut self) {
        self.move_agent_selection(-1);
    }

    pub fn select_first_agent(&mut self) {
        self.set_agent_selection(self.domain.agents.keys().next().cloned());
    }

    pub fn select_last_agent(&mut self) {
        self.set_agent_selection(self.domain.agents.keys().next_back().cloned());
    }

    pub fn select_agent(&mut self, agent: &AgentKey) {
        if self.domain.agents.contains_key(agent) {
            self.set_agent_selection(Some(agent.clone()));
        }
    }

    fn set_agent_selection(&mut self, selected: Option<AgentKey>) {
        self.work_selection = None;
        if self.domain.selected_agent != selected {
            self.invalidate_output();
        }
        self.domain.selected_agent = selected;
    }

    /// The adventurers waiting on a human, most urgent first.
    ///
    /// Ordering is by what the party actually needs rather than by name. An
    /// unanswered call for counsel outranks one somebody has already seen,
    /// which outranks the quieter summons; within a rank the adventurer who
    /// has waited longest comes first, because waiting is the whole cost being
    /// measured. Deliberately deferred summons are excluded until their snooze
    /// expires — that is what deferring meant.
    #[must_use]
    pub fn adventurers_awaiting_a_human(&self) -> Vec<AgentKey> {
        let mut waiting = self
            .domain
            .agents
            .values()
            .filter_map(|agent| {
                let rank = agent.urgency(self.now)?;
                let since = agent
                    .attention
                    .since()
                    .unwrap_or(agent.presence_since)
                    .as_millis();
                Some((rank, since, agent.key.clone()))
            })
            .collect::<Vec<_>>();
        waiting.sort_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then(left.1.cmp(&right.1))
                .then_with(|| left.2.cmp(&right.2))
        });
        waiting.into_iter().map(|(_, _, key)| key).collect()
    }

    /// Moves the selection to the next adventurer waiting on a human, wrapping.
    ///
    /// Selection was sequential only: reaching the one adventurer that needed
    /// something meant stepping past every adventurer that did not. Returns
    /// false when nobody is waiting, so the caller can say so rather than
    /// moving the selection somewhere arbitrary.
    pub fn select_next_agent_awaiting_a_human(&mut self) -> bool {
        let waiting = self.adventurers_awaiting_a_human();
        let Some(first) = waiting.first() else {
            return false;
        };
        let next = self
            .domain
            .selected_agent
            .as_ref()
            .and_then(|selected| waiting.iter().position(|key| key == selected))
            .map_or(first, |position| &waiting[(position + 1) % waiting.len()]);
        self.set_agent_selection(Some(next.clone()));
        true
    }

    fn move_agent_selection(&mut self, direction: i8) {
        let keys = self.domain.agents.keys().cloned().collect::<Vec<_>>();
        if keys.is_empty() {
            self.set_agent_selection(None);
            return;
        }
        let current = self
            .domain
            .selected_agent
            .as_ref()
            .and_then(|selected| keys.iter().position(|key| key == selected))
            .unwrap_or(0);
        let next = if direction.is_positive() {
            current.saturating_add(1).min(keys.len() - 1)
        } else {
            current.saturating_sub(1)
        };
        self.set_agent_selection(Some(keys[next].clone()));
    }

    /// Records what a search matched, in the party's own order.
    pub fn set_search_results(&mut self, query: String, matched: Vec<AgentKey>) {
        self.search = SearchResults { query, matched };
    }

    #[must_use]
    pub fn search_query(&self) -> &str {
        &self.search.query
    }

    /// Live matches only: the party changes under a stale result set, and
    /// cycling onto an adventurer who has left would be worse than saying the
    /// search is spent.
    #[must_use]
    pub fn search_results(&self) -> Vec<AgentKey> {
        self.search
            .matched
            .iter()
            .filter(|key| self.domain.agents.contains_key(*key))
            .cloned()
            .collect()
    }

    /// Steps to the next or previous search match, wrapping.
    ///
    /// Returns the one-based position and the total, so the caller can say
    /// which of how many you are looking at — the thing the old
    /// first-match-only search could never tell you.
    pub fn cycle_search_result(&mut self, forward: bool) -> Option<(usize, usize)> {
        let matched = self.search_results();
        if matched.is_empty() {
            return None;
        }
        let current = self
            .domain
            .selected_agent
            .as_ref()
            .and_then(|selected| matched.iter().position(|key| key == selected));
        let next = match current {
            Some(position) if forward => (position + 1) % matched.len(),
            Some(position) => (position + matched.len() - 1) % matched.len(),
            None => 0,
        };
        self.set_agent_selection(Some(matched[next].clone()));
        Some((next + 1, matched.len()))
    }

    /// The guild's lifetime experience.
    #[must_use]
    pub const fn experience(&self) -> u64 {
        self.durable_intent.experience()
    }

    /// Records the standing earned by a Chronicle event.
    ///
    /// Called where the reducer has already decided the event is new — the
    /// Chronicle dedupes by `EventId`, so the same returned spoils cannot be
    /// paid for twice. Startup replay assigns the Chronicle wholesale rather
    /// than appending through the reducer, so a restart re-reads history
    /// without re-earning it.
    pub const fn earn_experience(&mut self, experience: u64) {
        self.durable_intent.earn(experience);
    }

    pub fn open_chronicle(&mut self) {
        self.chronicle_chapter = None;
        self.modal = Modal::Chronicle;
        self.reading_scroll = 0;
    }

    pub fn toggle_chronicle_chapter(&mut self) {
        if self.modal != Modal::Chronicle {
            return;
        }
        self.chronicle_chapter = if self.chronicle_chapter.is_some() {
            None
        } else {
            Some(ChapterRequest {
                window: ChapterWindow::last_hour(self.now),
                adventurer: None,
            })
        };
        self.reading_scroll = 0;
    }

    pub fn chronicle_chapter_lines(&self) -> Option<Vec<String>> {
        self.chronicle_chapter
            .as_ref()
            .map(|request| request.project(&self.domain.chronicle).lines())
    }

    /// The Chronicle entries the view should show, newest first.
    ///
    /// Scoped to the selected adventurer when there is one, because "what has
    /// this agent been doing" is the question the Hall is usually asked; with
    /// nothing selected it reads the whole guild's history.
    #[must_use]
    pub fn chronicle_entries(&self, limit: usize) -> Vec<&crate::domain::ChronicleEntry> {
        let selected = self.domain.selected_agent.as_ref();
        self.domain
            .chronicle
            .entries()
            .iter()
            .rev()
            .filter(|entry| {
                selected.is_none_or(|key| entry.adventurer().is_some_and(|had| had == key))
            })
            .take(limit)
            .collect()
    }

    /// Moves the selection to the first adventurer of the next campaign.
    ///
    /// Replaces `cycle_guild_focus`, which walked eight "landmark" variants
    /// that no renderer, overlay or command ever read — `Tab` changed a field
    /// and nothing else, and its test asserted precisely that: a deterministic
    /// cycle producing no commands and no effects.
    ///
    /// Campaigns are the grouping the party actually has, and until now there
    /// was no way to move between them. Returns false when there is nothing to
    /// move to, so a single campaign does not pretend to cycle.
    pub fn select_next_campaign(&mut self) -> bool {
        let campaigns = self
            .domain
            .campaigns
            .values()
            .filter(|campaign| !campaign.party.is_empty())
            .collect::<Vec<_>>();
        if campaigns.len() < 2 {
            return false;
        }
        let current = self
            .selected_agent()
            .map(|agent| agent.workspace_id.clone());
        let position = current
            .as_ref()
            .and_then(|workspace| {
                campaigns
                    .iter()
                    .position(|campaign| &campaign.workspace_id == workspace)
            })
            .map_or(0, |position| (position + 1) % campaigns.len());
        let next = campaigns[position]
            .party
            .iter()
            .find(|key| self.domain.agents.contains_key(key))
            .cloned();
        let Some(next) = next else {
            return false;
        };
        self.set_agent_selection(Some(next));
        true
    }

    pub const fn modal(&self) -> &Modal {
        &self.modal
    }

    pub const fn adventurer_card_visible(&self) -> bool {
        self.adventurer_card_visible
    }

    pub const fn show_adventurer_card(&mut self) {
        self.adventurer_card_visible = true;
    }

    pub const fn dismiss_adventurer_card(&mut self) {
        self.adventurer_card_visible = false;
    }

    /// Opens the counsel parchment, restoring whatever was last drafted for
    /// this adventurer.
    ///
    /// `Esc` used to discard the draft outright, so a slip mid-sentence cost
    /// the whole message. Drafts are kept per adventurer rather than in a
    /// single slot: restoring someone else's half-written counsel would be a
    /// worse failure than losing it.
    pub fn open_counsel(&mut self) {
        let selected = self.domain.selected_agent.as_ref();
        if let Some((request, attempt)) = self
            .active_counsel
            .iter()
            .find(|(_, attempt)| Some(&attempt.agent_key) == selected)
        {
            let phase = match &attempt.stage {
                CounselAttemptStage::Sending => CounselPhase::Sending {
                    request: *request,
                    pane_id: attempt.pane_id.clone(),
                },
                CounselAttemptStage::SubmissionFailed { message } => {
                    CounselPhase::SubmissionFailed {
                        request: *request,
                        pane_id: attempt.pane_id.clone(),
                        message: message.clone(),
                    }
                }
                CounselAttemptStage::TextUncertain { message } => CounselPhase::TextUncertain {
                    request: *request,
                    pane_id: attempt.pane_id.clone(),
                    message: message.clone(),
                },
            };
            self.modal = Modal::Counsel {
                draft: attempt.draft.clone(),
                phase,
            };
            return;
        }
        let draft = self
            .selected_agent()
            .and_then(|agent| self.counsel_drafts.get(&agent.key).cloned())
            .unwrap_or_default();
        self.modal = Modal::Counsel {
            draft,
            phase: CounselPhase::Drafting,
        };
    }

    /// The phase of the open counsel parchment, if one is open.
    #[must_use]
    pub const fn counsel_phase(&self) -> Option<&CounselPhase> {
        match &self.modal {
            Modal::Counsel { phase, .. } => Some(phase),
            _ => None,
        }
    }

    /// Moves an open parchment from drafting to in-flight, returning the
    /// request its result must quote to be believed.
    pub fn begin_counsel_send(&mut self, pane_id: PaneId) -> Option<CounselRequest> {
        let Modal::Counsel { draft, phase } = &self.modal else {
            return None;
        };
        // A second `Enter` while the first send is still out would issue the
        // same counsel twice.
        if matches!(phase, CounselPhase::Sending { .. }) {
            return None;
        }
        let agent_key = self.domain.selected_agent.clone()?;
        if self
            .active_counsel
            .values()
            .any(|attempt| attempt.agent_key == agent_key)
        {
            return None;
        }
        let draft = draft.clone();
        self.next_counsel_request = self.next_counsel_request.wrapping_add(1);
        let request = CounselRequest(self.next_counsel_request);
        self.active_counsel.insert(
            request,
            CounselAttempt {
                agent_key,
                pane_id: pane_id.clone(),
                draft,
                stage: CounselAttemptStage::Sending,
            },
        );
        let Modal::Counsel { phase, .. } = &mut self.modal else {
            unreachable!("the counsel modal was checked above")
        };
        *phase = CounselPhase::Sending { request, pane_id };
        Some(request)
    }

    /// Settles a send that succeeded: the parchment closes and the draft is
    /// spent, including any copy set aside earlier for this adventurer.
    ///
    /// Returns false for an unknown or already settled request. A late result
    /// still settles its own operation without closing somebody else's draft.
    pub fn complete_counsel(&mut self, request: CounselRequest) -> bool {
        let Some(attempt) = self.active_counsel.remove(&request) else {
            return false;
        };
        if self.counsel_request_matches(request) {
            self.modal = Modal::None;
        }
        self.counsel_drafts.remove(&attempt.agent_key);
        true
    }

    /// Settles a send that failed: the parchment stays open with the draft
    /// intact, so the message can be retried rather than retyped.
    pub fn fail_counsel(&mut self, request: CounselRequest, message: String) -> bool {
        let Some(attempt) = self.active_counsel.remove(&request) else {
            return false;
        };
        if self.counsel_request_matches(request)
            && let Modal::Counsel { phase, .. } = &mut self.modal
        {
            *phase = CounselPhase::TextFailed { message };
        } else if !attempt.draft.trim().is_empty() {
            self.counsel_drafts.insert(attempt.agent_key, attempt.draft);
        }
        true
    }

    /// Records that the text reached the pane but pressing Enter did not.
    /// The attempt stays active so a later retry can submit only the text
    /// already present on the prompt.
    pub fn fail_counsel_submission(&mut self, request: CounselRequest, message: String) -> bool {
        let pane_id = {
            let Some(attempt) = self.active_counsel.get_mut(&request) else {
                return false;
            };
            attempt.stage = CounselAttemptStage::SubmissionFailed {
                message: message.clone(),
            };
            attempt.pane_id.clone()
        };
        if self.counsel_request_matches(request)
            && let Modal::Counsel { phase, .. } = &mut self.modal
        {
            *phase = CounselPhase::SubmissionFailed {
                request,
                pane_id,
                message,
            };
        }
        true
    }

    /// Records a send-text request whose acknowledgement was not trustworthy.
    /// No resend transition is provided: the pane must be inspected first.
    pub fn mark_counsel_text_uncertain(
        &mut self,
        request: CounselRequest,
        message: String,
    ) -> bool {
        let pane_id = {
            let Some(attempt) = self.active_counsel.get_mut(&request) else {
                return false;
            };
            attempt.stage = CounselAttemptStage::TextUncertain {
                message: message.clone(),
            };
            attempt.pane_id.clone()
        };
        if self.counsel_request_matches(request)
            && let Modal::Counsel { phase, .. } = &mut self.modal
        {
            *phase = CounselPhase::TextUncertain {
                request,
                pane_id,
                message,
            };
        }
        true
    }

    /// Returns the original pane for an uncertain text delivery. Observation
    /// is the only safe immediate action because neither resend nor submit can
    /// prove what is currently on the prompt.
    pub fn uncertain_counsel_pane(&self) -> Option<PaneId> {
        let Modal::Counsel {
            phase: CounselPhase::TextUncertain { pane_id, .. },
            ..
        } = &self.modal
        else {
            return None;
        };
        Some(pane_id.clone())
    }

    /// Explicitly abandons tracking for an uncertain text delivery. The draft
    /// is not restored as unsent because it may already exist on the prompt.
    pub fn abandon_uncertain_counsel(&mut self) -> bool {
        let Modal::Counsel {
            phase: CounselPhase::TextUncertain { request, .. },
            ..
        } = &self.modal
        else {
            return false;
        };
        let request = *request;
        self.active_counsel.remove(&request);
        true
    }

    /// Moves a failed Enter submission back in flight without exposing the
    /// original counsel text to the command layer again.
    pub fn retry_counsel_submission(&mut self) -> Option<(PaneId, CounselRequest)> {
        let Modal::Counsel {
            phase:
                CounselPhase::SubmissionFailed {
                    request, pane_id, ..
                },
            ..
        } = &self.modal
        else {
            return None;
        };
        let request = *request;
        let pane_id = pane_id.clone();
        let attempt = self.active_counsel.get_mut(&request)?;
        attempt.stage = CounselAttemptStage::Sending;
        let Modal::Counsel { phase, .. } = &mut self.modal else {
            unreachable!("the counsel modal was checked above")
        };
        *phase = CounselPhase::Sending {
            request,
            pane_id: pane_id.clone(),
        };
        Some((pane_id, request))
    }

    fn counsel_request_matches(&self, request: CounselRequest) -> bool {
        matches!(
            &self.modal,
            Modal::Counsel {
                phase:
                    CounselPhase::Sending { request: open, .. }
                    | CounselPhase::TextUncertain { request: open, .. }
                    | CounselPhase::SubmissionFailed { request: open, .. },
                ..
            } if *open == request
        )
    }

    /// Sets the open counsel draft aside for the adventurer it was meant for.
    /// Returns true when there was something worth keeping.
    pub fn keep_counsel_draft(&mut self) -> bool {
        let Modal::Counsel { draft, phase } = &self.modal else {
            return false;
        };
        // An in-flight draft is owned by `active_counsel`. Saving it as an
        // ordinary draft would offer already-written counsel as a fresh send.
        if matches!(
            phase,
            CounselPhase::Sending { .. }
                | CounselPhase::TextUncertain { .. }
                | CounselPhase::SubmissionFailed { .. }
        ) {
            return false;
        }
        if draft.trim().is_empty() {
            return false;
        }
        let Some(key) = self.domain.selected_agent.clone() else {
            return false;
        };
        let draft = draft.clone();
        self.counsel_drafts.insert(key, draft);
        true
    }

    pub fn open_search(&mut self) {
        self.modal = Modal::Search {
            query: String::new(),
        };
    }

    pub fn toggle_ledger(&mut self) {
        self.modal = if matches!(self.modal, Modal::LibrarianLedger { .. }) {
            Modal::None
        } else {
            Modal::LibrarianLedger {
                page: LedgerPageId::Welcome,
            }
        };
    }

    pub fn open_ledger(&mut self) {
        self.modal = Modal::LibrarianLedger {
            page: LedgerPageId::Welcome,
        };
    }

    pub const fn ledger_page(&self) -> Option<LedgerPageId> {
        match self.modal {
            Modal::LibrarianLedger { page } => Some(page),
            _ => None,
        }
    }

    pub fn next_ledger_page(&mut self) {
        if let Modal::LibrarianLedger { page } = &mut self.modal {
            *page = page.next();
        }
    }

    pub fn previous_ledger_page(&mut self) {
        if let Modal::LibrarianLedger { page } = &mut self.modal {
            *page = page.previous();
        }
    }

    pub fn first_ledger_page(&mut self) {
        if let Modal::LibrarianLedger { page } = &mut self.modal {
            *page = LedgerPageId::Welcome;
        }
    }

    pub fn last_ledger_page(&mut self) {
        if let Modal::LibrarianLedger { page } = &mut self.modal {
            *page = LedgerPageId::SafeChronicle;
        }
    }

    pub fn counsel_draft(&self) -> Option<&str> {
        match &self.modal {
            Modal::Counsel { draft, .. } => Some(draft),
            Modal::None
            | Modal::LibrarianLedger { .. }
            | Modal::Search { .. }
            | Modal::PartyName { .. }
            | Modal::Scrying
            | Modal::Chronicle
            | Modal::OwnedAuthoring
            | Modal::QuestBoard
            | Modal::QuestInbox { .. }
            | Modal::WarTable
            | Modal::CharacterPicker { .. }
            | Modal::StatusExplanation(_) => None,
        }
    }

    pub fn push_counsel_character(&mut self, character: char) {
        self.push_modal_character(character);
    }

    pub fn push_modal_character(&mut self, character: char) {
        match &mut self.modal {
            Modal::Counsel { draft, phase } => {
                if Self::counsel_accepts_input(phase) {
                    draft.push(character);
                }
            }
            Modal::Search { query } => query.push(character),
            Modal::PartyName { draft, .. } => {
                if !character.is_control()
                    && draft.chars().count() < crate::persistence::PARTY_NAME_LIMIT
                {
                    draft.push(character);
                }
            }
            Modal::None
            | Modal::LibrarianLedger { .. }
            | Modal::Scrying
            | Modal::Chronicle
            | Modal::OwnedAuthoring
            | Modal::QuestBoard
            | Modal::QuestInbox { .. }
            | Modal::WarTable
            | Modal::CharacterPicker { .. }
            | Modal::StatusExplanation(_) => {}
        }
    }

    /// Editing is refused while a send is in flight, because the text on
    /// screen is the text being sent. Editing after a failure clears the
    /// error: the message should describe the draft in front of you, not the
    /// one you have already started rewriting.
    fn counsel_accepts_input(phase: &mut CounselPhase) -> bool {
        match phase {
            CounselPhase::Sending { .. }
            | CounselPhase::TextUncertain { .. }
            | CounselPhase::SubmissionFailed { .. } => false,
            CounselPhase::TextFailed { .. } => {
                *phase = CounselPhase::Drafting;
                true
            }
            CounselPhase::Drafting => true,
        }
    }

    pub fn backspace_counsel(&mut self) {
        self.backspace_modal_input();
    }

    pub fn backspace_modal_input(&mut self) {
        match &mut self.modal {
            Modal::Counsel { draft, phase } => {
                if Self::counsel_accepts_input(phase) {
                    draft.pop();
                }
            }
            Modal::Search { query } | Modal::PartyName { draft: query, .. } => {
                query.pop();
            }
            Modal::None
            | Modal::LibrarianLedger { .. }
            | Modal::Scrying
            | Modal::Chronicle
            | Modal::OwnedAuthoring
            | Modal::QuestBoard
            | Modal::QuestInbox { .. }
            | Modal::WarTable
            | Modal::CharacterPicker { .. }
            | Modal::StatusExplanation(_) => {}
        }
    }

    pub fn clear_modal_input(&mut self) {
        match &mut self.modal {
            Modal::Counsel { draft, phase } => {
                if Self::counsel_accepts_input(phase) {
                    draft.clear();
                }
            }
            Modal::Search { query } | Modal::PartyName { draft: query, .. } => query.clear(),
            Modal::None
            | Modal::LibrarianLedger { .. }
            | Modal::Scrying
            | Modal::Chronicle
            | Modal::OwnedAuthoring
            | Modal::QuestBoard
            | Modal::QuestInbox { .. }
            | Modal::WarTable
            | Modal::CharacterPicker { .. }
            | Modal::StatusExplanation(_) => {}
        }
    }

    pub fn dismiss_modal(&mut self) {
        self.modal = Modal::None;
        self.inbox_is_home = false;
        self.reading_scroll = 0;
    }

    pub(crate) fn open_party_name(&mut self) {
        if let Some(return_page) = self.ledger_page() {
            self.modal = Modal::PartyName {
                draft: self
                    .durable_intent
                    .party_name()
                    .unwrap_or_default()
                    .to_owned(),
                return_page,
            };
        }
    }

    pub(crate) fn cancel_party_name(&mut self) {
        if let Modal::PartyName { return_page, .. } = self.modal {
            self.modal = Modal::LibrarianLedger { page: return_page };
        }
    }

    pub(crate) fn open_status_explanation(&mut self) -> Option<StatusRequest> {
        let Some(agent) = self.selected_agent() else {
            self.set_action_feedback("Select an adventurer first.".into());
            return None;
        };
        if self.managed_pane_id() == Some(&agent.pane_id) || agent.presence == Presence::Exited {
            self.set_action_feedback("Status requires an available adventurer.".into());
            return None;
        }
        let request = StatusRequest {
            serial: self.next_status_request.checked_add(1)?,
            pane: agent.pane_id.clone(),
            identity: agent.capture_identity.clone(),
            presence: agent.presence,
            presence_since: agent.presence_since,
        };
        let outcome = if self.connection != ConnectionState::Connected {
            Some(Err(
                "Herdr is unavailable. Reopen after reconnecting.".into()
            ))
        } else if request.identity == crate::domain::CaptureIdentity::Unqualified {
            Some(Err("This pane has no unambiguous terminal identity.".into()))
        } else {
            None
        };
        let pending = outcome.is_none();
        self.next_status_request = request.serial;
        self.modal = Modal::StatusExplanation(Box::new(StatusExplanation {
            request: request.clone(),
            outcome,
        }));
        pending.then_some(request)
    }

    pub(crate) fn finish_status_explanation(
        &mut self,
        serial: u64,
        result: Result<crate::herdr::protocol::AgentExplanation, String>,
    ) {
        if let Modal::StatusExplanation(status) = &mut self.modal
            && status.request.serial == serial
            && status.outcome.is_none()
        {
            status.outcome = Some(result);
        }
    }

    pub fn open_character_picker(&mut self) {
        let Some(agent) = self.selected_agent() else {
            self.set_action_feedback("Select an adventurer to choose a character.".to_owned());
            return;
        };
        let adventurer = agent.key.clone();
        let persona = agent.persona.key.clone();
        let Some(agent) = self.character_picker_target(&adventurer, &persona) else {
            self.set_action_feedback(
                "Character changes need a uniquely identified adventurer.".to_owned(),
            );
            return;
        };
        let choice = AdventurerClass::ALL
            .iter()
            .position(|class| *class == agent.persona.class)
            .unwrap_or(AdventurerClass::ALL.len());
        self.modal = Modal::CharacterPicker {
            adventurer,
            persona,
            choice,
            return_page: self.ledger_page(),
        };
    }

    pub(crate) fn cancel_character_picker(&mut self) {
        if let Modal::CharacterPicker { return_page, .. } = self.modal {
            self.modal = return_page.map_or(Modal::None, |page| Modal::LibrarianLedger { page });
        }
    }

    pub(crate) fn character_picker_target(
        &self,
        key: &AgentKey,
        persona: &PersonaKey,
    ) -> Option<&Agent> {
        let agent = self.domain.agents.get(key)?;
        (agent.presence != Presence::Exited
            && &agent.persona.key == persona
            && !self.domain.agents.values().any(|other| {
                &other.key != key
                    && &other.persona.key == persona
                    && other.presence != Presence::Exited
            }))
        .then_some(agent)
    }

    pub(crate) fn character_change(
        &mut self,
    ) -> Option<(AgentKey, crate::domain::AdventurerPersona)> {
        let Modal::CharacterPicker {
            adventurer,
            persona,
            choice,
            ..
        } = &self.modal
        else {
            return None;
        };
        let Some(agent) = self.character_picker_target(adventurer, persona) else {
            self.set_action_feedback(
                "Adventurer changed. Select them again to choose a character.".to_owned(),
            );
            return None;
        };
        let class = AdventurerClass::ALL
            .get(*choice)
            .copied()
            .unwrap_or_else(|| agent.persona.automatic_class());
        let mut updated = agent.persona.clone();
        updated.class = class;
        Some((adventurer.clone(), updated))
    }

    pub(crate) fn apply_character_change(
        &mut self,
        adventurer: &AgentKey,
        persona: crate::domain::AdventurerPersona,
    ) {
        self.durable_intent.remember_persona(&persona);
        if let Some(agent) = self.domain.agents.get_mut(adventurer) {
            agent.persona = persona;
        }
        self.dismiss_modal();
    }

    pub(crate) fn move_character_choice(&mut self, update: impl FnOnce(usize) -> usize) {
        if let Modal::CharacterPicker { choice, .. } = &mut self.modal {
            *choice = update(*choice).min(AdventurerClass::ALL.len());
        }
    }

    pub fn open_scrying(&mut self) {
        self.modal = Modal::Scrying;
        self.reading_scroll = 0;
    }

    /// How far the open reading surface has been scrolled, in lines.
    #[must_use]
    pub const fn reading_scroll(&self) -> u16 {
        self.reading_scroll
    }

    /// Scrolls the open parchment by one line.
    ///
    /// Scrying asks Herdr for `output_preview_lines` — eighty by default — and
    /// the parchment could show about fourteen of them. The rest were fetched,
    /// held in memory and unreachable. The Chronicle had the same shape: it
    /// rendered one screenful of a record with no way to reach the rest.
    ///
    /// The clamp lives here rather than in the renderer so that scrolling past
    /// the end cannot run the offset away and leave the user pressing `k`
    /// twenty times to get back.
    pub fn scroll_reading(&mut self, down: bool) {
        let last = u16::try_from(self.reading_line_count().saturating_sub(1)).unwrap_or(u16::MAX);
        self.reading_scroll = if down {
            self.reading_scroll.saturating_add(1).min(last)
        } else {
            self.reading_scroll.saturating_sub(1)
        };
    }

    fn reading_line_count(&self) -> usize {
        match self.modal {
            Modal::Scrying => self
                .output_preview
                .as_ref()
                .map_or(1, |preview| preview.text.lines().count().max(1)),
            Modal::WarTable => crate::ui::war_table::detail_line_count(
                self,
                self.board_viewport_width,
                self.work_viewport_height,
            ),
            Modal::QuestBoard => {
                crate::ui::work_board::detail_line_count(self, self.board_viewport_width)
            }
            Modal::QuestInbox { detail: true } => {
                crate::ui::quest_inbox::detail_lines(self, self.board_viewport_width).len()
            }
            Modal::Chronicle => self.chronicle_chapter_lines().map_or_else(
                || self.chronicle_entries(usize::MAX).len().max(1),
                |lines| lines.len().max(1),
            ),
            _ => 1,
        }
    }

    pub fn note_interaction(&mut self) {
        self.last_interaction_at = Some(self.now);
    }

    /// Whether to show the command ribbon.
    ///
    /// The ribbon fades a few seconds after you touch anything, which is right
    /// — the room is the point, and hints permanently across the bottom of it
    /// are clutter for anyone who knows the keys.
    ///
    /// But it keyed solely off the last interaction, so somebody who opened
    /// Questmancer and sat looking at it saw no hints at all: the one person
    /// who needed them got nothing, and the people who did not need them got
    /// them on every keypress. It now stays up until the first interaction and
    /// fades normally from then on.
    pub fn command_ribbon_visible(&self) -> bool {
        self.last_interaction_at
            .is_none_or(|started| started.elapsed_until(self.now) <= Duration::from_millis(3_000))
    }

    pub const fn output_preview(&self) -> Option<&OutputPreview> {
        self.output_preview.as_ref()
    }

    pub fn set_output_preview(&mut self, preview: Option<OutputPreview>) {
        self.output_context = None;
        self.output_preview = preview;
    }

    fn invalidate_output(&mut self) {
        self.output_context = None;
        self.output_preview = None;
        if self.modal == Modal::Scrying {
            self.reading_scroll = 0;
        }
    }

    fn reconcile_output(&mut self) {
        if let Some(context) = &self.output_context {
            let eligible = self.selected_agent().is_some_and(|agent| {
                agent.key == context.agent_key
                    && agent.capture_identity == context.identity
                    && agent.pane_id == context.pane_id
                    && agent.presence != Presence::Exited
                    && self.managed_pane_id.as_ref() != Some(&agent.pane_id)
            });
            if !eligible {
                self.invalidate_output();
            }
        }
    }

    /// Starts a read only for a live, selected adventurer on this connection.
    /// Superseding a request preserves the last revision as a lower bound.
    pub fn begin_output_request(&mut self, pane_id: &PaneId) -> Option<OutputRequest> {
        self.reconcile_output();
        let agent = self.selected_agent()?;
        if self.connection != ConnectionState::Connected
            || agent.pane_id != *pane_id
            || agent.presence == Presence::Exited
            || self.managed_pane_id.as_ref() == Some(pane_id)
        {
            return None;
        }
        let agent_key = agent.key.clone();
        let identity = agent.capture_identity.clone();
        self.next_output_request = self.next_output_request.checked_add(1)?;
        let request = OutputRequest(self.next_output_request);
        if self.output_context.is_none() {
            self.output_preview = None;
        }
        self.output_context = Some(OutputContext {
            identity,
            agent_key,
            pane_id: pane_id.clone(),
            pending: Some(request),
        });
        let preview = self.output_preview.get_or_insert_with(|| OutputPreview {
            pane_id: pane_id.clone(),
            revision: 0,
            text: String::new(),
            loading: true,
            error: None,
        });
        preview.loading = true;
        preview.error = None;
        Some(request)
    }

    fn finish_output_request(&mut self, request: OutputRequest, pane_id: &PaneId) -> bool {
        self.reconcile_output();
        let Some(context) = &mut self.output_context else {
            return false;
        };
        if self.connection != ConnectionState::Connected
            || context.pane_id != *pane_id
            || context.pending != Some(request)
        {
            return false;
        }
        context.pending = None;
        true
    }

    pub fn complete_output_request(
        &mut self,
        request: OutputRequest,
        pane_id: &PaneId,
        revision: u64,
        text: String,
    ) -> bool {
        if !self.finish_output_request(request, pane_id) {
            return false;
        }
        let preview = self
            .output_preview
            .as_mut()
            .expect("a read owns its preview");
        preview.loading = false;
        if revision < preview.revision {
            return false;
        }
        preview.revision = revision;
        preview.text = text;
        preview.error = None;
        true
    }

    pub fn fail_output_request(&mut self, request: OutputRequest, pane_id: &PaneId, message: &str) {
        if self.finish_output_request(request, pane_id) {
            let preview = self
                .output_preview
                .as_mut()
                .expect("a read owns its preview");
            preview.loading = false;
            preview.text.clear();
            preview.error = Some(format!("load output failed: {message}"));
        }
    }

    pub fn status_message(&self) -> Option<&str> {
        self.notice().map(Notice::message)
    }

    pub fn notice(&self) -> Option<&Notice> {
        self.notices.primary()
    }

    pub fn connection_diagnostic(&self) -> Option<&str> {
        self.notices.connection.as_ref().map(Notice::message)
    }

    pub fn action_feedback(&self) -> Option<&str> {
        self.notices.action.as_ref().map(Notice::message)
    }

    pub fn persistence_diagnostic(&self) -> Option<&str> {
        self.notices.persistence.as_ref().map(Notice::message)
    }

    pub fn reviewr_availability_diagnostic(&self) -> Option<&str> {
        self.notices.reviewr.as_ref().map(Notice::message)
    }

    pub fn integration_diagnostic(&self) -> Option<&str> {
        self.notices.integration.as_ref().map(Notice::message)
    }

    pub fn set_connection_diagnostic(&mut self, message: String) {
        self.notices.connection = Some(Notice::ConnectionDiagnostic(message));
    }

    pub fn set_action_feedback(&mut self, message: String) {
        self.notices.action = Some(Notice::ActionFeedback(message));
        self.action_feedback_at = Some(self.now);
    }

    pub(crate) fn set_counsel_issued(&mut self, message: String) {
        self.notices.action = Some(Notice::CounselIssued(message));
        self.action_feedback_at = Some(self.now);
    }

    pub fn set_persistence_diagnostic(&mut self, message: String) {
        self.notices.persistence = Some(Notice::PersistenceDiagnostic(message));
    }

    pub fn set_integration_diagnostic(&mut self, message: String) {
        self.notices.integration = Some(Notice::IntegrationDiagnostic(message));
    }

    pub fn set_reviewr_availability_diagnostic(&mut self, message: String) {
        self.notices.reviewr = Some(Notice::ReviewrAvailabilityDiagnostic(message));
    }

    pub fn clear_connection_notice(&mut self) {
        self.notices.connection = None;
    }

    pub fn clear_reviewr_availability_notice(&mut self) {
        self.notices.reviewr = None;
    }

    pub fn clear_action_feedback(&mut self) {
        self.notices.action = None;
        self.action_feedback_at = None;
    }

    pub fn selected_agent_key(&self) -> Option<&AgentKey> {
        self.domain.selected_agent.as_ref()
    }

    pub const fn reviewr_available(&self) -> bool {
        self.reviewr_available
    }

    pub const fn set_reviewr_available(&mut self, available: bool) {
        self.reviewr_available = available;
    }

    pub const fn now(&self) -> Timestamp {
        self.now
    }

    pub fn set_now(&mut self, now: Timestamp) {
        self.now = now;
        self.expire_action_feedback();
    }

    /// Action feedback is a confirmation, not a status. It answers "did that
    /// work?" for the thing you just did, and it stops being true almost
    /// immediately afterwards.
    ///
    /// Nothing used to retire it. `clear_action_feedback` had exactly one
    /// caller — the branch of search where a query matches a single
    /// adventurer — so every other message was permanent. Press `o` on an
    /// agent with long output and "output preview was truncated" stayed
    /// pinned to the bottom of the room for the rest of the session, long
    /// after the preview it described had been closed, describing an
    /// adventurer you were no longer looking at.
    ///
    /// It expires on the clock rather than on a list of transitions, because
    /// a list of transitions is what we had: one entry long, and wrong for the
    /// ten messages nobody remembered to add. The search position notice is
    /// safe here — `n` and `N` rewrite it on every press.
    fn expire_action_feedback(&mut self) {
        let stale = self
            .action_feedback_at
            .is_some_and(|shown| shown.elapsed_until(self.now) > ACTION_FEEDBACK_TTL);
        if stale {
            self.clear_action_feedback();
        }
    }

    pub const fn preferences(&self) -> &DisplayPreferences {
        &self.preferences
    }

    pub const fn set_preferences(&mut self, preferences: DisplayPreferences) {
        self.preferences = preferences;
    }

    /// Cycles motion and reports the new setting.
    ///
    /// Motion, glyphs and colour depth were configuration-file only: changing
    /// any of them meant editing a file and restarting, which is a poor answer
    /// for reduced motion in particular. All three now persist through the
    /// same durable state the file seeds, so a runtime change survives a
    /// restart without the file needing to change.
    pub fn cycle_motion(&mut self) -> &'static str {
        self.preferences.motion = self.preferences.motion.next();
        self.preferences.motion.label()
    }

    pub fn cycle_character_set(&mut self) -> &'static str {
        self.preferences.character_set = self.preferences.character_set.next();
        self.preferences.character_set.label()
    }

    pub fn cycle_color_mode(&mut self) -> &'static str {
        self.preferences.color_mode = self.preferences.color_mode.next();
        self.preferences.color_mode.label()
    }

    pub const fn settings(&self) -> &RuntimeSettings {
        &self.settings
    }

    pub fn set_settings(&mut self, settings: RuntimeSettings) {
        self.settings = settings;
    }

    pub const fn durable_intent(&self) -> &DurableIntent {
        &self.durable_intent
    }

    pub fn durable_intent_mut(&mut self) -> &mut DurableIntent {
        &mut self.durable_intent
    }

    pub const fn managed_pane_id(&self) -> Option<&PaneId> {
        self.managed_pane_id.as_ref()
    }

    pub fn set_managed_pane_id(&mut self, pane_id: Option<PaneId>) {
        self.managed_pane_id = pane_id;
        self.reconcile_output();
    }

    pub const fn goblins(&self) -> &GoblinState {
        &self.goblins
    }

    pub const fn goblins_mut(&mut self) -> &mut GoblinState {
        &mut self.goblins
    }
}
