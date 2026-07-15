use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::{
    domain::{Agent, AgentKey, DomainState, PaneId, Timestamp},
    persistence::DurableIntent,
    update::{AppEvent, update},
};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum View {
    #[default]
    Desk,
    Cafe,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Motion {
    #[default]
    Full,
    Reduced,
    None,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CharacterSet {
    #[default]
    Unicode,
    Ascii,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorMode {
    #[default]
    Xterm256,
    Ansi16,
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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Region {
    #[default]
    Sites,
    Inbox,
    Guestbook,
    Agent,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum Modal {
    #[default]
    None,
    Help,
    Reply {
        draft: String,
    },
    Search {
        query: String,
    },
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
    region: Region,
    modal: Modal,
    output_preview: Option<OutputPreview>,
    status_message: Option<String>,
    reviewr_available: bool,
    now: Timestamp,
    preferences: DisplayPreferences,
    settings: RuntimeSettings,
    durable_intent: DurableIntent,
    managed_pane_id: Option<PaneId>,
}

impl Model {
    pub fn new(view: View) -> Self {
        Self {
            view,
            domain: DomainState::default(),
            connection: ConnectionState::Offline,
            region: Region::Sites,
            modal: Modal::None,
            output_preview: None,
            status_message: None,
            reviewr_available: false,
            now: Timestamp::from_millis(0),
            preferences: DisplayPreferences::default(),
            settings: RuntimeSettings::default(),
            durable_intent: DurableIntent::default(),
            managed_pane_id: None,
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

    pub fn mark_selected_attention_seen(&mut self) {
        let Some(agent_key) = self.selected_agent_key().cloned() else {
            return;
        };
        let domain = self.take_domain();
        let (domain, _commands) = update(domain, AppEvent::MarkSeen(agent_key));
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

    pub const fn region(&self) -> Region {
        self.region
    }

    pub const fn set_region(&mut self, region: Region) {
        self.region = region;
    }

    pub fn cycle_region(&mut self) {
        self.region = match self.region {
            Region::Sites => Region::Inbox,
            Region::Inbox => Region::Guestbook,
            Region::Guestbook => Region::Agent,
            Region::Agent => Region::Sites,
        };
    }

    pub const fn modal(&self) -> &Modal {
        &self.modal
    }

    pub fn open_reply(&mut self) {
        self.modal = Modal::Reply {
            draft: String::new(),
        };
    }

    pub fn open_search(&mut self) {
        self.modal = Modal::Search {
            query: String::new(),
        };
    }

    pub fn reply_draft(&self) -> Option<&str> {
        match &self.modal {
            Modal::Reply { draft } => Some(draft),
            Modal::None | Modal::Help | Modal::Search { .. } => None,
        }
    }

    pub fn push_reply_character(&mut self, character: char) {
        self.push_modal_character(character);
    }

    pub fn push_modal_character(&mut self, character: char) {
        match &mut self.modal {
            Modal::Reply { draft } => draft.push(character),
            Modal::Search { query } => query.push(character),
            Modal::None | Modal::Help => {}
        }
    }

    pub fn backspace_reply(&mut self) {
        self.backspace_modal_input();
    }

    pub fn backspace_modal_input(&mut self) {
        match &mut self.modal {
            Modal::Reply { draft } => {
                draft.pop();
            }
            Modal::Search { query } => {
                query.pop();
            }
            Modal::None | Modal::Help => {}
        }
    }

    pub fn clear_modal_input(&mut self) {
        match &mut self.modal {
            Modal::Reply { draft } => draft.clear(),
            Modal::Search { query } => query.clear(),
            Modal::None | Modal::Help => {}
        }
    }

    pub fn dismiss_modal(&mut self) {
        self.modal = Modal::None;
    }

    pub fn take_reply(&mut self) -> Option<String> {
        match std::mem::take(&mut self.modal) {
            Modal::Reply { draft } => Some(draft),
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
        self.status_message.as_deref()
    }

    pub fn set_status_message(&mut self, message: Option<String>) {
        self.status_message = message;
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
}
