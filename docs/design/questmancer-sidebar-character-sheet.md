# Questmancer sidebar character sheet

Status: implemented. Questmancer publishes the tokens; Herdr renders them.

The sidebar is Herdr's, not Questmancer's. Questmancer only reports custom
`$name` metadata tokens per pane and per workspace, exactly as Herdr's
[UI and sidebar configuration](https://herdr.dev/docs/configuration/#ui-and-sidebar)
describes. Nothing here changes an agent's title, display name or state text —
Herdr's own `state_icon` and `state_text` remain the authority on status.

## The rule these tokens follow

The vocabulary is a tabletop character sheet. The **values are facts
Questmancer already holds**. A sigil is the adventurer's class, a condition is
its presence, a vigil is how long a summons has actually gone unanswered, a
hoard counts spoils actually recorded in the Chronicle. No token invents a
number to look like a game.

That rule is what keeps the sidebar an instrument rather than decoration. If a
future token cannot be traced to something Questmancer knows, it does not
belong here.

## Agent tokens

| Token | Reads as | Derived from |
| --- | --- | --- |
| `$quest_sigil` | `✧` | the adventurer's class, one glyph |
| `$quest_role` | `Elf Wizard` | ancestry and class |
| `$quest_epithet` | `Keeper of Schemas` | the persona's epithet |
| `$quest_condition` | `Concentrating` | presence, in condition vocabulary |
| `$quest_omen` | `on expedition` | presence, in prose |
| `$quest_trinket` | `Lucky Coin` | the persona's keepsake |
| `$quest_vigil` | `●●●○○○` | how long a summons has gone unanswered |
| `$quest_hoard` | `◈ 3 spoils` | spoils returned, per the Chronicle |

`$quest_vigil` is deliberately **empty unless the adventurer is blocked**. Only
a waiting summons keeps a vigil, and the track deepens through the six
tabletop exhaustion levels as the wait lengthens: one pip under a minute, six
past an hour. It is an urgency signal, not a health bar.

`$quest_hoard` counts what the Chronicle still remembers. The Chronicle is
bounded, so this is recent standing rather than lifetime totals.

## Space tokens

| Token | Reads as | Derived from |
| --- | --- | --- |
| `$quest_campaign` | `3 adventurers · 1 summons` | party size and open summons |
| `$quest_party` | `✧⚔✚` | the party's classes, in marching order |
| `$quest_hoard` | `◈ 7 spoils` | the guild's recorded spoils |

## Class sigils

| Class | Glyph | Class | Glyph |
| --- | --- | --- | --- |
| Barbarian | `⚔` | Artificer | `⚙` |
| Bard | `♪` | Runewright | `ᚱ` |
| Cleric | `✚` | Testmender | `✎` |
| Druid | `❧` | Pathseeker | `⌖` |
| Paladin | `✜` | Mage | `☠` |
| Ranger | `↟` | Sorcerer | `☼` |
| Rogue | `✦` | Wizard | `✧` |

Set `character_set = "ascii"` in Questmancer's configuration and every glyph
becomes a two-letter abbreviation, the vigil becomes `###---`, and the hoard
marker becomes `*`. The rows never collapse into replacement boxes.

## A sidebar to paste into Herdr

Add to your Herdr configuration. Each inner array is one rendered line.

```toml
[ui.sidebar.agents]
row_gap = 1
rows = [
  ["state_icon", "$quest_sigil", "agent"],
  ["$quest_role", "$quest_condition"],
  ["$quest_vigil"],
  ["$quest_hoard"],
]

[ui.sidebar.spaces]
rows = [
  ["state_icon", "workspace", "$quest_party"],
  ["$quest_campaign"],
  ["$quest_hoard"],
]
```

A tighter two-line variant for narrow sidebars:

```toml
[ui.sidebar.agents]
rows = [
  ["$quest_sigil", "agent", "$quest_vigil"],
  ["$quest_condition"],
]
```

A full character sheet, for a wide sidebar:

```toml
[ui.sidebar.agents]
row_gap = 1
rows = [
  ["$quest_sigil", "agent"],
  ["$quest_role"],
  ["$quest_epithet"],
  ["state_icon", "$quest_condition"],
  ["$quest_omen"],
  ["$quest_trinket"],
  ["$quest_vigil"],
  ["$quest_hoard"],
]
```

## What Herdr 0.7.4 actually accepts

Every configuration above is verified against the binary we target, not
against the published documentation. The two differ, and this document got it
wrong twice before anyone ran `herdr config check`:

- **A row element is a plain string, and it must name a token.** Herdr 0.7.4
  rejects an inline table outright — `invalid type: map, expected a string` —
  so the `{ token = "…", fg = "…", bold = true }` styling shown on
  herdr.dev belongs to a later schema than the one we pin. There is no
  styling mechanism in 0.7.4.
- **There is no literal-text element.** `" "` and `"Trinket: "` are read as
  token names and rejected: `unknown sidebar token; custom tokens must start
  with $`. Herdr separates adjacent values with `·` itself and puts a single
  space after `state_icon`, so spacers are unnecessary as well as invalid.

Because a value cannot be styled or labelled by the configuration, anything it
needs in order to explain itself has to travel inside the token. That is why
`$quest_hoard` reads `◈ 3 spoils`, `$quest_omen` reads `seeks counsel`, and
`$quest_trinket` reads `❖ Silver Compass` rather than relying on a prefix the
row cannot supply.

One trap worth knowing: Herdr 0.7.4 **ignores keys it does not recognise** but
validates every token strictly. A misspelled key fails silently while a
misspelled token takes the whole file down to defaults — losing every unrelated
setting with it, which is how a broken sidebar row brought back the onboarding
dialog.

`tests/sidebar_documentation.rs` parses every example above and holds it to
these rules. `herdr config check` remains the authority; run it after editing.

## Non-goals

- No replacement of Herdr's `state_icon` or `state_text`. Questmancer adds a
  second vocabulary beside them; it never overrides the authority on status.
- No token that cannot be traced to something Questmancer already knows.
- No Questmancer-drawn sidebar. The sidebar stays Herdr UI, configured by the
  user, and works with none of these tokens enabled.
