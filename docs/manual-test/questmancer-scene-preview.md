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

Review all sixteen fixed stories, including the core world masters, core
portrait masters, native Barbarian, Rogue, Wizard and Goblin cards, and the
Goblin Easter egg. Storybook does
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
| Help | `?` opens the contextual help parchment. |
| Native portrait | Barbarian, Rogue and Wizard classes use their transparent PNGs when the complete pane transport reports Kitty, Sixel or iTerm2 support. Goblin ancestry uses its PNG ahead of class. Other identities and unsupported transports retain the authored sprite. |
| Narrow viewport | The world camera crops without switching to a text dashboard. |
| View continuity | Selection remains coherent when switching with `1` and `2`. |

Do not infer unobserved states. In particular, Herdr `0.7.4` cannot synthesize
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
