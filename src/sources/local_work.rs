//! Complete owned snapshots enter the existing acknowledged work boundary.
use crate::{
    domain::Timestamp,
    owned_work::OwnedState,
    work::{Message, Projection, SourceId, source::Envelope},
};
use std::collections::BTreeMap;

#[derive(Debug, thiserror::Error)]
pub enum LocalSourceError {
    #[error("owned source or snapshot is invalid")]
    Invalid,
    #[error("owned store is older than its accepted projection; reconcile selection")]
    OlderStore,
    #[error("owned observer generation exhausted")]
    Exhausted,
}
#[derive(Debug)]
pub struct LocalWorkSource {
    source: SourceId,
    generation: u64,
    minimum_revision: u64,
}
impl LocalWorkSource {
    pub fn new(state: &OwnedState, projection: &Projection) -> Result<Self, LocalSourceError> {
        state.validate().map_err(|_| LocalSourceError::Invalid)?;
        Self::from_inventory(
            state.source(),
            state.revision(),
            state.inventory(),
            projection,
        )
    }
    pub(crate) fn from_inventory(
        source: &SourceId,
        revision: u64,
        inventory: &crate::work::Inventory,
        projection: &Projection,
    ) -> Result<Self, LocalSourceError> {
        let previous = projection.sources.get(source);
        if previous.is_some_and(|p| p.cursor > revision) {
            return Err(LocalSourceError::OlderStore);
        }
        if previous.is_some_and(|p| p.cursor == revision && p.inventory != *inventory) {
            return Err(LocalSourceError::Invalid);
        }
        let generation = previous
            .map_or(0, |p| p.generation)
            .checked_add(1)
            .ok_or(LocalSourceError::Exhausted)?;
        Ok(Self {
            source: source.clone(),
            generation,
            minimum_revision: revision,
        })
    }
    /// No output, topology, intent, answer text or live bindings leak into the cache.
    pub fn baseline(
        &self,
        state: &OwnedState,
        at: Timestamp,
    ) -> Result<Envelope, LocalSourceError> {
        state.validate().map_err(|_| LocalSourceError::Invalid)?;
        if state.source() != &self.source {
            return Err(LocalSourceError::Invalid);
        }
        if state.revision() < self.minimum_revision {
            return Err(LocalSourceError::OlderStore);
        }
        self.inventory_baseline(state.inventory(), state.revision(), at)
    }
    pub(crate) fn inventory_baseline(
        &self,
        inventory: &crate::work::Inventory,
        revision: u64,
        at: Timestamp,
    ) -> Result<Envelope, LocalSourceError> {
        if revision < self.minimum_revision {
            return Err(LocalSourceError::OlderStore);
        }
        Ok(Envelope {
            version: 1,
            message: Message::Baseline {
                source: self.source.clone(),
                generation: self.generation,
                captured_at: at,
                cursor: revision,
                inventory: inventory.clone(),
                complete: true,
                bindings: BTreeMap::new(),
            },
        })
    }
}
