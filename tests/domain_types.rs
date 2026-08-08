use std::time::Duration;

use questmancer::{
    domain::{AgentKey, GuildAttention, GuildSummons, PaneId, Presence, Timestamp, WorkspaceId},
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
fn marking_attention_read_retains_summons_and_original_time() {
    let since = Timestamp::from_millis(1_000);
    let attention = GuildAttention::unread(GuildSummons::CounselRequested, since);

    let read = attention.mark_read();

    assert_eq!(read.summons(), Some(GuildSummons::CounselRequested));
    assert_eq!(read.since(), Some(since));
    assert!(!read.is_unread());
    assert!(matches!(read, GuildAttention::Read { .. }));
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

/// Urgency was `Option<u8>` with a bare `3` written into the one place that
/// meant "nobody is waiting". The type admitted `Some(47)`, the priority lived
/// in the numbers rather than anything named, and the meaning of `3` was
/// recorded in prose in a different file.
///
/// Two things now order by this enum — the `!` jump and the digit Herdr sorts
/// its sidebar by — so its `Ord` and its digits have to agree. Nothing else
/// makes them.
#[test]
fn urgency_digits_follow_the_same_order_as_the_enum() {
    use questmancer::domain::Urgency;

    let mut by_ord = Urgency::ALL.to_vec();
    by_ord.sort();
    assert_eq!(
        by_ord,
        Urgency::ALL,
        "ALL must already be in priority order, most pressing first"
    );

    let digits = Urgency::ALL.iter().map(|u| u.digit()).collect::<Vec<_>>();
    let mut sorted = digits.clone();
    sorted.sort_unstable();
    assert_eq!(digits, sorted, "digits must ascend with the enum order");
    sorted.dedup();
    assert_eq!(sorted.len(), digits.len(), "two urgencies share a digit");

    assert!(
        digits.iter().all(|digit| *digit < Urgency::NOTHING_WANTED),
        "an adventurer nobody is waiting on must sort after every real urgency"
    );
    assert!(
        Urgency::NOTHING_WANTED < 10,
        "the sort key is one character; Herdr compares these tokens as strings"
    );
}

/// The identifier newtypes are the crate's main defence against passing an
/// agent where a pane belongs. This pins the traits callers actually reach
/// for, so the macro cannot quietly lose one.
#[test]
fn identifier_newtypes_carry_the_traits_callers_expect() {
    use questmancer::domain::{AgentKey, PaneId};
    use std::collections::BTreeMap;

    let key: AgentKey = "codex".parse().expect("parsing an id is infallible");
    assert_eq!(key.as_str(), "codex");
    assert_eq!(key.as_ref() as &str, "codex");
    assert_eq!(key.to_string(), "codex");
    assert_eq!(AgentKey::from("codex"), key);

    // Ord and Hash, so they key the maps the domain is built from.
    let mut map = BTreeMap::new();
    map.insert(key.clone(), 1);
    assert_eq!(map.get(&AgentKey::new("codex")), Some(&1));

    // And they do not interchange: this is the whole point of separate types.
    let pane = PaneId::new("codex");
    assert_eq!(pane.as_str(), key.as_str());
    // `map.get(&pane)` would not compile, which is the property under test.
}

/// A bounded setting cannot exist outside its range. The range used to be
/// checked once in the config loader and then forgotten by the type, so
/// anything downstream could hold a value the loader would have rejected.
#[test]
fn bounded_settings_cannot_be_built_out_of_range() {
    use questmancer::config::{ChronicleMaxEntries, OutputPreviewLines};

    assert!(OutputPreviewLines::new(80).is_ok());
    assert!(OutputPreviewLines::new(9).is_err(), "below the range");
    assert!(OutputPreviewLines::new(501).is_err(), "above the range");
    assert_eq!(OutputPreviewLines::default().get(), 80);

    assert!(ChronicleMaxEntries::new(500).is_ok());
    assert!(ChronicleMaxEntries::new(49).is_err());
    assert!(ChronicleMaxEntries::new(10_001).is_err());

    // The bounds are the type's, not a constant sitting somewhere else.
    assert!(OutputPreviewLines::new(*OutputPreviewLines::RANGE.start()).is_ok());
    assert!(OutputPreviewLines::new(*OutputPreviewLines::RANGE.end()).is_ok());
}
