use crate::{
    app::{DisplayPreferences, Model, Motion, View},
    domain::{Agent, GuildAttention, GuildSummons, Presence, Timestamp},
};

use ratatui::layout::Rect;
use std::time::Duration;

use super::{delve_projection::visible_agent_keys, views::guild_hall::next_elapsed_label_in};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TheatrePose {
    Delving,
    SeekingCounsel,
    SpoilsUnopened,
    VictoryRecorded,
    Resting,
    Departed,
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
        Presence::Working => (TheatrePose::Delving, "DELVING"),
        Presence::Blocked => (TheatrePose::SeekingCounsel, "COUNSEL REQUESTED"),
        Presence::Done if unseen_completion_since(&agent.attention).is_some() => {
            (TheatrePose::SpoilsUnopened, "SPOILS RETURNED")
        }
        Presence::Done => (TheatrePose::VictoryRecorded, "VICTORY RECORDED"),
        Presence::Idle => (TheatrePose::Resting, "RESTING"),
        Presence::Exited => (TheatrePose::Departed, "DEPARTED"),
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
    if model.view() == View::Guild {
        return if model.preferences().motion == Motion::Full
            && model.goblins().is_visible(model.now())
        {
            RenderCadence::Fps(4)
        } else {
            RenderCadence::EventDriven
        };
    }

    let fps = model
        .domain()
        .agents
        .values()
        .filter_map(|agent| cadence_for_agent(agent, model.now(), model.preferences().motion))
        .max();

    fps.map_or(RenderCadence::EventDriven, RenderCadence::Fps)
}

/// Returns the phase-aware delay until any visible Delve animation changes.
///
/// This is deliberately derived from every agent rather than from the highest
/// nominal FPS. Different agents can have interleaved boundaries, and a done
/// transition has an exact terminal boundary at one second.
pub fn next_visible_frame_in(model: &Model, render_area: Rect) -> Option<Duration> {
    next_projected_frame_in(model, render_area, Some(model.preferences().motion))
}

pub(crate) fn next_projected_frame_in(
    model: &Model,
    render_area: Rect,
    guild_goblin_motion: Option<Motion>,
) -> Option<Duration> {
    if model.view() == View::Guild {
        let goblins = guild_goblin_motion
            .and_then(|motion| model.goblins().next_visible_frame_in(model.now(), motion));
        let elapsed = next_elapsed_label_in(model, render_area);
        return goblins.into_iter().chain(elapsed).min();
    }
    if model.preferences().motion == Motion::None {
        return None;
    }

    visible_agent_keys(model, render_area)
        .into_iter()
        .filter_map(|key| model.domain().agents.get(&key))
        .filter_map(|agent| next_frame_for_agent(agent, model.now(), model.preferences().motion))
        .min()
}

fn animation_frame(agent: &Agent, pose: TheatrePose, now: Timestamp, motion: Motion) -> u8 {
    match motion {
        Motion::None => 0,
        Motion::Reduced => match pose {
            TheatrePose::Resting => looping_frame(agent.presence_since, now, 1, 4),
            TheatrePose::Delving
            | TheatrePose::SeekingCounsel
            | TheatrePose::SpoilsUnopened
            | TheatrePose::VictoryRecorded
            | TheatrePose::Departed
            | TheatrePose::Unknown => 0,
        },
        Motion::Full => match pose {
            TheatrePose::Delving => looping_frame(agent.presence_since, now, 6, 4),
            TheatrePose::SeekingCounsel => looping_frame(agent.presence_since, now, 2, 2),
            TheatrePose::SpoilsUnopened => done_transition_frame(agent, now),
            TheatrePose::Resting => looping_frame(agent.presence_since, now, 1, 4),
            TheatrePose::VictoryRecorded | TheatrePose::Departed | TheatrePose::Unknown => 0,
        },
    }
}

fn done_transition_frame(agent: &Agent, now: Timestamp) -> u8 {
    unseen_completion_since(&agent.attention).map_or(0, |since| {
        let elapsed = since.elapsed_until(now);
        if now >= since && elapsed < Duration::from_secs(1) {
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

fn next_frame_for_agent(agent: &Agent, now: Timestamp, motion: Motion) -> Option<Duration> {
    match motion {
        Motion::None => None,
        Motion::Reduced => (agent.presence == Presence::Idle)
            .then(|| next_loop_boundary(agent.presence_since, now, 1)),
        Motion::Full => match agent.presence {
            Presence::Working => Some(next_loop_boundary(agent.presence_since, now, 6)),
            Presence::Blocked => Some(next_loop_boundary(agent.presence_since, now, 2)),
            Presence::Done => next_done_boundary(agent, now),
            Presence::Idle => Some(next_loop_boundary(agent.presence_since, now, 1)),
            Presence::Exited | Presence::Unknown => None,
        },
    }
}

fn next_loop_boundary(since: Timestamp, now: Timestamp, fps: u8) -> Duration {
    if now < since {
        return now.elapsed_until(since) + first_frame_delay(fps);
    }

    next_step_delay(since.elapsed_until(now), fps)
}

fn next_done_boundary(agent: &Agent, now: Timestamp) -> Option<Duration> {
    let since = unseen_completion_since(&agent.attention)?;
    if now < since {
        return None;
    }
    let elapsed = since.elapsed_until(now);
    (elapsed < Duration::from_secs(1)).then(|| next_step_delay(elapsed, 8))
}

fn first_frame_delay(fps: u8) -> Duration {
    Duration::from_millis(1_000_u64.div_ceil(u64::from(fps)))
}

fn next_step_delay(elapsed: Duration, fps: u8) -> Duration {
    let elapsed_millis = elapsed.as_millis();
    let completed_steps = elapsed_millis * u128::from(fps) / 1_000;
    let next_boundary = ((completed_steps + 1) * 1_000).div_ceil(u128::from(fps));
    let delay = next_boundary.saturating_sub(elapsed_millis).max(1);
    Duration::from_millis(u64::try_from(delay).unwrap_or(u64::MAX))
}

fn done_transition_is_active(agent: &Agent, now: Timestamp) -> bool {
    unseen_completion_since(&agent.attention)
        .is_some_and(|since| now >= since && since.elapsed_until(now) < Duration::from_secs(1))
}

fn unseen_completion_since(attention: &GuildAttention) -> Option<Timestamp> {
    match attention {
        GuildAttention::Unread {
            summons: GuildSummons::SpoilsReturned,
            since,
        } => Some(*since),
        GuildAttention::Clear
        | GuildAttention::Unread { .. }
        | GuildAttention::Read { .. }
        | GuildAttention::Deferred { .. } => None,
    }
}
