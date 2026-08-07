//! Guards the paste-ready sidebar configurations we publish.
//!
//! Every `[ui.sidebar]` example we shipped was rejected by Herdr, and the
//! first attempt to fix them was rejected too. The rules below come from
//! feeding candidate rows to `herdr config check` on the 0.8.0 binary we
//! target, not from reading the published documentation:
//!
//! - a row element is a token name, or an inline table keyed `token`;
//!   `value =` is rejected, and so is a table with styling but no token;
//! - the only style keys are `token`, `fg`, `bold` and `dim` — `italic` is
//!   rejected — and `fg` takes strict hex, so `"red"` is rejected;
//! - there is still no literal text: `" "` and `"Trinket: "` are read as token
//!   names and give `unknown sidebar token`.
//!
//! Styling itself arrived in Herdr 0.7.5; on 0.7.4 an inline table fails with
//! `invalid type: map, expected a string`.
//!
//! Herdr ignores keys it does not recognise but validates tokens and styles
//! strictly, so a bad token takes the entire file down to defaults and
//! silently discards every unrelated setting with it.
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

/// A styled element is an inline table keyed `token`, carrying only `fg`,
/// `bold` and `dim`. Our first version keyed it `value`, which Herdr rejects,
/// taking the whole file down with it.
#[test]
fn documented_rows_use_the_styling_keys_herdr_accepts() {
    for block in toml_blocks() {
        for (panel, row) in rows_of(&block) {
            for element in row {
                let Value::Table(table) = &element else {
                    assert!(
                        element.is_str(),
                        "a {panel} row element is {element}; Herdr accepts a token \
                         name or an inline table and nothing else"
                    );
                    continue;
                };
                assert!(
                    table.contains_key("token"),
                    "a {panel} row element is keyed {:?}; Herdr requires `token`",
                    table.keys().collect::<Vec<_>>()
                );
                for key in table.keys() {
                    assert!(
                        ["token", "fg", "bold", "dim"].contains(&key.as_str()),
                        "`{key}` is not a Herdr row-style key; only token, fg, \
                         bold and dim are accepted"
                    );
                }
                for flag in ["bold", "dim"] {
                    if let Some(set) = table.get(flag) {
                        assert!(set.is_bool(), "`{flag}` must be a boolean, got {set}");
                    }
                }
                if let Some(colour) = table.get("fg") {
                    let colour = colour
                        .as_str()
                        .unwrap_or_else(|| panic!("fg must be a string, got {colour}"));
                    let digits = colour.strip_prefix('#').unwrap_or("");
                    assert!(
                        (digits.len() == 3 || digits.len() == 6)
                            && digits.chars().all(|c| c.is_ascii_hexdigit()),
                        "fg {colour:?} is not strict hex; Herdr rejects named colours"
                    );
                }
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
