# Installed Herdr upgrade — 2026-10-15

The user approved upgrading the host. The official stable updater selected
**Herdr 0.9.5 / protocol 22**, released after the earlier 0.9.4 qualification.
[0.9.5 is a hotfix for 0.9.4](https://github.com/herdrdev/herdr/releases/tag/v0.9.5),
including prefix/input fixes and revised Codex integration behaviour.
Questmancer's source minimum remains 0.9.4. Source changes remain uncommitted;
no Questmancer release was published.

## Installation and settings

`/Users/alancurrie/.local/bin/herdr` reports 0.9.5 and matches the official
macOS ARM64 asset SHA-256 recorded in [verification.json](verification.json).
The prior binary and integration/config files are backed up in the temporary
upgrade directory recorded there. No server was running at the start or end.
No user server, pane or plugin link was stopped or changed by qualification.

The updater identified two previously installed outdated integrations:

- Claude upgraded from v7 to v11; JSON settings parse, and settings outside
  Herdr's hook configuration are unchanged.
- Codex upgraded from v6 to v10; its TOML parses and is semantically unchanged.
  Hook JSON parses. Upstream removes Codex turn hooks; current status comes
  from screen detection, so earlier hook-based completion evidence does not
  qualify this new integration's real-agent behaviour.

All other agent integrations remain uninstalled. `herdr integration status
--outdated-only` reports no outstanding updates.

## Verification

- Current `cargo build --release` passed; candidate hash matches the October 14
  compatibility candidate. No production Rust changed during this host upgrade.
- The installed 0.9.5 binary passed a fresh [isolated production-plugin runtime](isolated-runtime.json):
  four synthetic states, singleton open, Guild/Delve/Guild actions, status changes,
  managed-pane exclusion and complete owned-resource cleanup.
- [Automated PTY authoring/restart](pty-receipt.json) passed: explicit store,
  campaign, quest, request, multiline answer and artifact; acknowledged revisions
  0–5; identical authority after restart; answer visible; processes exited and
  temporary state removed. This test creates no Herdr resources and does not
  establish native appearance or unassisted usability.
- The older PTY harness writes the repeat screen to the October 3 review folder.
  That file is now explicitly labelled as regenerated on October 15. Its current
  [synthetic screen](restart-pty-screen.txt) is retained here too. The structural
  receipt equals the original historical log; approved production images remain
  unchanged.
- Latest complete engineering gate remains October 14: 789 Rust tests / 64 runs,
  28 shell tests, formatting, denied-warning Clippy and shell syntax passed.
- The isolated helper now accepts `--expected-version` while requiring the exact
  supplied version and protocol 22. Repeat this run with:

```bash
python3 docs/reviews/2026-10-14-herdr-094/qualify.py \
  --herdr /Users/alancurrie/.local/bin/herdr --expected-version 0.9.5
```

## Next gate

Native acceptance remains unperformed. Computer Use previously denied Ghostty
access; no alternate app or shell UI automation was used. The prepared
[human native trial](../../plans/2026-10-03-owned-authoring-native.md) now checks
this current candidate hash before launching twice against its own temporary
state directory. It covers forms, attention, answer/artifact reads, resize and
restart. Attached-client portraits, mouse and enhanced-key behaviour also need
native observation. Real Codex/Claude behaviour is not inferred from integration
installation or synthetic reports. No agent was launched for this upgrade.
