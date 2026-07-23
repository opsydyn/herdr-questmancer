use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize)]
pub struct Request<P> {
    pub id: String,
    pub method: String,
    pub params: P,
}

impl<P> Request<P> {
    pub fn new(id: impl Into<String>, method: impl Into<String>, params: P) -> Self {
        Self {
            id: id.into(),
            method: method.into(),
            params,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct EmptyParams {}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct SuccessResponse<T> {
    pub id: String,
    pub result: T,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct ErrorResponse {
    pub id: String,
    pub error: ErrorBody,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Pong {
    #[serde(rename = "type")]
    pub kind: String,
    pub version: String,
    pub protocol: u32,
    #[serde(default)]
    pub capabilities: Option<ServerCapabilities>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct ServerCapabilities {
    pub live_handoff: bool,
    pub detached_server_daemon: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct SessionSnapshotResult {
    #[serde(rename = "type")]
    pub kind: String,
    pub snapshot: SessionSnapshot,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct SessionSnapshot {
    pub version: String,
    pub protocol: u32,
    #[serde(default)]
    pub focused_workspace_id: Option<String>,
    #[serde(default)]
    pub focused_tab_id: Option<String>,
    #[serde(default)]
    pub focused_pane_id: Option<String>,
    pub workspaces: Vec<WorkspaceInfo>,
    pub tabs: Vec<TabInfo>,
    pub panes: Vec<PaneInfo>,
    pub layouts: Vec<PaneLayoutSnapshot>,
    pub agents: Vec<AgentInfo>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Idle,
    Working,
    Blocked,
    Done,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WorkspaceInfo {
    pub workspace_id: String,
    pub number: usize,
    pub label: String,
    pub focused: bool,
    pub pane_count: usize,
    pub tab_count: usize,
    pub active_tab_id: String,
    pub agent_status: AgentStatus,
    #[serde(default)]
    pub worktree: Option<WorkspaceWorktreeInfo>,
    #[serde(default)]
    pub tokens: HashMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WorkspaceWorktreeInfo {
    pub repo_key: String,
    pub repo_name: String,
    pub repo_root: String,
    pub checkout_path: String,
    pub is_linked_worktree: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TabInfo {
    pub tab_id: String,
    pub workspace_id: String,
    pub number: usize,
    pub label: String,
    pub focused: bool,
    pub pane_count: usize,
    pub agent_status: AgentStatus,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct PaneInfo {
    pub pane_id: String,
    pub terminal_id: String,
    pub workspace_id: String,
    pub tab_id: String,
    pub focused: bool,
    pub agent_status: AgentStatus,
    pub revision: u64,
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(default)]
    pub agent_session: Option<AgentSessionInfo>,
    #[serde(default)]
    pub custom_status: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub display_agent: Option<String>,
    #[serde(default)]
    pub foreground_cwd: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub scroll: Option<PaneScrollInfo>,
    #[serde(default)]
    pub state_labels: HashMap<String, String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub tokens: HashMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct AgentInfo {
    pub terminal_id: String,
    pub agent_status: AgentStatus,
    pub workspace_id: String,
    pub tab_id: String,
    pub pane_id: String,
    pub focused: bool,
    pub revision: u64,
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(default)]
    pub agent_session: Option<AgentSessionInfo>,
    #[serde(default)]
    pub custom_status: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub display_agent: Option<String>,
    #[serde(default)]
    pub foreground_cwd: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub screen_detection_skipped: bool,
    #[serde(default)]
    pub state_labels: HashMap<String, String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub tokens: HashMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct AgentSessionInfo {
    pub source: String,
    pub agent: String,
    pub kind: String,
    pub value: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
pub struct PaneScrollInfo {
    pub offset_from_bottom: u64,
    pub max_offset_from_bottom: u64,
    pub viewport_rows: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct PaneLayoutSnapshot {
    pub workspace_id: String,
    pub tab_id: String,
    pub zoomed: bool,
    pub area: PaneLayoutRect,
    pub focused_pane_id: String,
    pub panes: Vec<PaneLayoutPane>,
    pub splits: Vec<PaneLayoutSplit>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
pub struct PaneLayoutRect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct PaneLayoutPane {
    pub pane_id: String,
    pub focused: bool,
    pub rect: PaneLayoutRect,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct PaneLayoutSplit {
    pub id: String,
    pub direction: String,
    pub ratio: f64,
    pub rect: PaneLayoutRect,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct WireEvent {
    pub event: String,
    pub data: Value,
}

#[derive(Clone, Debug, Serialize)]
pub struct PaneTarget {
    pub pane_id: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct PaneSendTextParams {
    pub pane_id: String,
    pub text: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadSource {
    Visible,
    Recent,
    RecentUnwrapped,
    Detection,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadFormat {
    Text,
    Ansi,
}

#[derive(Clone, Debug, Serialize)]
pub struct PaneReadParams {
    pub pane_id: String,
    pub source: ReadSource,
    pub lines: Option<u32>,
    pub format: ReadFormat,
    pub strip_ansi: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct PaneReportMetadataParams {
    pub pane_id: String,
    pub source: String,
    pub tokens: BTreeMap<String, Option<String>>,
    pub seq: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct WorkspaceReportMetadataParams {
    pub workspace_id: String,
    pub source: String,
    pub tokens: BTreeMap<String, Option<String>>,
    pub seq: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct OkResult {
    #[serde(rename = "type")]
    pub kind: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct PaneInfoResult {
    #[serde(rename = "type")]
    pub kind: String,
    pub pane: PaneInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct PaneReadResultEnvelope {
    #[serde(rename = "type")]
    pub kind: String,
    pub read: PaneReadResult,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct PaneReadResult {
    pub pane_id: String,
    pub workspace_id: String,
    pub tab_id: String,
    pub source: ReadSource,
    pub format: ReadFormat,
    pub text: String,
    pub revision: u64,
    pub truncated: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PluginActionContext {
    Global,
    Workspace,
    Tab,
    Pane,
    Selection,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct PluginActionInfo {
    pub plugin_id: String,
    pub action_id: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub contexts: Vec<PluginActionContext>,
    #[serde(default)]
    pub command: Vec<String>,
}

impl PluginActionInfo {
    #[must_use]
    pub fn qualified_id(&self) -> String {
        format!("{}.{}", self.plugin_id, self.action_id)
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct PluginActionListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct PluginActionListResult {
    #[serde(rename = "type")]
    pub kind: String,
    pub actions: Vec<PluginActionInfo>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct PluginInvocationContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tab_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focused_pane_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invocation_source: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PluginActionInvokeParams {
    pub action_id: String,
    pub plugin_id: Option<String>,
    pub context: Option<PluginInvocationContext>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct PluginActionInvokedResult {
    #[serde(rename = "type")]
    pub kind: String,
    pub action: PluginActionInfo,
    pub context: Value,
    pub log: Value,
}
