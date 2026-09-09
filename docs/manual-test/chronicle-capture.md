# Chronicle capture acceptance

C1/C2 are implemented locally. C3 automated and isolated synthetic qualification
has passed; the user approved the Chronicle screen pack on 2026-09-09. See the
[C3 receipt](../reviews/2026-09-09-chronicle-c3/README.md). Follow the resource-ownership rules in
[the guarded scene preview](questmancer-scene-preview.md) before any live work.
No shared Herdr server may be stopped or reconfigured without its owner's approval.

## Expected behaviour

1. Establish an authoritative post-subscription baseline. Existing blocked/done
   adventurers create no retrospective history or XP. Retained history remains.
2. With complete session/terminal identity, observe a known working adventurer
   become blocked in a qualified snapshot. One record says it was **observed**
   needing counsel, with local observation time. Repeats and matching metadata
   create no extra record. Mere fallback identity cannot qualify this history.
3. Open `c`, then use `Tab` for a fixed-window chapter. Check the separate presence,
   unknown-whereabouts, membership and campaign-removal counts, captured summaries,
   source IDs and local-observation labels. Legacy joined records remain identity
   events; an empty/partial chapter never proves that no work happened.
4. Treat close/exit events as evidence with limits. Workspace-close hints require
   an eligible snapshot confirming absence. If the workspace remains, retain it
   and consume the hint. Campaign removal gives no XP and absorbs membership
   losses caused by that same absence. Unversioned pane-exit hints cannot prove
   departure. Subscription-membership changes instead create a quiet baseline.
5. Reconnect/resync and repeat already-observed facts: no backfill, duplicates or
   re-awarded standing. A changed terminal invalidates old scrying output and
   establishes a quiet subject baseline. Delayed responses from old ownership
   must remain inert. Counsel/focus task ownership remains independent.
6. Preserve mixed history bytes across restart. V1 records keep their original IDs
   and wording; v2 records retain captured source context. Confirm the existing
   append diagnostic/acknowledgement path without treating state/history as an
   atomic transaction. Do not rewrite or deliberately corrupt a user's history.

For C3 live work, use only an isolated test server and owned synthetic identities,
with temporary state/history files. Record the exact source, baseline subscriptions,
resources, observation source and cleanup. Herdr 0.9 cannot synthesize explicit
done: snapshot-first spoils and later status reward suppression remain fixture
cases unless independently observed with authorisation. Never claim live
snapshot capture merely from a status event, or infer complete upstream history.

## Replay and downgrade

New records use `record_version: 2`; the reader also accepts unchanged flat v1
records. Unknown versions/payloads and malformed/truncated lines produce bounded
diagnostics. Existing valid bytes remain untouched. Published 0.1.9 readers may
skip v2 records with diagnostics on downgrade. Replay never reconstructs awards
from bounded history. If startup cannot obtain a capture-run ID, a diagnostic is
shown and new capture/awards stay disabled while live presentation continues.

## Repeating isolated synthetic qualification

The review's `isolated_capture_check.py` creates temporary XDG directories, its
own headless server and plugin link, four dummy Pi processes, and fresh synthetic
UUIDs. It compiles a sleeper named `pi`; it never starts an AI agent or resumes a
session. Agent restore and update/manifest checks are disabled in that isolated
config. Follow the guarded ownership rules and inspect the receipt on exit.

Herdr 0.9 does not retain native session references from arbitrary custom sources.
Official Pi lifecycle reports additionally need a matching process and a session
start. After detection, report `pane.report_agent_session` with `source=herdr:pi`,
`agent=pi`, a fresh session ID, `session_start_source=startup`, and sequence 0;
then use ordered `pane.report_agent` reports from sequence 1. The source must use
Herdr's official value; the temporary server, pane and UUID establish test
ownership. Assert four non-empty, complete session identities before opening
Questmancer. A report acknowledgement alone does not establish acceptance.

Release each owned report before closing its pane. Herdr may retain the session
reference while the dummy process still exists; do not mistake that for active
hook authority. Verify pane absence after closing, remove only the test-created
link, verify the final empty pane inventory, and stop only the server this run
launched. Native UI approval, real-agent completion and snapshot-only live capture
remain separate from these synthetic metadata observations.
