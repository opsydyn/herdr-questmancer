# Real-agent Chronicle acceptance — 2026-09-09

The user approved continuing with the recommended guarded real-agent check after
v0.1.10 publication. The test used source `44eae4e` (documentation only after the
`9f2f58f` release), the built 0.1.10 binary, Herdr 0.9.0, and an actual interactive
Pi agent using its configured openai-codex/gpt-5.5 model.

## Observed result

- A fresh isolated server and one owned Pi pane established a quiet idle baseline
  with complete native session identity. Questmancer initially wrote no history.
- Pi answered the no-tools arithmetic prompt with `QUESTMANCER_CHECK 3973`.
  Herdr's actual integration reported working and then done; no synthetic status
  reports were sent.
- Questmancer persisted exactly two v2 pane-metadata records: working and done.
  The completed adventurer projected `Triumphant` and `◈ 1 spoil`.
- Confirmed completion earned 10 XP. Closing/reopening Questmancer added no
  history, and persisted XP remained 10 after the final shutdown.
- All owned panes and the test plugin link were removed. The owned server exited
  normally and removed its socket. Plugin action logs were inspected; no cleanup
  errors remained. Shared Herdr resources and global integrations were untouched.

The bundled Pi extension v8 was installed only into temporary storage. Pi used a
fresh configuration and session directory, with tools, context files, skills,
prompt templates, other extensions and startup network checks disabled. Only the
arithmetic prompt was sent to the configured provider. Existing authentication
was referenced temporarily; that reference was removed after cleanup. Credentials
and full agent session files are not included in this review.

`verification.json` records the result, binary hash and observation evidence.
Full local receipts are under `/tmp/questmancer-real-agent-acceptance/` and its
recorded case directory. Two earlier attempts reached real completion but stopped
on harness terminal-output decoding mistakes; both cleaned up their resources.
The final corrected run completed all persistence and cleanup checks.

## Evidence limits and next step

This closes real-agent completion through pane metadata for this Pi configuration.
It does not establish native animation appearance, other agent integrations,
real blocked/counsel cycles or live snapshot-only capture. No production code
changed, so the existing release gate was not rerun for this evidence-only update.

The next bounded recommendation is to design a controlled live snapshot-only
acceptance check, preserving source attribution and quiet reconnect baselines.
Do not treat this metadata run as that evidence or start that separate scope
without approval.
