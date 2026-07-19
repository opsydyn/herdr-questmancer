# Scene-first cutover evidence ledger

Status: **NOT YET DECIDED**

This ledger records visual and operational evidence. It does not authorize a production renderer cutover, removal of the legacy UI, or release packaging of the preview binary.

| Evidence | Result | Notes / command / capture |
|---|---|---|
| Guild Hall visual approval | NOT REVIEWED | |
| Delve visual approval | NOT REVIEWED | |
| Minimum viewport | NOT REVIEWED | 80x24 terminal / 40x18 logical pixels. |
| Full motion | NOT REVIEWED | |
| Reduced motion | NOT REVIEWED | |
| No motion | NOT REVIEWED | |
| Working state truth | NOT REVIEWED | |
| Blocked state truth | NOT REVIEWED | |
| Done/fresh-spoils/settled truth | NOT REVIEWED | |
| Idle state truth | NOT REVIEWED | |
| Exited state truth | NOT REVIEWED | |
| Reconnect behaviour | NOT REVIEWED | |
| Preview ignores legacy action keys | NOT REVIEWED | |
| Terminal restore after `q` | NOT REVIEWED | |
| Idle CPU after 30 seconds static | NOT MEASURED | |
| Active CPU with working animation | NOT MEASURED | |
| Known visual defects | NOT REVIEWED | |
| ANSI-256 decision | NOT DECIDED | Legacy renderer remains authoritative for compatibility modes. |
| Legacy renderer delete-or-dev-only decision | NOT DECIDED | |
| Cutover decision | **NOT YET DECIDED** | Explicit product review required. |

Automated evidence to record alongside manual observations:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
PROPTEST_CASES=4096 cargo test --test scene_pixel_properties --test scene_stage_properties
bash tests/scripts.sh
cargo build --release
```
