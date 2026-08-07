use std::time::Duration;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::{
    domain::{
        Agent, AgentKey, DomainState, GuildAttention, GuildSummons, PaneId, Presence, Timestamp,
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
    pub output_preview_lines: u32,
    pub reviewr_action: String,
    pub show_elapsed_time: bool,
}

impl Default for RuntimeSettings {
    fn default() -> Self {
        Self {
            output_preview_lines: 80,
            reviewr_action: "persiyanov.reviewr.open".to_owned(),
            show_elapsed_time: true,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Notice {
    ConnectionDiagnostic(String),
    ActionFeedback(String),
    PersistenceDiagnostic(String),
    ReviewrAvailabilityDiagnostic(String),
    IntegrationDiagnostic(String),
}

impl Notice {
    pub fn message(&self) -> &str {
        match self {
            Self::ConnectionDiagnostic(message)
            | Self::ActionFeedback(message)
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
    },
    Search {
        query: String,
    },
    Scrying,
    Chronicle,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputPreview {
    pub pane_id: PaneId,
    pub revision: u64,
    pub text: String,
    pub loading: bool,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Model {
    view: View,
    domain: DomainState,
    connection: ConnectionState,
    modal: Modal,
    adventurer_card_visible: bool,
    output_preview: Option<OutputPreview>,
    notices: Box<Notices>,
    reviewr_available: bool,
    now: Timestamp,
    preferences: DisplayPreferences,
    settings: RuntimeSettings,
    durable_intent: DurableIntent,
    managed_pane_id: Option<PaneId>,
    goblins: GoblinState,
    last_interaction_at: Option<Timestamp>,
    search: SearchResults,
    reading_scroll: u16,
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
            view,
            domain: DomainState::default(),
            connection: ConnectionState::Offline,
            modal: Modal::None,
            adventurer_card_visible: false,
            output_preview: None,
            notices: Box::default(),
            reviewr_available: false,
            now: Timestamp::from_millis(0),
            preferences: DisplayPreferences::default(),
            settings: RuntimeSettings::default(),
            durable_intent: DurableIntent::default(),
            managed_pane_id: None,
            goblins: GoblinState::default(),
            last_interaction_at: None,
            search: SearchResults::default(),
            reading_scroll: 0,
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
        self.connection = connection;
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
        self.domain.selected_agent = self.domain.agents.keys().next().cloned();
    }

    pub fn select_last_agent(&mut self) {
        self.domain.selected_agent = self.domain.agents.keys().next_back().cloned();
    }

    pub fn select_agent(&mut self, agent: &AgentKey) {
        if self.domain.agents.contains_key(agent) {
            self.domain.selected_agent = Some(agent.clone());
        }
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
                let rank = urgency_rank(agent, self.now)?;
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
        self.domain.selected_agent = Some(next.clone());
        true
    }

    fn move_agent_selection(&mut self, direction: i8) {
        let keys = self.domain.agents.keys().cloned().collect::<Vec<_>>();
        if keys.is_empty() {
            self.domain.selected_agent = None;
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
        self.domain.selected_agent = Some(keys[next].clone());
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
        self.domain.selected_agent = Some(matched[next].clone());
        Some((next + 1, matched.len()))
    }

    pub fn open_chronicle(&mut self) {
        self.modal = Modal::Chronicle;
        self.reading_scroll = 0;
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
                selected.is_none_or(|key| entry.adventurer.as_ref().is_some_and(|had| had == key))
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
        self.domain.selected_agent = Some(next);
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

    pub fn open_counsel(&mut self) {
        self.modal = Modal::Counsel {
            draft: String::new(),
        };
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
            Modal::Counsel { draft } => Some(draft),
            Modal::None
            | Modal::LibrarianLedger { .. }
            | Modal::Search { .. }
            | Modal::Scrying
            | Modal::Chronicle => None,
        }
    }

    pub fn push_counsel_character(&mut self, character: char) {
        self.push_modal_character(character);
    }

    pub fn push_modal_character(&mut self, character: char) {
        match &mut self.modal {
            Modal::Counsel { draft } => draft.push(character),
            Modal::Search { query } => query.push(character),
            Modal::None | Modal::LibrarianLedger { .. } | Modal::Scrying | Modal::Chronicle => {}
        }
    }

    pub fn backspace_counsel(&mut self) {
        self.backspace_modal_input();
    }

    pub fn backspace_modal_input(&mut self) {
        match &mut self.modal {
            Modal::Counsel { draft } => {
                draft.pop();
            }
            Modal::Search { query } => {
                query.pop();
            }
            Modal::None | Modal::LibrarianLedger { .. } | Modal::Scrying | Modal::Chronicle => {}
        }
    }

    pub fn clear_modal_input(&mut self) {
        match &mut self.modal {
            Modal::Counsel { draft } => draft.clear(),
            Modal::Search { query } => query.clear(),
            Modal::None | Modal::LibrarianLedger { .. } | Modal::Scrying | Modal::Chronicle => {}
        }
    }

    pub fn dismiss_modal(&mut self) {
        self.modal = Modal::None;
        self.reading_scroll = 0;
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
            Modal::Chronicle => self.chronicle_entries(usize::MAX).len().max(1),
            _ => 1,
        }
    }

    pub fn note_interaction(&mut self) {
        self.last_interaction_at = Some(self.now);
    }

    pub fn command_ribbon_visible(&self) -> bool {
        self.last_interaction_at
            .is_some_and(|started| started.elapsed_until(self.now) <= Duration::from_millis(3_000))
    }

    pub fn take_counsel(&mut self) -> Option<String> {
        match std::mem::take(&mut self.modal) {
            Modal::Counsel { draft } => Some(draft),
            modal => {
                self.modal = modal;
                None
            }
        }
    }

    pub const fn output_preview(&self) -> Option<&OutputPreview> {
        self.output_preview.as_ref()
    }

    pub fn set_output_preview(&mut self, preview: Option<OutputPreview>) {
        self.output_preview = preview;
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

    pub const fn set_now(&mut self, now: Timestamp) {
        self.now = now;
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
    }

    pub const fn goblins(&self) -> &GoblinState {
        &self.goblins
    }

    pub const fn goblins_mut(&mut self) -> &mut GoblinState {
        &mut self.goblins
    }
}

/// How loudly an adventurer is asking for a human, lowest first.
///
/// `None` means nobody is waiting on you for this one. The three ranks are the
/// only distinctions the Hall actually draws: an unanswered call for counsel,
/// a call somebody has seen but not resolved, and the quieter summons that
/// still deserve a look.
fn urgency_rank(agent: &Agent, now: Timestamp) -> Option<u8> {
    if let GuildAttention::Deferred { until, .. } = agent.attention
        && until.as_millis() > now.as_millis()
    {
        // Deferring said "not now". Honour it until it expires.
        return None;
    }
    match (&agent.attention, agent.presence) {
        (
            GuildAttention::Unread {
                summons: GuildSummons::CounselRequested,
                ..
            },
            _,
        ) => Some(0),
        (_, Presence::Blocked) => Some(1),
        (GuildAttention::Unread { .. } | GuildAttention::Deferred { .. }, _) => Some(2),
        _ => None,
    }
}
