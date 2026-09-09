#[test]
fn standing_handbook_distinguishes_rewards_from_observations() {
    let text = questmancer::ledger::standing_page_body(0).join(" ");
    assert!(text.contains("status-confirmed spoils"));
    assert!(text.contains("Snapshot observations and campaign removal earn no XP."));
    assert!(!text.contains("spoils returned, and campaigns closed"));
}
