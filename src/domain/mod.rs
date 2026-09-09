mod agent;
mod attention;
mod campaign;
mod capture;
mod chronicle;
mod chronicle_observation;
mod ids;
mod persona;
mod state;

pub use agent::{Agent, Presence, Urgency};
pub use attention::{GuildAttention, GuildSummons};
pub use campaign::{Campaign, CampaignStatus};
pub use capture::{CaptureClock, CaptureIdentity, CaptureRunId, ObservationStamp};
pub use chronicle::{Chronicle, ChronicleEntry, ChronicleEvent, LegacyChronicleEntry};
pub use chronicle_observation::{
    CapturedObservation, ObservationEvidence, ObservationSubject, ObservedPresence,
    RemovalEvidence, SnapshotObservation,
};
pub use ids::{AgentKey, EventId, PaneId, PersonaKey, TabId, Timestamp, WorkspaceId};
pub use persona::{
    AccentTone, AdventurerClass, AdventurerPersona, AdventuringGear, Ancestry, BodyProportions,
    Epithet, FaceDetail, Footwear, Garb, HairShape, HairTone, HeadShape, Keepsake, Legwear,
    PersonaAppearance, PersonaGeneration, SkinTone,
};
pub use state::DomainState;
