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

If `herdr plugin list` already shows `opsydyn.questmancer` as a local link to
this checkout, rebuilding the release binary is sufficient. Close and reopen
the Questmancer pane; do not relink it.

For terminal-free visual review:

```bash
just storybook
```

Review all twenty-five fixed stories, including the core world masters, core
portrait masters, native Artificer, Barbarian, Bard, Cleric, Druid, Paladin, Ranger, Rogue, Testmender, Wizard, Goblin and Orc cards, the Goblin
Easter egg, the persistent Librarian, and the Librarian's Ledger. Storybook does
not connect to Herdr, invoke plugin actions, write state or send text. In
Ghostty, confirm the header reports `portrait: native Kitty`; on unsupported
terminals it must report `portrait: authored sprite fallback` and preserve the
24x32 authored card sprite.

For a native portrait in a Herdr-managed pane, opt into Herdr's graphics bridge:

```toml
[experimental]
kitty_graphics = true
```

Run `herdr config check`, then `herdr server reload-config`. Only change or
reload a shared server when its owner has approved the operation. A compatible
outer terminal without this Herdr setting must use the authored sprite fallback
and must never leave the portrait region blank.

### Optional sidebar marginalia

In a user-owned Herdr `config.toml`, add the opt-in rows from
[the README](../../README.md#optional-herdr-sidebar-marginalia). Run
`herdr config check`, then reload the server configuration only with the
owner's approval. Open or reconnect Questmancer and confirm:

| Herdr sidebar target | Expected display-only token |
|---|---|
| Each agent row | `$quest_role` and a truthful `$quest_omen` |
| Each workspace row | `$quest_campaign` with party and summons count |

The rows must not change Herdr's title, state icon, focus, agent identity or
task. Use a synthetic blocked report to confirm `seeks counsel` and an updated
campaign summons count. Restore the user's original sidebar configuration after
the test if it was changed solely for this procedure.

Row styling requires Herdr `0.7.5` or newer; `herdr config check` must report
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

Confirm version `0.1.0`, local source, all five actions, and exactly one
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
| Counsel | `r` opens parchment; submit only to the disposable synthetic agent. |
| Acknowledge | `Space` marks the current blocked episode seen locally. |
| Librarian | The Librarian remains visible and clickable in every full-party Guild Hall composition. Clicking the Librarian opens the same fixed handbook as `?` without changing adventurer selection. |
| Librarian's Ledger | `?` opens or closes the handbook; left/right pages it; `Esc` closes it. No counsel, observe or selection command passes through the open Ledger. |
| Native portrait | Artificer, Barbarian, Bard, Cleric, Druid, Paladin, Ranger, Rogue, Testmender and Wizard classes use their transparent PNGs when the complete pane transport reports Kitty, Sixel or iTerm2 support. Class remains primary regardless of ancestry: an Orc Ranger uses the Ranger portrait. Goblin and Orc art is reserved for future event/NPC storytelling. The Librarian's Ledger uses its native Librarian illustration when available. Other identities and unsupported transports retain the authored class sprite. |
| Narrow viewport | The Guild Hall recomposes from a compact whole-party room to a priority-adventurer vignette and finally status-only rendering; the Delve retains its authored camera crop. Neither switches to a text dashboard. |
| View continuity | Selection remains coherent when switching with `1` and `2`. |
| Urgency jump | `!` selects an adventurer that is waiting on you, in one press, and cycles them when several are. With nobody waiting the selection does not move and the notice reads `No adventurer is waiting on you.` |
| Set aside | `s` on a summoned adventurer reports `Set aside for 15 minutes.`; `!` then skips that adventurer while the summons and its `NEEDS COUNSEL` state both remain visible. `s` on an adventurer with no summons says so instead. |
| Campaign navigation | `Tab` moves the selection into another campaign's party and wraps. With the whole party on one campaign it stays put and says so. |
| Chronicle | `c` opens the guild's record, newest first, scoped to the selected adventurer or the whole guild when none is selected. `j`/`k`, arrows and the wheel scroll it. No key moves the party while it is open. `Esc` or `c` closes it. |
| Search cycling | `/` with a query matching several adventurers reports `1/N matching …`; `n` and `N` walk the matches in both directions and wrap. |
| Scrying scroll | `o` on an adventurer with long output scrolls with `j`/`k`, arrows or the wheel, reaches text below the first screenful, and stops at the last line rather than scrolling into blank space. |
| Counsel draft | Type into the `r` parchment, press `Esc`, and the notice reads `Draft kept.`; pressing `r` again restores the text. Selecting a different adventurer shows a blank parchment, and returning restores the first draft. Sending clears it. |
| Display toggles | `m` cycles motion through full, reduced and still; `u` switches Unicode and ASCII glyphs; `p` switches truecolour and sixteen colours. Each reports the setting it landed on, and each survives closing and reopening Questmancer. |
| Keyring | `?` reaches the Questmancer's Keyring page. Every binding used above appears there, including `!`, `s`, `c`, `n`/`N` and `Tab`. |
| Guild standing | A badge sits in the top-right corner reading rank and experience. Opening an adventurer card does not cover it. The Ledger's Guild's Standing page shows the same figures with the amount owed to the next rank. |
| Standing is earned | With a disposable agent, a `working` to `idle` transition that records returned spoils raises the score; a `blocked` report does not. The score never falls, and it survives closing and reopening Questmancer. |
| Ribbon | Opening Questmancer without pressing anything shows the command ribbon, including `[?] Keys`. It fades a few seconds after the first keypress and returns on activity. |
| Goblin outbreak | `/`, then `release the goblins`, then `Enter` puts goblins on screen for about three seconds in both `1` and `2` — Guild Hall doorway and shelves, Delve entrance-left and centre-bottom. No adventurer moves or disappears while they are loose, the room returns to normal on its own without further input, and a close/reopen never brings them back. |

Do not infer unobserved states. In particular, Herdr `0.8.0` cannot synthesize
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
