//! Guards the paste-ready sidebar configurations we publish.
//!
//! We shipped four `[ui.sidebar]` examples that Herdr rejects outright. They
//! used `value = "…"` where Herdr's schema requires `token = "…"`, and they
//! used bare strings as literal text — `" "` and `"Trinket: "` — where a Herdr
//! row element is always a token name. A user pasted one in verbatim and got
//! `config.toml invalid; using defaults`.
//!
//! Nothing could have caught it: the examples were prose in a Markdown file,
//! and the only thing that reads them is a person. These tests parse the TOML
//! out of the document and hold it to the parts of Herdr's schema we can state
//! precisely — the shape of a row element, and whether a `$quest_*` token is
//! one Questmancer actually publishes.

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

/// A Herdr row element is a token name or an inline table keyed by `token`.
/// Ours used `value`, which Herdr rejects, taking the whole file down with it.
#[test]
fn documented_rows_use_the_token_key_herdr_expects() {
    for block in toml_blocks() {
        for (panel, row) in rows_of(&block) {
            for element in row {
                match element {
                    Value::String(_) => {}
                    Value::Table(table) => {
                        assert!(
                            table.contains_key("token"),
                            "a {panel} row element is keyed {:?}; Herdr requires `token`",
                            table.keys().collect::<Vec<_>>()
                        );
                        for key in table.keys() {
                            assert!(
                                ["token", "fg", "bold", "dim"].contains(&key.as_str()),
                                "`{key}` is not a Herdr row-style key"
                            );
                        }
                    }
                    other => panic!("a {panel} row element must be a token or table, got {other}"),
                }
            }
        }
    }
}

/// Herdr rows hold tokens and nothing else — it inserts its own separators.
/// Ours carried `" "` and `"Trinket: "` as if literal text were allowed, so
/// every one of those was read as the name of a token that does not exist.
#[test]
fn documented_rows_name_only_tokens_that_exist() {
    let published = published_tokens();
    for block in toml_blocks() {
        for (panel, row) in rows_of(&block) {
            for element in row {
                let name = match &element {
                    Value::String(name) => name.clone(),
                    Value::Table(table) => table
                        .get("token")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_owned(),
                    _ => continue,
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
