# Questmancer from Moshi Runbook Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add an end-user runbook that explains how to operate an installed Questmancer plugin from Moshi without changing Questmancer runtime behavior.

**Architecture:** Keep the runbook as a focused operator document under `docs/runbooks/`, with one discoverability link from `README.md`. It will describe Moshi as an optional outer client, while Herdr remains authoritative for sessions, workspaces, panes, and agent state and Questmancer remains the local presentation and command surface.

**Tech Stack:** Markdown, current Herdr 0.8.0 plugin actions, Moshi and `moshi-hook` operator commands, existing Bash/Ruby documentation-contract checks.

## Global Constraints

- Preserve the existing unrelated change in `src/scene/assets/archetypes.rs`.
- Use the current plugin actions exactly: `opsydyn.questmancer.open`, `close`, `toggle`, `guild`, and `delve`.
- Do not add a Questmancer dependency on Moshi, a Moshi webhook, cloud transcription, or a second renderer.
- State that Herdr owns topology and agent facts; never advise sending Chronicle/state data to Moshi.
- Label `moshi-hook`, Chat View, diff viewer, browser preview, native graphics, and live visual checks as optional or evidence-dependent.
- Do not instruct the reader to stop, reload, unlink, or reconfigure a Herdr server they do not own.

---

### Task 1: Write the Moshi operator runbook

**Files:**
- Create: `docs/runbooks/questmancer-from-moshi.md`

**Interfaces:**
- Consumes: the commands and controls documented in `README.md`, the actions in `herdr-plugin.toml`, and Moshi’s current Herdr, hooks, voice, keyboard, Chat View, diff, browser-preview, and deep-link documentation.
- Produces: a standalone end-user procedure linked by the README in Task 2.

- [ ] **Step 1: Create the document structure and scope statement.**

  Start with the operator outcome and explicitly say the reader must already
  have Questmancer installed. Define Questmancer, Herdr, Moshi, and optional
  `moshi-hook` in one short terminology table or paragraph. State that Moshi is
  an outer access layer, not a Questmancer runtime dependency.

- [ ] **Step 2: Document first connection and PATH checks.**

  Include the Moshi/Herdr prerequisites and the exact non-interactive check:

  ```bash
  ssh your-host 'echo PATH=$PATH; command -v herdr'
  ```

  Explain that `herdr` must be visible to Moshi’s non-interactive SSH shell,
  that Moshi lists running Herdr sessions/workspaces, and that the user should
  attach to the existing workspace rather than create a duplicate session.
  Link to the official Moshi Herdr and troubleshooting pages.

- [ ] **Step 3: Document the daily mobile loop.**

  Describe this exact safe flow: open the relevant Moshi inbox event; resume
  the Herdr workspace; invoke Questmancer `open`, `guild`, or `delve`; use `!`
  to select the next adventurer waiting on the user; use `r` to draft counsel;
  review it; press `Enter`; then use `Enter` on an adventurer when a deeper
  terminal/agent view is needed. Include phone and tablet guidance for zoom,
  landscape layout, Herdr sidebar collapse, taps, and Questmancer’s documented
  `80x24` minimum.

- [ ] **Step 4: Add optional shortcut and voice instructions.**

  Show the Herdr `[[keys.command]]` shape with `type = "plugin_action"` and the
  three Questmancer action ids. Use example safe chords `prefix+shift+q`,
  `prefix+shift+e`, and `prefix+shift+o`, and require checking `prefix+?`
  before applying them so an existing user binding is not overwritten. Tell
  readers to merge existing `config.toml` content and reload only a server they
  own. Explain that Moshi direct-terminal dictation can fill the counsel
  parchment, local speech engines avoid sending audio away from the phone, and
  auto-send should remain off until the draft is reviewed.

- [ ] **Step 5: Explain complementary inspection surfaces and boundaries.**

  Link to Moshi Chat View, diff viewer, browser preview, voice, hooks, and
  deep-link documentation. Explain that these surfaces complement Questmancer
  Scrying and optional Reviewr; Moshi transcript parsing and notifications do
  not replace Herdr or Questmancer truth. Include a privacy note distinguishing
  host-to-phone local gateway traffic from notification summaries/metadata sent
  to Moshi’s service, and advise against adding Questmancer webhook publishing.

- [ ] **Step 6: Add troubleshooting and evidence limits.**

  Cover an undetected Herdr binary, an empty session picker, a wrong shortcut
  prefix, a notification that returns to a workspace rather than a pane, and a
  missing `moshi-hook` daemon. State that Moshi deep links target Herdr sessions
  and workspaces, not Questmancer panes; native portraits require the existing
  Herdr graphics bridge and a compatible transport; and no mobile visual
  acceptance claim is valid without a fresh direct visual check.

- [ ] **Step 7: Review the runbook for copy and safety.**

  Confirm all user-facing terms use Questmancer vocabulary, every command is
  copyable, optional features are labelled, no destructive/shared-server step is
  implied, and no claim exceeds the documented or directly tested evidence.

- [ ] **Step 8: Commit the standalone runbook.**

  ```bash
  git add docs/runbooks/questmancer-from-moshi.md
  git commit -m "docs: add Questmancer Moshi runbook"
  ```

### Task 2: Link the runbook from the README

**Files:**
- Modify: `README.md` near the installed-plugin quick start

**Interfaces:**
- Consumes: `docs/runbooks/questmancer-from-moshi.md` from Task 1.
- Produces: a discoverable link for installed Questmancer users without
  changing installation commands or the existing manual acceptance links.

- [ ] **Step 1: Add one short operator link.**

  Place a sentence after the installed-plugin instructions stating that users
  who operate Questmancer from a phone or tablet can follow the [Questmancer
  from Moshi](docs/runbooks/questmancer-from-moshi.md) runbook. Keep contributor
  build/link instructions separate.

- [ ] **Step 2: Verify the link and scope.**

  ```bash
  test -f docs/runbooks/questmancer-from-moshi.md
  rg -n "Questmancer from Moshi|docs/runbooks/questmancer-from-moshi.md" README.md
  git diff --check
  ```

- [ ] **Step 3: Commit the README link.**

  ```bash
  git add README.md
  git commit -m "docs: link Moshi operator runbook"
  ```

### Task 3: Run documentation and repository verification

**Files:**
- Test: `docs/runbooks/questmancer-from-moshi.md`, `README.md`

**Interfaces:**
- Consumes: the completed documents from Tasks 1 and 2.
- Produces: verification evidence distinguishing text-contract success from
  unperformed live Moshi or visual acceptance.

- [ ] **Step 1: Run the repository script and contract checks.**

  ```bash
  bash tests/scripts.sh
  ruby tests/workflow_contract.rb
  git diff --check
  ```

  Expected: all existing script/workflow checks pass; documentation changes do
  not alter plugin action or release contracts.

- [ ] **Step 2: Run the normal full gate when the checkout is otherwise ready.**

  ```bash
  just verify
  ```

  Record any environment or pre-existing failure separately from the runbook
  result. Do not claim that this verifies Moshi connectivity, mobile rendering,
  native graphics, notifications, or visual quality.

- [ ] **Step 3: Inspect final status and summarize evidence.**

  ```bash
  git status --short --branch
  git log -3 --oneline
  ```

  Confirm the unrelated `src/scene/assets/archetypes.rs` change remains
  present and that only the intended documentation commits were added.
