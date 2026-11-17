# Tokenmaxx workshop and Questmancer follow-up

## Recommended layout

Use a standalone participant repository alongside the pinned starter and the Questmancer checkout. The participant repo gets its own Git history, pnpm workspace and Turbo task graph. This follows the starter's two-prompt setup. The sibling repositories now exist. The participant adds the embedded workshop during Step 0. [Pinned starter README](https://github.com/joelhooks/rat-stack/blob/tokenmaxx-starter-v3/examples/tokenmaxx-starter/README.md)

```text
Projects/
  herdr-web-master/       Questmancer product
  tokenmaxx/              Upstream starter and initial Pi launcher
  tokenmaxx-lab/          Participant repo for prompts, code, hooks and commits
    packages/agent-tools/ Our Pi tooling and behavior tests
    workshop/             Pinned exercises added by the second setup prompt
```

The later exercises check committed work. The final exercise puts three workers in separate copies, checks which files reach the participant branch, and requires a recoverable reference or patch for rejected work. A separate repo makes those checks meaningful while Questmancer has ongoing owned-authoring and native-capability changes. A branch in the existing checkout would still carry those working-tree edits. The trade-off is a second repo to maintain and a separate integration test for anything we adopt. [Wait exercise](https://github.com/joelhooks/rat-stack/blob/tokenmaxx-starter-v3/examples/tokenmaxx-starter/exercises/02-loop/02.1-wake-me-when-done/readme.md) · [Delegation exercise](https://github.com/joelhooks/rat-stack/blob/tokenmaxx-starter-v3/examples/tokenmaxx-starter/exercises/06-split-the-work/06.1-split-three-ways/readme.md)

## Before the workshop

1. **Prepared locally.** The standalone starter at `../tokenmaxx/` has the 129 source files from pinned commit `b89bf6155c0beb22ed2ca152f535f8ca9c5209fb`. Its frozen install used upstream pnpm 11.3.0. The participant lab at `../tokenmaxx-lab/` has its own Git baseline at `56957ed`, fnm Node 26.13.1, pnpm 12.21.0, Turbo 2.11.11 and a frozen lockfile. Turbo discovered `packages/agent-tools/`.
2. **Model setup.** From the upstream starter, run `fnm use 26.13.1` and `pnpm pi`. Accept project trust, use `/login` if needed, select a model with `/model`, and ask Pi to read the package name from package.json. Quit and run `pnpm ex 00.1 check` there. [Setup guide](https://ratstack.sh/tokenmaxx/setup.md)
3. **Participant repo.** In the lab, run `fnm use` and start `../tokenmaxx/node_modules/.bin/pi`. Use the starter README's first prompt with its Turborepo instruction. Verify strict types, type-aware lint, format checks, behavior tests and the commit hook. Restart Pi, accept trust and follow the second prompt to add the upstream exercises under `workshop/` and root scripts that invoke their Node entry points. The lab workspace selects only `packages/*`; the embedded starter keeps its own lockfile and pnpm version. [Pinned prompts](https://github.com/joelhooks/rat-stack/blob/b89bf6155c0beb22ed2ca152f535f8ca9c5209fb/examples/tokenmaxx-starter/README.md#step-0-before-the-day)
4. **Qualify the lab.** Run `pnpm ex 00.1 check` and `pnpm ex 01.1 done` at the lab root after adding the launchers. Record the model, package and concurrency passes, the valid/broken commit pair, and the Pi bypass stretch. The v3 probe uses the root pnpm lockfile to install the disposable copy offline. [Commit probe](https://github.com/joelhooks/rat-stack/blob/tokenmaxx-starter-v3/examples/tokenmaxx-starter/shared/first-package/fence.ts)

## What comes back to Questmancer

Start with exercise 05.1, which supplies five Pi sessions with known states. Compare the question awaiting a human with the parked build, peer wait and stopped process. Use those cases to ask which distinctions Herdr actually exposes and whether Guild Hall or summons misrepresent any of them. A scripted Pi state supplies an expected outcome for the exercise. A Questmancer change also needs corresponding Herdr evidence. [Exercise 05.1](https://github.com/joelhooks/rat-stack/blob/tokenmaxx-starter-v3/examples/tokenmaxx-starter/exercises/05-several-agents/05.1-who-needs-me/readme.md)

If one case exposes a product gap, bring over its input, expected behavior and rationale. Add a focused failing test against the existing Questmancer adapter, reducer or attention projection, implement the smallest fix there and run `just verify`. If current behavior already covers it, record the mapping. Generic Pi extensions can continue to run from the lab alongside Questmancer. Worker spawning, inter-agent messaging and automated merges are separate product decisions.

## Status

The initial lab commit is clean. The frozen installs, version pins, workspace discovery and starter exercise listing passed locally. Participant trust, model setup, both Step 0 prompts, the live exercise checks and any Questmancer adoption remain pending.
