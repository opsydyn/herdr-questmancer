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
  ["state_icon", "$quest_sigil", { token = "agent", bold = true }],
  [{ token = "$quest_role", dim = true }, { token = "$quest_condition", fg = "#c9a227" }],
  [{ token = "$quest_vigil", fg = "#c2413f" }],
  [{ token = "$quest_hoard", dim = true }],
]

[ui.sidebar.spaces]
rows = [
  ["state_icon", { token = "workspace", bold = true }, "$quest_party"],
  [{ token = "$quest_campaign", dim = true }],
  [{ token = "$quest_hoard", dim = true }],
]
```

A tighter two-line variant for narrow sidebars:

```toml
[ui.sidebar.agents]
rows = [
  ["$quest_sigil", { token = "agent", bold = true }, { token = "$quest_vigil", fg = "#c2413f" }],
  [{ token = "$quest_condition", dim = true }],
]
```

A full character sheet, for a wide sidebar:

```toml
[ui.sidebar.agents]
row_gap = 1
rows = [
  ["$quest_sigil", { token = "agent", bold = true }],
  [{ token = "$quest_role", dim = true }],
  [{ token = "$quest_epithet", dim = true }],
  ["state_icon", { token = "$quest_condition", fg = "#c9a227" }],
  [{ token = "$quest_omen", dim = true }],
  [{ token = "$quest_trinket", dim = true }],
  [{ token = "$quest_vigil", fg = "#c2413f" }],
  [{ token = "$quest_hoard", fg = "#c9a227" }],
]
```

## What Herdr 0.8.0 actually accepts

Every configuration above is verified by feeding it to `herdr config check`,
not by reading the published documentation. This document got the schema wrong
twice before anyone ran the binary, and each time a pasted example took a whole
`config.toml` down to defaults:

- **A row element is a token name, or an inline table keyed `token`.**
  `{ token = "$quest_vigil", fg = "#c2413f" }`. `value =` is rejected. So is a
  table with styling but no `token`.
- **The only style keys are `token`, `fg`, `bold` and `dim`.** `italic` is
  rejected, and `bold` must be a boolean rather than a string.
- **`fg` takes strict hex only** — `#RGB` or `#RRGGBB`, either case. A named
  colour such as `"red"` is rejected.
- **There is still no literal-text element.** `" "` and `"Trinket: "` are read
  as token names and rejected: `unknown sidebar token; custom tokens must start
  with $`. Herdr separates adjacent values with `·` itself and puts a single
  space after `state_icon`, so spacers are unnecessary as well as invalid.

Styling arrived in Herdr 0.7.5. On 0.7.4 an inline table fails with
`invalid type: map, expected a string`, so these configurations require the
0.8.0 the plugin now targets.

Because a row cannot supply literal text, anything a value needs in order to
explain itself has to travel inside the token. That is why `$quest_hoard`
reads `◈ 3 spoils`, `$quest_omen` reads `seeks counsel`, and `$quest_trinket`
reads `❖ Silver Compass` rather than relying on a prefix the row cannot give.

One trap worth knowing: Herdr **ignores keys it does not recognise** but
validates tokens and styles strictly. A misspelled key fails silently while a
misspelled token takes the whole file down to defaults — losing every unrelated
setting with it, which is how a broken sidebar row brought back the onboarding
dialog.

`tests/sidebar_documentation.rs` parses every example above and holds it to
these rules. `herdr config check` remains the authority; run it after editing.

## Letting Herdr order its own list by urgency

Herdr 0.7.5 added `agent.view.set`, which takes a declarative sort and applies
it to the sidebar, mobile, mouse and agent-keybind navigation order — and one
of the fields it can sort by is a plugin's own metadata token.

Questmancer publishes `$quest_rank` for exactly that: a single digit, `0` for
an unanswered call for counsel, `1` for one already seen, `2` for the quieter
summons, `3` for nothing wanted. Turn the view on and Herdr's own agent list
leads with whoever needs a human, without Questmancer's pane being open.

```toml
# Questmancer's config.toml, not Herdr's
sidebar_urgency_order = true
```

Four rules this follows:

- **Off by default.** It changes shared Herdr UI rather than anything inside
  Questmancer's pane. The sidebar belongs to the user.
- **Sort, never filter.** `agent.view.set` can filter too. Hiding an agent from
  Herdr's own sidebar because Questmancer judged it uninteresting would take
  authority the plugin does not have; reordering leaves every agent reachable.
- **Cleared on shutdown.** A Questmancer that has stopped must not leave
  Herdr's list sorted on its behalf, so the view is released when the pane
  closes and re-requested on every fresh connection — Herdr's view is
  transient, so a server restart drops it.
- **One definition of urgent.** The rank comes from `Agent::urgency_rank`, the
  same function behind the `!` jump. Two definitions would drift and the
  sidebar would start contradicting the key.

`$quest_rank` is not for display. It is a bare digit because Herdr compares
custom tokens as the strings they are, and a single digit sorts identically
whether that comparison is lexicographic or numeric — which is not something
the plugin can see from outside.

## Non-goals

- No replacement of Herdr's `state_icon` or `state_text`. Questmancer adds a
  second vocabulary beside them; it never overrides the authority on status.
- No token that cannot be traced to something Questmancer already knows.
- No Questmancer-drawn sidebar. The sidebar stays Herdr UI, configured by the
  user, and works with none of these tokens enabled.
