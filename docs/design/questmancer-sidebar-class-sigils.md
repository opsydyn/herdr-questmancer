# Class sigils in the Herdr sidebar

2026-09-08: requested by the Questmancer. Applied to the local Herdr configuration;
native visual review remains pending. This changes only the existing class-sigil
occurrence beside the adventurer name. All other rows and settings are preserved.

Sigils use bold text with faint disabled. Their colours identify class families;
Herdr's separate state icon and Questmancer condition still communicate urgency.
No image support or extra font installation is required by this recipe, although
glyph appearance depends on the terminal font. ASCII mode retains the existing
unique two-letter class abbreviations in bold gold.

| Class | Sigil | Colour |
| --- | --- | --- |
| Barbarian | ⚔ | `#e5b95c` |
| Bard | ♪ | `#f5c2e7` |
| Cleric | ✚ | `#f9e2af` |
| Druid | ❧ | `#a6e3a1` |
| Paladin | ✜ | `#f9e2af` |
| Ranger | ↟ | `#a6e3a1` |
| Rogue | ✦ | `#89b4fa` |
| Wizard | ✧ | `#cba6f7` |
| Artificer | ⚙ | `#94e2d5` |
| Runewright | ᚱ | `#94e2d5` |
| Testmender | ✎ | `#a6e3a1` |
| Pathseeker | ⌖ | `#89b4fa` |
| Mage | ☠ | `#cba6f7` |
| Sorcerer | ☼ | `#f5c2e7` |

Replace the existing `"$quest_sigil"` row element with this inline table:

```toml
{ token = "$quest_sigil", fg = "#e5b95c", bold = true, dim = false, rules = [
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
  ] }
```

The installed Herdr 0.9.0 configuration checker accepts the complete configuration.
All foregrounds clear 4.5:1 against `#1e1e2e`; selected backgrounds and native
font appearance require visual review. The rules match current production sigils
from `src/sidebar.rs`, with one rule per class (fourteen, below Herdr's limit of
sixteen). Unmatched values retain the readable gold base style.

The server was stopped when this configuration was applied. Start Herdr normally
to load it; Questmancer must be running to publish its existing metadata. The plugin
itself never edits Herdr configuration. A backup of the prior configuration is
retained at `~/.config/herdr/config.toml.before-class-sigils-20260908`.

Next review: check the sigil beside each available adventurer at narrow sidebar
widths, then decide whether to adopt the separate
[conditional three-row layout](questmancer-sidebar-09.md).
