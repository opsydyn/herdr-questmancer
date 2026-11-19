use super::{CheckReceipt, Review, ReviewReport, ReviewSettlement, ReviewTarget};
use crate::{
    domain::Timestamp,
    work::{
        Artifact, ArtifactId, ArtifactKind, CampaignId, CampaignState, Inventory, Message,
        Obligation, ObligationId, ObligationKind, ObligationState, Projection, Quest, QuestId,
        QuestKind, QuestState, Responsible, SourceId, WorkCampaign,
    },
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

const RECEIPTS: usize = 128;
const TEXT_BYTES: usize = 8192;

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct CommandId(String);
impl CommandId {
    pub fn new(value: &str) -> Self {
        Self(value.to_owned())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub id: CommandId,
    pub source: SourceId,
    pub expected_revision: u64,
    pub actor: String,
    pub at: Timestamp,
    pub action: Action,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum QuestLifecycleChange {
    MarkActive,
    MarkComplete,
    Reopen,
}
impl QuestLifecycleChange {
    pub(crate) const fn for_state(state: QuestState) -> Option<Self> {
        match state {
            QuestState::Planned => Some(Self::MarkActive),
            QuestState::Active => Some(Self::MarkComplete),
            QuestState::Completed => Some(Self::Reopen),
            _ => None,
        }
    }

    pub(crate) const fn states(self) -> (QuestState, QuestState) {
        match self {
            Self::MarkActive => (QuestState::Planned, QuestState::Active),
            Self::MarkComplete => (QuestState::Active, QuestState::Completed),
            Self::Reopen => (QuestState::Completed, QuestState::Active),
        }
    }

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::MarkActive => "Mark active",
            Self::MarkComplete => "Mark complete",
            Self::Reopen => "Reopen quest",
        }
    }
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "operation", deny_unknown_fields)]
pub enum Action {
    CreateCampaign {
        id: CampaignId,
        name: String,
        objective: Option<String>,
    },
    CreateQuest {
        id: QuestId,
        campaign: CampaignId,
        title: String,
        description: Option<String>,
        kind: QuestKind,
    },
    ChangeQuestLifecycle {
        id: QuestId,
        change: QuestLifecycleChange,
    },
    RequestDecision {
        id: ObligationId,
        campaign: CampaignId,
        quest: Option<QuestId>,
        incarnation: String,
        prompt: String,
        context: Option<String>,
    },
    AnswerDecision {
        id: ObligationId,
        incarnation: String,
        answer: String,
    },
    RegisterArtifact {
        id: ArtifactId,
        campaign: CampaignId,
        quest: Option<QuestId>,
        kind: ArtifactKind,
        title: String,
        uri: Option<String>,
    },
    RequestReview {
        id: ObligationId,
        quest: QuestId,
        incarnation: String,
        criterion: String,
        target: ReviewTarget,
        report: Option<ReviewReport>,
    },
    RecordCheck {
        id: ObligationId,
        incarnation: String,
        receipt: CheckReceipt,
    },
    SettleReview {
        id: ObligationId,
        incarnation: String,
    },
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Answer {
    pub text: String,
    pub actor: String,
    pub at: Timestamp,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionDetail {
    pub requested_by: String,
    pub answer: Option<Answer>,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub command: Command,
    pub revision: u64,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedState {
    source: SourceId,
    revision: u64,
    inventory: Inventory,
    decisions: BTreeMap<ObligationId, DecisionDetail>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    reviews: BTreeMap<ObligationId, Review>,
    receipts: VecDeque<Receipt>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transition {
    pub state: OwnedState,
    pub receipt: Receipt,
    pub replayed: bool,
}
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum CommandError {
    #[error("invalid owned work or command")]
    Invalid,
    #[error("command targets another source")]
    ForeignSource,
    #[error("command was based on a stale revision")]
    Stale,
    #[error("command identity was reused with a different payload")]
    RetryConflict,
    #[error("entity or decision incarnation already exists or is resolved")]
    Conflict,
    #[error("campaign, quest or decision reference does not match")]
    Reference,
    #[error("requested change does not match the quest's current state")]
    QuestLifecycle,
    #[error("check receipt does not match the pinned repository, revision, recipe or runner")]
    ReviewMismatch,
    #[error(
        "check receipt is newer than the current time or no newer than the last recorded check"
    )]
    StaleCheck,
    #[error("review needs a passing check for the pinned revision")]
    ReviewNotReady,
    #[error("owned revision exhausted")]
    Exhausted,
}
impl OwnedState {
    pub fn new(source: SourceId) -> Result<Self, CommandError> {
        if !owned_source(&source) {
            return Err(CommandError::Invalid);
        }
        Ok(Self {
            source,
            revision: 0,
            inventory: Inventory::default(),
            decisions: BTreeMap::new(),
            reviews: BTreeMap::new(),
            receipts: VecDeque::new(),
        })
    }
    pub fn source(&self) -> &SourceId {
        &self.source
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn inventory(&self) -> &Inventory {
        &self.inventory
    }
    pub fn answer(&self, id: &ObligationId) -> Option<&Answer> {
        self.decisions.get(id)?.answer.as_ref()
    }
    pub fn decision_detail(&self, id: &ObligationId) -> Option<&DecisionDetail> {
        self.decisions.get(id)
    }
    pub fn receipt(&self, id: &CommandId) -> Option<&Receipt> {
        self.receipts.iter().find(|r| &r.command.id == id)
    }

    pub fn review(&self, id: &ObligationId) -> Option<&Review> {
        self.reviews.get(id)
    }

    pub(crate) fn has_reviews(&self) -> bool {
        !self.reviews.is_empty()
    }

    pub(crate) fn has_quest_lifecycle(&self) -> bool {
        // Lifecycle changes never return a quest to Planned, so v3 outlives its receipts.
        self.inventory
            .quests
            .values()
            .any(|quest| quest.state != QuestState::Planned)
    }

    /// Protected on load as well as before publication. No live/session facts exist here.
    pub fn validate(&self) -> Result<(), CommandError> {
        if !owned_source(&self.source)
            || self.receipts.len()
                != usize::try_from(self.revision.min(RECEIPTS as u64))
                    .map_err(|_| CommandError::Invalid)?
        {
            return Err(CommandError::Invalid);
        }
        crate::work::reduce(
            Projection::default(),
            Message::Baseline {
                source: self.source.clone(),
                generation: 1,
                captured_at: Timestamp::from_millis(0),
                cursor: self.revision,
                inventory: self.inventory.clone(),
                complete: true,
                bindings: BTreeMap::new(),
            },
        )
        .map_err(|_| CommandError::Invalid)?;
        let mut incarnations = BTreeSet::new();
        let mut open_reviews = BTreeSet::new();
        let mut valid = self.inventory.campaigns.values().all(|c| {
            c.state == CampaignState::Planned
                && c.revision <= self.revision
                && text(&c.name)
                && optional(c.objective.as_deref())
        }) && self.inventory.quests.values().all(|q| {
            QuestLifecycleChange::for_state(q.state).is_some()
                && q.assignment.is_none()
                && q.dependencies.is_empty()
                && q.kind != QuestKind::Unspecified
                && q.revision <= self.revision
                && text(&q.title)
                && optional(q.description.as_deref())
        }) && self.inventory.artifacts.values().all(|a| {
            a.validation.is_none()
                && a.head_revision.is_none()
                && a.revision <= self.revision
                && text(&a.title)
                && optional(a.uri.as_deref())
        });
        for o in self.inventory.obligations.values() {
            valid &= o.responsible == Responsible::Human
                && token(&o.incarnation)
                && incarnations.insert(&o.incarnation)
                && o.revision <= self.revision
                && text(&o.prompt)
                && optional(o.context.as_deref());
            valid &= match o.kind {
                ObligationKind::Decision => self.valid_decision(o),
                ObligationKind::Review => {
                    self.valid_review(o)
                        && (o.state != ObligationState::Open
                            || open_reviews.insert(o.quest.as_ref()))
                }
                _ => false,
            };
        }
        valid &= self.decisions.keys().all(|id| {
            self.inventory
                .obligations
                .get(id)
                .is_some_and(|o| o.kind == ObligationKind::Decision)
        });
        valid &= self.reviews.keys().all(|id| {
            self.inventory
                .obligations
                .get(id)
                .is_some_and(|o| o.kind == ObligationKind::Review)
        });
        let mut ids = BTreeSet::new();
        let mut quest_receipts = BTreeMap::new();
        let first = self.revision - self.receipts.len() as u64;
        for (offset, r) in self.receipts.iter().enumerate() {
            valid &= r.revision == first + offset as u64 + 1
                && r.command.expected_revision == r.revision - 1
                && r.command.source == self.source
                && command_valid(&r.command)
                && ids.insert(&r.command.id)
                && self.has_effect(r, &mut quest_receipts);
        }
        valid &= self
            .inventory
            .quests
            .values()
            .all(|quest| match quest_receipts.get(&quest.id) {
                Some(&(revision, state)) => quest.revision == revision && quest.state == state,
                None => quest.revision <= first,
            });
        if valid {
            Ok(())
        } else {
            Err(CommandError::Invalid)
        }
    }

    fn valid_decision(&self, o: &Obligation) -> bool {
        self.decisions
            .get(&o.id)
            .is_some_and(|d| text(&d.requested_by))
            && match (o.state, self.answer(&o.id)) {
                (ObligationState::Open, None) => true,
                (ObligationState::Resolved, Some(a)) => text(&a.text) && text(&a.actor),
                _ => false,
            }
    }

    fn valid_review(&self, o: &Obligation) -> bool {
        self.review(&o.id).is_some_and(|review| {
            o.quest.is_some()
                && review.target.valid()
                && review.report.as_ref().is_none_or(ReviewReport::valid)
                && (1..=o.revision).contains(&review.requested_revision)
                && match (&review.check, review.check_revision) {
                    (Some(receipt), Some(revision)) => {
                        receipt.valid()
                            && review.target.matches(receipt)
                            && revision > review.requested_revision
                            && revision <= o.revision
                    }
                    (None, None) => true,
                    _ => false,
                }
                && match (o.state, &review.settlement) {
                    (ObligationState::Open | ObligationState::Superseded, None) => true,
                    (ObligationState::Resolved, Some(s)) => {
                        text(&s.actor)
                            && review.check_passed()
                            && review
                                .check
                                .as_ref()
                                .is_some_and(|receipt| receipt.captured_at <= s.at)
                            && review
                                .check_revision
                                .is_some_and(|revision| revision < o.revision)
                    }
                    _ => false,
                }
        })
    }
    fn has_effect<'a>(
        &self,
        r: &'a Receipt,
        quest_receipts: &mut BTreeMap<&'a QuestId, (u64, QuestState)>,
    ) -> bool {
        match &r.command.action {
            Action::CreateCampaign {
                id,
                name,
                objective,
            } => self.inventory.campaigns.get(id).is_some_and(|c| {
                c.revision == r.revision && &c.name == name && &c.objective == objective
            }),
            Action::CreateQuest {
                id,
                campaign,
                title,
                description,
                kind,
            } => self.inventory.quests.get(id).is_some_and(|q| {
                &q.campaign == campaign
                    && &q.title == title
                    && &q.description == description
                    && &q.kind == kind
                    && quest_receipts
                        .insert(id, (r.revision, QuestState::Planned))
                        .is_none()
            }),
            Action::ChangeQuestLifecycle { id, change } => {
                let (from, to) = change.states();
                if !self.inventory.quests.contains_key(id)
                    || quest_receipts
                        .get(id)
                        .is_some_and(|&(_, prior)| prior != from)
                {
                    return false;
                }
                quest_receipts.insert(id, (r.revision, to));
                true
            }
            Action::RequestDecision {
                id,
                campaign,
                quest,
                incarnation,
                prompt,
                context,
            } => {
                self.inventory.obligations.get(id).is_some_and(|o| {
                    o.revision >= r.revision
                        && &o.campaign == campaign
                        && &o.quest == quest
                        && &o.incarnation == incarnation
                        && &o.prompt == prompt
                        && &o.context == context
                        && o.requested_at == r.command.at
                }) && self
                    .decisions
                    .get(id)
                    .is_some_and(|d| d.requested_by == r.command.actor)
            }
            Action::AnswerDecision {
                id,
                incarnation,
                answer,
            } => {
                self.inventory
                    .obligations
                    .get(id)
                    .is_some_and(|o| o.revision == r.revision && &o.incarnation == incarnation)
                    && self.answer(id).is_some_and(|a| {
                        &a.text == answer && a.actor == r.command.actor && a.at == r.command.at
                    })
            }
            Action::RegisterArtifact {
                id,
                campaign,
                quest,
                kind,
                title,
                uri,
            } => self.inventory.artifacts.get(id).is_some_and(|a| {
                a.revision == r.revision
                    && &a.campaign == campaign
                    && &a.quest == quest
                    && &a.kind == kind
                    && &a.title == title
                    && &a.uri == uri
            }),
            Action::RequestReview { .. }
            | Action::RecordCheck { .. }
            | Action::SettleReview { .. } => self.has_review_effect(r),
        }
    }

    fn has_review_effect(&self, r: &Receipt) -> bool {
        match &r.command.action {
            Action::RequestReview {
                id,
                quest,
                incarnation,
                criterion,
                target,
                report,
            } => {
                self.inventory.obligations.get(id).is_some_and(|o| {
                    o.quest.as_ref() == Some(quest)
                        && &o.incarnation == incarnation
                        && &o.prompt == criterion
                        && o.requested_at == r.command.at
                }) && self.review(id).is_some_and(|review| {
                    review.requested_revision == r.revision
                        && &review.target == target
                        && &review.report == report
                })
            }
            Action::RecordCheck {
                id,
                incarnation,
                receipt,
            } => {
                self.inventory
                    .obligations
                    .get(id)
                    .is_some_and(|o| &o.incarnation == incarnation)
                    && self.review(id).is_some_and(|review| {
                        review.target.matches(receipt)
                            && receipt.captured_at <= r.command.at
                            && review.check_revision.is_some_and(|revision| {
                                revision > r.revision
                                    || revision == r.revision
                                        && review.check.as_ref() == Some(receipt)
                            })
                    })
            }
            Action::SettleReview { id, incarnation } => {
                self.inventory
                    .obligations
                    .get(id)
                    .is_some_and(|o| o.revision == r.revision && &o.incarnation == incarnation)
                    && self
                        .review(id)
                        .and_then(|review| review.settlement.as_ref())
                        .is_some_and(|s| s.actor == r.command.actor && s.at == r.command.at)
            }
            _ => unreachable!("typed review receipt dispatcher"),
        }
    }
}

/// Deterministic transition: IDs, time and attribution arrive as command facts.
pub fn transition(state: &OwnedState, command: &Command) -> Result<Transition, CommandError> {
    state.validate()?;
    if command.source != state.source {
        return Err(CommandError::ForeignSource);
    }
    if let Some(receipt) = state.receipt(&command.id) {
        return if &receipt.command == command {
            Ok(Transition {
                state: state.clone(),
                receipt: receipt.clone(),
                replayed: true,
            })
        } else {
            Err(CommandError::RetryConflict)
        };
    }
    if command.expected_revision != state.revision {
        return Err(CommandError::Stale);
    }
    if !command_valid(command) {
        return Err(CommandError::Invalid);
    }
    let revision = state
        .revision
        .checked_add(1)
        .ok_or(CommandError::Exhausted)?;
    let mut next = state.clone();
    match &command.action {
        Action::CreateCampaign { .. } => create_campaign(&mut next, command, revision)?,
        Action::CreateQuest { .. } => create_quest(&mut next, command, revision)?,
        Action::ChangeQuestLifecycle { id, change } => {
            own(id.belongs_to(&next.source))?;
            let quest = next
                .inventory
                .quests
                .get_mut(id)
                .ok_or(CommandError::Reference)?;
            let (from, to) = change.states();
            if quest.state != from {
                return Err(CommandError::QuestLifecycle);
            }
            quest.state = to;
            quest.revision = revision;
        }
        Action::RequestDecision { .. } => request_decision(&mut next, command, revision)?,
        Action::AnswerDecision { .. } => answer_decision(&mut next, command, revision)?,
        Action::RegisterArtifact { .. } => register_artifact(&mut next, command, revision)?,
        Action::RequestReview { .. } => request_review(&mut next, command, revision)?,
        Action::RecordCheck { .. } | Action::SettleReview { .. } => {
            update_review(&mut next, command, revision)?;
        }
    }
    next.revision = revision;
    let receipt = Receipt {
        command: command.clone(),
        revision,
    };
    next.receipts.push_back(receipt.clone());
    if next.receipts.len() > RECEIPTS {
        next.receipts.pop_front();
    }
    next.validate()?;
    Ok(Transition {
        state: next,
        receipt,
        replayed: false,
    })
}

fn request_review(
    state: &mut OwnedState,
    command: &Command,
    revision: u64,
) -> Result<(), CommandError> {
    let Action::RequestReview {
        id,
        quest,
        incarnation,
        criterion,
        target,
        report,
    } = &command.action
    else {
        unreachable!("typed command dispatcher")
    };
    own(id.belongs_to(&state.source) && quest.belongs_to(&state.source))?;
    let campaign = state
        .inventory
        .quests
        .get(quest)
        .ok_or(CommandError::Reference)?
        .campaign
        .clone();
    vacant(
        state.inventory.obligations.contains_key(id)
            || state
                .inventory
                .obligations
                .values()
                .any(|o| &o.incarnation == incarnation),
    )?;
    for o in state.inventory.obligations.values_mut().filter(|o| {
        o.kind == ObligationKind::Review
            && o.state == ObligationState::Open
            && o.quest.as_ref() == Some(quest)
    }) {
        o.state = ObligationState::Superseded;
        o.revision = revision;
    }
    state.inventory.obligations.insert(
        id.clone(),
        Obligation {
            id: id.clone(),
            campaign,
            quest: Some(quest.clone()),
            revision,
            incarnation: incarnation.clone(),
            kind: ObligationKind::Review,
            responsible: Responsible::Human,
            state: ObligationState::Open,
            prompt: criterion.clone(),
            context: None,
            action: "Review changes".into(),
            resolution: "Explicit settlement for the checked candidate".into(),
            requested_at: command.at,
        },
    );
    state.reviews.insert(
        id.clone(),
        Review {
            target: target.clone(),
            report: report.clone(),
            check: None,
            settlement: None,
            requested_revision: revision,
            check_revision: None,
        },
    );
    Ok(())
}

fn update_review(
    state: &mut OwnedState,
    command: &Command,
    revision: u64,
) -> Result<(), CommandError> {
    let (Action::RecordCheck {
        id, incarnation, ..
    }
    | Action::SettleReview { id, incarnation }) = &command.action
    else {
        unreachable!("typed command dispatcher")
    };
    own(id.belongs_to(&state.source))?;
    let obligation = state
        .inventory
        .obligations
        .get_mut(id)
        .filter(|o| o.kind == ObligationKind::Review && &o.incarnation == incarnation)
        .ok_or(CommandError::Reference)?;
    if obligation.state != ObligationState::Open {
        return Err(CommandError::Conflict);
    }
    let review = state.reviews.get_mut(id).ok_or(CommandError::Reference)?;
    if let Action::RecordCheck { receipt, .. } = &command.action {
        if !review.target.matches(receipt) {
            return Err(CommandError::ReviewMismatch);
        }
        if receipt.captured_at > command.at
            || review
                .check
                .as_ref()
                .is_some_and(|last| receipt.captured_at <= last.captured_at)
        {
            return Err(CommandError::StaleCheck);
        }
        review.check = Some(receipt.clone());
        review.check_revision = Some(revision);
    } else {
        if !review.check_passed()
            || review
                .check
                .as_ref()
                .is_some_and(|receipt| receipt.captured_at > command.at)
        {
            return Err(CommandError::ReviewNotReady);
        }
        review.settlement = Some(ReviewSettlement {
            actor: command.actor.clone(),
            at: command.at,
        });
        obligation.state = ObligationState::Resolved;
    }
    obligation.revision = revision;
    Ok(())
}
fn create_campaign(
    state: &mut OwnedState,
    command: &Command,
    revision: u64,
) -> Result<(), CommandError> {
    let Action::CreateCampaign {
        id,
        name,
        objective,
    } = &command.action
    else {
        unreachable!("typed command dispatcher")
    };
    let s = &state.source;
    own(id.belongs_to(s))?;
    vacant(state.inventory.campaigns.contains_key(id))?;
    state.inventory.campaigns.insert(
        id.clone(),
        WorkCampaign {
            id: id.clone(),
            revision,
            name: name.clone(),
            objective: objective.clone(),
            state: CampaignState::Planned,
        },
    );
    Ok(())
}

fn create_quest(
    state: &mut OwnedState,
    command: &Command,
    revision: u64,
) -> Result<(), CommandError> {
    let Action::CreateQuest {
        id,
        campaign,
        title,
        description,
        kind,
    } = &command.action
    else {
        unreachable!("typed command dispatcher")
    };
    let s = &state.source;
    own(id.belongs_to(s) && campaign.belongs_to(s))?;
    campaign_exists(state, campaign)?;
    vacant(state.inventory.quests.contains_key(id))?;
    state.inventory.quests.insert(
        id.clone(),
        Quest {
            id: id.clone(),
            campaign: campaign.clone(),
            revision,
            title: title.clone(),
            description: description.clone(),
            kind: *kind,
            state: QuestState::Planned,
            assignment: None,
            dependencies: Vec::new(),
        },
    );
    Ok(())
}

fn request_decision(
    state: &mut OwnedState,
    command: &Command,
    revision: u64,
) -> Result<(), CommandError> {
    let Action::RequestDecision {
        id,
        campaign,
        quest,
        incarnation,
        prompt,
        context,
    } = &command.action
    else {
        unreachable!("typed command dispatcher")
    };
    let s = &state.source;
    own(id.belongs_to(s))?;
    references(state, campaign, quest.as_ref())?;
    vacant(
        state.inventory.obligations.contains_key(id)
            || state
                .inventory
                .obligations
                .values()
                .any(|o| &o.incarnation == incarnation),
    )?;
    state.inventory.obligations.insert(
        id.clone(),
        Obligation {
            id: id.clone(),
            campaign: campaign.clone(),
            quest: quest.clone(),
            revision,
            incarnation: incarnation.clone(),
            kind: ObligationKind::Decision,
            responsible: Responsible::Human,
            state: ObligationState::Open,
            prompt: prompt.clone(),
            context: context.clone(),
            action: "Answer decision".into(),
            resolution: "Explicit human answer committed to the owned store".into(),
            requested_at: command.at,
        },
    );
    state.decisions.insert(
        id.clone(),
        DecisionDetail {
            requested_by: command.actor.clone(),
            answer: None,
        },
    );
    Ok(())
}

fn answer_decision(
    state: &mut OwnedState,
    command: &Command,
    revision: u64,
) -> Result<(), CommandError> {
    let Action::AnswerDecision {
        id,
        incarnation,
        answer,
    } = &command.action
    else {
        unreachable!("typed command dispatcher")
    };
    own(id.belongs_to(&state.source))?;
    let o = state
        .inventory
        .obligations
        .get_mut(id)
        .ok_or(CommandError::Reference)?;
    if &o.incarnation != incarnation {
        return Err(CommandError::Reference);
    }
    if o.kind != ObligationKind::Decision {
        return Err(CommandError::Reference);
    }
    if o.state != ObligationState::Open {
        return Err(CommandError::Conflict);
    }
    o.state = ObligationState::Resolved;
    o.revision = revision;
    state
        .decisions
        .get_mut(id)
        .ok_or(CommandError::Invalid)?
        .answer = Some(Answer {
        text: answer.clone(),
        actor: command.actor.clone(),
        at: command.at,
    });
    Ok(())
}

fn register_artifact(
    state: &mut OwnedState,
    command: &Command,
    revision: u64,
) -> Result<(), CommandError> {
    let Action::RegisterArtifact {
        id,
        campaign,
        quest,
        kind,
        title,
        uri,
    } = &command.action
    else {
        unreachable!("typed command dispatcher")
    };
    let s = &state.source;
    own(id.belongs_to(s))?;
    references(state, campaign, quest.as_ref())?;
    vacant(state.inventory.artifacts.contains_key(id))?;
    state.inventory.artifacts.insert(
        id.clone(),
        Artifact {
            id: id.clone(),
            campaign: campaign.clone(),
            quest: quest.clone(),
            revision,
            kind: *kind,
            title: title.clone(),
            uri: uri.clone(),
            head_revision: None,
            validation: None,
        },
    );
    Ok(())
}

fn own(valid: bool) -> Result<(), CommandError> {
    if valid {
        Ok(())
    } else {
        Err(CommandError::ForeignSource)
    }
}
fn vacant(present: bool) -> Result<(), CommandError> {
    if present {
        Err(CommandError::Conflict)
    } else {
        Ok(())
    }
}
fn campaign_exists(state: &OwnedState, id: &CampaignId) -> Result<(), CommandError> {
    if state.inventory.campaigns.contains_key(id) {
        Ok(())
    } else {
        Err(CommandError::Reference)
    }
}
fn references(
    state: &OwnedState,
    campaign: &CampaignId,
    quest: Option<&QuestId>,
) -> Result<(), CommandError> {
    own(campaign.belongs_to(&state.source) && quest.is_none_or(|q| q.belongs_to(&state.source)))?;
    campaign_exists(state, campaign)?;
    if quest.is_none_or(|q| {
        state
            .inventory
            .quests
            .get(q)
            .is_some_and(|q| &q.campaign == campaign)
    }) {
        Ok(())
    } else {
        Err(CommandError::Reference)
    }
}
fn owned_source(s: &SourceId) -> bool {
    s.valid() && s.0.strip_prefix("owned:").is_some_and(token)
}
fn token(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 256
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._:-".contains(&b))
}
fn text(s: &str) -> bool {
    !s.trim().is_empty() && s.len() <= TEXT_BYTES && !s.chars().any(|c| c.is_control() && c != '\n')
}
fn optional(s: Option<&str>) -> bool {
    s.is_none_or(text)
}
fn command_valid(c: &Command) -> bool {
    token(c.id.as_str())
        && text(&c.actor)
        && match &c.action {
            Action::CreateCampaign {
                name, objective, ..
            } => text(name) && optional(objective.as_deref()),
            Action::CreateQuest {
                title,
                description,
                kind,
                ..
            } => text(title) && optional(description.as_deref()) && *kind != QuestKind::Unspecified,
            Action::ChangeQuestLifecycle { .. } => true,
            Action::RequestDecision {
                incarnation,
                prompt,
                context,
                ..
            } => token(incarnation) && text(prompt) && optional(context.as_deref()),
            Action::AnswerDecision {
                incarnation,
                answer,
                ..
            } => token(incarnation) && text(answer),
            Action::RegisterArtifact { title, uri, .. } => text(title) && optional(uri.as_deref()),
            Action::RequestReview {
                incarnation,
                criterion,
                target,
                report,
                ..
            } => {
                token(incarnation)
                    && text(criterion)
                    && target.valid()
                    && report.as_ref().is_none_or(ReviewReport::valid)
            }
            Action::RecordCheck {
                incarnation,
                receipt,
                ..
            } => token(incarnation) && receipt.valid(),
            Action::SettleReview { incarnation, .. } => token(incarnation),
        }
}
