use crate::{
    app::{DisplayPreferences, Model, Motion, View},
    domain::{Agent, Presence, Timestamp},
};

use std::time::Duration;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TheatrePose {
    Working,
    Blocked,
    DoneUnseen,
    DoneSeen,
    Idle,
    Exited,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TheatreFrame {
    pub pose: TheatrePose,
    pub animation_frame: u8,
    pub focused: bool,
    pub label: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderCadence {
    EventDriven,
    Fps(u8),
}

pub fn frame_for(agent: &Agent, now: Timestamp, preferences: &DisplayPreferences) -> TheatreFrame {
    let (pose, label) = match agent.presence {
        Presence::Working => (TheatrePose::Working, "BUILDING"),
        Presence::Blocked => (TheatrePose::Blocked, "HELP!"),
        Presence::Done if agent.attention.is_unseen() => (TheatrePose::DoneUnseen, "UPDATE READY"),
        Presence::Done => (TheatrePose::DoneSeen, "DONE"),
        Presence::Idle => (TheatrePose::Idle, "IDLE"),
        Presence::Exited => (TheatrePose::Exited, "BROKEN LINK"),
        Presence::Unknown => (TheatrePose::Unknown, "UNKNOWN"),
    };

    TheatreFrame {
        pose,
        animation_frame: animation_frame(agent, pose, now, preferences.motion),
        focused: agent.focused,
        label,
    }
}

pub fn cadence_for(model: &Model) -> RenderCadence {
    if model.view() != View::Cafe {
        return RenderCadence::EventDriven;
    }

    let fps = model
        .domain()
        .agents
        .values()
        .filter_map(|agent| cadence_for_agent(agent, model.now(), model.preferences().motion))
        .max();

    fps.map_or(RenderCadence::EventDriven, RenderCadence::Fps)
}

fn animation_frame(agent: &Agent, pose: TheatrePose, now: Timestamp, motion: Motion) -> u8 {
    match motion {
        Motion::None => 0,
        Motion::Reduced => match pose {
            TheatrePose::Idle => looping_frame(agent.presence_since, now, 1, 4),
            TheatrePose::Working
            | TheatrePose::Blocked
            | TheatrePose::DoneUnseen
            | TheatrePose::DoneSeen
            | TheatrePose::Exited
            | TheatrePose::Unknown => 0,
        },
        Motion::Full => match pose {
            TheatrePose::Working => looping_frame(agent.presence_since, now, 6, 4),
            TheatrePose::Blocked => looping_frame(agent.presence_since, now, 2, 2),
            TheatrePose::DoneUnseen => done_transition_frame(agent, now),
            TheatrePose::Idle => looping_frame(agent.presence_since, now, 1, 4),
            TheatrePose::DoneSeen | TheatrePose::Exited | TheatrePose::Unknown => 0,
        },
    }
}

fn done_transition_frame(agent: &Agent, now: Timestamp) -> u8 {
    agent.attention.since().map_or(0, |since| {
        let elapsed = since.elapsed_until(now);
        if elapsed < Duration::from_secs(1) {
            frame_from_elapsed(elapsed, 8, 8) + 1
        } else {
            0
        }
    })
}

fn looping_frame(since: Timestamp, now: Timestamp, fps: u8, frame_count: u8) -> u8 {
    frame_from_elapsed(since.elapsed_until(now), fps, frame_count)
}

fn frame_from_elapsed(elapsed: Duration, fps: u8, frame_count: u8) -> u8 {
    let frame = elapsed.as_millis() * u128::from(fps) / 1_000 % u128::from(frame_count);
    u8::try_from(frame).unwrap_or_default()
}

fn cadence_for_agent(agent: &Agent, now: Timestamp, motion: Motion) -> Option<u8> {
    match motion {
        Motion::None => None,
        Motion::Reduced => (agent.presence == Presence::Idle).then_some(1),
        Motion::Full => match agent.presence {
            Presence::Working => Some(6),
            Presence::Blocked => Some(2),
            Presence::Done if done_transition_is_active(agent, now) => Some(8),
            Presence::Idle => Some(1),
            Presence::Done | Presence::Exited | Presence::Unknown => None,
        },
    }
}

fn done_transition_is_active(agent: &Agent, now: Timestamp) -> bool {
    agent.attention.is_unseen()
        && agent
            .attention
            .since()
            .is_some_and(|since| since.elapsed_until(now) < Duration::from_secs(1))
}
