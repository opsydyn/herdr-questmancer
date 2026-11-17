# L1b native acceptance

The user approved the [production form images](../reviews/2026-10-03-owned-authoring/visual-approval.md)
on 2026-10-03. The next gate is one isolated native workflow. Computer Use refuses
Ghostty at the access check, so automated native execution is blocked. The manual
trial below is prepared, not performed. No install, relink, shared server change,
real adventurer, commit or release is part of this gate. L2–L5 remain unpromoted.

## Scope and success

Run the qualified release candidate directly in Ghostty, offline, against a new
physical temporary state directory. Strip inherited Herdr environment so it cannot
attach to a shared server or pane. Keep the new directory for both launches. The
ordinary plugin runner and managed-pane transport are separate future checks.

The human should complete these steps using native input:

1. Open Board (`b`), then Actions (`a`). Explicitly Create local store at the proposed
   path within the test directory. Review path/source in Store details.
2. Create **Native authentication trial** and verify the empty campaign is selectable.
   Create one Ship quest, **Migrate API authentication**; it remains Planned.
3. Request **Keep the old endpoint for one release?**, associated with that quest.
   In the Board, `!` reaches the open request. Space/seen and `s`/defer retain source
   Open; deferral may remove it from the immediate attention queue.
4. Use Actions → Decisions to inspect the same request and explicitly answer:
   **Keep it for one release. Document removal.** Verify the answer is readable,
   the request is Resolved and its incarnation no longer demands attention.
5. Register a Report artifact, **Native migration report**, with URI
   `https://example.invalid/native-report`. Read it in quest or Store details.
   Registration must neither open that URI nor claim validation.
6. Resize to 40×20 cells while reading/editing a long unsent answer or campaign draft.
   Verify wrapping, scroll, fixed heading, Tab focus and literal multiline paste.
   Resize to 27×11: the resize message is clear, draft survives and submission is
   disabled. Restore the size, Resume/Discard explicitly. Do not submit extra work
   accidentally. Retain screenshots only of this synthetic test.
7. Return to the world with Esc, then `q`. Relaunch against the same directory;
   select the owned campaign and Decisions, then read the identical saved answer
   and artifact. Record findings and quit. Nothing may be recreated implicitly.

Review saved/view-pending and uncertain-state wording through the approved fixed
Storybook images; do not damage files or force faults in the native test. Do not
equate their fixture approval with physically observing those failures in Ghostty.

## Manual launch

Run this from the repository root in Ghostty. It checks the candidate hash, creates
only its own state directory, and launches the same candidate twice with native
stdin/stdout. Press Return at each launch prompt; finish each UI with `q` before
continuing. Keep store setup at the proposed test path. The directory is retained
for evidence and explicit cleanup, and printed before any UI starts.

```bash
python3 -c '
import hashlib, json, os, pathlib, subprocess, tempfile
root = pathlib.Path.cwd()
proof = json.loads((root / "docs/reviews/2026-10-15-herdr-upgrade/verification.json").read_text())
binary = root / proof["candidate_binary"]["path"]
assert hashlib.sha256(binary.read_bytes()).hexdigest() == proof["candidate_binary"]["sha256"], "Candidate changed; qualify before testing"
state = pathlib.Path(tempfile.mkdtemp(prefix="questmancer-native-", dir=str(pathlib.Path(tempfile.gettempdir()).resolve())))
environment = {key: value for key, value in os.environ.items() if not key.startswith("HERDR_")}
environment["HERDR_PLUGIN_STATE_DIR"] = str(state)
print("TEST STATE:", state, flush=True)
for label in ("Create and answer the synthetic work", "Restart: inspect the same answer and artifact"):
    input(label + " — press Return to launch: ")
    result = subprocess.run([str(binary), "ui"], env=environment)
    if result.returncode:
        raise SystemExit("UI failed; retain test state and report its exit code: " + str(result.returncode))
print("Both UIs exited. Test state retained for review:", state)
'
```

This command is for the human to run. The agent has not executed it or automated
Ghostty. An interrupted trial is incomplete; retain the directory and report the
last completed step rather than infer restart/cleanup acceptance.

## Evidence and cleanup

Record candidate SHA-256, terminal/OS, actual viewports, test directory, owned source,
what passed/failed and native screenshots. Do not copy real terminal output or live
topology into evidence. Separate user visual acceptance from file/restart proof.

After both test UIs exit, inspect only the printed test directory. Check the answer,
Resolved request and artifact in `owned-work.json`, the same path/source pin in
`state.json`, and no literal answer in `work-state.json` or Chronicle. Record the
files created. Remove only this trial-owned directory after evidence is retained;
the persistent `.lock` file is not a running writer and must not be removed while
the owner is active. Do not remove any existing plugin state, registration or link.

Native acceptance requires observed completion of the checklist and explicit user
confirmation of readability. The current [receipt](../reviews/2026-10-03-owned-authoring-native/README.md)
records only the blocked availability check. Engineering/PTY qualification remains
valid, and production fixture visuals are approved. Managed-pane/live use,
installation, commit and release remain separate.

## Candidate refresh — 2026-10-15

The approved host upgrade installed Herdr 0.9.5, the current stable hotfix for
0.9.4, and refreshed the existing Claude/Codex integrations. The launcher above
now checks the current release candidate against the October 15 receipt; the
October 3 binary hash belongs to its historical qualification. The current
[upgrade receipt](../reviews/2026-10-15-herdr-upgrade/README.md) records repeated
isolated Herdr runtime and automated PTY authoring/restart checks. Native human
execution and readability approval remain unperformed.
