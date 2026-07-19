# Scene-first cutover evidence ledger

Status: **APPROVED FOR PRODUCTION CUTOVER**

This ledger separates the approved visual direction from live-state evidence
that has not yet been observed. Approval does not manufacture acceptance data.

| Evidence | Result | Notes / command / capture |
|---|---|---|
| Guild Hall visual direction | APPROVED | 2026-07-19 production RGB Guild Hall capture. |
| Delve visual direction | APPROVED | 2026-07-19 production RGB Delve capture. |
| RGB renderer as sole production renderer | APPROVED | User approved the renderer and hard cutover. |
| Legacy renderer removal | APPROVED | Removed from production, Storybook and build surfaces. |
| Production linked-pane RGB render | PASS | 2026-07-19: fresh `target/release/questmancer` produced RGB half-block output in `w2:p16`. |
| Guild Hall and Delve switching | PASS | Distinct live frame hashes after `1` and `2`: `14d8d59a...` and `f1940a3e...`. |
| Search parchment | PASS | Opened with `/` and cancelled with `Escape`; no text submitted. |
| Counsel parchment | PASS | Opened with `r` and cancelled with `Escape`; no counsel submitted. |
| Scrying parchment | PASS | Opened with `o` for the selected Codex pane and cancelled. |
| Field guide | PASS | Opened with `?` and cancelled. |
| Observe selected adventurer | PASS | `Enter` focused the original Codex pane `w2:pM`. |
| Multi-adventurer selection | NOT REVIEWED | The live herd contained only one real agent; reducer and scene tests pass. |
| Singleton lifecycle | PASS | Concurrent `open` calls created one pane; test-created pane and tab were removed. |
| Source-link runner freshness | PASS | Release build now precedes stale `bin/questmancer`; shell regression suite covers both paths. |
| Minimum viewport | NOT REVIEWED | 80x24 acceptance remains open. |
| Full motion | NOT REVIEWED | |
| Reduced motion | NOT REVIEWED | |
| No motion | NOT REVIEWED | |
| Working state truth | NOT REVIEWED | |
| Blocked state truth | NOT REVIEWED | |
| Done/fresh-spoils/settled truth | NOT REVIEWED | Herdr 0.7.4 cannot synthesize `done`. |
| Idle state truth | NOT REVIEWED | |
| Exited state truth | NOT REVIEWED | |
| Reconnect behaviour | NOT REVIEWED | |
| Terminal restore after `q` | NOT REVIEWED | |
| Idle CPU after 30 seconds static | NOT MEASURED | |
| Active CPU with working animation | NOT MEASURED | |
| Known visual defects | NONE RECORDED AT APPROVAL | New defects remain reportable. |
| ANSI-256 compatibility | NOT REVIEWED | Configuration remains accepted; RGB scene stays authoritative. |
| Cutover decision | **APPROVED** | Production uses one scene engine and no legacy fallback. |

Engineering verification for the cutover is recorded with:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
PROPTEST_CASES=4096 cargo test --test scene_pixel_properties --test scene_stage_properties
bash tests/scripts.sh
bash -n herdr/install.sh herdr/run.sh herdr/control.sh
cargo build --release
```

The guarded production acceptance procedure lives at
`docs/manual-test/questmancer-scene-preview.md`; its historical filename is kept
only for link stability.
