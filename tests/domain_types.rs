use std::time::Duration;

use herdr_webmaster::{
    domain::{AgentKey, Attention, AttentionReason, PaneId, Presence, Timestamp, WorkspaceId},
    herdr::protocol::AgentStatus,
};

#[test]
fn ids_keep_their_types_while_serializing_as_strings() {
    let workspace = WorkspaceId::new("w1");
    let pane = PaneId::new("w1:p1");
    let agent = AgentKey::new("agent-1");

    assert_eq!(workspace.as_str(), "w1");
    assert_eq!(pane.to_string(), "w1:p1");
    assert_eq!(serde_json::to_string(&agent).unwrap(), "\"agent-1\"");
    assert_ne!(
        std::any::type_name_of_val(&workspace),
        std::any::type_name_of_val(&pane)
    );
}

#[test]
fn protocol_status_converts_to_domain_presence() {
    assert_eq!(Presence::from(AgentStatus::Working), Presence::Working);
    assert_eq!(Presence::from(AgentStatus::Blocked), Presence::Blocked);
    assert_eq!(Presence::from(AgentStatus::Done), Presence::Done);
    assert_eq!(Presence::from(AgentStatus::Idle), Presence::Idle);
    assert_eq!(Presence::from(AgentStatus::Unknown), Presence::Unknown);
}

#[test]
fn marking_attention_seen_retains_reason_and_original_time() {
    let since = Timestamp::from_millis(1_000);
    let attention = Attention::unseen(AttentionReason::NeedsInput, since);

    let seen = attention.mark_seen();

    assert_eq!(seen.reason(), Some(AttentionReason::NeedsInput));
    assert_eq!(seen.since(), Some(since));
    assert!(!seen.is_unseen());
    assert!(matches!(seen, Attention::Seen { .. }));
}

#[test]
fn timestamp_elapsed_is_deterministic_and_saturating() {
    let since = Timestamp::from_millis(1_500);

    assert_eq!(
        since.elapsed_until(Timestamp::from_millis(4_000)),
        Duration::from_millis(2_500)
    );
    assert_eq!(
        since.elapsed_until(Timestamp::from_millis(1_000)),
        Duration::ZERO
    );
}
