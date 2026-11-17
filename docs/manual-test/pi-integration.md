# Pi integration activation and acceptance

Use Herdr's supported installer:

```bash
herdr integration install pi
herdr integration status
```

On the qualified 2026-11-17 host, Herdr 0.9.5 installs Pi extension v9 into
`~/.pi/agent/extensions/herdr-agent-state.ts`; Pi 1.5.2 discovers it automatically.
Do not edit the managed extension. Custom extensions belong beside it.

For an already running Pi session, wait until it is ready and enter:

```text
/reload
```

Pi should confirm that extensions were reloaded. A new interactive Pi session
also loads the extension. Neither action requires restarting a shared Herdr
server. Never reload another person's or an unknown agent's session.

Inspect `herdr agent list` or `herdr pane get <fresh-pane-id>` without reading
terminal output. The Pi row should have a non-empty `terminal_id` and complete
`agent_session` fields: `source`, `agent`, `kind`, `value`. The native source is
`herdr:pi`, agent `pi`. Keep raw identity metadata out of public receipts.

Once Questmancer has a qualified baseline, a user-authorized test prompt should
produce working then resting and semantic Chronicle records without duplicates.
The current hook's idle report does not prove success and must not award XP.
Reload/restart should preserve history rather than backfill it.

A disposable real-agent check passed with `openai-codex/gpt-6.1-sol`; the host's
configured `gpt-5.5` was rejected by its ChatGPT account. The test did not change
global defaults. Select a model supported by the account before using a test
answer as evidence; an error can also settle to idle.

The retained synthetic dogfood identity intentionally lacks native session
identity, so a remaining withheld-observations warning can still be accurate.
Verify metadata and the warning's count independently. See the
[qualification receipt](../reviews/2026-11-17-pi-integration/README.md).
