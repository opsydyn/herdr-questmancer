use crate::{
    domain::BodyProportions,
    ui::theatre::{TheatreFrame, TheatrePose},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SeatedPose {
    CrtFacing { hand_phase: bool },
    RaisedHand,
    Relaxed,
    Absent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SeatedLayout {
    pub head_x: u16,
    pub head_y: u16,
    pub head_width: u16,
    pub torso_x: u16,
    pub torso_y: u16,
    pub torso_width: u16,
    pub torso_height: u16,
}

impl SeatedLayout {
    pub(super) const fn for_proportions(proportions: BodyProportions) -> Self {
        match proportions {
            BodyProportions::Compact => Self {
                head_x: 2,
                head_y: 0,
                head_width: 5,
                torso_x: 3,
                torso_y: 4,
                torso_width: 4,
                torso_height: 4,
            },
            BodyProportions::Average | BodyProportions::Tall => Self {
                head_x: 3,
                head_y: 0,
                head_width: 4,
                torso_x: 3,
                torso_y: 4,
                torso_width: 4,
                torso_height: 5,
            },
            BodyProportions::Broad => Self {
                head_x: 2,
                head_y: 0,
                head_width: 6,
                torso_x: 1,
                torso_y: 4,
                torso_width: 8,
                torso_height: 4,
            },
        }
    }
}

pub(super) const fn seated_pose(frame: TheatreFrame) -> SeatedPose {
    match frame.pose {
        TheatrePose::Working => SeatedPose::CrtFacing {
            hand_phase: frame.animation_frame % 2 == 1,
        },
        TheatrePose::Blocked => SeatedPose::RaisedHand,
        TheatrePose::DoneUnseen
        | TheatrePose::DoneSeen
        | TheatrePose::Idle
        | TheatrePose::Unknown => SeatedPose::Relaxed,
        TheatrePose::Exited => SeatedPose::Absent,
    }
}
