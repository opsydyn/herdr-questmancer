# Questmancer scene-first preview: guarded manual test

The scene-first renderer is an experimental developer preview. It is not registered as a Herdr plugin pane, is not packaged for release, and does not replace Questmancer's production UI.

The preview reads the same Herdr session model as Questmancer, but has no interactive agent controls. Codex CLI and Herdr remain authoritative for reporting state and sending work to agents.

## Preconditions

- Work from the Questmancer checkout you intend to test.
- Herdr `0.7.4` is already running. Do not stop it.
- An existing Questmancer plugin link may remain linked. Do not unlink it.
- Start from an existing Herdr shell or pane where `HERDR_SOCKET_PATH` is exported.
- Do not create, close, focus, reply to, or report against another agent just to make the preview interesting. Use only a previously approved disposable test source if state synthesis is required.

## Offline Storybook review

Storybook is the asset and composition review surface. It needs neither Herdr nor running agents, and its controls are development-only.

```bash
cd /Users/alancurrie/Projects/herdr-web-master
just storybook-test
just storybook
```

Inspect every scene-first entry below at its reference viewport and at the listed minimum viewport. The north-star criterion is a dense, continuous 16-bit pixel world where compact adventurers are embedded in the environment rather than sitting inside dashboard panels.

| Story | Reference | Minimum | Truth to inspect |
|---|---:|---:|---|
| RGB Calibration Room | 120x36 | 40x18 | RGB-to-half-block rendering is opaque and continuous. |
| Guild Hall Empty | 160x45 | 80x24 | A prepared hall contains no invented activity. |
| Guild Hall Mixed Party | 160x45 | 80x24 | Working, resting and settled adventurers occupy truthful stations once. |
| Guild Hall Counsel Requested | 160x45 | 80x24 | A blocked adventurer is at the Counsel Bell. |
| Guild Hall Spoils Returned | 160x45 | 80x24 | Fresh completion is a bounded, one-shot effect. |
| Guild Hall Reconnecting | 160x45 | 80x24 | Connection truth appears without erasing the room. |
| Guild Hall Minimum Viewport | 80x24 | 40x18 | Focused crop stays authored; it is never scaled. |
| Delve Active Party | 160x45 | 80x24 | Working party occupies the connected active passage. |
| Delve Mixed States | 160x45 | 80x24 | Every station remains part of one dungeon. |
| Delve Sealed Gate | 160x45 | 80x24 | Blocked adventurer waits at the sealed gate. |
| Delve Reconnecting | 160x45 | 80x24 | Connection truth is visible at the entrance. |
| Delve Minimum Viewport | 80x24 | 40x18 | Focused dungeon crop retains material and scale. |
| Scene-First Full Motion | 160x45 | 80x24 | Visible motion has purposeful, deterministic cadence. |
| Scene-First Reduced Motion | 160x45 | 80x24 | Only reduced idle movement remains. |
| Scene-First No Motion | 160x45 | 80x24 | Static scene has no animation wake-up. |
| Scene-First Minimum Viewport | 80x24 | 40x18 | RGB scene remains legible at the review minimum. |

Use `j`/`k` to select a Storybook entry, `Enter` to inspect, `Esc` to return, `?` for help and `q` to quit. These keys belong to Storybook only.

## Guarded live preview

Verify the socket is inherited, then build and launch only the feature-gated binary. This does not link, open, close, or modify the production plugin.

```bash
test -n "$HERDR_SOCKET_PATH"
cargo build --features scene-preview --bin questmancer-scene-preview
cargo run --features scene-preview --bin questmancer-scene-preview
```

Observe the following state truth. Mark unavailable synthetic transitions as `BLOCKED`, never as passed by inference.

| Check | Expected preview result |
|---|---|
| Working | Delve, with the adventurer at an active passage. |
| Blocked | Guild Hall, with the adventurer at the Counsel Bell. |
| Done | Fresh Spoils theatre, then the settled return after its bounded window. |
| Idle | Guild Hall Hearth. |
| Exited | No adventurer body remains in either scene. |
| Reconnecting | Current room/dungeon remains visible with connection truth. |
| Narrow terminal | Automatic camera crop, no scaling or replacement dashboard. |
| Full/reduced/no motion | Cadence follows the selected display preference. |

The preview accepts only plain `q`, `Ctrl-C`, input-stream closure, and process signals as exits. `1`, `2`, arrows, Enter, `r`, `/`, Space, mouse events and paste are intentionally ignored: the preview must not focus panes, send counsel, read output, mark summons read, persist preference changes, or mutate agent state.

Exit with plain `q`, then confirm the normal terminal returns. Do not stop Herdr or unlink the existing plugin as part of this test.

## Record, do not decide by implication

Copy observations into [`docs/superpowers/reviews/2026-07-17-scene-first-cutover.md`](../superpowers/reviews/2026-07-17-scene-first-cutover.md). The preview provides evidence only. Production cutover remains a separate, explicit decision.
