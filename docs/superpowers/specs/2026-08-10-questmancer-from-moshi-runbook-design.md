# Questmancer from Moshi runbook design

## Status

Approved design for an end-user operator runbook. Moshi is an optional outer
mobile-access layer for an already-installed Questmancer plugin; it is not a
Questmancer runtime dependency or release acceptance surface.

## Audience and outcome

The document targets a user who already has Questmancer installed and wants to
reach an existing Herdr workspace from Moshi on a phone or tablet. A reader
should be able to connect, open the Guild Hall or Delve, respond to a summons,
and move into deeper agent review without touching Rust or changing shared
Herdr state.

## Runbook shape

Create `docs/runbooks/questmancer-from-moshi.md` and link it from the user-facing
README. Organize it around:

- prerequisites and the distinction between Questmancer, Herdr, Moshi, and
  optional `moshi-hook`;
- first connection, including non-interactive SSH `PATH` and workspace/session
  selection;
- a recommended mobile operating loop from Moshi notification to Questmancer
  summons, counsel, observation, and review;
- optional Herdr `plugin_action` keybindings for Questmancer `open`, `guild`,
  and `delve`, using user-selected safe chords;
- local voice dictation with review-before-send, plus Moshi Chat View, diff
  viewer, and browser preview as complementary inspection surfaces;
- privacy, authority, and safety boundaries;
- troubleshooting and explicit evidence limits for pane targeting, native
  portraits, and visual acceptance.

## Authority and privacy contract

The runbook must state that Herdr remains authoritative for topology, pane
identity, agent presence, and agent attention. Questmancer remains the local
party-level presentation and command surface. The runbook must not recommend
publishing Questmancer Chronicle/state data to Moshi webhooks or treating Moshi
notifications, Chat View, or transcript parsing as Questmancer domain truth.

Moshi's local transcription options should be identified separately from its
cloud transcription option. The reader must be told to review dictated counsel
before pressing Enter.

## Acceptance

Documentation review must confirm that every command and action matches the
current plugin manifest and README, that no step asks the reader to stop or
reconfigure an unowned Herdr server, and that optional Moshi capabilities are
labelled as optional. Run the repository's documentation/workflow checks and
`git diff --check`; no live Moshi or visual success should be claimed without
fresh direct evidence.
