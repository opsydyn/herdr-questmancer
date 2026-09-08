# Questmancer production pixel world: guarded manual acceptance

This guide tests the production Guild Hall and Delve without damaging an
existing Herdr session. The historical filename is retained so existing links
continue to work; there is no separate scene-preview binary.

## Safety boundary

- Record the current focused pane, open tabs and plugin registration first.
- Do not stop a Herdr server that this test did not start.
- Do not unlink a plugin link that this test did not create.
- Do not send counsel to a real or unknown agent.
- Use a newly created plain pane for synthetic status reports.
- Track every test-created pane and close only those panes during cleanup.

## Build and automated checks

From the linked checkout:

```bash
cargo build --release
bash tests/scripts.sh
cargo test --test cli
```

Scrying ordering and cancellation have deterministic local-socket coverage:

```bash
cargo test --lib runtime_loop::tests
cargo test --test runtime_loop --test command --test interaction --test app
```

These checks cover stale success/failure, selection changes, pane replacement,
connection changes and refresh coalescing while counsel completes. They do
not establish live Herdr acceptance. For a guarded manual check, rapidly
refresh the disposable synthetic adventurer, select another disposable
adventurer and return: the parchment must show only the current selection's
output. Reconnect testing requires a test-owned server; the old preview must
clear and the selected adventurer must load again even if its status did not
change. Record the actual result separately from the automated gate.

If `herdr plugin list` already shows `opsydyn.questmancer` as a local link to
this checkout, rebuilding the release binary is sufficient. Close and reopen
the Questmancer pane; do not relink it.

For review without Herdr or agent processes:

```bash
just storybook
```

Review all thirty-four fixed stories: two worlds, ten sprite/palette galleries,
fourteen native class cards, two reserved Goblin/Orc art cards, and six
interactions. `j`/`k` select within a category; `h`/`l` change category. Include
the Librarian's world sprite and Ledger fallback separately from its native
illustration. Storybook does
not connect to Herdr, invoke plugin actions, write state or send text. In
Ghostty, confirm the header reports `portrait: native Kitty`; on unsupported
terminals it must report `portrait: authored sprite fallback` and preserve the
`24x32` authored card canvas. Wizard, Ranger and Barbarian must show their
new stocky world sprite at native proportions, with matching persona colours
and readable card text. The other classes retain their independent fallbacks.

Press Enter to inspect at the full terminal size. World, card and interaction
stories now reach production layouts at every positive size; sprite galleries
keep their `80x28` minimum. The [2026-09-06 resize receipt](../design/reviews/2026-09-06-terminal-resize/README.md)
records the executable PTY check and the blocked native Ghostty review. Do not
substitute that executable check for visual acceptance.

The three pilot pose galleries show production Wizard, Ranger and Barbarian
frames. Storybook time is fixed; use the [pilot review playback](../design/reviews/2026-09-05-party-pilot/README.md)
to inspect the production renderer sampled at explicit fixture times. This
is not live Herdr acceptance.

For a native portrait in a Herdr-managed pane, Herdr 0.9 enables the bridge
by default. An explicit `terminal.kitty_graphics = false` disables it; the legacy
experimental key remains accepted. Validate with `herdr config check` and follow
[the current transport guide](../troubleshooting/native-portrait-rendering.md).
A compatible outer terminal alone does not establish pane transport support.
Do not restart a shared server without the owner's approval.

### Optional sidebar marginalia

In a user-owned Herdr `config.toml`, add the opt-in rows from
[the README](../../README.md#optional-herdr-sidebar-marginalia). Run
`herdr config check`, then reload the viewing client configuration. Open or reconnect Questmancer and confirm:

| Herdr sidebar target | Expected display-only token |
|---|---|
| Each agent row | `$quest_role` and a truthful `$quest_omen` |
| Each workspace row | `$quest_campaign` with party and summons count |

The rows must not change Herdr's title, state icon, focus, agent identity or
task. Use a synthetic blocked report to confirm `seeks counsel` and an updated
campaign summons count. Restore the user's original sidebar configuration after
the test if it was changed solely for this procedure.

Use the supported Herdr `0.9.0` / protocol `22`; `herdr config check` must report
`config: ok` before opening Questmancer, and an invalid row takes the whole
file down to defaults rather than being skipped.

### Optional urgency ordering

With `sidebar_urgency_order = true` in Questmancer's own configuration, confirm
that Herdr's agent list leads with an adventurer that is waiting on a human,
matching the order `!` walks inside Questmancer. Confirm also that **no agent
disappears** — the view sorts and must never filter — and that closing
Questmancer returns Herdr's list to its own order. Reconnecting after a server
restart must restore the ordering, since Herdr's view is transient.

## Registration and singleton

```bash
herdr plugin list --json
herdr plugin action invoke opsydyn.questmancer.open
herdr plugin action invoke opsydyn.questmancer.open
```

Confirm the version matches the current `herdr-plugin.toml` (`0.1.9` at this
revision), local source, all five actions, and exactly one
Questmancer pane after the repeated `open`.

## Production interaction pass

| Check | Expected evidence |
|---|---|
| Guild Hall | `1` shows the full RGB guild scene. |
| Delve | `2` shows the full RGB dungeon scene. |
| Selection | `j`/`k`, arrows and `g`/`G` move one in-world selection rune. |
| Observe | `Enter` focuses the selected real Herdr pane. |
| Search | `/` opens parchment, filters the party, and `Esc` cancels. |
| Scrying | `o` opens recent output for the selected adventurer. |
| Counsel | `r` opens parchment; submit only to the disposable synthetic agent. A correlated confirmed text-and-submit result adds a red seal to the existing notice for six seconds. Draft/sending/rejected/uncertain/submit-failed outcomes have no seal and keep their recovery actions. Confirmation must not change blocked presence. |
| Pilot working and counsel | Wizard, Ranger and Barbarian alternate two working frames every 500 ms. A new blocked episode gestures for 600 ms and then stays still. Repeated reports, a room switch or reconnect must not restart the gesture. Only a later working report resumes the ritual. |
| Pilot spoils | Fixture coverage: a confirmed done transition carries the class-specific parcel, places it at 1000 ms and ends effects at 3000 ms. Completed, resting and unknown remain distinct. Herdr 0.9.0 synthetic reports cannot establish a live done transition. |
| Quiet motion | Reduced/still mode shows a static meaningful pose for all pilot states, including fresh done, with no decorative or cleanup wake. |
| Acknowledge | `Space` marks the current blocked episode seen locally. |
| Librarian | The revised Librarian sprite is visible and clickable in canonical and compact Halls. Clicking it opens the same fixed handbook as `?` without changing adventurer selection. Roster, vignette and status-only tiers omit the actor; `?` remains available. Review the enlarged face, books and short robe; the wide Ledger must show its independently authored fallback when native graphics are unavailable. |
| Librarian's Ledger | `?` opens or closes the handbook; left/right pages it; `Esc` closes it. No counsel, observe or selection command passes through the open Ledger. |
| Native portrait | All fourteen classes (including Mage, Sorcerer, Runewright and Pathseeker) use distinct transparent PNGs when the complete pane transport reports Kitty, Sixel or iTerm2 support. Class remains primary regardless of ancestry: an Orc Ranger uses the Ranger portrait. Goblin and Orc art is reserved for future event/NPC storytelling. The Librarian's Ledger uses its native Librarian illustration when available. Unsupported transports or preparation failures retain the authored class portrait; the Ledger retains its Librarian fallback. |
| Narrow viewport | The Guild Hall checks canonical, compact, authored `8x12` roster, priority `16x24` vignette and status-only tiers in that order. The Delve uses a roster below `100x56` RGB when the viewport is at least `20x27` and the whole party fits; otherwise it retains its camera crop and station overflow. Neither switches to a text dashboard. |
| View continuity | Selection remains coherent when switching with `1` and `2`. |
| Urgency jump | `!` selects an adventurer that is waiting on you, in one press, and cycles them when several are. With nobody waiting the selection does not move and the notice reads `No adventurer is waiting on you.` |
| Set aside | `s` on a summoned adventurer reports `Set aside for 15 minutes.`; `!` then skips that adventurer while the summons and its `NEEDS COUNSEL` state both remain visible. `s` on an adventurer with no summons says so instead. |
| Campaign navigation | `Tab` moves the selection into another campaign's party and wraps. With the whole party on one campaign it stays put and says so. |
| Chronicle | `c` opens the guild's record, newest first, scoped to the selected adventurer or the whole guild when none is selected. `j`/`k`, arrows and the wheel scroll it. No key moves the party while it is open. `Esc` or `c` closes it. |
| Search cycling | `/` with a query matching several adventurers reports `1/N matching …`; `n` and `N` walk the matches in both directions and wrap. |
| Scrying scroll | `o` on an adventurer with long output scrolls with `j`/`k`, arrows or the wheel, reaches text below the first screenful, and stops at the last line rather than scrolling into blank space. |
| Counsel draft | Type into the `r` parchment, press `Esc`, and the notice reads `Draft kept.`; pressing `r` again restores the text. Selecting a different adventurer shows a blank parchment, and returning restores the first draft. Sending clears it. Closing an in-flight parchment does not turn its text back into a fresh draft; its correlated result still settles the original attempt. |
| Display toggles | `m` cycles motion through full, reduced and still; `u` switches Unicode and ASCII glyphs; `p` switches truecolour and sixteen colours. Each reports the setting it landed on, and each survives closing and reopening Questmancer. |
| Keyring | `?` reaches the Questmancer's Keyring page. Every binding used above appears there, including `!`, `s`, `c`, `n`/`N` and `Tab`. |
| Roster state cues | At 30 columns × 15 terminal rows (`30x30` RGB), working shows a tool, counsel a lantern, resting a Z, completed a check, and unknown a question mark in both rooms. At 30 × 24 terminal rows, six actors and their cues fit in two rows; selection rings and connection indicators stay clear of the cues. At 60 × 15, available nameplates sit clear of cues and rings. Repeat with `p` and `u` for 16 colours and ASCII. |
| Roster completion | Use a fixture for synthetic done: full motion shows gutter sparkles until three seconds and then only the completed check; reduced/still modes retain one static check. A newer presence or disconnect ends the return. Reconnect retains the summons without replaying it. Confirm real-agent completion only after actually observing it. |
| Guild standing | A badge sits in the top-right corner reading rank and experience. Opening an adventurer card does not cover it. The Ledger's Guild's Standing page shows the same figures with the amount owed to the next rank. |
| Standing is earned | Only an explicit done transition recording returned spoils earns 10 points; a campaign-closed event earns 25. Working to idle, blocked, and other presence events earn none. Use fixtures for synthetic done. The score never falls, and it survives closing and reopening Questmancer. |
| Ribbon | Opening Questmancer without pressing anything shows the command ribbon, including `[?] Keys`. It fades a few seconds after the first keypress and returns on activity. |
| Goblin outbreak | `/`, then `release the goblins`, then `Enter` puts goblins on screen for about three seconds in both `1` and `2` — Guild Hall doorway and shelves, Delve entrance-left and centre-bottom. No adventurer moves or disappears while they are loose, the room returns to normal on its own without further input, and a close/reopen never brings them back. |

Do not infer unobserved states. In particular, Herdr `0.9.0` cannot synthesize
`done`; fixture tests are not a substitute for live visual acceptance.

## Optional disposable agent

Create a plain pane, capture its ID, then use one unique source:

```bash
PANE_ID=$(herdr pane current | jq -r '.result.pane.pane_id')
SOURCE_ID="questmancer-smoke-$(date +%s)-$$-$RANDOM"

herdr pane report-agent "$PANE_ID" \
  --source "$SOURCE_ID" \
  --agent smoke-adventurer \
  --state working \
  --message "mapping the dungeon" \
  --seq 1

herdr pane report-agent "$PANE_ID" \
  --source "$SOURCE_ID" \
  --agent smoke-adventurer \
  --state blocked \
  --message "needs counsel" \
  --seq 2
```

Confirm the blocked pose, summons marker, search result, selected-output read and
counsel parchment. Then release the same identity:

```bash
herdr pane report-agent "$PANE_ID" \
  --source "$SOURCE_ID" \
  --agent smoke-adventurer \
  --state working \
  --message "manual test complete" \
  --seq 3

herdr pane release-agent "$PANE_ID" \
  --source "$SOURCE_ID" \
  --agent smoke-adventurer \
  --seq 4
```

## Cleanup and report

1. Release the synthetic report before closing its disposable pane.
2. Close only Questmancer panes created by this test.
3. Restore the original focused pane when it still exists.
4. Leave the pre-existing Herdr server and plugin link running.
5. Run `git status --short --branch` and confirm the test changed no tracked files.
6. Report every item as `PASS`, `FAIL`, `BLOCKED` or `NOT REVIEWED`.

Inspect plugin logs before declaring the environment restored:

```bash
herdr plugin log list --plugin opsydyn.questmancer --limit 50
```
