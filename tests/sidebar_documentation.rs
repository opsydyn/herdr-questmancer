//! Guards the paste-ready sidebar configurations we publish.
//!
//! Every `[ui.sidebar]` example we shipped was rejected by Herdr, and the
//! first attempt to fix them was rejected too. The rules below come from
//! running `herdr config check` against the 0.7.4 binary we target, not from
//! the published documentation, which describes a later schema:
//!
//! - a row element is a plain string; an inline table gives
//!   `invalid type: map, expected a string`;
//! - every string must name a builtin token or a `$`-prefixed custom one;
//!   literal text like `" "` or `"Trinket: "` gives `unknown sidebar token`.
//!
//! Herdr ignores keys it does not recognise but validates tokens strictly, so
//! a bad token takes the entire file down to defaults and silently discards
//! every unrelated setting with it.
//!
//! Nothing could have caught this: the examples were prose in a Markdown file
//! and the only thing that read them was a person.

use std::collections::BTreeSet;

use questmancer::sidebar::ALL_QUEST_TOKENS;
use toml::{Table, Value};

const DOC: &str = include_str!("../docs/design/questmancer-sidebar-character-sheet.md");

/// Herdr's built-in row tokens, from its configuration documentation.
const HERDR_TOKENS: &[&str] = &[
    "state_icon",
    "state_text",
    "workspace",
    "tab",
    "pane",
    "agent",
    "terminal_title",
    "terminal_title_stripped",
    "branch",
    "git_status",
];

fn toml_blocks() -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current: Option<String> = None;
    for line in DOC.lines() {
        match (&mut current, line.trim()) {
            (None, "```toml") => current = Some(String::new()),
            (Some(_), "```") => blocks.push(current.take().expect("inside a block")),
            (Some(block), _) => {
                block.push_str(line);
                block.push('\n');
            }
            _ => {}
        }
    }
    assert!(current.is_none(), "an unterminated ```toml block");
    blocks
}

/// Every `$quest_*` token Questmancer actually reports for an agent or space.
fn published_tokens() -> BTreeSet<String> {
    ALL_QUEST_TOKENS
        .iter()
        .map(|name| format!("${name}"))
        .collect()
}

fn rows_of(block: &str) -> Vec<(String, Vec<Value>)> {
    let parsed: Table = toml::from_str(block)
        .unwrap_or_else(|error| panic!("documented config is not valid TOML: {error}\n{block}"));
    let mut found = Vec::new();
    for panel in ["agents", "spaces"] {
        let Some(rows) = parsed
            .get("ui")
            .and_then(|ui| ui.get("sidebar"))
            .and_then(|sidebar| sidebar.get(panel))
            .and_then(|table| table.get("rows"))
            .and_then(Value::as_array)
        else {
            continue;
        };
        for row in rows {
            let row = row
                .as_array()
                .unwrap_or_else(|| panic!("a {panel} row must be an array"));
            found.push((panel.to_owned(), row.clone()));
        }
    }
    found
}

#[test]
fn every_documented_config_is_valid_toml() {
    let blocks = toml_blocks();
    assert!(blocks.len() >= 3, "the document lost its examples");
    for block in blocks {
        assert!(
            toml::from_str::<Table>(&block).is_ok(),
            "a published configuration does not parse:\n{block}"
        );
    }
}

/// Herdr 0.7.4 takes plain strings and nothing else. Our first correction
/// reached for the inline-table styling documented on herdr.dev, which belongs
/// to a later schema; the binary answers `invalid type: map, expected a
/// string` and falls back to defaults.
#[test]
fn documented_rows_hold_plain_strings() {
    for block in toml_blocks() {
        for (panel, row) in rows_of(&block) {
            for element in row {
                assert!(
                    element.is_str(),
                    "a {panel} row element is {element}; Herdr 0.7.4 accepts only \
                     plain strings, and rejects the whole file on anything else"
                );
            }
        }
    }
}

/// Herdr rows hold token names and nothing else — it inserts its own
/// separators. Ours carried `" "` and `"Trinket: "` as if literal text were
/// allowed, and each was read as the name of a token that does not exist.
#[test]
fn documented_rows_name_only_tokens_that_exist() {
    let published = published_tokens();
    for block in toml_blocks() {
        for (panel, row) in rows_of(&block) {
            for element in row {
                let Some(name) = element.as_str().map(str::to_owned) else {
                    continue;
                };
                let known = HERDR_TOKENS.contains(&name.as_str()) || published.contains(&name);
                assert!(
                    known,
                    "a {panel} row names {name:?}, which is neither a Herdr token nor one \
                     Questmancer publishes. Literal text is not a row element: Herdr \
                     separates adjacent tokens itself, and any label a value needs must \
                     travel inside the token."
                );
            }
        }
    }
}
