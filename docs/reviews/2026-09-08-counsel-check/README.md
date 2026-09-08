# Counsel input and delivery check — 2026-09-08

The user reported that counsel text entry produced no state changes. Their
subsequent screenshot identifies `rune-scholar`, one of the synthetic visual
review adventurers created with a fixed blocked report. The user subsequently confirmed that the unchanged synthetic state explained
the issue; no counsel input defect remains reported.

The current production release binary was checked in a fresh, test-owned
Herdr 0.9 server. Literal key presses appeared in the managed Questmancer
counsel editor; Backspace removed one character and typing restored it.
Submitting delivered exactly `counsel-probe` followed by a newline to one
owned Python line receiver, and the editor closed after confirmation. The
receiver never evaluated the message as shell code. All four synthetic
presence reports stayed unchanged until explicitly updated by the harness.
See `isolated-runtime.json` for the binary hash and complete cleanup receipt.
All test identities, panes, plugin link and the owned server were cleaned up.
The shared server and existing demo panes were not changed.

The focused input, interaction and scene-overlay suites also passed: 105 tests.
No counsel implementation change was needed for the exercised path. This
check does not prove physical Ghostty typing on the user’s pane, paste support,
or a real agent reacting to counsel.

Herdr owns presence. Successful counsel confirms text and Enter delivery; it
does not manufacture a working report. The synthetic demo adventurers have no
agent process that reacts to counsel by reporting a new state. Their blocked
poses therefore remain until the test source supplies a working report.
