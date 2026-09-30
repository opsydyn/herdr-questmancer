//! I/O boundary: validate/reduce, acknowledge publication, then expose facts.
use super::{Message, source::Envelope};
use crate::{app::Model, persistence::PersistenceClient};

#[derive(Debug, thiserror::Error)]
pub enum ObservationError {
    #[error(transparent)]
    Mapping(#[from] crate::sources::herdr_projects::MappingError),
    #[error("{0}")]
    Publication(String),
}
impl ObservationError {
    /// The source producer must stop until the owner reconciles registration.
    pub fn requires_rebind(&self) -> bool {
        matches!(self, Self::Mapping(error) if error.requires_rebind())
    }
}

/// Consumer mapping boundary; attachment leases and scheduling belong to the producer.
pub async fn accept_snapshot(
    client: &PersistenceClient,
    model: &mut Model,
    registration: &crate::sources::herdr_projects::Registration,
    generation: u64,
    snapshot: &crate::sources::herdr_projects::Snapshot,
    checkpoint: Option<&crate::sources::herdr_projects::Checkpoint>,
) -> Result<crate::sources::herdr_projects::Checkpoint, ObservationError> {
    use crate::sources::herdr_projects::{MappingError, map_snapshot};
    match map_snapshot(registration, generation, snapshot, model.work(), checkpoint) {
        Ok(Some(proposal)) => accept_proposal(client, model, proposal)
            .await
            .map_err(ObservationError::Publication),
        Ok(None) => checkpoint
            .cloned()
            .ok_or_else(|| MappingError::Invalid.into()),
        Err(error) => {
            if error == MappingError::Stale {
                return Err(error.into());
            }
            let mut retained = model.work().clone();
            if error == MappingError::Unlocked {
                retained = super::reduce(
                    retained,
                    Message::Disconnected(registration.source().clone()),
                )
                .expect("disconnection cannot fail");
            } else {
                retained.reconcile(registration.source());
            }
            model.replace_work(retained);
            model.set_action_feedback(format!("Work source observation rejected: {error}"));
            Err(error.into())
        }
    }
}

/// Expose facts only after the projection/checkpoint pair is durable.
pub async fn accept_proposal(
    client: &PersistenceClient,
    model: &mut Model,
    proposal: crate::sources::herdr_projects::Proposal,
) -> Result<crate::sources::herdr_projects::Checkpoint, String> {
    if proposal.base != *model.work() {
        return Err("obsolete work proposal discarded".into());
    }
    let source = proposal.source().clone();
    let result = client
        .publish_work_proposal(proposal.clone())
        .await
        .map_err(|e| e.to_string());
    if let Err(error) = result {
        let mut retained = model.work().clone();
        retained.reconcile(&source);
        model.replace_work(retained);
        model.set_action_feedback(
            "Work observation was not acknowledged; source reconciliation required.".into(),
        );
        return Err(error);
    }
    model.replace_work(proposal.projection);
    Ok(proposal.checkpoint)
}
pub async fn accept(
    client: &PersistenceClient,
    model: &mut Model,
    envelope: Envelope,
) -> Result<(), String> {
    let source = match &envelope.message {
        Message::Baseline { source, .. }
        | Message::Update { source, .. }
        | Message::Disconnected(source)
        | Message::BindingRemoved { source, .. } => source.clone(),
    };
    let next = if envelope.version == 1 {
        super::reduce(model.work().clone(), envelope.message)
    } else {
        Err(super::WorkError::Invalid)
    };
    let next = match next {
        Ok(next) => next,
        Err(error) => {
            let mut retained = model.work().clone();
            retained.reconcile(&source);
            model.replace_work(retained);
            model.set_action_feedback(format!("Work source requires a baseline: {error}"));
            return Err(error.to_string());
        }
    };
    if next == *model.work() {
        return Ok(());
    }
    if let Err(error) = client.publish_work(next.clone()).await {
        let mut retained = model.work().clone();
        retained.reconcile(&source);
        model.replace_work(retained);
        model.set_action_feedback(
            "Work update was not acknowledged by storage; source reconciliation required.".into(),
        );
        return Err(error.to_string());
    }
    model.replace_work(next);
    Ok(())
}
