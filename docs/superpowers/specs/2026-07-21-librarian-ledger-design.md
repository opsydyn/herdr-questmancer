# Questmancer Librarian and Ledger

**Status:** Approved design

## Objective

Add the Librarian, an orangutan knowledge keeper who permanently inhabits the
Guild Hall. He is a visible, clickable non-agent character whose dedicated
Librarian's Ledger teaches the user how to operate Questmancer.

Phase 1 is a fixed handbook. A later phase may add project-progress and
agent-state insights, but that future work must reuse the Ledger presentation
boundary rather than expanding Phase 1 into a project scanner, background
service or AI subsystem.

## Product contract

- The Librarian appears exactly once in the canonical and compact Guild Hall.
- He is absent from the Delve.
- He yields minimum-vignette and status-only layouts to the priority
  adventurer; the Ledger remains reachable with `?`.
- Clicking his visible world sprite opens the Librarian's Ledger.
- `?` opens the same Ledger and replaces the separate generic Help parchment.
- He is not an adventurer, Herdr agent, persona, campaign member or Chronicle
  participant.
- He never changes party counts, search results, selection history or durable
  state and can never receive focus, output, counsel or Reviewr commands.
- His world pose and location are deterministic and presentation-owned.
- His card uses `src/assets/librarian.png` when the complete pane transport
  supports native images and an authored RGB portrait otherwise.
- No supported path may leave the Ledger portrait region blank.

## Interaction model

The Ledger is the only help system:

```text
? or click Librarian   open Ledger
j / Right              next page
k / Left               previous page
g / G                  first / last page
Esc or ?               close Ledger
```

Ledger input is modal. While it is open, paging keys must not select or focus
an adventurer behind the parchment. Opening, paging and closing the Ledger emit
no Herdr commands and no persistence writes.

Each fresh open starts on the first page. Page position is transient and is not
restored across closes or application restarts.

Pointer precedence is explicit:

```text
click adventurer -> select it and show its adventurer card
click Librarian  -> open the Librarian's Ledger
click empty room -> dismiss the active card
```

The Librarian is a non-agent scene interactable, so opening the Ledger does not
change the selected adventurer.

## Fixed handbook

Phase 1 contains four typed pages:

1. **Welcome to the Guild**
   - Defines the Questmancer, campaigns, adventurers, Guild Hall and Delve.
   - Explains that the scene projects Herdr facts rather than controlling agent
     state.

2. **Reading the Party**
   - Explains working, needs counsel, completed, resting and unknown states.
   - States that Questmancer never infers an unobserved successful outcome.

3. **Questmancer's Tools**
   - Covers selection, observe, counsel, scrying, search, acknowledgement and
     optional Reviewr inspection.
   - Uses the current production key bindings verbatim.

4. **Keeping a Safe Chronicle**
   - Explains local-only operation, managed-pane exclusion and the difference
     between Herdr-owned facts and Questmancer-owned intent.
   - Describes guarded synthetic testing without embedding run-specific pane
     IDs or claiming that Herdr `0.7.4` can synthesize `done`.

Copy is cozy, dry and practical. Humour belongs in headings and short asides;
instructions and system-state language remain literal. The character must be
original and must not copy a named character, catchphrase or text from another
fantasy property.

## Architecture

### Typed scene interaction

Extend the scene output with non-agent interactables:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SceneInteractable {
    Librarian,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SceneInteractableRegion {
    pub kind: SceneInteractable,
    pub bounds: PixelRect,
}

pub struct SceneFrame {
    pub world: WorldScene,
    pub next_frame_in: Option<Duration>,
    pub actors: Vec<SceneActorRegion>,
    pub interactables: Vec<SceneInteractableRegion>,
}
```

`SceneActorRegion` remains agent-only. Pointer resolution exposes distinct
agent and interactable lookups rather than overloading `AgentKey` or inventing a
synthetic agent identity.

The Guild Hall renderer paints the Librarian and returns his final visible
bounds. The Delve returns no Librarian region. A region is returned only when
the complete world sprite is on-screen.

### Responsive placement

- **Canonical `160x90`:** place him at a stable library station beside the
  shelves or a small authored reading desk.
- **Compact:** calculate a responsive anchor after room composition. Preserve
  his complete native-scale sprite and ensure that he does not overlap any
  returned adventurer region.
- **Vignette/status-only:** omit the world sprite and region. `?` remains the
  accessible entry point.

He is environmental theatre, not an actor in the capacity calculation. Compact
composition must nevertheless reserve his final occupied rectangle before
placing adventurers, or choose a station proven not to intersect their layout.

### Presentation state

Replace the generic help state with a typed Ledger modal and overlay:

```rust
enum Modal {
    None,
    Counsel { /* existing fields */ },
    Search { /* existing fields */ },
    Scrying,
    LibrarianLedger { page: LedgerPageId },
}

enum SceneOverlay {
    None,
    Counsel,
    Search,
    Scrying,
    LibrarianLedger,
}
```

The existing `ShowHelp` action may be renamed to `ToggleLedger` so source and
tests express the product language. `?` maps to that action. Pointer selection
of `SceneInteractable::Librarian` reduces through the same action path; there
must not be a second way to open or manage the Ledger.

### Page model

Use stable typed IDs and fixed content:

```rust
pub enum LedgerPageId {
    Welcome,
    ReadingTheParty,
    QuestmancersTools,
    SafeChronicle,
}

pub struct LedgerPage {
    pub id: LedgerPageId,
    pub title: &'static str,
    pub body: &'static [&'static str],
}
```

Navigation follows catalogue order and clamps at the first and last page.
Rendering receives the selected `LedgerPage`; it does not read or alter domain
state.

This page model is the future extension seam. A later read-only insight
projector may derive additional pages from `Model` facts and feed them to the
same renderer. Phase 1 does not add a provider trait, network call, filesystem
scanner, background task, generic plugin API or persistence schema.

## Art and card presentation

The supplied PNG is card artwork, not a world sprite.

- Author a separate native-scale RGB Librarian world master that matches the
  approved Questmancer sprite family.
- Author an RGB Ledger portrait fallback with a non-empty silhouette.
- Prepare the PNG through the existing Ratatui-image capability path without
  routing it through `AdventurerPersona`.
- Prefer a small general prepared-illustration lookup or an explicit Librarian
  field over a fake class/persona mapping.
- Invalid data, unsupported protocols and pane-transport incompatibility fall
  back to the authored RGB portrait.

The Ledger layout is responsive:

- **Wide:** centered parchment, native or fallback portrait on the left, page
  title and body on the right.
- **Compact:** text-first parchment with a small fallback portrait only when it
  does not reduce readability.
- **Tiny:** page title, concise body and navigation state; omit imagery instead
  of cropping text or creating an empty image box.

The footer states the active page and only the valid modal controls.

## Data and command flow

```text
Guild Hall paint
  -> Librarian RGB sprite
  -> SceneInteractableRegion::Librarian

mouse hit or ?
  -> ToggleLedger action
  -> transient LibrarianLedger modal
  -> fixed LedgerPage projection
  -> parchment + optional native image
```

There is no `AgentCommand`, update-reducer event, Chronicle entry or persistence
command in this flow.

## Failure behaviour

- Missing or invalid native artwork uses the authored RGB fallback.
- Unsupported terminal or Herdr image transport uses the authored RGB fallback.
- A viewport too small for imagery renders readable text without an image.
- A viewport too small for a complete world sprite returns no Librarian hit
  region; `?` still opens the Ledger.
- Unknown page IDs cannot be deserialized because page state is transient.
- Navigation clamps safely and never wraps into unrelated UI state.
- Zero-sized areas remain panic-free and produce no interactable region.

## Storybook and documentation

Storybook must exercise production paths and own every new authored asset once:

- a Guild Hall story showing the persistent Librarian;
- an asset story showing the world master and RGB card fallback;
- an interaction story showing the wide Ledger with native-image capability
  clearly reported;
- compact and tiny Ledger rendering through production layout paths.

Update the Storybook inventory/count, README controls and help description,
manual acceptance guide, workflow contract and `AGENTS.md` architecture notes in
the same implementation slice.

## Automated acceptance

Tests must prove:

1. The Librarian appears exactly once in canonical and compact Guild Halls.
2. His returned region is complete, on-screen and clickable.
3. He is absent from Delve, vignette and status-only layouts.
4. His region never overlaps a returned adventurer region.
5. He never changes party counts, search results or agent regions.
6. Clicking him and pressing `?` produce the same Ledger modal.
7. Ledger paging is bounded and blocks underlying adventurer actions.
8. Opening, paging and closing emit no Herdr commands or persistence writes.
9. Every fixed page is reachable and uses current key bindings.
10. Native PNG and authored fallback paths both render non-empty content.
11. Invalid PNG and unsupported transport cannot produce a blank card.
12. Storybook owns the Librarian world sprite, fallback portrait and Ledger
    interaction exactly once.
13. Arbitrary viewports never return a cropped Librarian region or panic.
14. Formatting, Clippy, all-target/all-feature tests, property tests and shell
    contracts pass.

## Manual acceptance

In Storybook, review:

- canonical and compact Guild Hall placement;
- silhouette fidelity beside the current adventurer masters;
- pointer target correspondence;
- wide native-PNG Ledger;
- authored fallback Ledger;
- compact and tiny text hierarchy;
- page navigation and dismissal.

In a guarded Herdr session, verify `?`, pointer opening, view switching and
selection continuity without sending counsel or changing synthetic agent state.
Do not claim native-image acceptance unless the complete Herdr pane transport
reports the relevant capability.

## Non-goals for Phase 1

- Dynamic project, repository, progress or agent-state insights.
- AI-generated advice or network services.
- Scanning Git, source files, terminal output or worktrees.
- Adding the Librarian to the Herdr/domain agent model.
- Making him searchable, selectable as an adventurer or commandable.
- Persisting Ledger position or read state.
- Adding him to the Delve.
- Sound, speech bubbles, autonomous movement or animation beyond a bounded
  ambient idle pose.
- A generic NPC, dialogue-tree or plugin framework.

## Future direction

A later phase may project truthful, read-only model facts into additional
Ledger pages such as campaign progress, unattended summons or stale activity.
Those insights must remain deterministic projections of already-owned facts,
label inference clearly, avoid polling and never invent agent success. Any
dynamic phase requires a separate design and acceptance boundary.
