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
| Selection, search, counsel and scrying controls | RETAINED | Contextual overlays over the world; live retest still required. |
| `1` / `2` Guild Hall and Delve switching | RETAINED | Production input contract; live retest still required. |
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
