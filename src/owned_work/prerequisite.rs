//! Prerequisite edits and graph integrity for the authoritative owned inventory.

use super::CommandError;
use crate::work::{Inventory, QuestId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// An explicit change to the directed link from a prerequisite to its dependent quest.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PrerequisiteChange {
    Add,
    Remove,
}

impl PrerequisiteChange {
    pub(crate) fn apply(
        self,
        inventory: &mut Inventory,
        quest: &QuestId,
        prerequisite_id: &QuestId,
        revision: u64,
    ) -> Result<(), CommandError> {
        let campaign = &inventory
            .quests
            .get(prerequisite_id)
            .ok_or(CommandError::Reference)?
            .campaign;
        let dependent = inventory.quests.get(quest).ok_or(CommandError::Reference)?;
        if quest == prerequisite_id {
            return Err(CommandError::PrerequisiteSelf);
        }
        if &dependent.campaign != campaign {
            return Err(CommandError::PrerequisiteCampaign);
        }
        let dependent = inventory
            .quests
            .get_mut(quest)
            .ok_or(CommandError::Reference)?;
        match (self, dependent.dependencies.binary_search(prerequisite_id)) {
            (Self::Add, Err(index)) => dependent
                .dependencies
                .insert(index, prerequisite_id.clone()),
            (Self::Remove, Ok(index)) => {
                dependent.dependencies.remove(index);
            }
            _ => return Err(CommandError::PrerequisitePresence),
        }
        dependent.revision = revision;
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum Visit {
    Visiting,
    Done,
}

/// Walk each edge once without depending on the call stack for deep graphs.
pub(super) fn validate(inventory: &Inventory) -> Result<(), CommandError> {
    let mut visited = BTreeMap::new();
    let mut stack = Vec::new();
    for root in inventory.quests.values() {
        stack.push((root, Visit::Visiting));
        while let Some((quest, visit)) = stack.pop() {
            if matches!(visit, Visit::Done) {
                visited.insert(&quest.id, Visit::Done);
                continue;
            }
            match visited.get(&quest.id) {
                Some(Visit::Done) => continue,
                Some(Visit::Visiting) => return Err(CommandError::PrerequisiteCycle),
                None => visited.insert(&quest.id, Visit::Visiting),
            };
            if quest.dependencies.windows(2).any(|pair| pair[0] >= pair[1]) {
                return Err(CommandError::Invalid);
            }
            stack.push((quest, Visit::Done));
            for id in &quest.dependencies {
                let prerequisite = inventory.quests.get(id).ok_or(CommandError::Reference)?;
                if quest.campaign != prerequisite.campaign {
                    return Err(CommandError::PrerequisiteCampaign);
                }
                stack.push((prerequisite, Visit::Visiting));
            }
        }
    }
    Ok(())
}
