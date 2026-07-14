# herdr-webmaster design

## Product boundary

`herdr-webmaster` is a local-only Herdr plugin that gives one shared model two
interactive views. The webmaster desk optimizes attention routing; the
cybercafe turns the same state into ambient theatre without hiding actions.
The user, never an agent, is the webmaster.

The visual vocabulary is functional: under construction means working,
contact webmaster means blocked, update available means done and unseen, and a
broken link means an exited pane. Original Unicode/ASCII art is canonical.

## Runtime architecture

The TUI owns two socket connections: one serial request connection and one
long-lived subscription connection. Startup validates the plugin environment,
pings Herdr, checks protocol compatibility, takes `session.snapshot`, subscribes,
and renders. Reconnect preserves the visible model, shows its state, takes a new
snapshot, replaces runtime cache, and resumes subscriptions.

A pure reducer returns a new `Model` plus effect `Command` values. Effect
handlers perform socket I/O and persistence outside the reducer. Both views
render the same model. Output reads are selection-driven rather than global.

## Domain rules

Agent presence and user attention are independent types. `SiteStatus` is
derived with priority `NeedsWebmaster`, `UpdateReady`, `Updating`, `Online`,
then `Offline`. Guestbook entries are JSONL records with deterministic IDs.
Personas use a stable identity preference order and persist by persona key.

Animation has three layers: durable domain state, one-shot transition effects,
and clock-derived frames. Tests inject time, so confetti and CRT frames are
repeatable. Done confetti ends; it is not a permanent state marker.

## Failure behavior

Unknown JSON fields are ignored. Unsupported protocols produce a useful screen.
Socket loss never discards the last visible state. Invalid configuration emits
a warning and uses defaults. Zero-sized or tiny terminal areas use a safe list
fallback. Terminal restoration is guarded on every exit path and in the panic
hook.

## Verification strategy

Domain behavior uses reducer unit tests. Ratatui output uses `TestBackend` with
an injected clock. Socket parsing uses split/coalesced/interleaved fixtures.
Lifecycle scripts use shell-level tests before live plugin linking. Live
acceptance requires Herdr `0.7.3`; this machine currently has `0.7.0`, so that
environmental gap must be reported separately from code failures.

## Scope

v0.1 includes the two views, focus/reply, attention, lazy output, deterministic
personas, local persistence, optional reviewr invocation, plugin lifecycle,
macOS/Linux binaries, and release automation. It excludes images, sound,
telemetry, cloud/network services, GitHub integration, Git worktree decoration,
multiple themes, Windows, and SQLite.

