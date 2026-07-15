use questmancer::{
    app::{CharacterSet, ColorMode, DisplayPreferences, Model, Motion, View},
    domain::{Agent, DomainState, GuildAttention, GuildSummons, Presence, Timestamp},
    herdr::protocol::{SessionSnapshotResult, SuccessResponse},
    ui::theatre::{RenderCadence, TheatrePose, cadence_for, frame_for},
};
use std::time::Duration;

fn agent() -> Agent {
    let response: SuccessResponse<SessionSnapshotResult> =
        serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();
    DomainState::from_snapshot(&response.result.snapshot, Timestamp::from_millis(0))
        .agents
        .into_values()
        .next()
        .unwrap()
}

fn preferences() -> DisplayPreferences {
    DisplayPreferences {
        motion: Motion::Full,
        character_set: CharacterSet::Unicode,
        color_mode: ColorMode::Xterm256,
    }
}

fn frame_at(agent: &Agent, milliseconds: i64, motion: Motion) -> u8 {
    frame_for(
        agent,
        Timestamp::from_millis(milliseconds),
        &DisplayPreferences {
            motion,
            character_set: CharacterSet::Unicode,
            color_mode: ColorMode::Xterm256,
        },
    )
    .animation_frame
}

fn model_with(agent: Agent, now: i64, motion: Motion) -> Model {
    let mut domain = DomainState::default();
    domain.agents.insert(agent.key.clone(), agent);
    let mut model = Model::new(View::Delve);
    model.replace_domain(domain);
    model.set_now(Timestamp::from_millis(now));
    model.set_preferences(DisplayPreferences {
        motion,
        character_set: CharacterSet::Unicode,
        color_mode: ColorMode::Xterm256,
    });
    model
}

fn attention_variants() -> Vec<GuildAttention> {
    let since = Timestamp::from_millis(1_000);
    let until = Timestamp::from_millis(10_000);
    let reasons = [
        GuildSummons::CounselRequested,
        GuildSummons::SpoilsReturned,
        GuildSummons::AdventurerDeparted,
    ];
    let mut variants = vec![GuildAttention::Clear];
    for reason in reasons {
        variants.push(GuildAttention::Unread {
            summons: reason,
            since,
        });
        variants.push(GuildAttention::Read {
            summons: reason,
            since,
        });
        variants.push(GuildAttention::Deferred {
            summons: reason,
            since,
            until,
        });
    }
    variants
}

#[test]
fn presence_maps_to_explicit_theatre_poses_and_labels() {
    let cases = [
        (Presence::Working, TheatrePose::Delving, "DELVING"),
        (
            Presence::Blocked,
            TheatrePose::SeekingCounsel,
            "COUNSEL REQUESTED",
        ),
        (Presence::Idle, TheatrePose::Resting, "RESTING"),
        (Presence::Exited, TheatrePose::Departed, "DEPARTED"),
        (Presence::Unknown, TheatrePose::Unknown, "UNKNOWN"),
    ];

    for (presence, expected_pose, expected_label) in cases {
        let mut agent = agent();
        agent.presence = presence;
        agent.attention = GuildAttention::Clear;

        let frame = frame_for(&agent, Timestamp::from_millis(5_000), &preferences());

        assert_eq!(frame.pose, expected_pose);
        assert_eq!(frame.label, expected_label);
    }
}

#[test]
fn unseen_and_seen_done_attention_map_to_distinct_explicit_states() {
    let mut done = agent();
    done.presence = Presence::Done;
    done.attention =
        GuildAttention::unread(GuildSummons::SpoilsReturned, Timestamp::from_millis(1_000));

    let unseen = frame_for(&done, Timestamp::from_millis(1_500), &preferences());
    assert_eq!(unseen.pose, TheatrePose::SpoilsUnopened);
    assert_eq!(unseen.label, "SPOILS RETURNED");

    done.attention = done.attention.mark_read();
    let seen = frame_for(&done, Timestamp::from_millis(1_500), &preferences());
    assert_eq!(seen.pose, TheatrePose::VictoryRecorded);
    assert_eq!(seen.label, "VICTORY RECORDED");
}

#[test]
fn done_pose_and_cadence_follow_complete_attention_semantics() {
    for attention in attention_variants() {
        let completion_unseen = matches!(
            &attention,
            GuildAttention::Unread {
                summons: GuildSummons::SpoilsReturned,
                ..
            }
        );
        let mut done = agent();
        done.presence = Presence::Done;
        done.attention = attention;

        let frame = frame_for(&done, Timestamp::from_millis(1_500), &preferences());

        let expected_pose = if completion_unseen {
            TheatrePose::SpoilsUnopened
        } else {
            TheatrePose::VictoryRecorded
        };
        let expected_label = if completion_unseen {
            "SPOILS RETURNED"
        } else {
            "VICTORY RECORDED"
        };
        let expected_frame = if completion_unseen { 5 } else { 0 };
        let expected_cadence = if completion_unseen {
            RenderCadence::Fps(8)
        } else {
            RenderCadence::EventDriven
        };

        assert_eq!(frame.pose, expected_pose);
        assert_eq!(frame.label, expected_label);
        assert_eq!(frame.animation_frame, expected_frame);
        assert_eq!(
            cadence_for(&model_with(done, 1_500, Motion::Full)),
            expected_cadence
        );
    }
}

#[test]
fn non_done_presence_is_authoritative_for_every_attention_variant() {
    let cases = [
        (Presence::Working, TheatrePose::Delving, "DELVING"),
        (
            Presence::Blocked,
            TheatrePose::SeekingCounsel,
            "COUNSEL REQUESTED",
        ),
        (Presence::Idle, TheatrePose::Resting, "RESTING"),
        (Presence::Exited, TheatrePose::Departed, "DEPARTED"),
        (Presence::Unknown, TheatrePose::Unknown, "UNKNOWN"),
    ];

    for (presence, expected_pose, expected_label) in cases {
        for attention in attention_variants() {
            let mut agent = agent();
            agent.presence = presence;
            agent.attention = attention;

            let frame = frame_for(&agent, Timestamp::from_millis(5_000), &preferences());

            assert_eq!(frame.pose, expected_pose);
            assert_eq!(frame.label, expected_label);
        }
    }
}

#[test]
fn done_without_unseen_attention_is_stable_done_seen() {
    let mut done = agent();
    done.presence = Presence::Done;
    done.attention = GuildAttention::Clear;

    let frame = frame_for(&done, Timestamp::from_millis(1_500), &preferences());

    assert_eq!(frame.pose, TheatrePose::VictoryRecorded);
    assert_eq!(frame.label, "VICTORY RECORDED");
}

#[test]
fn focus_is_projected_without_replacing_pose_or_label() {
    let mut working = agent();
    working.presence = Presence::Working;
    working.focused = true;

    let frame = frame_for(&working, Timestamp::from_millis(1_500), &preferences());

    assert!(frame.focused);
    assert_eq!(frame.pose, TheatrePose::Delving);
    assert_eq!(frame.label, "DELVING");
}

#[test]
fn full_motion_frames_are_derived_from_presence_time() {
    let mut working = agent();
    working.presence = Presence::Working;
    working.presence_since = Timestamp::from_millis(1_000);
    assert_eq!(frame_at(&working, 1_000, Motion::Full), 0);
    assert_eq!(frame_at(&working, 1_167, Motion::Full), 1);
    assert_eq!(frame_at(&working, 1_667, Motion::Full), 0);

    working.presence = Presence::Blocked;
    assert_eq!(frame_at(&working, 1_499, Motion::Full), 0);
    assert_eq!(frame_at(&working, 1_500, Motion::Full), 1);
    assert_eq!(frame_at(&working, 2_000, Motion::Full), 0);

    working.presence = Presence::Idle;
    assert_eq!(frame_at(&working, 1_999, Motion::Full), 0);
    assert_eq!(frame_at(&working, 2_000, Motion::Full), 1);
}

#[test]
fn done_unseen_animation_uses_attention_time_and_ends_exactly_at_one_second() {
    let mut done = agent();
    done.presence = Presence::Done;
    done.presence_since = Timestamp::from_millis(10);
    done.attention =
        GuildAttention::unread(GuildSummons::SpoilsReturned, Timestamp::from_millis(2_000));
    let original_attention = done.attention.clone();

    assert_eq!(frame_at(&done, 1_999, Motion::Full), 0);
    assert_eq!(
        cadence_for(&model_with(done.clone(), 1_999, Motion::Full)),
        RenderCadence::EventDriven
    );
    assert_eq!(frame_at(&done, 2_000, Motion::Full), 1);
    assert_eq!(frame_at(&done, 2_125, Motion::Full), 2);
    assert_eq!(frame_at(&done, 2_875, Motion::Full), 8);
    assert_eq!(frame_at(&done, 2_999, Motion::Full), 8);
    assert_eq!(frame_at(&done, 3_000, Motion::Full), 0);
    assert_eq!(done.attention, original_attention);
}

#[test]
fn reduced_motion_freezes_rapid_effects_but_retains_slow_idle_frames() {
    let mut agent = agent();
    agent.presence_since = Timestamp::from_millis(0);

    for presence in [Presence::Working, Presence::Blocked] {
        agent.presence = presence;
        assert_eq!(frame_at(&agent, 750, Motion::Reduced), 0);
    }

    agent.presence = Presence::Done;
    agent.attention =
        GuildAttention::unread(GuildSummons::SpoilsReturned, Timestamp::from_millis(0));
    assert_eq!(frame_at(&agent, 875, Motion::Reduced), 0);

    agent.presence = Presence::Idle;
    assert_eq!(frame_at(&agent, 1_000, Motion::Reduced), 1);
}

#[test]
fn no_motion_freezes_every_animation_frame() {
    let mut agent = agent();
    agent.presence_since = Timestamp::from_millis(0);

    for presence in [
        Presence::Working,
        Presence::Blocked,
        Presence::Done,
        Presence::Idle,
        Presence::Exited,
        Presence::Unknown,
    ] {
        agent.presence = presence;
        if presence == Presence::Done {
            agent.attention =
                GuildAttention::unread(GuildSummons::SpoilsReturned, Timestamp::from_millis(0));
        }
        assert_eq!(frame_at(&agent, 10_000, Motion::None), 0);
    }
}

#[test]
fn full_motion_cadence_tracks_the_fastest_visible_animation() {
    let mut agent = agent();
    agent.presence_since = Timestamp::from_millis(0);

    agent.presence = Presence::Working;
    assert_eq!(
        cadence_for(&model_with(agent.clone(), 500, Motion::Full)),
        RenderCadence::Fps(6)
    );

    agent.presence = Presence::Blocked;
    assert_eq!(
        cadence_for(&model_with(agent.clone(), 500, Motion::Full)),
        RenderCadence::Fps(2)
    );

    agent.presence = Presence::Idle;
    assert_eq!(
        cadence_for(&model_with(agent.clone(), 500, Motion::Full)),
        RenderCadence::Fps(1)
    );

    agent.presence = Presence::Done;
    agent.attention =
        GuildAttention::unread(GuildSummons::SpoilsReturned, Timestamp::from_millis(0));
    assert_eq!(
        cadence_for(&model_with(agent.clone(), 999, Motion::Full)),
        RenderCadence::Fps(8)
    );
    assert_eq!(
        cadence_for(&model_with(agent, 1_000, Motion::Full)),
        RenderCadence::EventDriven
    );
}

#[test]
fn stable_states_are_event_driven() {
    let mut agent = agent();
    for presence in [Presence::Exited, Presence::Unknown] {
        agent.presence = presence;
        assert_eq!(
            cadence_for(&model_with(agent.clone(), 5_000, Motion::Full)),
            RenderCadence::EventDriven
        );
    }

    agent.presence = Presence::Done;
    agent.attention = GuildAttention::Clear;
    assert_eq!(
        cadence_for(&model_with(agent, 5_000, Motion::Full)),
        RenderCadence::EventDriven
    );
}

#[test]
fn reduced_and_no_motion_cadence_only_schedule_visible_changes() {
    let mut agent = agent();
    for presence in [Presence::Working, Presence::Blocked, Presence::Done] {
        agent.presence = presence;
        assert_eq!(
            cadence_for(&model_with(agent.clone(), 500, Motion::Reduced)),
            RenderCadence::EventDriven
        );
    }

    agent.presence = Presence::Idle;
    assert_eq!(
        cadence_for(&model_with(agent.clone(), 500, Motion::Reduced)),
        RenderCadence::Fps(1)
    );
    assert_eq!(
        cadence_for(&model_with(agent, 500, Motion::None)),
        RenderCadence::EventDriven
    );
}

#[test]
fn no_motion_never_requests_a_future_frame_even_during_completion_transition() {
    let mut done = agent();
    done.presence = Presence::Done;
    done.attention =
        GuildAttention::unread(GuildSummons::SpoilsReturned, Timestamp::from_millis(0));

    let model = model_with(done, 500, Motion::None);
    assert_eq!(cadence_for(&model), RenderCadence::EventDriven);
    assert_eq!(
        questmancer::ui::theatre::next_visible_frame_in(&model),
        None
    );
}

#[test]
fn cadence_is_event_driven_when_the_delve_theatre_is_not_visible_or_empty() {
    let mut working = agent();
    working.presence = Presence::Working;
    let mut desk = model_with(working, 500, Motion::Full);
    desk.switch_to(View::Guild);
    assert_eq!(cadence_for(&desk), RenderCadence::EventDriven);

    let empty_delve = Model::new(View::Delve);
    assert_eq!(cadence_for(&empty_delve), RenderCadence::EventDriven);
}

#[test]
fn mixed_delve_uses_the_fastest_visible_adventurer_cadence() {
    let mut working = agent();
    working.presence = Presence::Working;
    let mut done = working.clone();
    done.key = "agent-done".into();
    done.presence = Presence::Done;
    done.attention =
        GuildAttention::unread(GuildSummons::SpoilsReturned, Timestamp::from_millis(0));

    let mut domain = DomainState::default();
    domain.agents.insert(working.key.clone(), working);
    domain.agents.insert(done.key.clone(), done);
    let mut model = Model::new(View::Delve);
    model.replace_domain(domain);
    model.set_now(Timestamp::from_millis(500));

    assert_eq!(cadence_for(&model), RenderCadence::Fps(8));
}

#[test]
fn display_preferences_are_model_state_with_accessible_defaults() {
    let mut model = Model::new(View::Delve);
    assert_eq!(model.preferences(), &DisplayPreferences::default());

    let configured = DisplayPreferences {
        motion: Motion::Reduced,
        character_set: CharacterSet::Ascii,
        color_mode: ColorMode::Ansi16,
    };
    model.set_preferences(configured);
    assert_eq!(model.preferences(), &configured);
}

#[test]
fn next_visible_frame_delay_is_phase_aware_and_exact() {
    let mut working = agent();
    working.presence = Presence::Working;
    working.presence_since = Timestamp::from_millis(0);
    assert_eq!(
        questmancer::ui::theatre::next_visible_frame_in(&model_with(
            working.clone(),
            166,
            Motion::Full
        )),
        Some(Duration::from_millis(1))
    );
    assert_eq!(
        questmancer::ui::theatre::next_visible_frame_in(&model_with(working, 167, Motion::Full)),
        Some(Duration::from_millis(167))
    );

    let mut done = agent();
    done.presence = Presence::Done;
    done.attention =
        GuildAttention::unread(GuildSummons::SpoilsReturned, Timestamp::from_millis(0));
    assert_eq!(
        questmancer::ui::theatre::next_visible_frame_in(&model_with(done, 999, Motion::Full)),
        Some(Duration::from_millis(1))
    );
}
