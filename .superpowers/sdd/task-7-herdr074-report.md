# Task 7: Herdr 0.7.4 verification report

Date: 2026-07-15
Repository: `/Users/alancurrie/Projects/herdr-web-master`

## Environment protection

- Pre-existing Herdr server was left running.
- Pre-existing local `opsydyn.webmaster` link was preserved.
- Only the webmaster pane and one dedicated plain test pane created by this run were closed.
- The original Codex pane `w2:p1` remains.

## Verification evidence

| Check | Result | Evidence |
|---|---|---|
| Herdr version | PASS | `herdr --version` → `herdr 0.7.4` |
| Protocol/version snapshot | PASS | `herdr api snapshot` → `version: "0.7.4"`, `protocol: 16` |
| Release build | PASS | `cargo build --release` completed successfully |
| Local plugin link | PASS | `herdr plugin link .` returned `plugin_linked` |
| Registration/actions | PASS | `opsydyn.webmaster` enabled; `open`, `close`, `toggle`, `desk`, `cafe` present |
| Singleton open | PASS | `open` reused the managed pane; only one webmaster pane existed during the run |
| Desk render | PASS | `pane read w2:p4 --source recent-unwrapped` rendered the control centre |
| Café render | PASS | Same read after `cafe` rendered the connected-bay scene |
| Desk/café switching | PASS | `desk`, `cafe`, and `desk` action logs all succeeded |
| Managed pane excluded | PASS | Desk showed only the real Codex contributor; the webmaster pane was absent from sites/mail/agent selection |
| Managed pane excluded from café | PASS | Fresh 0.7.4 smoke opened managed pane `w2:p6` (`label: webmaster`), switched to café, and read its output. The café contained only `codex` as a workstation; neither `webmaster`, `w2:p6`, nor `unknown agent` appeared. |
| Dedicated blocked path | PASS | Plain pane `w2:p5` in `/private/tmp` accepted `webmaster-smoke` blocked report; snapshot contained blocked agent |
| Blocked UI | PASS | Desk showed `NEW webmaster-smoke - NEEDS WEBMASTER`; café showed `HELP!` |
| Done transition | BLOCKED | Herdr 0.7.4 CLI still has no supported `done` report state |
| Logs | PASS | Logs 1–6 (`open`, `desk`, `cafe`, `desk`, `cafe`, `close`) all succeeded with empty stderr |
| Cleanup | PASS | Synthetic source released; `w2:p5` closed; webmaster pane closed |
| Final environment | PASS | Final snapshot contains only pre-existing Codex pane `w2:p1`; Herdr remains 0.7.4 |
| Git status | PASS | `## main` |

## Concerns

- `herdr api ping` is not exposed by the installed 0.7.4 CLI; the snapshot command is the available version/protocol proof.
- The earlier blocked test over an agent-owned pane was invalid. This run used a dedicated unowned plain pane and the blocked state was visible end-to-end.
- The café still needs the approved authored pixel-world redesign work; this task only verifies the current build and records the 0.7.4 sidebar follow-up.
- A second live café assertion was run after the original cleanup. The temporary managed pane `w2:p6` was visible to Herdr but absent from café workstations; only the pre-existing Codex workstation rendered.

## Follow-up

The smallest sidebar opportunity is recorded in `docs/superpowers/plans/2026-07-15-herdr-074-sidebar-integration.md`. No sidebar behavior was implemented here.
