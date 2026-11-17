# Supported Pi integration qualification — 2026-11-17

- **result:** complete
- **claim:** Installed Herdr's supported Pi integration and qualified native session identity and real working/resting capture in a disposable isolated session.
- **evidence:** Host-captured installation status, one successful Pi turn, two persisted Chronicle records, reload/restart checks and owned-resource cleanup; see [verification.json](verification.json).
- **remaining uncertainty:** Existing user Pi sessions have not loaded the integration; blocked transitions and native visual acceptance were not exercised. The user's configured model was rejected by the account.
- **next:** Manually `/reload` each existing Pi session when ready, then inspect fresh identity metadata. Address the model choice separately.

## Installation and scope

Executed `herdr integration install pi`. `herdr integration status` reports Pi
**current (v9)** at `~/.pi/agent/extensions/herdr-agent-state.ts`. Herdr is
0.9.5 / protocol 22; Pi is 1.5.2. Other integrations were not changed.

The test used an isolated named Herdr server, two disposable panes, temporary Pi
configuration and Questmancer state. Pi discovered the installed extension through
a symlink in its temporary extensions directory. A temporary symlink referenced
the existing authentication store; it was removed after Pi stopped. No credentials
or raw session identity are included in this review pack.

The current release binary was used without rebuilding. Its SHA-256 is recorded
in the receipt. No application source, user model settings, real agent session,
shared-server configuration or plugin registration was changed. Existing unrelated
worktree changes were preserved; nothing was committed or published.

## Observed behaviour

1. A new interactive Pi session supplied a complete native `herdr:pi` / `pi`
   path identity with a terminal ID. Questmancer startup established a quiet
   baseline with no retrospective records.
2. With tools, MCP, skills, context files and prompt templates disabled, the
   disposable agent answered the arithmetic prompt with `PI_QUALIFY 3973`.
   The inference used the existing account; startup offline mode does not make
   inference local or offline.
3. Herdr reported `idle → working → idle` with unchanged session identity.
   Questmancer persisted exactly two v2 records with `pane_metadata` evidence:
   working and idle. Experience remained zero.
4. `/reload` confirmed that extensions were reloaded. The complete identity
   remained stable and no history was added. An initial harness expectation that
   an identical report must increment the pane revision was incorrect; reload
   was instead confirmed from the owned Pi's explicit reload acknowledgement.
5. Closing and reopening only the disposable Questmancer UI preserved Chronicle
   bytes and zero experience.

The first attempt used the user's configured `openai-codex/gpt-5.5` and received
an explicit unsupported-model error for the ChatGPT account. Even that failed
turn reported working then idle. The successful qualification used
`openai-codex/gpt-6.1-sol`, low thinking, **only in temporary settings**. The
global default remains `gpt-5.5`. Resting is not proof of successful work.

This v9 hook publishes working, blocked and idle, with no explicit done report.
This run qualifies working/resting and session identity, not blocked/completion
visuals, snapshot-only capture or a successful-delivery/XP transition. Historical
Pi completion evidence applies only to its recorded configuration.

## Cleanup and remaining rollout

The owned Pi report was released before its pane was closed. Both test panes
returned to shells and were closed. The isolated server's final pane, agent,
plugin and plugin-log inventories were empty; only that verified named server
was stopped, and its socket disappeared. Private test evidence was retained.

All nine original shared-session panes retained their terminal IDs and focus
was unchanged. The existing five Pi sessions and synthetic dogfood party were
left open and received no input. The final metadata snapshot still has six
unqualified adventurers: five Pi sessions awaiting reload and one synthetic
identity. The warning therefore need not disappear immediately after installation.

The separate potentially stale integration-banner behaviour was not changed.
After reloading sessions, verify actual metadata before judging the banner; a
fresh Questmancer process can distinguish stale UI text from missing identity.

Recommendation: **needs follow-up** — finish manual activation in the existing
Pi sessions and resolve their model configuration when convenient.
