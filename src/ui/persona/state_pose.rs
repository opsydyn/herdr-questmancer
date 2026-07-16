use crate::domain::{Ancestry, BodyProportions};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum BodyPose {
    Chamber { compact: bool, broad: bool },
    Profile { compact: bool, broad: bool },
}

impl BodyPose {
    pub(super) const fn chamber(ancestry: Ancestry, proportions: BodyProportions) -> Self {
        Self::Chamber {
            compact: is_compact(ancestry) || matches!(proportions, BodyProportions::Compact),
            broad: matches!(proportions, BodyProportions::Broad),
        }
    }

    pub(super) const fn profile(ancestry: Ancestry, proportions: BodyProportions) -> Self {
        Self::Profile {
            compact: is_compact(ancestry) || matches!(proportions, BodyProportions::Compact),
            broad: matches!(proportions, BodyProportions::Broad),
        }
    }

    pub(super) const fn compact(self) -> bool {
        match self {
            Self::Chamber { compact, .. } | Self::Profile { compact, .. } => compact,
        }
    }

    pub(super) const fn broad(self) -> bool {
        match self {
            Self::Chamber { broad, .. } | Self::Profile { broad, .. } => broad,
        }
    }
}

const fn is_compact(ancestry: Ancestry) -> bool {
    matches!(
        ancestry,
        Ancestry::Dwarf | Ancestry::Halfling | Ancestry::Gnome | Ancestry::Goblin
    )
}
