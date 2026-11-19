//! Source-owned work facts beside the live Herdr domain.
mod types;
pub use types::*;

pub(crate) mod association;
mod attention;
mod reducer;
pub use attention::{AttentionTarget, WorkSelection, bound_agent, queue};
pub use reducer::reduce;
pub mod fixture;
pub mod graph;
pub(crate) mod inbox;
pub mod runtime;
pub mod source;
pub mod view;
