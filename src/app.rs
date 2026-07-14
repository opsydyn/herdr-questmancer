use clap::ValueEnum;

use crate::domain::{Agent, AgentKey, DomainState, PaneId, Timestamp};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum View {
    #[default]
    Desk,
    Cafe,
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

    pub fn replace_domain(&mut self, mut domain: DomainState) {
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
        self.domain = domain;
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

    pub const fn modal(&self) -> &Modal {
        &self.modal
    }

    pub fn open_reply(&mut self) {
        self.modal = Modal::Reply {
            draft: String::new(),
        };
    }

    pub fn push_reply_character(&mut self, character: char) {
        if let Modal::Reply { draft } = &mut self.modal {
            draft.push(character);
        }
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
}
