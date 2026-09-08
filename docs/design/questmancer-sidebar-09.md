# Conditional sidebar proposal for Herdr 0.9

Date: 2026-09-08. Status: proposed for visual review. The Questmancer identified
Herdr 0.9's conditional styles as a design opportunity during the upgrade.
This three-row layout remains unapplied; the later four-row candidate was
applied on 2026-09-08 with visual sign-off deferred at the user's request. The separate
[class-sigil styling](questmancer-sidebar-class-sigils.md) was requested and
applied on 2026-09-08, preserving the existing rows.

Use the existing character sheet with three compact rows. The name stays
readable, the machine stays visible, and condition/vigil values carry emphasis.
This works with existing Questmancer tokens and Herdr's own `machine` token.

| Fact | Treatment | Meaning remains visible as text |
| --- | --- | --- |
| Working | Warm gold condition | `Concentrating` |
| Needs counsel | Bold coral condition | `Restrained`, Herdr state icon and vigil pips |
| Returned spoils | Bold green condition | `Triumphant`; it does not imply reviewed or accepted work |
| Resting | Muted readable condition | `Resting` |
| Unknown | Bold lavender condition | `Blinded` plus Herdr's own state icon; never a success colour |
| Counsel waits at least five minutes | Vigil becomes bold coral | Three or more filled pips, in Unicode or ASCII |
| No remembered spoils | Hoard is muted | Explicit empty bag remains visible |

Keep `dim = false`. Our earlier faint-text measurements showed why reducing
contrast is a poor way to communicate calm. Colour and weight establish the
hierarchy; the existing words, state icon and pips carry meaning independently.
No blinking, animation, new timers, rank digits or invented counters are needed.

## Proposed three-row recipe

This is an optional Herdr configuration fragment. Review it before replacing
existing rows; the user owns that global configuration. It does not enable
Questmancer's separate urgency sort or configure any SSH connections.

```toml
[ui.sidebar.agents]
row_gap = 1
rows = [
  ["state_icon", { token = "agent", fg = "#cdd6f4", bold = true, dim = false }, { token = "machine", fg = "#9399b2", dim = false }],
  [{ token = "$quest_sigil", fg = "#e5b95c", bold = true, dim = false, rules = [
    { equals = "⚔", fg = "#e5b95c" },
    { equals = "♪", fg = "#f5c2e7" },
    { equals = "✚", fg = "#f9e2af" },
    { equals = "❧", fg = "#a6e3a1" },
    { equals = "✜", fg = "#f9e2af" },
    { equals = "↟", fg = "#a6e3a1" },
    { equals = "✦", fg = "#89b4fa" },
    { equals = "✧", fg = "#cba6f7" },
    { equals = "⚙", fg = "#94e2d5" },
    { equals = "ᚱ", fg = "#94e2d5" },
    { equals = "✎", fg = "#a6e3a1" },
    { equals = "⌖", fg = "#89b4fa" },
    { equals = "☠", fg = "#cba6f7" },
    { equals = "☼", fg = "#f5c2e7" },
  ] }, { token = "$quest_role", fg = "#9399b2", dim = false }, { token = "$quest_condition", fg = "#9399b2", bold = false, dim = false, rules = [
    { equals = "Restrained", fg = "#f38ba8", bold = true },
    { equals = "Concentrating", fg = "#e5b95c" },
    { equals = "Triumphant", fg = "#a6e3a1", bold = true },
    { equals = "Blinded", fg = "#cba6f7", bold = true },
  ] }],
  [{ token = "$quest_vigil", fg = "#e5b95c", bold = false, dim = false, rules = [
    { starts_with = "●●●", fg = "#f38ba8", bold = true },
    { starts_with = "###", fg = "#f38ba8", bold = true },
  ] }, { token = "$quest_hoard", fg = "#e5b95c", dim = false, rules = [
    { equals = "◈ empty", fg = "#9399b2" },
    { equals = "* empty", fg = "#9399b2" },
  ] }],
]
```

## What the rules can express

The [0.9 implementation](https://github.com/herdrdev/herdr/blob/v0.9.0/src/config/sidebar/rules.rs)
uses the first matching rule, then inherits unspecified colour/weight/dimming
from the element's base style. Each rule has exactly one condition:
`equals`, `contains`, `starts_with`, `gt` or `lt`. More-specific overlapping
rules must come first. Numeric comparisons require the entire token value to
be a finite number; they are strict comparisons.

Rules inspect their **own token's displayed value**. `$quest_rank` cannot
style `agent` or `$quest_condition` from another element. `$quest_hoard`
contains a glyph, count and noun, so a numeric `gt` rule would never match it.
Matching the two explicit empty-bag strings keeps the current truthful vocabulary.
The vigil already supplies a factual duration ladder; matching its pips avoids
adding a second stored time or another presentation token.

Do not add rules to `state_icon` or `git_status`: Herdr rejects rules on those
non-text tokens. Machine names are client-provided context; the Questmancer
socket and adventurer IDs remain local to their server.

## Review gate

On 2026-09-08, the installed Herdr 0.9.0 accepted this recipe with
`herdr config check`. All explicit foreground colours measure at least 5.81:1
against Catppuccin Mocha's `#1e1e2e` base with faint disabled; this does not certify selected backgrounds,
other themes, fonts or actual terminal rendering.

Before enabling the recipe, review working, new/older counsel, resting,
completed and unknown adventurers at narrow and normal sidebar widths. Check
that role and machine text remain useful and that coral draws attention
without making the whole list loud. Review Unicode and ASCII vigils and a
zero/nonzero hoard. A local-only view can establish layout; remote-machine
acceptance requires an actual configured remote and remains separate.

The previous [static character-sheet recipes](questmancer-sidebar-character-sheet.md)
remain available. The next bounded design step is a user-approved native trial
of this fragment, recording the original rows and restoring them if the new
hierarchy is less readable.

## Layout comparison — 2026-09-08

The [review comparison](reviews/2026-09-08-sidebar-comparison/README.md)
retains the current layout and this three-row proposal, plus a recommended
four-row option. The latter reserves a line for condition and vigil so a long
ancestry/class does not displace them. All candidates retain the requested
class-sigil colours. User selection and native visual acceptance remain pending.
