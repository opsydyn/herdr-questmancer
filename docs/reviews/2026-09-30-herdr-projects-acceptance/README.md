# M3a.4 prerequisite audit

The user requested “Commit and move to next”. Accumulated Questmancer work was
committed at `4923a05f310477fba6e67e6c89f36b2ea6df47ec` after fresh full
verification: 754 Rust tests across 62 runs, 28 shell tests, formatting,
warnings-denied Clippy and shell syntax. The worktree was clean after that commit.
See the [commit qualification receipt](../2026-09-30-committed-control-plane/README.md).

The promoted [acceptance plan](../../plans/2026-09-30-herdr-projects-acceptance.md)
then stopped at its prerequisite gate. The audit is complete; the operational
trial is blocked and has not run. [Machine-readable evidence](inspection.json)
records binary, registry and pinned-source hashes and the limited read-only scope.

| Current evidence | Consequence |
| --- | --- |
| `/Users/alancurrie/.local/bin/herdr` reports `herdr 0.9.0`; SHA-256 `32b53df09872628059c789a69f02a6b8e29e14ddf26711421f3463f70c1aef17` | Below Herdr Projects' pinned manifest minimum of 0.9.1. No compatible runtime was qualified; the actual server protocol was not queried. |
| No executable `herdr-projects` on the current PATH; local plugin registry contains only `opsydyn.questmancer` | No installed Projects binary/version is available through the examined routes. This is not an exhaustive search of the filesystem. |
| Original temporary source checkout is clean at `4e4548c3c43888e6be1c96906215998f07dc63f0`, version 0.2.34 | The earlier contract remains available for source inspection. It is not an installed or executed binary. |
| [Thread start](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/src/threads.rs#L135-L153) calls `ticker::start` before thread allocation | Normal thread creation cannot satisfy this trial's no-ticker scope. |
| [Adoption](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/src/adopt.rs#L48-L62) requires a previously opened project and starts the ticker; later it can deliver an agent brief | A disposable plain pane alone does not supply a supported no-agent creation path. |
| [CLI context](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/src/cli.rs#L626-L638) sets `detached_ticker = true`; [ticker start](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/src/ticker.rs#L97-L118) can spawn once a project exists | The test-only flag is not a supported production CLI opt-out. Do not pretend pausing or quick cleanup removes launch effects. |
| [Path resolution](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/src/paths.rs#L46-L50) fixes config under the process home; CLI `--root` isolates project data only | Root selection alone is not full configuration isolation. Any later command trial must establish this separately. |

No server API was contacted. Local plugin registry metadata and directory names
were read; no real project home, terminal output or user project record was read.
No upstream binary was built, run or installed. No project, server, socket, pane,
agent, ticker, coordinator or plugin registration was created; no live cleanup
was necessary. The existing registry, installed Herdr and pinned checkout remain
unchanged. Neither installation nor agent/ticker launch was implicitly authorised
to bypass the plan's stop conditions.

## Recommended next slice

Promote the [upstream-writer fixture plan](../../plans/2026-09-30-herdr-projects-writer-fixtures.md).
Use the pinned upstream writer functions in a test-only temporary copy to produce
records, then consume them with Questmancer's actual reader/mapper. This addresses
record compatibility without changing the user's installation or launching an
orchestrator. It would establish upstream-writer fixture evidence only.
Installed-source, compatible Herdr runtime, native interaction and real-work
usefulness remain open; M3b and later roadmap work remain unpromoted.
