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

/// Herdr's default theme is `catppuccin`, whose base is this. Contrast is
/// judged against it because it is the background our published examples
/// actually land on for a reader who has not changed themes.
const THEME_BACKGROUND: (u8, u8, u8) = (0x1e, 0x1e, 0x2e);

/// The WCAG AA floor for normal-size text. Sidebar rows are small text in a
/// narrow rail; there is no "large text" exemption available to them.
const CONTRAST_FLOOR: f64 = 4.5;

fn parse_hex(colour: &str) -> (u8, u8, u8) {
    let digits = colour.strip_prefix('#').expect("fg is strict hex");
    let expand = |c: char| u8::from_str_radix(&c.to_string(), 16).expect("hex digit") * 0x11;
    match digits.len() {
        3 => {
            let mut chars = digits.chars();
            let mut next = || expand(chars.next().expect("three digits"));
            (next(), next(), next())
        }
        6 => {
            let byte = |at: usize| u8::from_str_radix(&digits[at..at + 2], 16).expect("hex pair");
            (byte(0), byte(2), byte(4))
        }
        _ => panic!("fg {colour:?} is neither #RGB nor #RRGGBB"),
    }
}

/// Relative luminance, per WCAG 2.1.
fn luminance((r, g, b): (u8, u8, u8)) -> f64 {
    let channel = |value: u8| {
        let value = f64::from(value) / 255.0;
        if value <= 0.040_45 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)
}

fn contrast(foreground: (u8, u8, u8), background: (u8, u8, u8)) -> f64 {
    let (a, b) = (luminance(foreground), luminance(background));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

/// Herdr renders sidebar rows faint unless an element opts out, and terminals
/// implement faint as a multiplier — Ghostty applies about `0.63`. Measured on
/// the default catppuccin theme, that put every published row between 1.77:1
/// and 2.78:1, well under the AA floor, including the vigil whose entire job is
/// to be noticed.
///
/// `dim = true` was therefore never an effect these examples added; it was the
/// default they restated. Removing it would have changed nothing. Only
/// `dim = false` turns faint off.
#[test]
fn documented_rows_never_request_faint() {
    for block in toml_blocks() {
        for (panel, row) in rows_of(&block) {
            for element in row {
                let Value::Table(table) = &element else {
                    continue;
                };
                let token = table["token"].as_str().unwrap_or("<unnamed>");
                match table.get("dim").and_then(Value::as_bool) {
                    Some(false) => {}
                    Some(true) => panic!(
                        "the {panel} element {token:?} sets `dim = true`, which is already \
                         Herdr's default. Faint drops it to roughly 1.9:1 on the default \
                         theme; set `dim = false`."
                    ),
                    None => panic!(
                        "the {panel} element {token:?} does not set `dim`, so Herdr renders \
                         it faint. Every styled element must set `dim = false`."
                    ),
                }
            }
        }
    }
}

/// A colour that cannot be read is not styling. Faint is off by the test above,
/// so these values land at face value and can be judged directly.
///
/// This also encodes a limit worth knowing: while faint was on, no red cleared
/// the floor — not even `#ff0000`, at 1.79:1 — so the vigil could not be fixed
/// by choosing a better red. It needed `dim = false` first.
#[test]
fn documented_colours_clear_the_contrast_floor() {
    let mut checked = 0;
    for block in toml_blocks() {
        for (panel, row) in rows_of(&block) {
            for element in row {
                let Value::Table(table) = &element else {
                    continue;
                };
                let Some(colour) = table.get("fg").and_then(Value::as_str) else {
                    let token = table["token"].as_str().unwrap_or("<unnamed>");
                    panic!(
                        "the {panel} element {token:?} has no `fg`. Unstyled row text uses \
                         Herdr's muted `overlay0`, which cannot reach {CONTRAST_FLOOR}:1 on \
                         the default theme even with faint off."
                    );
                };
                let ratio = contrast(parse_hex(colour), THEME_BACKGROUND);
                assert!(
                    ratio >= CONTRAST_FLOOR,
                    "the {panel} colour {colour} is {ratio:.2}:1 against the default theme \
                     background, under the {CONTRAST_FLOOR}:1 floor for small text"
                );
                checked += 1;
            }
        }
    }
    assert!(
        checked >= 20,
        "only {checked} styled elements were checked; the examples lost their styling"
    );
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
