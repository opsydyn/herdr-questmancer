use super::Command;
use crate::{
    domain::{
        CapturedObservation, ChronicleEntry, DomainState, ObservationEvidence, ObservationSubject,
        ObservedPresence, Presence, RemovalEvidence, SnapshotObservation, Timestamp,
    },
    snapshot_refresh::SnapshotRequest,
};

pub(super) fn append_observation(
    state: &mut DomainState,
    subject: ObservationSubject,
    evidence: ObservationEvidence,
    at: Timestamp,
) -> Option<Command> {
    let chronicle = &mut state.chronicle;
    state.capture.accept(|stamp| {
        let entry = ChronicleEntry::observed(
            at,
            CapturedObservation {
                stamp,
                subject,
                evidence,
            },
        )?;
        chronicle
            .append(entry.clone())
            .then_some(Command::AppendChronicle(entry))
    })
}

pub(super) fn capture_snapshot(
    previous: &DomainState,
    replacement: &mut DomainState,
    request: SnapshotRequest,
    at: Timestamp,
) -> Vec<Command> {
    if !replacement.capture.qualifies(request.epoch) {
        return Vec::new();
    }
    // Ordered maps make each accepted batch deterministic. Campaign absence
    // absorbs membership losses caused solely by that same missing campaign.
    let mut observations = Vec::new();
    for (id, campaign) in &previous.campaigns {
        if !replacement.campaigns.contains_key(id) {
            let evidence = if previous.capture.close_was_hinted(id) {
                RemovalEvidence::CloseHintConfirmed
            } else {
                RemovalEvidence::NoLongerVisible
            };
            observations.push((
                ObservationSubject::campaign(campaign),
                SnapshotObservation::CampaignRemoved { evidence },
            ));
        }
    }
    for (key, agent) in &replacement.agents {
        let Some(subject) = ObservationSubject::adventurer(agent) else {
            continue;
        };
        let Some(presence) = ObservedPresence::from_presence(agent.presence) else {
            continue;
        };
        match previous.agents.get(key) {
            None => observations.push((
                subject,
                SnapshotObservation::AdventurerObserved { presence },
            )),
            Some(old)
                if old
                    .capture_identity
                    .same_incarnation(&agent.capture_identity)
                    && old.presence != agent.presence =>
            {
                observations.push((subject, SnapshotObservation::PresenceObserved { presence }));
            }
            // A changed/insufficient incarnation establishes a quiet subject baseline.
            Some(_) => {}
        }
    }
    for (key, agent) in &previous.agents {
        if !replacement.agents.contains_key(key)
            && replacement.campaigns.contains_key(&agent.workspace_id)
            && agent.presence != Presence::Exited
            && !replacement.agents.values().any(|candidate| {
                candidate.pane_id == agent.pane_id && !candidate.capture_identity.is_qualified()
            })
            && let Some(subject) = ObservationSubject::adventurer(agent)
        {
            observations.push((subject, SnapshotObservation::AdventurerNoLongerVisible));
        }
    }
    observations
        .into_iter()
        .filter_map(|(subject, observation)| {
            append_observation(
                replacement,
                subject,
                ObservationEvidence::Snapshot {
                    request,
                    observation,
                },
                at,
            )
        })
        .collect()
}
