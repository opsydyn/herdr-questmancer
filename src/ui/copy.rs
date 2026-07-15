pub const EMPTY_GUILD: &str = "The hearth is warm. The guild awaits its next commission.";
pub const SCRYING_CLOUDED: &str = "The scrying pool has clouded. Reconnecting...";
pub const SCRYING_STILL: &str = "The scrying pool is still.";
pub const COUNSEL_ISSUED: &str = "Counsel issued.";
pub const SUMMONS_ACKNOWLEDGED: &str = "Summons acknowledged.";

#[must_use]
pub fn counsel_requested(name: &str) -> String {
    format!("{name} requests counsel at a sealed gate.")
}

#[must_use]
pub fn spoils_returned(name: &str) -> String {
    format!("{name} has returned with unopened spoils.")
}

#[must_use]
pub fn no_match(query: &str) -> String {
    format!("No adventurer or campaign answers {query:?}.")
}
