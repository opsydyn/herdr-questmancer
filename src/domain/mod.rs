mod agent;
mod attention;
mod guestbook;
mod ids;
mod persona;
mod site;
mod state;

pub use agent::{Agent, Presence};
pub use attention::{Attention, AttentionReason};
pub use guestbook::{Guestbook, GuestbookEntry, GuestbookEvent};
pub use ids::{AgentKey, EventId, PaneId, PersonaKey, TabId, Timestamp, WorkspaceId};
pub use persona::{
    AccentTone, Accessory, AgentPersona, BodyProportions, DeskProp, FaceDetail, HairShape,
    HairTone, HeadShape, OutfitBottom, OutfitTop, PersonaAppearance, Shoes, SkinTone,
};
pub use site::{Site, SiteStatus};
pub use state::DomainState;
