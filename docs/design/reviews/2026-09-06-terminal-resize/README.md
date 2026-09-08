# Terminal resize acceptance attempt

Date: 2026-09-06. The Questmancer authorised a terminal resize pass after
approving the updated card fallbacks. Native Ghostty visual acceptance remains
blocked: Computer Use refused access to `com.mitchellh.ghostty` for safety
reasons. No alternative route was used to control or capture that application.

## Preview defect corrected

The current Storybook rejected world scenes below `80x24` terminal cells,
although production supports smaller roster, vignette and status-only layouts.
Its card stories similarly stopped at `80x28`. This prevented the documented
resize review from reaching those production paths.

World, card and interaction stories now delegate every positive size to their
production renderer. Authored sprite galleries retain their `80x28` minimum,
so they still explain when there is insufficient room for their layout.
Reference viewports, the 34-story catalogue and production application
behaviour are unchanged.

A regression first failed at Guild Hall `64x20`. It now checks every scene
fixture at `64x20`, `30x15`, `24x13` and `16x9`, and verifies that galleries
retain their minimum-size message. All six Storybook tests pass. The full
`just verify` gate passed: **552 Rust tests across 50 test runs, 26 shell tests,
formatting, Clippy with warnings denied and script syntax**.

## Executable check

The companion check launches only its own Storybook child process and PTY.
It obtains story identities and navigation positions from the compiled
production catalogue. No Herdr server, pane, registration, focus or counsel
command is involved. The PTY reports no native graphics capability, so card
and Ledger inspection uses the authored sprite fallback.

See [the machine-readable result](result.json). Ten selected stories cover both
worlds, the three updated cards, the Librarian's Ledger, Librarian artwork and
three pose galleries. Each runs the terminal-cell sequence `160x45 → 100x30 →
64x20 → 30x15 → 24x13 → 16x9 → 160x45`. The harness checks emitted redraws,
minimum messages and cursor bounds, then normal exit, restored terminal modes,
visible cursor and departure from the alternate screen.

The check passed **all 70 resize cases** on 2026-09-06. The owned child exited
with code 0, restored the original terminal modes, made the cursor visible and
left the alternate screen. The result records the tested binary's SHA-256.

The Questmancer approved the reported Storybook correction and executable
resize results on 2026-09-06 with “Approved”. No native terminal observation
accompanied that approval; the Ghostty visual review below remains open.

This exercises executable input and resize handling. It does not establish
native font appearance, colour fidelity, visual recognition, window resizing,
mouse interaction or live Herdr acceptance. The separate Librarian and full
pilot visual approvals remain open.

To reproduce the executable check:

```bash
bash docs/design/reviews/2026-09-06-terminal-resize/run.sh
```

The script builds the current development binary and compiles a small catalogue
reader against those exact Cargo artifacts. It owns and cleans up its child,
PTY and temporary files. It does not create a second renderer.

## Remaining native review

Run `just storybook` in a new Ghostty window. Use Enter to inspect the selected
story at the full terminal size. Review Guild Hall and Delve while resizing
through the sizes above; `j`/`k` change stories and `h`/`l` change categories.
Check the Librarian, three pilot pose galleries and class-card stories, then
use `q` to exit. Asset galleries deliberately show their minimum-size message
when too small. Observe native and authored portrait routes separately; a
working native image does not certify the fallback, and neither certifies a
Herdr-managed graphics transport.

Record the actual terminal, dimensions, screenshot and reviewer decision in
this receipt before marking native visual acceptance complete. No commit or
release was made during this pass.
