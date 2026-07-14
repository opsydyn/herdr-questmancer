use herdr_webmaster::{
    app::{CharacterSet, DisplayPreferences, Model, Motion, View},
    domain::{Agent, Attention, AttentionReason, DomainState, Presence, Timestamp},
    herdr::protocol::{SessionSnapshotResult, SuccessResponse},
    ui::theatre::{RenderCadence, TheatrePose, cadence_for, frame_for},
};

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
    }
}

fn frame_at(agent: &Agent, milliseconds: i64, motion: Motion) -> u8 {
    frame_for(
        agent,
        Timestamp::from_millis(milliseconds),
        &DisplayPreferences {
            motion,
            character_set: CharacterSet::Unicode,
        },
    )
    .animation_frame
}

fn model_with(agent: Agent, now: i64, motion: Motion) -> Model {
    let mut domain = DomainState::default();
    domain.agents.insert(agent.key.clone(), agent);
    let mut model = Model::new(View::Cafe);
    model.replace_domain(domain);
    model.set_now(Timestamp::from_millis(now));
    model.set_preferences(DisplayPreferences {
        motion,
        character_set: CharacterSet::Unicode,
    });
    model
}

#[test]
fn presence_maps_to_explicit_theatre_poses_and_labels() {
    let cases = [
        (Presence::Working, TheatrePose::Working, "BUILDING"),
        (Presence::Blocked, TheatrePose::Blocked, "HELP!"),
        (Presence::Idle, TheatrePose::Idle, "IDLE"),
        (Presence::Exited, TheatrePose::Exited, "BROKEN LINK"),
        (Presence::Unknown, TheatrePose::Unknown, "UNKNOWN"),
    ];

    for (presence, expected_pose, expected_label) in cases {
        let mut agent = agent();
        agent.presence = presence;
        agent.attention = Attention::Clear;

        let frame = frame_for(&agent, Timestamp::from_millis(5_000), &preferences());

        assert_eq!(frame.pose, expected_pose);
        assert_eq!(frame.label, expected_label);
    }
}

#[test]
fn unseen_and_seen_done_attention_map_to_distinct_explicit_states() {
    let mut done = agent();
    done.presence = Presence::Done;
    done.attention = Attention::unseen(
        AttentionReason::WorkCompleted,
        Timestamp::from_millis(1_000),
    );

    let unseen = frame_for(&done, Timestamp::from_millis(1_500), &preferences());
    assert_eq!(unseen.pose, TheatrePose::DoneUnseen);
    assert_eq!(unseen.label, "UPDATE READY");

    done.attention = done.attention.mark_seen();
    let seen = frame_for(&done, Timestamp::from_millis(1_500), &preferences());
    assert_eq!(seen.pose, TheatrePose::DoneSeen);
    assert_eq!(seen.label, "DONE");
}

#[test]
fn done_without_unseen_attention_is_stable_done_seen() {
    let mut done = agent();
    done.presence = Presence::Done;
    done.attention = Attention::Clear;

    let frame = frame_for(&done, Timestamp::from_millis(1_500), &preferences());

    assert_eq!(frame.pose, TheatrePose::DoneSeen);
    assert_eq!(frame.label, "DONE");
}

#[test]
fn focus_is_projected_without_replacing_pose_or_label() {
    let mut working = agent();
    working.presence = Presence::Working;
    working.focused = true;

    let frame = frame_for(&working, Timestamp::from_millis(1_500), &preferences());

    assert!(frame.focused);
    assert_eq!(frame.pose, TheatrePose::Working);
    assert_eq!(frame.label, "BUILDING");
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
    done.attention = Attention::unseen(
        AttentionReason::WorkCompleted,
        Timestamp::from_millis(2_000),
    );
    let original_attention = done.attention.clone();

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
    agent.attention = Attention::unseen(AttentionReason::WorkCompleted, Timestamp::from_millis(0));
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
                Attention::unseen(AttentionReason::WorkCompleted, Timestamp::from_millis(0));
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
    agent.attention = Attention::unseen(AttentionReason::WorkCompleted, Timestamp::from_millis(0));
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
    agent.attention = Attention::Clear;
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
fn cadence_is_event_driven_when_the_cafe_theatre_is_not_visible_or_empty() {
    let mut working = agent();
    working.presence = Presence::Working;
    let mut desk = model_with(working, 500, Motion::Full);
    desk.switch_to(View::Desk);
    assert_eq!(cadence_for(&desk), RenderCadence::EventDriven);

    let empty_cafe = Model::new(View::Cafe);
    assert_eq!(cadence_for(&empty_cafe), RenderCadence::EventDriven);
}

#[test]
fn mixed_cafe_uses_the_fastest_visible_agent_cadence() {
    let mut working = agent();
    working.presence = Presence::Working;
    let mut done = working.clone();
    done.key = "agent-done".into();
    done.presence = Presence::Done;
    done.attention = Attention::unseen(AttentionReason::WorkCompleted, Timestamp::from_millis(0));

    let mut domain = DomainState::default();
    domain.agents.insert(working.key.clone(), working);
    domain.agents.insert(done.key.clone(), done);
    let mut model = Model::new(View::Cafe);
    model.replace_domain(domain);
    model.set_now(Timestamp::from_millis(500));

    assert_eq!(cadence_for(&model), RenderCadence::Fps(8));
}

#[test]
fn display_preferences_are_model_state_with_accessible_defaults() {
    let mut model = Model::new(View::Cafe);
    assert_eq!(model.preferences(), &DisplayPreferences::default());

    let configured = DisplayPreferences {
        motion: Motion::Reduced,
        character_set: CharacterSet::Ascii,
    };
    model.set_preferences(configured);
    assert_eq!(model.preferences(), &configured);
}
