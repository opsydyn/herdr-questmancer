# Sidebar layout comparison

Date: 2026-09-08. Status: four-row candidate applied; visual acceptance deferred
to the final review by explicit user request. The captured current.toml remains
the original eight-row comparison baseline.
The user approved the sequence: operating-note corrections, sidebar comparison,
Librarian/pilot acceptance, release acceptance, then campaign heraldry.

The protocol note now says 22. Empty-hoard rules now match `◈ empty` and
`* empty`, which are the current production values. No runtime code changed.

## Compared configurations

- [Current](current.toml): eight configured rows with the applied class colours.
- [Three rows](three.toml): compact proposal, retaining the same class colours.
- [Four rows](four.toml): recommended for this local sidebar. Name and class sigil;
  state icon, condition and vigil; ancestry/class; remembered spoils.

The four-row option makes urgency readable before supporting identity details.
At 24 columns, `Elf Wizard` plus the separator and `Concentrating` already
requires 26 columns, before adding the sigil. This makes the three-row design's
combined role/condition line unreliable at narrow widths. Herdr controls actual
truncation, so terminal verification remains necessary.

The four-row candidate omits the separate epithet, omen and keepsake rows.
It also omits the machine token for this local-only comparison; SSH use requires
an explicit machine-context review. No campaign rows, sorting or global
configuration were changed.

## Review and evidence

The inline comparison uses illustrative Wizard, Ranger and Barbarian fixtures,
current token vocabulary and the validated configuration tables. It allows
24/32-column comparisons, two sets of states and ASCII fallback. Missing vigil
rows disappear in the current configuration, so visible row counts differ from
the configured maximum. Spoils examples are fixtures, not observed completions.

The preview approximates column clipping and state icons using browser text.
It is not Herdr's renderer, a terminal screenshot or live-agent evidence. The
static dark palette is deliberate; it does not establish light-theme acceptance.
Browser Use rejected automated local-file inspection under its URL policy.
JavaScript syntax and static element references were checked; interactive and
native appearance remain for user review.

All three configurations pass installed Herdr 0.9.0 `config check`:
[validation receipt](validation.json). The preparation script reads only the
sidebar section of the local config and validates candidates in isolated temporary
configuration directories. The recorded output contains only sidebar rows.

The four-row candidate is now applied and passes the active configuration check.
The prior configuration is retained at
`~/.config/herdr/config.toml.before-four-row-20260908`. Campaign rows and all
unrelated configuration were preserved. The server remained stopped; no reload
was needed or attempted. Native appearance and user visual acceptance remain
pending in the final review queue. Release preparation and heraldry engineering
proceeded under the revised ordering.
