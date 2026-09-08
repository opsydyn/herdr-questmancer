# Herdr 0.8.2 and Questmancer release readiness

Date: 2026-09-06. The Questmancer authorised reviewing Herdr's latest release
notes before upgrading, followed by the bounded release-readiness pass.

## Release assessment before implementation

The latest stable release is [Herdr 0.8.2](https://github.com/herdrdev/herdr/releases/tag/v0.8.2),
published 2026-08-19. Newer preview builds exist; this upgrade stays on stable.
The client installed at the start was 0.8.0. Fresh `herdr status` and `herdr session list`
showed the default session stopped and no other sessions running.

Herdr 0.8.2 declares [protocol 20](https://github.com/herdrdev/herdr/blob/v0.8.2/src/protocol/wire.rs).
Questmancer required protocol 19 exactly at the start, so changing the executable
alone would leave the plugin incompatible. Audit the socket methods it uses,
exercise a test-owned server, then move the manifest and protocol gate together.
Retain exact version rejection rather than assuming future protocols work.

| Upstream change | Consequence for Questmancer |
| --- | --- |
| Better blocked/working detection, including Qwen Code and Claude confirmation prompts | Keep Herdr authoritative. Test state adaptation without adding terminal scraping or tool-specific detection to Questmancer. |
| `agent prompt` rejects agents at approval/question dialogs | Preserve explicit, user-composed counsel and its correlated text/submit outcomes. Do not replace that path with an automatic agent-prompt call. |
| Experimental graphics layers, acknowledged direct RGBA files and placement-only resize replay | A future bounded native-card/portrait experiment may improve transport ownership and resize behaviour. Keep the sole RGB half-block world renderer and authored fallbacks; this is not approval for a new renderer. |
| Fewer hidden-pane wakes and faster alternate-screen reads | Retain lazy selected-output reads and bounded animation deadlines. Include headless/hidden-pane conditions in future performance evidence. |
| Headless default grows to 120x40 terminal cells | Do not assume the old 80x24 default in integration checks. Continue explicit viewport tests; this does not establish native visual acceptance. |
| Marketplace reports plugin versions and exact default-branch commits | Prioritise matching manifests, tags and downloadable archives. The existing 0.1.8 versus published 0.1.3 gap affects installation. |
| Windows becomes generally available in Herdr | Treat a Windows Questmancer port as separate work: its current Unix socket client and Bash lifecycle scripts do not become portable through this upgrade. |

The graphics contract is described in the [tagged socket API documentation](https://github.com/herdrdev/herdr/blob/v0.8.2/docs/next/website/src/content/docs/socket-api.mdx).
These roadmap consequences are assessments, not additional feature commitments.
The current Librarian and pilot visual reviews remain pending manual evidence.

## Authorised bounded pass

1. Verify the stable Herdr binary against its release-asset SHA-256 and update
   the installed client. Preserve the original binary for rollback.
2. Capture protocol evidence from an isolated, test-owned server. Upgrade
   Questmancer test-first, including rejection of unsupported versions, and
   reconcile current operating guidance.
3. Run the full verification gate, build the release binary, and verify the
   actual packaged crate and its 10 MiB limit.
4. Check all four existing published archives and checksums, exercise the real
   installer in a temporary directory, diagnose missing matching releases and
   prepare a locally reviewable repair. Update the hand-written changelog.

No commit, tag, workflow dispatch, registry publication or GitHub release is
included. Final clean-commit release acceptance and native visual evidence
remain separate gates. Every live test must own and clean up its own server,
panes, synthetic agent sources and registration.

## Execution receipt

The authorised local pass is complete. See the [detailed receipt](../reviews/2026-09-06-release-readiness/README.md).

- Installed the checksum-verified official Herdr 0.8.2 macOS ARM64 binary and
  retained the previous executable for rollback. No shared server was changed.
- Updated the manifest and exact protocol gate together. Captured real 0.8.2
  fixtures and verified rejection of 19 and 21 at ping, initial snapshot and
  topology refresh. An isolated server exercised the actual production client,
  supervisor, event adapter and reducer; its resources were cleaned up.
- Full `just verify` passes 555 Rust tests across 50 runs and 28 shell tests,
  formatting, strict Clippy and shell syntax. The release build and crate
  verification pass; packaging used `--allow-dirty` because earlier authorised
  work remains uncommitted. This is preflight, not clean-commit acceptance.
- All four published 0.1.3 archives pass checksums, layout and architecture
  checks. Its native installer path passes in a temporary directory. A 0.1.8
  release is still absent.
- Confirmed the historical tag-to-build failure and prepared an explicit
  dispatch using the scoped workflow token. All release jobs select the
  requested tag. Mocked dispatch and workflow-contract regressions pass.

Native Librarian/pilot and room review, clean-candidate live acceptance, the
eventual matching publication, and live repository registry settings remain
unverified. No commit, tag or external publication was performed.
