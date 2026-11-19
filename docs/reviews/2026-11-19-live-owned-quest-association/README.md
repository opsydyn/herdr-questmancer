# Guarded live owned quest association

- **Result** · Passed the guarded association checks on live Herdr 0.9.5 and observed Kitty room fitting and a native portrait in Ghostty 1.4.0 after a window resize and warm launch.
- **Claim** · An explicit link from an owned quest authorizes Observe only while the selected adventurer's Herdr identity remains valid.
- **Evidence** · [Live receipt](receipt.json) for commit `db6be41` and the SHA-256 of the release binary used in the PTY and managed pane.
- **Remaining uncertainty** · Fresh managed panes can still start with zero pixel dimensions. Real-agent execution and the full resize and portrait matrix remain open.

## What ran

The default server accepted the trial's synthetic working report but returned no
session identity. Questmancer also publishes sidebar metadata for its connected
server. The trial released its synthetic identity and closed its disposable
workspace before launching Questmancer on a test-owned server. The default server
retained its original focus and all pre-existing panes.

The test-owned server used a separate configuration, socket and state directory.
A dummy Pi process reported a complete session through Herdr's Pi session route.
Two plain test panes reported idle and blocked without qualified identity. The
release binary exercised the owned-work fixture through a controlled PTY and the
real Herdr socket. This ran before starting the retained managed Questmancer pane.

| Check | Observed result |
| --- | --- |
| Full release gate | `just release-verify` passed 887 Rust tests in 68 runs and 28 shell checks, including formatting, Clippy, script syntax and the release build. |
| Guild Hall and Delve | Keyboard navigation changed the saved view Guild → Delve → Guild, with three adventurers. The managed plugin actions repeated that sequence. |
| Link eligibility | The metadata-only adventurer was refused. Explicitly linking the dummy Pi made Observe available. |
| Verified focus | Board Enter, War Table Enter and Inbox `o` each moved actual Herdr focus from the idle sentinel to the qualified Pi pane. |
| Removal and replacement | Explicit `L` and a replacement session in the same pane each revoked Observe and left sentinel focus in place. Selecting the new incarnation and pressing `l` restored verified focus. |
| Restart and durable state | Restart required a fresh link. The owned file stayed byte-for-byte identical, the work cache contained no bindings, and saved profile files contained neither synthetic session ID. |
| Managed pane | The tracked process ran the qualified release binary. Open reused the existing pane, the pane stayed out of the party, and all recorded plugin actions exited successfully. |

The first PTY pass reached revocation but needed an explicit re-selection of the
replacement adventurer for re-linking. The corrected pass completed end to end.
An initial managed-pane bookkeeping error was also corrected. The retained setup
revalidated the created panes, recorded the managed PID and repeated the view checks.
The local run directory keeps the terminal transcripts and Herdr action log.

## Review the open party

From a plain shell in a new Ghostty window (`⌘N`) on the same Mac, attach to
the isolated server with

```sh
python3 /tmp/qm-live-association.USm1QV/isolated.py attach
```

The **Questmancer association review** workspace retains the managed pane and
temporary owned store. The three test receivers exited after the captures, and
the user opened a shell tab. The isolated server uses a plain non-login `/bin/sh`
to keep the user's shell configuration out of the test.

While the party was present, `2` opened the Delve and `1` returned to the Hall.
Selecting `pi`, opening the Board with `b`, selecting **Web migration**, and
pressing `l` linked the quest. Enter observed the verified pane and `L` cleared
the link. Restart required a fresh link.

## Native Ghostty evidence

The review rechecked the retained managed pane and confirmed the same three live
reports (working, idle and blocked) on Herdr 0.9.5. Ghostty 1.4.0 is installed.
Computer Use denied access to Ghostty. The user captured both rooms from the
attached client.

The first Ghostty capture showed white stripes in the Guild Hall. The isolated
server had inherited `NO_COLOR=1` from the test launcher and passed it to the
managed process. Crossterm suppressed ANSI colors, leaving the half-block glyphs
in the default foreground color. The Guild Hall snapshot contained 8,978 half
blocks and zero RGB sequences.

We reopened only the managed pane with `NO_COLOR=` in its launch environment.
The same binary, server and three adventurer sessions stayed in place. Guild
and Delve snapshots then contained 3,506 and 7,648 RGB sequences, respectively.
This recovered color in the fallback renderer. The [live receipt](receipt.json)
records both sides of the check.

The next capture showed the compact Hall inside a second Herdr tab bar. It
records the appearance of the fallback in that terminal chain. The attach
helper now rejects shells marked as Herdr panes. Questmancer negotiates Kitty
graphics and cell dimensions at startup.

### Ledger portrait fallback follow-up

A later Ghostty capture showed the authored Librarian sprite in the Ledger on
the **regular** Herdr server. Its managed Questmancer pane reported valid PTY
geometry at inspection (211×52 cells, 1688×936 pixels). This case also occurs
with a directly attached Ghostty client, so nesting alone cannot explain it.

- The isolated headless managed probe acknowledged the exact Kitty query but
  returned zero PTY pixels and no CSI 16t reply. This reproduces the headless
  startup limitation.
- An unfocused disposable shell on the regular server returned Kitty `OK`
  and 8×18 cells in the same batch before the terminal-ready reply.
- A second disposable shell on that server ran the exact Questmancer release
  binary offline with a temporary profile. It emitted a Kitty upload in the
  Hall and another when opening the Ledger. Escape and quit passed on a repeat
  with separate key presses. Both disposable workspaces were removed and the
  pre-existing panes and original focus were verified after each probe.

This establishes working protocol output for a fresh plain pane. The cause
of the existing regular managed pane's fallback remains open.

### Direct attachment and resize

With Ghostty attached directly, the existing isolated pane reported nonzero
pixel dimensions. A fresh managed pane on that same client started with a
209×54 grid and zero pixel dimensions. A separate disposable managed probe
returned Kitty `OK`, no CSI 16t reply and zero pixel dimensions. The refreshed
Questmancer pane used the RGB half-block fallback.

A second refresh delayed only the test-owned pane's launch until the kernel
reported real pixel dimensions. A manual Ghostty window resize supplied
1672×972 pixels for the 209×54 grid. The same release binary then selected the
Kitty room path. Both the Hall and Delve terminal frames contained 4,900 Kitty
image placeholders and zero half-block glyphs. Both reported three adventurers.
The cell snapshot can record placements, but it cannot show image pixels or
confirm visual fit. The pane returned to the Hall with the original party.

Earlier Hall and Delve captures showed the separate regular session with one
Codex adventurer. The accepted captures below show the isolated **Questmancer
association review** workspace and the three `qmusm1q-` synthetic tabs.

| Native capture | Observed result |
| --- | --- |
| [Guild Hall with Adventurer card](ghostty-guild-hall.png) | The room stays fitted with the profile open, and the card displays the detailed Mage portrait. |
| [Delve](ghostty-delve.png) | The full room is visible with all three adventurers. The resting, working and counsel cues and selection marker are distinguishable. |

These captures qualify appearance at the observed sizes after the resize and
warm launch. They leave animation and the rest of the resize range open.
