# Herdr 0.8.2 protocol captures

Captured on 2026-09-06 from the official stable macOS ARM64 binary, whose SHA-256
is `a5d4f4d504d8b309c91f811050559300faba31258425f53c50852fc96f6ae574`.
The test owned a fresh headless server, configuration/state directories and a
plain `/bin/sh` pane. It reported one synthetic working agent, captured these
responses, released that source and stopped its own server cleanly.

Only the generated terminal ID and temporary working-directory path are
normalised. Field names, missing optional fields, protocol/version and viewport
values are retained. The default 120x40 terminal gives this pane 94x39 cells
after the server's sidebar and tab row. These are protocol fixtures, not native
visual or real-agent completion evidence.
