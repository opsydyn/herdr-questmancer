//! One serial filesystem owner. Bounded requests/results; no detached writer on shutdown.
use super::authoring::{
    Draft, FormKind, FormTarget, Intent, Metadata, Phase, ReviewDetail, SelectedDetail,
    artifact_kinds, quest_kind,
};
use super::{
    Action, CheckReceipt, Command, CommandError, CommandId, OwnedStore, ReportAuthor, ReviewReport,
    ReviewTarget, StoreError, StoreSelection, random_source, random_token,
};
use crate::{
    app::Model,
    domain::Timestamp,
    persistence::{PersistedStateV1, PersistenceClient, WorkerPaths},
    sources::local_work::LocalWorkSource,
    work::{ArtifactId, CampaignId, ObligationId, QuestId},
};
use tokio::sync::mpsc;

#[derive(Debug)]
enum Operation {
    Setup(StoreSelection, bool, std::path::PathBuf),
    Submit(Box<Command>),
    Snapshot,
    Reconcile,
    Read(ObligationId, String, u64),
    ReadReview(ObligationId, String, u64),
}
#[derive(Debug)]
struct Request {
    tag: u64,
    operation: Operation,
}
#[derive(Debug)]
enum ResultValue {
    Snapshot(StoreSelection, Metadata),
    Detail(SelectedDetail),
    Review(Box<ReviewDetail>),
}
#[derive(Debug)]
struct Response {
    tag: u64,
    result: Result<ResultValue, StoreError>,
}
#[derive(Debug)]
pub struct OwnedRuntime {
    requests: Option<mpsc::Sender<Request>>,
    responses: mpsc::Receiver<Response>,
    worker: Option<std::thread::JoinHandle<()>>,
    next: u64,
    waiting: Option<u64>,
    snapshot: Option<(StoreSelection, Metadata)>,
    remember: bool,
    buffered: Option<Response>,
    submitted: Option<Command>,
    at: Timestamp,
    intent_path: Option<std::path::PathBuf>,
}
impl OwnedRuntime {
    pub fn start(paths: &WorkerPaths, model: &mut Model) -> Result<Self, String> {
        let (requests, mut rx) = mpsc::channel::<Request>(1);
        let (tx, responses) = mpsc::channel(1);
        let worker = std::thread::Builder::new()
            .name("questmancer-owned-work".into())
            .spawn(move || {
                let mut store: Option<OwnedStore> = None;
                let mut selection: Option<StoreSelection> = None;
                while let Some(request) = rx.blocking_recv() {
                    let result = process(&request.operation, &mut store, &mut selection);
                    if tx
                        .blocking_send(Response {
                            tag: request.tag,
                            result,
                        })
                        .is_err()
                    {
                        break;
                    }
                }
            })
            .map_err(|e| e.to_string())?;
        let available = paths.state.is_some() && paths.work_path().is_some();
        model.authoring_mut().available = available;
        model.authoring_mut().default_path = paths
            .state
            .as_ref()
            .and_then(|p| p.parent())
            .map_or_else(String::new, |p| {
                p.join("owned-work.json").display().to_string()
            });
        if available
            && model.authoring().blocked.is_none()
            && let Some(selection) = model.durable_intent().owned_store().cloned()
        {
            model.authoring_mut().request_setup(selection, false);
        }
        Ok(Self {
            requests: Some(requests),
            responses,
            worker: Some(worker),
            next: 0,
            waiting: None,
            snapshot: None,
            remember: false,
            buffered: None,
            submitted: None,
            at: model.now(),
            intent_path: paths.state.clone(),
        })
    }
    /// Only this I/O boundary allocates command identity and freezes its supplied timestamp.
    pub fn dispatch(&mut self, model: &mut Model) {
        if self.waiting.is_some() {
            return;
        }
        Self::load_selected_review(model);
        if model.modal() == &crate::app::Modal::QuestBoard
            && model.authoring().ready()
            && model.authoring().fields.is_empty()
            && model.authoring().pending.is_none()
        {
            let selected = match model.work_selection() {
                Some(crate::work::WorkSelection::Obligation(id)) => model
                    .authoring()
                    .metadata
                    .as_ref()
                    .and_then(|m| m.inventory.obligations.get(id))
                    .filter(|o| o.kind == crate::work::ObligationKind::Decision)
                    .cloned(),
                _ => None,
            };
            if let Some(o) = selected
                && model.authoring().detail.as_ref().is_none_or(|d| {
                    d.id != o.id
                        || d.incarnation != o.incarnation
                        || model
                            .authoring()
                            .metadata
                            .as_ref()
                            .is_none_or(|m| m.revision != d.revision)
                })
            {
                model.authoring_mut().read(o.id, o.incarnation);
            }
        }
        let Some(intent) = model.authoring_mut().pending.take() else {
            return;
        };
        if !model.authoring().available || model.authoring().blocked.is_some() {
            model.authoring_mut().phase = Phase::Unavailable;
            model.authoring_mut().feedback = model
                .authoring()
                .blocked
                .clone()
                .unwrap_or("Durable state directory unavailable.".into());
            return;
        }
        let operation = match self.prepare(intent, model) {
            Ok(op) => op,
            Err(e) => {
                model.authoring_mut().phase = Phase::Ready;
                model.authoring_mut().feedback = e;
                return;
            }
        };
        let Some(tag) = self.next.checked_add(1) else {
            model.authoring_mut().phase = Phase::Unavailable;
            model.authoring_mut().feedback = "Owned request sequence exhausted.".into();
            return;
        };
        let reading = matches!(&operation, Operation::Read(..) | Operation::ReadReview(..));
        self.next = tag;
        self.at = model.now();
        if self
            .requests
            .as_ref()
            .is_none_or(|s| s.try_send(Request { tag, operation }).is_err())
        {
            model.authoring_mut().phase = Phase::Unavailable;
            model.authoring_mut().feedback =
                "Owned worker unavailable; outcome must be checked before retry.".into();
            return;
        }
        self.waiting = Some(tag);
        model.authoring_mut().phase = if reading {
            Phase::Reading
        } else {
            Phase::Saving
        };
    }
    fn load_selected_review(model: &mut Model) {
        if model.modal() != &(crate::app::Modal::QuestInbox { detail: true })
            || !model.authoring().ready()
            || model.authoring().pending.is_some()
        {
            return;
        }
        let Some((o, revision)) = crate::work::inbox::owned_review(model) else {
            return;
        };
        if model
            .authoring()
            .review
            .as_ref()
            .is_none_or(|detail| detail.id != o.id || detail.revision != revision)
        {
            let (id, incarnation) = (o.id.clone(), o.incarnation.clone());
            model.authoring_mut().read_review(id, incarnation, revision);
        }
    }
    fn prepare(&mut self, intent: Intent, model: &mut Model) -> Result<Operation, String> {
        match intent {
            Intent::Setup { selection, create } => {
                selection.validate().map_err(|e| e.to_string())?;
                if let Some(old) = model.authoring().metadata.as_ref() {
                    let work = crate::work::reduce(
                        model.work().clone(),
                        crate::work::Message::Disconnected(old.source.clone()),
                    )
                    .map_err(|e| e.to_string())?;
                    model.replace_work(work);
                }
                model.authoring_mut().metadata = None;
                model.authoring_mut().detail = None;
                model.authoring_mut().review = None;
                self.snapshot = None;
                self.submitted = None;
                self.remember = true;
                model.authoring_mut().selection = Some(selection.clone());
                Ok(Operation::Setup(
                    selection,
                    create,
                    self.intent_path
                        .clone()
                        .ok_or("Intent storage unavailable")?,
                ))
            }
            Intent::Submit(draft) if draft.kind == FormKind::CreateStore => {
                let selection = StoreSelection::new(
                    draft.values[0].clone().into(),
                    random_source().map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?;
                self.prepare(
                    Intent::Setup {
                        selection,
                        create: true,
                    },
                    model,
                )
            }
            Intent::Submit(draft) => {
                let command = make_command(&draft, model)?;
                self.submitted = Some(command.clone());
                Ok(Operation::Submit(Box::new(command)))
            }
            Intent::Read {
                id,
                incarnation,
                revision,
            } => Ok(Operation::Read(id, incarnation, revision)),
            Intent::ReadReview {
                id,
                incarnation,
                revision,
            } => Ok(Operation::ReadReview(id, incarnation, revision)),
            Intent::Refresh => Ok(Operation::Snapshot),
            Intent::Reconcile => Ok(Operation::Reconcile),
        }
    }
    pub async fn wait_result(&mut self) {
        if self.waiting.is_some() {
            if self.buffered.is_none() {
                self.buffered = self.responses.recv().await;
            }
        } else {
            std::future::pending::<()>().await;
        }
    }
    pub async fn consume_next(&mut self, client: &mut PersistenceClient, model: &mut Model) {
        let Some(response) = (if self.buffered.is_some() {
            self.buffered.take()
        } else {
            self.responses.recv().await
        }) else {
            model.authoring_mut().phase = Phase::Unavailable;
            model.authoring_mut().feedback =
                "Owned worker closed; reopen the exact selection to check outcome.".into();
            self.waiting = None;
            return;
        };
        if self.waiting != Some(response.tag) {
            return;
        }
        self.waiting = None;
        match response.result {
            Ok(ResultValue::Snapshot(selection, metadata)) => {
                // Reconciliation may prove that the original file is still current.
                // Such an outcome must preserve the draft, not invent a saved receipt.
                let not_committed = self
                    .submitted
                    .as_ref()
                    .is_some_and(|command| metadata.revision == command.expected_revision);
                if not_committed {
                    self.submitted = None;
                }
                self.snapshot = Some((selection.clone(), metadata));
                model.authoring_mut().selection = Some(selection);
                self.publish(client, model).await;
                if not_committed {
                    model.authoring_mut().feedback = "Outcome checked: the command was not saved. Draft retained; review before submitting again.".into();
                }
            }
            Ok(ResultValue::Detail(detail)) => {
                model.authoring_mut().detail = Some(detail);
                model.authoring_mut().phase = Phase::Ready;
                model.authoring_mut().feedback.clear();
                if model.authoring().return_after_read {
                    model.authoring_mut().return_after_read = false;
                    model.authoring_mut().form(FormKind::Answer);
                }
            }
            Ok(ResultValue::Review(detail)) => {
                model.authoring_mut().review = Some(*detail);
                model.authoring_mut().phase = Phase::Ready;
            }
            Err(error) => self.report_operation_error(model, &error),
        }
    }
    fn report_operation_error(&mut self, model: &mut Model, error: &StoreError) {
        if matches!(error, StoreError::Command(CommandError::PrerequisiteCycle)) {
            self.submitted = None;
            model.authoring_mut().phase = Phase::Ready;
            model.authoring_mut().feedback =
                "This link would create a cycle. Choose another prerequisite.".into();
            return;
        }
        if matches!(
            error,
            StoreError::ReviewEvidence(_)
                | StoreError::Command(
                    CommandError::ReviewMismatch
                        | CommandError::ReviewNotReady
                        | CommandError::StaleCheck
                )
        ) || matches!(error, StoreError::Command(CommandError::Invalid))
            && self.submitted.as_ref().is_some_and(|c| {
                matches!(
                    c.action,
                    Action::RequestReview { .. } | Action::RecordCheck { .. }
                )
            })
        {
            self.submitted = None;
            model.authoring_mut().phase = Phase::Ready;
            model.authoring_mut().feedback = format!("Review remains open. {error}");
            return;
        }
        let phase = if matches!(
            error,
            StoreError::Indeterminate(_) | StoreError::CreationIndeterminate
        ) {
            Phase::Indeterminate
        } else if matches!(error, StoreError::Command(_)) {
            Phase::Ready
        } else {
            Phase::Unavailable
        };
        model.authoring_mut().phase = phase;
        model.authoring_mut().feedback = if phase == Phase::Indeterminate {
            format!("May have saved. Check outcome before another submission: {error}")
        } else {
            format!(
                "Owned operation not acknowledged: {error}. Draft retained; reopen or refresh before resubmitting."
            )
        };
        if phase == Phase::Ready {
            self.submitted = None;
            // Stale references need a checked snapshot, not an automatic command retry.
            model.authoring_mut().metadata = None;
            model.authoring_mut().phase = Phase::Unavailable;
        }
        if phase == Phase::Unavailable
            && let Some(s) = &model.authoring().selection
            && let Ok(work) = crate::work::reduce(
                model.work().clone(),
                crate::work::Message::Disconnected(s.source.clone()),
            )
        {
            model.replace_work(work);
        }
    }
    async fn publish(&mut self, client: &mut PersistenceClient, model: &mut Model) {
        let Some((selection, metadata)) = self.snapshot.clone() else {
            return;
        };
        if self.remember {
            let mut state = PersistedStateV1::capture(model);
            state.owned_store = Some(selection.clone());
            if let Err(error) = client.remember_owner(state).await {
                model.authoring_mut().phase = Phase::RememberPending;
                model.authoring_mut().feedback = format!(
                    "Store open; remembering selection was not acknowledged. Retry the same path/source: {error}"
                );
                return;
            }
            model
                .durable_intent_mut()
                .set_owned_store(Some(selection.clone()));
            self.remember = false;
        }
        let result = match LocalWorkSource::from_inventory(
            &metadata.source,
            metadata.revision,
            &metadata.inventory,
            model.work(),
        ) {
            Ok(source) => {
                match source.inventory_baseline(&metadata.inventory, metadata.revision, self.at) {
                    Ok(envelope) => crate::work::runtime::accept(client, model, envelope).await,
                    Err(e) => Err(e.to_string()),
                }
            }
            Err(e) => Err(e.to_string()),
        };
        if let Err(error) = result {
            model.authoring_mut().phase = Phase::ViewPending;
            model.authoring_mut().feedback =
                format!("Store saved/open; Board not updated. Retry view only: {error}");
            return;
        }
        let campaign = match self.submitted.as_ref().map(|c| &c.action) {
            Some(Action::CreateCampaign { id, .. }) => Some(id.clone()),
            _ => model.authoring().campaign.clone(),
        };
        let answer_id = match self.submitted.as_ref().map(|c| &c.action) {
            Some(Action::AnswerDecision {
                id, incarnation, ..
            }) => Some((id.clone(), incarnation.clone())),
            _ => None,
        };
        model.authoring_mut().campaign = campaign;
        model.authoring_mut().metadata = Some(metadata.clone());
        model.authoring_mut().phase = Phase::Ready;
        model.authoring_mut().feedback = if self.submitted.is_some() {
            "Saved; Board updated.".into()
        } else {
            "Store open; Board updated. No new command was submitted.".into()
        };
        if self.submitted.take().is_some()
            || model.authoring().draft_kind == Some(FormKind::CreateStore)
            || model.authoring().draft_kind == Some(FormKind::OpenStore)
        {
            model.authoring_mut().fields.clear();
            model.authoring_mut().draft_kind = None;
            model.authoring_mut().form_target = None;
            model.authoring_mut().menu();
        }
        if let Some((id, incarnation)) = answer_id {
            model.authoring_mut().read(id, incarnation);
        }
    }
    pub async fn shutdown(
        &mut self,
        client: &mut PersistenceClient,
        model: &mut Model,
    ) -> Result<(), String> {
        if self.waiting.is_some() {
            self.consume_next(client, model).await;
        }
        // No new requests are dispatched here; accepted filesystem calls finish on their owner.
        self.requests.take();
        self.responses.close();
        if let Some(worker) = self.worker.take() {
            tokio::task::spawn_blocking(move || worker.join())
                .await
                .map_err(|e| e.to_string())?
                .map_err(|_| "Owned worker panicked".to_string())?;
        }
        Ok(())
    }
}
fn snapshot(store: &OwnedStore, selection: &StoreSelection) -> Result<ResultValue, StoreError> {
    let state = store.checked_state()?;
    Ok(ResultValue::Snapshot(
        selection.clone(),
        Metadata {
            source: state.source().clone(),
            revision: state.revision(),
            inventory: state.inventory().clone(),
        },
    ))
}
fn process(
    operation: &Operation,
    store: &mut Option<OwnedStore>,
    selection: &mut Option<StoreSelection>,
) -> Result<ResultValue, StoreError> {
    if let Operation::Setup(selected, create, intent_path) = operation {
        match std::fs::read(intent_path) {
            Ok(bytes) => {
                crate::persistence::parse_state(intent_path, &bytes)
                    .map_err(|e| StoreError::Protected(e.to_string()))?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(StoreError::Io(e.to_string())),
        }
        store.take();
        *selection = Some(selected.clone());
        *store = Some(if *create {
            OwnedStore::create(&selected.path, selected.source.clone())?
        } else {
            OwnedStore::open(&selected.path, &selected.source)?
        });
    }
    if matches!(operation, Operation::Reconcile) && store.is_none() {
        let selected = selection.as_ref().ok_or(StoreError::Selection)?;
        *store = Some(OwnedStore::open(&selected.path, &selected.source)?);
    }
    let owned = store.as_mut().ok_or(StoreError::Selection)?;
    let selected = selection.as_ref().ok_or(StoreError::Selection)?;
    match operation {
        Operation::Submit(command) => {
            owned.execute(command)?;
        }
        Operation::Reconcile => {
            owned.reconcile()?;
        }
        Operation::Read(id, incarnation, revision) => {
            let state = owned.checked_state()?;
            if state.revision() != *revision {
                return Err(super::CommandError::Stale.into());
            }
            let obligation = state
                .inventory()
                .obligations
                .get(id)
                .filter(|o| &o.incarnation == incarnation)
                .ok_or(super::CommandError::Reference)?;
            let detail = state
                .decision_detail(id)
                .ok_or(super::CommandError::Reference)?;
            return Ok(ResultValue::Detail(SelectedDetail {
                id: id.clone(),
                incarnation: incarnation.clone(),
                revision: *revision,
                prompt: obligation.prompt.clone(),
                context: obligation.context.clone(),
                requested_by: detail.requested_by.clone(),
                answer: detail.answer.clone(),
            }));
        }
        Operation::ReadReview(id, incarnation, revision) => {
            let state = owned.checked_state()?;
            if state.revision() != *revision {
                return Err(CommandError::Stale.into());
            }
            state
                .inventory()
                .obligations
                .get(id)
                .filter(|o| &o.incarnation == incarnation)
                .ok_or(CommandError::Reference)?;
            let review = state.review(id).ok_or(CommandError::Reference)?.clone();
            let changes = super::review_git::changes(&review.target);
            return Ok(ResultValue::Review(Box::new(ReviewDetail {
                id: id.clone(),
                revision: *revision,
                review,
                changes,
            })));
        }
        _ => {}
    }
    snapshot(owned, selected)
}
fn make_command(draft: &Draft, model: &Model) -> Result<Command, String> {
    let source = &model
        .authoring()
        .metadata
        .as_ref()
        .ok_or("Owner metadata unavailable")?
        .source;
    let token = random_token().map_err(|e| e.to_string())?;
    let v = &draft.values;
    let optional = |index: usize| v.get(index).filter(|s| !s.is_empty()).cloned();
    let campaign = || {
        draft
            .campaign
            .clone()
            .ok_or("Choose an owned campaign".to_string())
    };
    let quest = |index| -> Result<Option<QuestId>, String> {
        optional(index)
            .map(|id| {
                model
                    .authoring()
                    .metadata
                    .as_ref()
                    .and_then(|m| m.inventory.quests.keys().find(|q| q.as_str() == id))
                    .cloned()
                    .ok_or("Quest reference is stale".into())
            })
            .transpose()
    };
    let action = match draft.kind {
        FormKind::Campaign => Action::CreateCampaign {
            id: CampaignId::new(source, &token),
            name: v[0].clone(),
            objective: optional(1),
        },
        FormKind::Quest => Action::CreateQuest {
            id: QuestId::new(source, &token),
            campaign: campaign()?,
            title: v[0].clone(),
            description: optional(2),
            kind: quest_kind(&v[1]).ok_or("Choose quest kind")?,
        },
        FormKind::QuestLifecycle => match (&draft.form_target, v.as_slice()) {
            (Some(FormTarget::QuestLifecycle { id, change, .. }), [confirmation])
                if confirmation == "confirm" =>
            {
                Action::ChangeQuestLifecycle {
                    id: id.clone(),
                    change: *change,
                }
            }
            _ => return Err("Confirm the change for the selected owned quest".into()),
        },
        FormKind::QuestPrerequisites => prerequisite_action(draft, quest(0)?)?,
        FormKind::Request => Action::RequestDecision {
            id: ObligationId::new(source, &token),
            campaign: campaign()?,
            quest: quest(2)?,
            incarnation: random_token().map_err(|e| e.to_string())?,
            prompt: v[0].clone(),
            context: optional(1),
        },
        FormKind::Answer => {
            let (id, incarnation) = draft.decision.clone().ok_or("Choose an open decision")?;
            Action::AnswerDecision {
                id,
                incarnation,
                answer: v[0].clone(),
            }
        }
        FormKind::Artifact => Action::RegisterArtifact {
            id: ArtifactId::new(source, &token),
            campaign: campaign()?,
            quest: quest(3)?,
            kind: *artifact_kinds()
                .iter()
                .find(|k| format!("{k:?}") == v[1])
                .ok_or("Choose artifact kind")?,
            title: v[0].clone(),
            uri: optional(2),
        },
        FormKind::RequestReview | FormKind::RecordCheck | FormKind::SettleReview => {
            review_action(draft, source, &token)?
        }
        _ => return Err("Invalid command form".into()),
    };
    Ok(Command {
        id: CommandId::new(&random_token().map_err(|e| e.to_string())?),
        source: source.clone(),
        expected_revision: draft.revision,
        actor: "Questmancer".into(),
        at: model.now(),
        action,
    })
}

fn prerequisite_action(draft: &Draft, prerequisite_id: Option<QuestId>) -> Result<Action, String> {
    let Some(FormTarget::QuestPrerequisites { id, current, .. }) = &draft.form_target else {
        return Err("Select an owned quest to edit prerequisites".into());
    };
    let prerequisite_id = prerequisite_id.ok_or("Choose a prerequisite to add or remove")?;
    let change = if current.contains(&prerequisite_id) {
        super::PrerequisiteChange::Remove
    } else {
        super::PrerequisiteChange::Add
    };
    Ok(Action::ChangeQuestPrerequisite {
        id: id.clone(),
        prerequisite_id,
        change,
    })
}

fn review_action(
    draft: &Draft,
    source: &crate::work::SourceId,
    token: &str,
) -> Result<Action, String> {
    match (draft.kind, &draft.form_target, draft.values.as_slice()) {
        (
            FormKind::RequestReview,
            Some(FormTarget::Quest { id, .. }),
            [
                criterion,
                repository,
                base,
                head,
                recipe,
                runner,
                author,
                summary,
            ],
        ) => {
            let report = match (author.as_str(), summary.trim().is_empty()) {
                ("", true) => None,
                ("Pi", false) => Some(ReviewReport {
                    author: ReportAuthor::Pi,
                    summary: summary.clone(),
                }),
                ("Codex", false) => Some(ReviewReport {
                    author: ReportAuthor::Codex,
                    summary: summary.clone(),
                }),
                _ => {
                    return Err(
                        "Choose Pi or Codex and supply report text, or leave both empty".into(),
                    );
                }
            };
            let target = ReviewTarget {
                repository: repository.trim().into(),
                base: base.trim().to_owned(),
                head: head.trim().to_owned(),
                recipe: recipe.trim().to_owned(),
                runner: runner.trim().to_owned(),
            };
            if !target.valid() {
                return Err("Use the physical checkout path, two distinct full commit IDs, a recipe and a runner label".into());
            }
            Ok(Action::RequestReview {
                id: ObligationId::new(source, token),
                quest: id.clone(),
                incarnation: token.into(),
                criterion: criterion.clone(),
                target,
                report,
            })
        }
        (
            FormKind::RecordCheck,
            Some(FormTarget::Review {
                id, incarnation, ..
            }),
            [input],
        ) => {
            let receipt = CheckReceipt::parse(input)?;
            if !receipt.valid() {
                return Err(
                    "The check receipt has an invalid path, commit ID, label or timestamp".into(),
                );
            }
            Ok(Action::RecordCheck {
                id: id.clone(),
                incarnation: incarnation.clone(),
                receipt,
            })
        }
        (
            FormKind::SettleReview,
            Some(FormTarget::Review {
                id, incarnation, ..
            }),
            [confirmation],
        ) if confirmation == "settle" => Ok(Action::SettleReview {
            id: id.clone(),
            incarnation: incarnation.clone(),
        }),
        _ => Err("No matching owned quest or review selected".into()),
    }
}

impl Drop for OwnedRuntime {
    fn drop(&mut self) {
        self.requests.take();
        self.responses.close();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn reconciled_original_revision_is_not_reported_saved_or_discarded() {
        use crate::{
            app::View,
            work::{Inventory, SourceId},
        };
        let dir =
            tempfile::tempdir_in(std::fs::canonicalize(std::env::temp_dir()).unwrap()).unwrap();
        let paths = WorkerPaths::new(Some(dir.path().join("state.json")), None);
        let (mut client, _, writer) = crate::persistence::PersistenceWorker::start(paths.clone());
        let mut model = Model::new(View::Guild);
        let mut runtime = OwnedRuntime::start(&paths, &mut model).unwrap();
        let source = SourceId::new("owned:test");
        let metadata = Metadata {
            source: source.clone(),
            revision: 0,
            inventory: Inventory::default(),
        };
        model.authoring_mut().metadata = Some(metadata.clone());
        model.authoring_mut().phase = Phase::Ready;
        model.authoring_mut().form(FormKind::Campaign);
        model.authoring_mut().edit("Retain this unsaved draft");
        model.authoring_mut().phase = Phase::Indeterminate;
        runtime.submitted = Some(Command {
            id: CommandId::new("pending"),
            source: source.clone(),
            expected_revision: 0,
            actor: "Questmancer".into(),
            at: Timestamp::from_millis(1),
            action: Action::CreateCampaign {
                id: CampaignId::new(&source, "campaign"),
                name: "Retain this unsaved draft".into(),
                objective: None,
            },
        });
        runtime.waiting = Some(1);
        runtime.buffered = Some(Response {
            tag: 1,
            result: Ok(ResultValue::Snapshot(
                StoreSelection::new(dir.path().join("owned-work.json"), source).unwrap(),
                metadata,
            )),
        });
        runtime.consume_next(&mut client, &mut model).await;
        assert_eq!(
            model.authoring().fields[0].text,
            "Retain this unsaved draft"
        );
        assert!(model.authoring().feedback.contains("not saved"));
        assert!(
            model
                .authoring()
                .metadata
                .as_ref()
                .unwrap()
                .inventory
                .campaigns
                .is_empty()
        );
        runtime.shutdown(&mut client, &mut model).await.unwrap();
        client.shutdown().await.unwrap();
        writer.await.unwrap();
    }
    #[tokio::test]
    async fn late_response_cannot_replace_the_current_request_or_owner_metadata() {
        use crate::{
            app::View,
            work::{Inventory, SourceId},
        };
        let dir =
            tempfile::tempdir_in(std::fs::canonicalize(std::env::temp_dir()).unwrap()).unwrap();
        let paths = WorkerPaths::new(Some(dir.path().join("state.json")), None);
        let (mut client, _, writer) = crate::persistence::PersistenceWorker::start(paths.clone());
        let mut model = Model::new(View::Guild);
        let mut runtime = OwnedRuntime::start(&paths, &mut model).unwrap();
        runtime.waiting = Some(2);
        runtime.buffered = Some(Response {
            tag: 1,
            result: Ok(ResultValue::Snapshot(
                StoreSelection::new(dir.path().join("old.json"), SourceId::new("owned:old"))
                    .unwrap(),
                Metadata {
                    source: SourceId::new("owned:old"),
                    revision: 99,
                    inventory: Inventory::default(),
                },
            )),
        });
        runtime.consume_next(&mut client, &mut model).await;
        assert!(model.authoring().metadata.is_none());
        assert_eq!(runtime.waiting, Some(2));
        runtime.waiting = None;
        runtime.shutdown(&mut client, &mut model).await.unwrap();
        client.shutdown().await.unwrap();
        writer.await.unwrap();
    }
}
