//! Experimental, read-only snapshot proof for the pinned Herdr Projects layout.
//! Explicit opt-in inventory; no execution capability.
mod contract;
mod files;
mod mapping;
mod producer;
mod read;
pub(crate) use producer::fail as source_failure;
pub use producer::{Observation, Producer, RuntimeSelection};

pub use mapping::{Checkpoint, MappingError, Proposal, Registration, map_snapshot};

pub use contract::{
    Capabilities, Coherence, Coverage, Diagnostic, FORMAT, HomeReport, INSPECTED_REVISION,
    MAX_SNAPSHOT_BYTES, MAX_THREADS, Placement, ProjectRecord, ProjectStatus, ReadError,
    RecordedGroup, Selection, Snapshot, SupportState, ThreadRecord, ThreadStatus, decode_snapshot,
};
pub use read::capture;
