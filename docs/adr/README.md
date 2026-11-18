# Architecture decisions

- [0001 — Qualify proportional Hall fitting in Storybook](0001-storybook-hall-fit.md)
  — accepted experiment; production promotion is recorded in ADR-0002.
- [0002 — Integrate proportional Guild Hall fitting](0002-production-hall-fit.md)
  — accepted after explicit promotion; extends the experiment to the Hall only.
- [0003 — Extend proportional fitting to the Delve](0003-proportional-delve.md)
  — accepted; both rooms share the native output adapter and complete-party fallback.
- [0004 — Retain the dungeon when stations overflow](0004-crowded-delve-room.md)
  — accepted correction after screenshot review; preserves room and full masters for ordinary crowded parties.
- [0005 — Preserve fitted rooms beneath adventurer profiles](0005-profile-keeps-world-fit.md)
  — accepted correction; fixed-scale room images remain behind masked profile overlays.
- [0006 — Compare larger parties in Storybook](0006-storybook-party-capacity.md)
  — accepted experiment; compares full and small authored sprites up to thirty-two adventurers without production promotion.
- [0007 — Integrate automatic fitted-room density](0007-automatic-party-density.md)
  — accepted production promotion; full masters through eleven, smaller authored sprites through sixty-four, stable seats and unchanged full-size profiles.
- [0008 — One writer per Questmancer profile](0008-profile-writer.md)
  — serializes profile writes and supports read-only secondary panes.
