//! Deterministic synthetic source for tests and Storybook; never installed at startup.
use super::{
    Artifact, ArtifactId, ArtifactKind, Assignment, CampaignId, CampaignState, Change, Inventory,
    Message, Obligation, ObligationId, ObligationKind, ObligationState, Quest, QuestId, QuestKind,
    QuestState, Responsible, SourceId, WorkCampaign,
};
use crate::domain::Timestamp;
use std::collections::BTreeMap;
#[derive(Debug)]
pub struct FixtureSource;
impl FixtureSource {
    pub fn authentication(now: Timestamp) -> Vec<Message> {
        let source = SourceId::new("fixture-auth");
        let campaign = CampaignId::new(&source, "auth");
        let mut inventory = Inventory::default();
        inventory.campaigns.insert(
            campaign.clone(),
            WorkCampaign {
                id: campaign.clone(),
                revision: 1,
                name: "Replace Authentication Layer".into(),
                objective: Some("Migrate authentication while preserving compatibility.".into()),
                state: CampaignState::Active,
            },
        );
        add_quests(&mut inventory, &source, &campaign);
        let id = ObligationId::new(&source, "compatibility");
        inventory.obligations.insert(
            id.clone(),
            Obligation {
                id,
                campaign: campaign.clone(),
                quest: Some(QuestId::new(&source, "web")),
                revision: 1,
                incarnation: "compatibility-1".into(),
                kind: ObligationKind::Decision,
                responsible: Responsible::Human,
                state: ObligationState::Open,
                prompt: "Should the old token endpoint remain for backwards compatibility?".into(),
                context: Some(
                    "The web migration continues; endpoint compatibility needs your decision."
                        .into(),
                ),
                action: "Answer through the source; Questmancer can only mark seen or defer."
                    .into(),
                resolution: "A matching source resolution of compatibility-1.".into(),
                requested_at: now,
            },
        );
        let id = ArtifactId::new(&source, "pr-481");
        inventory.artifacts.insert(
            id.clone(),
            Artifact {
                id,
                campaign,
                quest: Some(QuestId::new(&source, "api")),
                revision: 1,
                kind: ArtifactKind::PullRequest,
                title: "PR #481 (illustrative)".into(),
                uri: Some("https://example.invalid/pull/481".into()),
                head_revision: None,
                validation: None,
            },
        );
        let mut api = inventory.quests[&QuestId::new(&source, "api")].clone();
        api.revision = 2;
        api.description = Some("Migration implementation continues.".into());
        vec![
            Message::Baseline {
                source: source.clone(),
                generation: 1,
                captured_at: now,
                cursor: 0,
                inventory,
                complete: true,
                bindings: BTreeMap::new(),
            },
            Message::Update {
                source,
                generation: 1,
                occurred_at: now,
                cursor: 1,
                event: "api-progress".into(),
                changes: vec![Change::Quest(api)],
            },
        ]
    }
    pub fn resolution(occurred_at: Timestamp) -> Message {
        let source = SourceId::new("fixture-auth");
        Message::Update {
            source: source.clone(),
            generation: 1,
            occurred_at,
            cursor: 2,
            event: "compatibility-resolved".into(),
            changes: vec![Change::Resolve {
                id: ObligationId::new(&source, "compatibility"),
                incarnation: "compatibility-1".into(),
                revision: 2,
            }],
        }
    }
}

fn add_quests(inventory: &mut Inventory, source: &SourceId, campaign: &CampaignId) {
    for (local, title, kind, state, worker) in [
        (
            "scout",
            "Investigate current auth",
            QuestKind::Scout,
            QuestState::Completed,
            None,
        ),
        (
            "api",
            "Migrate API auth",
            QuestKind::Ship,
            QuestState::Active,
            Some("Ada"),
        ),
        (
            "web",
            "Migrate web auth",
            QuestKind::Ship,
            QuestState::Active,
            Some("Linus"),
        ),
        (
            "tests",
            "Integration tests",
            QuestKind::Validate,
            QuestState::Planned,
            None,
        ),
    ] {
        let id = QuestId::new(source, local);
        inventory.quests.insert(
            id.clone(),
            Quest {
                id,
                campaign: campaign.clone(),
                revision: 1,
                title: title.into(),
                description: Some("Synthetic work supplied by FixtureSource.".into()),
                kind,
                state,
                assignment: worker.map(|name| Assignment {
                    worker: format!("fixture:{name}"),
                    name: name.into(),
                }),
                dependencies: if local == "tests" {
                    vec![QuestId::new(source, "api"), QuestId::new(source, "web")]
                } else {
                    vec![]
                },
            },
        );
    }
}
