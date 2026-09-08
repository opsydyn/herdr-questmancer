# Herdr 0.9.0 protocol captures

Captured on 2026-09-08 from the official stable macOS ARM64 binary, SHA-256
`32b53df09872628059c789a69f02a6b8e29e14ddf26711421f3463f70c1aef17`.
An isolated headless server owned its configuration/state/cache directories and
a plain `/bin/sh` pane. Its synthetic working agent was released before the
pane closed; the server exited zero and removed its socket.

Only the generated terminal ID and temporary working-directory paths are
normalised. Version 0.9.0, protocol 22, endpoint-generation capability fields,
optional fields and layout values are retained. The headless pane now occupies
the full 120x40 terminal area because the outer Herdr UI is client-rendered.

These fixtures prove JSON shapes, not native graphics, multi-machine scope,
real-agent completion or the user's release-candidate acceptance. The older
0.8.2 captures remain as historical rejection evidence.
