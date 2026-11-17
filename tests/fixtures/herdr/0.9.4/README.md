# Herdr 0.9.4 protocol captures

Captured 2026-10-14 from the official macOS ARM64 release binary, SHA-256
`ea20bfff9cb8c83a996081c926e6ff9370de8e543baa859bca48d19175d895ff`.
The digest matches GitHub's immutable release asset metadata. The annotated
v0.9.4 tag resolves to commit `7e26bad94044eba205033111a0ce8033ede7e24f`.

An owned headless server used temporary config/state/cache directories, a plain
`/bin/sh` pane and a synthetic working report. Its identity was released before
closing its pane; the server exited zero, left an empty snapshot and removed
its socket. Only the generated terminal ID and temporary cwd are normalised.
Version, protocol 22, capabilities, layout, optional fields and errors are kept.

- `pong.json` and `session_snapshot.json`: actual current release wire shapes.
- `removed-graphics.json`: `pane.graphics.info` returns `unknown_method`.
- `unsupported-header.json`: the experimental pane-header probe returns
  `invalid_request`; the plugin must treat it as unavailable.

Synthetic reports here have no exposed agent session, even with a terminal ID.
Do not invent Chronicle identity from them. Newer Herdr clears self-reported
agents at idle shell prompts; sustained runtime trials use owned foreground
receivers. These captures prove JSON contracts, not native appearance or
real-agent completion. The 0.9.0 fixtures remain historical protocol-22 coverage.

See [the compatibility review](../../../../docs/reviews/2026-10-14-herdr-094/README.md)
for qualification, checksums and cleanup receipts.
