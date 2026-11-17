# Storybook proportional Hall fit

This is an isolated prototype. It uses fixed fixture adventurers, opens no Herdr
socket and writes no Questmancer state. Keep the existing dogfood party open and
run this in a separate terminal/pane.

From the repository:

```sh
cargo run --features storybook --bin questmancer-storybook -- --story world.guild-fit --inspect
```

Alternatively, `just storybook` includes three new World-category stories after
Guild Hall: **Prototype / Hall Fit — Five**, **Eleven** and **Crowded**.

## Native review

Use an attached compatible terminal with the existing pane graphics transport.
Startup negotiates Kitty and cell-pixel geometry using the same Picker as the
cards. The top line should say **Kitty fit … / 5/5 adventurers**. A fallback
diagnostic means native fitting was not enabled; it is not a visual pass. Do not
force a protocol or change shared-server configuration for this check.

1. Resize the window around 120×34, 96×28 and 80×24 cells. The complete room
   should stay in proportion, with letterboxing rather than distorted pixels.
   All five adventurers and the Librarian should remain visible. The footer
   text retains the terminal font size independently of the picture.
2. Tab and Shift-Tab select fixture adventurers. Check the selection rune and
   footer refer to the same actor after every resize. Click an actor to open its
   card; click letterboxing and confirm it does not select anything.
3. `c` opens/closes the selected card. `L` opens/closes the Ledger; clicking the
   Librarian also opens it. Ledger `j/k` or arrows turn pages and `g/G` reach the
   ends. Esc closes the current overlay. During this experiment, open parchment
   uses the existing half-block backdrop. Closing it must restore the fitted room.
4. `?` opens Storybook help. Esc closes help and restores the fit.
5. With overlays closed, `j` selects **Eleven**. Check its header says 11/11 and
   every actor is visible/clickable. `j` again selects **Crowded**, which must
   report incomplete canonical station placement and use the existing responsive
   fallback. At very small sizes that fallback may be a single priority vignette;
   the prototype must not claim a complete native fit for that case.
6. `k` returns through the stories. Esc returns to the catalogue; `q` exits.
   Check no image remains over the shell or another story after exit/switching.

For graphics-off fallback:

```sh
cargo run --features storybook --bin questmancer-storybook -- --story world.guild-fit --inspect --no-graphics
```

Expect an explicit fallback diagnostic and a usable half-block scene. No native
world image should be emitted. Selection and card/Ledger controls still work.

## Limits and acceptance record

The fixture clock is fixed and motion is None. This does not qualify animation
upload cost, real-agent behaviour, live Herdr state or production responsiveness.
Cell geometry is negotiated at startup: **restart after changing terminal font
size**. Normal window resize uses that geometry. Font-zoom changes, reconnect,
visibility transitions and actual Ghostty rendering remain manual/unqualified.
Tmux is deliberately excluded from native fitting in this prototype.

Record the terminal/pane transport, dimensions, native/fallback status, actor
visibility, selection/click agreement, overlays, resize and exit separately.
The [engineering receipt](../reviews/2026-11-17-storybook-hall-fit/README.md)
contains deterministic and synthetic PTY evidence. It is not native acceptance.
