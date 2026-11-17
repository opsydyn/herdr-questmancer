# Herdr 0.9.4 compatibility and capability assessment

The user requested updating Questmancer for the latest Herdr and assessing useful
capabilities. GitHub's latest stable release was `v0.9.4` on 2026-10-14.
The current plugin source now targets Herdr **0.9.4 / protocol 22**. This work is
uncommitted and unreleased beside the existing owned-work/authoring changes.
At this qualification the installed Herdr remained 0.9.0; no user configuration
or shared server changed. The subsequent [approved host upgrade](../2026-10-15-herdr-upgrade/README.md)
installed the 0.9.5 hotfix.

## Compatibility changes

Herdr kept protocol 22 but removed `pane.graphics.info`, `set`, `clear` and
`stream` in 0.9.2. The old portrait startup shim called that API and changed PTY
pixel dimensions. It is removed, together with its DTO/client method and seven
obsolete shim tests. Card and Ledger images now rely entirely on the standard
terminal Picker query and Kitty output. Missing geometry, unsupported graphics
or failed preparation retain their authored sprite. The RGB world renderer,
class assets and all existing portrait capability/fallback tests remain in use.

Herdr 0.9.2 also introduced an explicit `events_lost` error when a subscription
falls behind retained history. Previously that response became a generic JSON
error. Questmancer now checks its correlation ID, preserves the server code and
message, then follows the existing reconnect/subscription/post-subscription
baseline path. Recovery does not pretend to replay missing events or earn XP.
A red-first fake-socket test reproduced the old failure; the green test and a
composed supervisor recovery test verify the new diagnostic and baseline.

The manifest minimum and current operating docs are updated. Captures from the
official binary retain unknown optional fields without inventing agent sessions.
The optional `pane.viewport_header.*` extension stays separately negotiated:
published 0.9.4 does not implement it, and the plugin quietly disables it.
Standard Kitty images cannot recreate its separately reserved header row.

## Useful capabilities and recommended scope

| Capability since our 0.9.0 baseline | Questmancer value | Decision |
| --- | --- | --- |
| Standard Kitty rendering, correct graphics disablement, pixel mouse and keyboard fixes | Better transport for card/Ledger images and contextual hit regions; fewer input leaks | Use through existing terminal APIs; perform current native acceptance next |
| Explicit lost-event errors and complete subscription bursts | Truthful recovery when events are dropped | Integrated diagnostic and fresh baseline; live overflow remains untested |
| `herdr plugin update` in 0.9.4 | Convenient updates for GitHub-installed releases while active panes finish on old files | Documented; reopen to use the new version; local links still require a release build |
| Improved Codex/Pi detection, startup/restore completion fixes and retained session identity | More reliable presence, summons and Chronicle inputs | Inherit authoritative Herdr facts; keep real-agent acceptance distinct |
| `--machine` forwarding since 0.9.1 | A future guarded remote campaign workflow | Assess later with explicit per-server identity and failure boundaries; no multi-server aggregator added |
| Self-reported resume commands since 0.9.2 | Third-party agent integrations can retain their own conversations | Herdr/agent-owned; Questmancer must not invent sessions or start agents |
| Durable saves, shutdown handling, process exhaustion and session restore fixes | Stronger execution substrate | Benefit automatically after host upgrade; no plugin persistence changes needed |

Upgrading Herdr would satisfy the historical Projects adapter version prerequisite
(0.9.1), but Projects remains uninstalled and its inventory still lacks native
structured obligations and verified bindings. No Projects trial or new source
attachment is promoted by this compatibility change. Owned-work authoring remains
in its current bounded scope.

The next bounded step is native acceptance on an attached 0.9.4 client: cards,
Ledger, graphics-off fallback, small sizes, mouse selection and authoring
Escape/Shift+Enter. Publication and installed-host upgrading remain separate.

## Qualification

[Machine-readable verification](verification.json) records:

- 47 protocol tests passed; the lost-event regression was first demonstrated red.
- `just verify`: **789 Rust tests / 64 runs, 28 shell tests**, formatting,
  Clippy with warnings denied and shell syntax passed. Baseline was 792 tests;
  seven retired shim tests were replaced by four current contract/recovery tests.
- `cargo build --release` passed. The exact binary checksum is in the receipt.
- An isolated official Herdr 0.9.4 server ran the linked production plugin:
  working, blocked, idle and unknown projections; repeated open; Guild → Delve
  → Guild actions; working → blocked → working; managed-pane exclusion.
- [Runtime cleanup](isolated-runtime.json) passed: released reports, closed only
  owned panes, unlinked only the test link, stopped only the test server, socket
  removed. Four `/bin/cat` receivers keep synthetic reports alive; newer Herdr
  otherwise releases self-reported identities when the pane is an idle shell.
- Headless managed-pane portrait probes with graphics
  [enabled](portrait-true.json) and [disabled](portrait-false.json) safely declined
  native images and kept the authored fallback route. No attached graphics client
  was present. This does not qualify native preparation or visible appearance.
- [Capture cleanup](capture-receipt.json) passed for the fixture generation server.

To repeat production runtime qualification after building the release binary:

```bash
python3 docs/reviews/2026-10-14-herdr-094/qualify.py \
  --herdr /absolute/path/to/official/herdr-0.9.4
```

The script uses fresh temporary config/state/cache directories and never operates
on the installed server. Receipts remain in its reported temporary directory.
Raw verification logs for this run are under `/tmp/questmancer-herdr-094/`.
Native images, actual lost-event overflow, non-macOS execution, installation,
publication and real-agent acceptance on 0.9.4 remain unqualified.

## Sources

- [Herdr v0.9.4 release](https://github.com/herdrdev/herdr/releases/tag/v0.9.4).
- [Pinned cumulative changelog](https://github.com/herdrdev/herdr/blob/7e26bad94044eba205033111a0ce8033ede7e24f/CHANGELOG.md), including 0.9.1–0.9.3 changes.
- [Protocol version](https://github.com/herdrdev/herdr/blob/7e26bad94044eba205033111a0ce8033ede7e24f/src/protocol/wire.rs).
- [Lost-event contract](https://github.com/herdrdev/herdr/blob/7e26bad94044eba205033111a0ce8033ede7e24f/src/api/subscriptions.rs).
- Official macOS ARM64 binary SHA-256:
  `ea20bfff9cb8c83a996081c926e6ff9370de8e543baa859bca48d19175d895ff`.
