# A quiet guild — one cat reaction

2026-09-08. The Questmancer approved implementation after approving the keepsake
artwork. The Questmancer approved this reaction on 2026-09-08. Native observation
remains separate.

[Production review sheet](cat-reaction.png) shows the existing sleeping cat,
head lift, stretch and settled pose, enlarged and at native scale, plus the
canonical Hall. [Playback](cat-reaction.gif) plays the event once and ends
asleep. These are production RGB exports, not native terminal captures.

## Behaviour

- The same known, non-empty party must change from not-all-resting to all Idle.
  This reflects presence only; it does not claim completed work or clear summons.
- Runtime compares accepted live facts before and after reduction, including
  refreshed snapshots used by Herdr 0.9. Party membership includes both stable
  adventurer identity and pane identity; departed adventurers are excluded.
- Startup and reconnect snapshots establish baselines. Unknown/empty parties,
  membership changes and stale/duplicate reports cannot start or replay it.
  Work resuming, changed membership or a connection boundary cancels it.
- Two authored 400 ms frames fit the sleeping cat's existing 10x5 reservation.
  The 800 ms boundary returns to the original sleeping master. Asset timing
  feeds `SceneFrame.next_frame_in`; expiration needs no cleanup timer.
- Only the canonical Hall in full motion animates. Smaller Hall tiers, Delve
  and reduced/still motion add no cat wakes. Scene time determines progress,
  so returning to the Hall does not restart the event.
- An ephemeral start timestamp belongs to Model/presentation. No live facts,
  scene snapshot, persistence schema, actor positions, targets or commands change.

## Verification and review

Tests cover runtime snapshot/status paths, two-adventurer aggregation, stale
reports, initial/reconnect baselines, membership changes, unknown/empty parties,
work resumption, view/size/motion limits, exact deadlines and the pixel reservation.
The reaction leaves persisted state and the live scene projection unchanged.
`just verify` passes 582 Rust tests across 53 runs and 28 shell tests,
formatting, Clippy with warnings denied and shell syntax. Release build and
review regeneration also pass. [Verification receipt](verification.json). Native visual and live Herdr observation remain separate.

Regenerate with a Python containing Pillow:

```bash
python3 docs/design/reviews/2026-09-08-cat-reaction/regenerate.py
```

The exporter drives the actual runtime reduction and production RGB renderer
with local fixtures; it creates no Herdr panes or agent processes. Reopen the
source-linked Questmancer pane to load a rebuilt release binary for native
review. Observe a real transition without sending synthetic counsel to real
adventurers. The guarded manual-test guide owns any disposable live test.

The previous clean 0.1.9 qualification still describes `c3720a9`; keepsakes and
this reaction are subsequent uncommitted changes. No tag or publication occurs
in this slice. Factual Chronicle chapters remain the next proposed delight item.
