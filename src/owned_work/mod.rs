//! Explicit human commands own this source. Work projections and Chronicle do not.
mod review;
mod review_git;
mod state;
mod store;
pub use review::{
    CheckReceipt, CheckResult, ReportAuthor, Review, ReviewReport, ReviewSettlement, ReviewTarget,
};
pub use state::{
    Action, Answer, Command, CommandError, CommandId, DecisionDetail, OwnedState,
    QuestLifecycleChange, Receipt, Transition, transition,
};
pub use store::{OwnedStore, StoreError};

/// Supply opaque identities at the I/O boundary; pure transitions never generate them.
pub fn random_token() -> Result<String, getrandom::Error> {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut bytes = [0_u8; 16];
    getrandom::fill(&mut bytes)?;
    Ok(bytes
        .iter()
        .flat_map(|b| {
            [
                char::from(HEX[usize::from(b >> 4)]),
                char::from(HEX[usize::from(b & 15)]),
            ]
        })
        .collect())
}
pub fn random_source() -> Result<crate::work::SourceId, getrandom::Error> {
    Ok(crate::work::SourceId::new(&format!(
        "owned:{}",
        random_token()?
    )))
}

mod selection;
pub use selection::StoreSelection;
pub mod authoring;
mod runtime;
pub use runtime::OwnedRuntime;
