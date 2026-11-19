//! Ephemeral forms and typed intent. No filesystem, clock, identity allocation or agent effects.
use super::{Answer, QuestLifecycleChange, Review, StoreSelection};
use crate::{
    app::{Modal, Model},
    work::{
        ArtifactKind, CampaignId, Inventory, ObligationId, ObligationKind, ObligationState,
        QuestId, QuestKind, SourceId,
    },
};
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Metadata {
    pub source: SourceId,
    pub revision: u64,
    pub inventory: Inventory,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedDetail {
    pub id: ObligationId,
    pub incarnation: String,
    pub revision: u64,
    pub prompt: String,
    pub context: Option<String>,
    pub requested_by: String,
    pub answer: Option<Answer>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewDetail {
    pub id: ObligationId,
    pub revision: u64,
    pub review: Review,
    pub changes: Result<Vec<String>, String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FormTarget {
    Quest {
        id: QuestId,
        title: String,
    },
    Review {
        id: ObligationId,
        incarnation: String,
        title: String,
        head: String,
    },
    QuestLifecycle {
        id: QuestId,
        title: String,
        change: QuestLifecycleChange,
        revision: u64,
    },
    QuestPrerequisites {
        id: QuestId,
        title: String,
        current: Vec<QuestId>,
        revision: u64,
    },
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Phase {
    #[default]
    Unavailable,
    Ready,
    Reading,
    Saving,
    RememberPending,
    ViewPending,
    Indeterminate,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FormKind {
    CreateStore,
    OpenStore,
    Campaign,
    Quest,
    QuestLifecycle,
    QuestPrerequisites,
    Request,
    Answer,
    Artifact,
    RequestReview,
    RecordCheck,
    SettleReview,
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum Surface {
    #[default]
    Menu,
    Campaigns,
    Decisions,
    Detail,
    Store,
    Form(FormKind),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Field {
    pub label: String,
    pub text: String,
    pub multiline: bool,
    pub choices: Vec<(String, String)>,
    pub choice: usize,
}
impl Field {
    fn text(label: &str, multiline: bool) -> Self {
        Self {
            label: label.into(),
            text: String::new(),
            multiline,
            choices: Vec::new(),
            choice: 0,
        }
    }
    fn choices(label: &str, choices: Vec<(String, String)>) -> Self {
        Self {
            label: label.into(),
            text: String::new(),
            multiline: false,
            choices,
            choice: 0,
        }
    }
    pub fn value(&self) -> &str {
        if self.choices.is_empty() {
            &self.text
        } else {
            &self.choices[self.choice].0
        }
    }
    pub fn display(&self) -> &str {
        if self.choices.is_empty() {
            &self.text
        } else {
            &self.choices[self.choice].1
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Intent {
    Setup {
        selection: StoreSelection,
        create: bool,
    },
    Submit(Draft),
    Read {
        id: ObligationId,
        incarnation: String,
        revision: u64,
    },
    ReadReview {
        id: ObligationId,
        incarnation: String,
        revision: u64,
    },
    Refresh,
    Reconcile,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Draft {
    pub kind: FormKind,
    pub values: Vec<String>,
    pub campaign: Option<CampaignId>,
    pub decision: Option<(ObligationId, String)>,
    pub revision: u64,
    pub(crate) form_target: Option<FormTarget>,
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Authoring {
    pub surface: Surface,
    pub phase: Phase,
    pub feedback: String,
    pub metadata: Option<Metadata>,
    pub selection: Option<StoreSelection>,
    pub campaign: Option<CampaignId>,
    pub detail: Option<SelectedDetail>,
    pub review: Option<ReviewDetail>,
    pub fields: Vec<Field>,
    pub focus: usize,
    pub cursor: usize,
    pub scroll: usize,
    pub follow_focus: bool,
    pub default_path: String,
    pub blocked: Option<String>,
    pub available: bool,
    pub(crate) pending: Option<Intent>,
    pub(crate) return_after_read: bool,
    pub(crate) draft_kind: Option<FormKind>,
    pub(crate) form_target: Option<FormTarget>,
    pub(crate) return_to_inbox: Option<bool>,
}
impl Phase {
    pub fn label(self) -> &'static str {
        match self {
            Self::Unavailable => "Owner unavailable",
            Self::Ready => "Ready",
            Self::Reading => "Checking selected work",
            Self::Saving => "Saving",
            Self::RememberPending => "Store open; selection pending",
            Self::ViewPending => "Saved; view pending",
            Self::Indeterminate => "May have saved; check outcome",
        }
    }
}
impl Authoring {
    pub fn ready(&self) -> bool {
        self.phase == Phase::Ready && self.metadata.is_some()
    }
    pub fn busy(&self) -> bool {
        matches!(
            self.phase,
            Phase::Reading
                | Phase::Saving
                | Phase::RememberPending
                | Phase::ViewPending
                | Phase::Indeterminate
        )
    }
    pub fn request_setup(&mut self, selection: StoreSelection, create: bool) {
        if self.busy() {
            return;
        }
        self.pending = Some(Intent::Setup { selection, create });
        self.phase = Phase::Saving;
    }
    pub fn menu(&mut self) {
        self.surface = Surface::Menu;
        self.cursor = 0;
        self.scroll = 0;
    }
    pub fn campaign_label(&self) -> String {
        self.campaign
            .as_ref()
            .and_then(|c| self.metadata.as_ref()?.inventory.campaigns.get(c))
            .map_or("Choose an owned campaign".into(), |c| {
                format!(
                    "{} [{}]",
                    c.name,
                    c.id.as_str().rsplit('/').next().unwrap_or(c.id.as_str())
                )
            })
    }
    pub fn form(&mut self, kind: FormKind) {
        self.form_with_target(kind, None);
    }
    pub(crate) fn form_with_target(&mut self, kind: FormKind, target: Option<FormTarget>) {
        if self.busy() {
            self.feedback =
                "An owned operation is outstanding. Return to its result or check outcome.".into();
            return;
        }
        if !self.fields.is_empty() {
            self.feedback = "A draft is retained. Resume it or choose Discard draft first.".into();
            return;
        }
        if matches!(kind, FormKind::CreateStore | FormKind::OpenStore) {
            if !self.available || self.blocked.is_some() {
                self.feedback = self
                    .blocked
                    .clone()
                    .unwrap_or("Durable state directory unavailable.".into());
                return;
            }
        } else if !self.ready() {
            self.feedback = "Open or create your local store first.".into();
            return;
        }
        if matches!(
            kind,
            FormKind::Quest | FormKind::Request | FormKind::Artifact
        ) && self.campaign.is_none()
        {
            self.surface = Surface::Campaigns;
            self.cursor = 0;
            self.feedback = "Choose an owned campaign, then reopen the action.".into();
            return;
        }
        let Some(fields) = self.form_fields(kind, target.as_ref()) else {
            return;
        };
        self.fields = fields;
        self.form_target = target;
        self.surface = Surface::Form(kind);
        self.draft_kind = Some(kind);
        self.focus = 0;
        self.scroll = 0;
        self.follow_focus = false;
        self.feedback.clear();
    }
    fn form_fields(&mut self, kind: FormKind, target: Option<&FormTarget>) -> Option<Vec<Field>> {
        let text = Field::text;
        Some(match kind {
            FormKind::CreateStore => {
                let mut f = text("Store path", false);
                f.text.clone_from(&self.default_path);
                vec![f]
            }
            FormKind::OpenStore => {
                vec![text("Store path", false), text("Expected source ID", false)]
            }
            FormKind::Campaign => vec![text("Name", false), text("Objective (optional)", true)],
            FormKind::Quest => vec![
                text("Title", false),
                Field::choices(
                    "Kind",
                    vec![
                        (String::new(), "Choose kind".into()),
                        ("Scout".into(), "Scout / Expedition".into()),
                        ("Ship".into(), "Ship / Commission".into()),
                        ("Review".into(), "Review / Inspection".into()),
                        ("Validate".into(), "Validate / Trial".into()),
                    ],
                ),
                text("Description (optional)", true),
            ],
            FormKind::Request => vec![
                text("Question", true),
                text("Context (optional)", true),
                self.quest_field(),
            ],
            FormKind::Artifact => vec![
                text("Title", false),
                Field::choices(
                    "Kind",
                    std::iter::once((String::new(), "Choose kind".into()))
                        .chain(
                            artifact_kinds()
                                .iter()
                                .map(|k| (format!("{k:?}"), format!("{k:?}"))),
                        )
                        .collect(),
                ),
                text("URI (optional)", false),
                self.quest_field(),
            ],
            FormKind::RequestReview => vec![
                text("Acceptance criterion", true),
                text("Physical Git checkout path", false),
                text("Base commit (full ID)", false),
                text("Candidate commit (full ID)", false),
                text("Required check recipe", false),
                text("Trusted local runner label", false),
                Field::choices(
                    "Report attribution",
                    vec![
                        (String::new(), "No report".into()),
                        ("Pi".into(), "Pi".into()),
                        ("Codex".into(), "Codex".into()),
                    ],
                ),
                text("Adventurer report (optional)", true),
            ],
            FormKind::RecordCheck => vec![text("Trusted local check receipt (JSON)", true)],
            FormKind::SettleReview => vec![Field::choices(
                "Decision for this revision",
                vec![
                    (String::new(), "Keep review open".into()),
                    ("settle".into(), "Settle this exact revision".into()),
                ],
            )],
            FormKind::QuestLifecycle => vec![Field::choices(
                "Confirmation",
                vec![
                    (String::new(), "Keep current state".into()),
                    ("confirm".into(), "Confirm this change".into()),
                ],
            )],
            FormKind::QuestPrerequisites => vec![self.prerequisite_field(target)?],
            FormKind::Answer => {
                let Some(detail) = &self.detail else {
                    self.feedback = "Load the selected decision before answering.".into();
                    return None;
                };
                if detail.answer.is_some() {
                    self.feedback = "This decision is already answered.".into();
                    return None;
                }
                if self
                    .metadata
                    .as_ref()
                    .is_none_or(|m| m.revision != detail.revision)
                {
                    self.feedback =
                        "Decision detail is stale. Load it again before answering.".into();
                    return None;
                }
                vec![text("Answer", true)]
            }
        })
    }
    fn quest_field(&self) -> Field {
        let choices = std::iter::once((String::new(), "Campaign only".into()))
            .chain(
                self.metadata
                    .iter()
                    .flat_map(|m| m.inventory.quests.values())
                    .filter(|q| Some(&q.campaign) == self.campaign.as_ref())
                    .map(|q| {
                        (
                            q.id.as_str().to_owned(),
                            format!("{} [{}]", q.title, q.id.as_str()),
                        )
                    }),
            )
            .collect();
        Field::choices("Quest (optional)", choices)
    }
    fn prerequisite_field(&mut self, target: Option<&FormTarget>) -> Option<Field> {
        let Some(FormTarget::QuestPrerequisites { id, current, .. }) = target else {
            return None;
        };
        let quests = &self.metadata.as_ref()?.inventory.quests;
        let selected = quests.get(id)?;
        let mut choices = vec![(String::new(), "Choose a prerequisite change".into())];
        choices.extend(
            quests
                .values()
                .filter(|quest| &quest.id != id && quest.campaign == selected.campaign)
                .map(|quest| {
                    let verb = if current.contains(&quest.id) {
                        "Remove"
                    } else {
                        "Add"
                    };
                    (
                        quest.id.as_str().to_owned(),
                        format!("{verb} {} [{}]", quest.title, quest.id.as_str()),
                    )
                }),
        );
        if choices.len() == 1 {
            self.feedback = "Create another owned quest in this campaign first.".into();
            return None;
        }
        Some(Field::choices("Prerequisite change", choices))
    }
    pub fn edit(&mut self, text: &str) {
        if self.busy() || !matches!(self.surface, Surface::Form(_)) {
            return;
        }
        let Some(field) = self.fields.get_mut(self.focus) else {
            return;
        };
        if !field.choices.is_empty() {
            return;
        }
        if field.text.len() + text.len() > 8192
            || text
                .chars()
                .any(|c| c.is_control() && !(field.multiline && c == '\n'))
        {
            self.feedback =
                "Input rejected: text exceeds 8,192 bytes or contains unsupported controls.".into();
            return;
        }
        field.text.push_str(text);
        self.follow_focus = true;
    }
    pub fn submit(&mut self) {
        if self.busy() {
            return;
        }
        let Surface::Form(kind) = self.surface else {
            return;
        };
        let values = self
            .fields
            .iter()
            .map(|f| f.value().to_owned())
            .collect::<Vec<_>>();
        if kind == FormKind::SettleReview && values.first().is_none_or(|s| s != "settle") {
            self.feedback = "Choose Settle this exact revision to confirm.".into();
            return;
        }
        if kind == FormKind::QuestLifecycle && values.first().is_none_or(|s| s != "confirm") {
            self.feedback = "Choose Confirm this change to save the new quest state.".into();
            return;
        }
        if kind == FormKind::QuestPrerequisites && values.first().is_none_or(String::is_empty) {
            self.feedback = "Choose a prerequisite to add or remove before saving.".into();
            return;
        }
        if kind == FormKind::RequestReview
            && values
                .get(..6)
                .is_none_or(|fields| fields.iter().any(|s| s.trim().is_empty()))
        {
            self.feedback =
                "Complete the criterion, checkout, commit IDs, recipe and runner.".into();
            return;
        }
        if values.first().is_none_or(|s| s.trim().is_empty())
            || matches!(kind, FormKind::Quest | FormKind::Artifact) && values[1].is_empty()
        {
            self.feedback = "Complete the required fields and choose an explicit kind.".into();
            return;
        }
        if kind == FormKind::OpenStore {
            match StoreSelection::new(values[0].clone().into(), SourceId::new(&values[1])) {
                Ok(selection) => self.request_setup(selection, false),
                Err(_) => {
                    self.feedback =
                        "Use an absolute physical path and the expected owned source ID.".into();
                }
            }
            return;
        }
        let revision = match self.form_target {
            Some(
                FormTarget::QuestLifecycle { revision, .. }
                | FormTarget::QuestPrerequisites { revision, .. },
            ) => revision,
            _ => self.metadata.as_ref().map_or(0, |m| m.revision),
        };
        // Random source for CreateStore is supplied only at the runtime effect boundary.
        self.pending = Some(Intent::Submit(Draft {
            kind,
            values,
            campaign: self.campaign.clone(),
            decision: self
                .detail
                .as_ref()
                .map(|d| (d.id.clone(), d.incarnation.clone())),
            revision,
            form_target: self.form_target.clone(),
        }));
        self.phase = Phase::Saving;
        self.feedback = "Saving; submitted text is frozen.".into();
    }
    pub fn discard(&mut self) {
        if self.busy() {
            return;
        }
        self.fields.clear();
        self.draft_kind = None;
        self.form_target = None;
        self.menu();
        self.feedback = "Draft discarded.".into();
    }
    pub fn read(&mut self, id: ObligationId, incarnation: String) {
        if self.busy() {
            return;
        }
        if !self.fields.is_empty() {
            self.feedback = "Resume or discard the retained draft before changing decision.".into();
            return;
        }
        let Some(m) = &self.metadata else {
            return;
        };
        self.pending = Some(Intent::Read {
            id,
            incarnation,
            revision: m.revision,
        });
        self.phase = Phase::Reading;
        self.detail = None;
        self.surface = Surface::Detail;
        self.scroll = 0;
    }

    pub(crate) fn read_review(&mut self, id: ObligationId, incarnation: String, revision: u64) {
        self.pending = Some(Intent::ReadReview {
            id,
            incarnation,
            revision,
        });
        self.phase = Phase::Reading;
        self.review = None;
    }
}
pub fn artifact_kinds() -> &'static [ArtifactKind] {
    &[
        ArtifactKind::PullRequest,
        ArtifactKind::Commit,
        ArtifactKind::Diff,
        ArtifactKind::Report,
        ArtifactKind::TestResult,
        ArtifactKind::Benchmark,
        ArtifactKind::Document,
        ArtifactKind::Screenshot,
        ArtifactKind::Other,
    ]
}
pub fn quest_kind(value: &str) -> Option<QuestKind> {
    match value {
        "Scout" => Some(QuestKind::Scout),
        "Ship" => Some(QuestKind::Ship),
        "Review" => Some(QuestKind::Review),
        "Validate" => Some(QuestKind::Validate),
        _ => None,
    }
}
pub fn menu_entries() -> &'static [&'static str] {
    &[
        "Create campaign",
        "Create quest",
        "Request decision",
        "Answer selected decision",
        "Register artifact",
        "Change owned campaign",
        "Decisions (open and answered)",
        "Create local store",
        "Open existing store",
        "Store details / result",
        "Retry view / remember selection / check outcome",
        "Discard draft",
        "Resume draft",
    ]
}

/// Production terminal input; paste stays literal and never becomes global shortcuts.
#[allow(clippy::too_many_lines)] // Modal routing; editing and menu transitions are separate helpers.
pub fn handle_event(model: &mut Model, event: &Event) -> bool {
    if model.modal() != &Modal::OwnedAuthoring {
        return false;
    }
    if matches!(event, Event::Resize(..)) {
        return false;
    }
    if let Event::Key(key) = event {
        if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
            return true;
        }
        if key.kind == KeyEventKind::Repeat && matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
            return true;
        }
        if key.code == KeyCode::Esc {
            if model.authoring().surface == Surface::Menu || model.authoring().busy() {
                let back_to_inbox = model.authoring_mut().return_to_inbox.take();
                model.authoring_mut().menu();
                if let Some(detail) = back_to_inbox {
                    model.open_inbox(detail);
                } else {
                    model.open_quest_board();
                }
            } else {
                model.authoring_mut().menu();
            }
            return true;
        }
    }
    let viewport_ok = model.owned_form_fits();
    let selected = model.work_selection().cloned();
    let a = model.authoring_mut();
    if let Event::Paste(text) = event {
        a.edit(text);
        return true;
    }
    if let Event::Mouse(mouse) = event {
        match mouse.kind {
            MouseEventKind::ScrollDown => {
                a.scroll = a.scroll.saturating_add(1);
                a.follow_focus = false;
            }
            MouseEventKind::ScrollUp => {
                a.scroll = a.scroll.saturating_sub(1);
                a.follow_focus = false;
            }
            _ => {}
        }
        return true;
    }
    let Event::Key(key) = event else {
        return true;
    };
    if let Surface::Form(kind) = a.surface {
        return handle_form(a, key, viewport_ok, kind);
    }
    if key.code == KeyCode::Down {
        let count = match a.surface {
            Surface::Menu => menu_entries().len(),
            Surface::Campaigns => a
                .metadata
                .as_ref()
                .map_or(0, |m| m.inventory.campaigns.len()),
            Surface::Decisions => a.metadata.as_ref().map_or(0, |m| {
                m.inventory
                    .obligations
                    .values()
                    .filter(|o| {
                        o.kind == ObligationKind::Decision
                            && a.campaign.as_ref().is_none_or(|c| c == &o.campaign)
                    })
                    .count()
            }),
            _ => usize::MAX,
        };
        a.cursor = a.cursor.saturating_add(1).min(count.saturating_sub(1));
        a.scroll = a.scroll.saturating_add(1);
        return true;
    }
    if key.code == KeyCode::Up {
        a.cursor = a.cursor.saturating_sub(1);
        a.scroll = a.scroll.saturating_sub(1);
        return true;
    }
    if key.code != KeyCode::Enter {
        return true;
    }
    if a.surface == Surface::Menu {
        a.cursor = a.cursor.min(menu_entries().len() - 1);
        handle_menu(a, selected);
    } else if a.surface == Surface::Campaigns {
        if a.ready() {
            a.campaign = a
                .metadata
                .as_ref()
                .and_then(|m| m.inventory.campaigns.keys().nth(a.cursor))
                .cloned();
            a.menu();
        }
    } else if a.surface == Surface::Decisions {
        let o = a
            .metadata
            .as_ref()
            .and_then(|m| {
                m.inventory
                    .obligations
                    .values()
                    .filter(|o| {
                        o.kind == ObligationKind::Decision
                            && a.campaign.as_ref().is_none_or(|c| c == &o.campaign)
                    })
                    .nth(a.cursor)
            })
            .cloned();
        if let Some(o) = o {
            a.return_after_read = false;
            a.read(o.id, o.incarnation);
        }
    } else if a.surface == Surface::Detail {
        a.form(FormKind::Answer);
    }
    true
}

fn handle_form(
    a: &mut Authoring,
    key: &crossterm::event::KeyEvent,
    viewport_ok: bool,
    kind: FormKind,
) -> bool {
    if a.busy() {
        return true;
    }
    match key.code {
        KeyCode::Tab => {
            a.focus = (a.focus + 1) % (a.fields.len() + 2);
            a.follow_focus = true;
        }
        KeyCode::BackTab => {
            a.focus = (a.focus + a.fields.len() + 1) % (a.fields.len() + 2);
            a.follow_focus = true;
        }
        KeyCode::Enter if a.focus == a.fields.len() => {
            if viewport_ok {
                a.submit();
            } else {
                a.feedback = "Resize to at least 28 columns and 12 rows before submitting.".into();
            }
        }
        KeyCode::Enter if a.focus == a.fields.len() + 1 => a.menu(),
        KeyCode::Enter if a.fields[a.focus].multiline => a.edit("\n"),
        KeyCode::Backspace => {
            if let Some(f) = a.fields.get_mut(a.focus) {
                f.text.pop();
            }
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if let Some(f) = a.fields.get_mut(a.focus) {
                f.text.clear();
            }
        }
        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            a.edit(&c.to_string());
        }
        KeyCode::Up | KeyCode::Down => {
            if let Some(f) = a.fields.get_mut(a.focus).filter(|f| !f.choices.is_empty()) {
                f.choice = if key.code == KeyCode::Down {
                    (f.choice + 1) % f.choices.len()
                } else {
                    (f.choice + f.choices.len() - 1) % f.choices.len()
                };
            } else {
                a.follow_focus = false;
                a.scroll = if key.code == KeyCode::Down {
                    a.scroll.saturating_add(1)
                } else {
                    a.scroll.saturating_sub(1)
                };
            }
        }
        _ => {}
    }
    // Keep the form kind (including a retained draft) independent of menu navigation.
    a.draft_kind = Some(kind);
    true
}

fn handle_menu(a: &mut Authoring, selected: Option<crate::work::WorkSelection>) {
    match a.cursor {
        0 => a.form(FormKind::Campaign),
        1 => a.form(FormKind::Quest),
        2 => a.form(FormKind::Request),
        3 => {
            if let Some(crate::work::WorkSelection::Obligation(id)) = selected {
                let obligation = a
                    .metadata
                    .as_ref()
                    .and_then(|m| m.inventory.obligations.get(&id))
                    .cloned();
                if let Some(o) = obligation.filter(|o| {
                    o.kind == ObligationKind::Decision && o.state == ObligationState::Open
                }) {
                    a.return_after_read = true;
                    a.read(o.id, o.incarnation);
                } else {
                    a.feedback =
                        "Select an owned open decision on the Board, or use Decisions.".into();
                }
            } else {
                a.feedback = "Select an owned open decision, or use Decisions.".into();
            }
        }
        4 => a.form(FormKind::Artifact),
        5 => {
            if a.fields.is_empty() {
                a.surface = Surface::Campaigns;
                a.cursor = 0;
                a.scroll = 0;
            } else {
                a.feedback =
                    "Discard or finish the retained draft before changing campaign.".into();
            }
        }
        6 => {
            a.surface = Surface::Decisions;
            a.cursor = 0;
            a.scroll = 0;
        }
        7 => a.form(FormKind::CreateStore),
        8 => a.form(FormKind::OpenStore),
        9 => {
            a.surface = Surface::Store;
            a.scroll = 0;
        }
        10 => match a.phase {
            Phase::Indeterminate => a.pending = Some(Intent::Reconcile),
            Phase::ViewPending | Phase::RememberPending | Phase::Unavailable => {
                a.pending = Some(Intent::Refresh);
            }
            _ => a.feedback = "No pending operation to retry.".into(),
        },
        11 => a.discard(),
        12 => {
            if let Some(k) = a.draft_kind {
                a.surface = Surface::Form(k);
                a.scroll = 0;
            } else {
                a.feedback = "No retained draft.".into();
            }
        }
        _ => {}
    }
}
