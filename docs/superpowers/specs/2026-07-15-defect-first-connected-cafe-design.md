# Webmaster defect-first remediation and connected café design

## Decision

Fix the current runtime defects and redesign the cybercafé before adding Herdr
0.7.4 integrations. Herdr 0.7.4 is installed and available as the verification
target after this slice; it is not a reason to expand the current scope.

## Defect order

### 1. Exclude the plugin's own pane at the normalization boundary

`DomainState::from_snapshot` currently consumes every agent in the Herdr
snapshot. The plugin's managed `HERDR_PANE_ID` therefore becomes an
`unknown agent` when the snapshot includes the webmaster pane.

This single defect caused the observed symptoms:

- the café rendered an extra unknown workstation;
- selecting it loaded the webmaster TUI output recursively;
- recreating the managed pane produced a misleading persona-stability failure.

The runtime must carry the managed pane identity into snapshot normalization
and exclude it before the pane can enter domain state. The same exclusion must
apply after reconnect and to event-driven additions. No output, focus, reply,
attention, guestbook, or persona operation may target the managed pane.

Add tests for startup snapshots, reconnect snapshots, status events, and pane
creation after reconnect.

### 2. Repair the manual synthetic-agent test path

The previous report targeted an existing Codex-owned pane. Herdr accepted the
command but did not expose `webmaster-smoke` in the snapshot, so the plugin
never received a testable blocked agent. This is a test-environment defect,
not a blocked-state rendering failure.

The manual guide must create or select a dedicated plain pane before sending
synthetic reports. It must record the pane identity and release the synthetic
source during cleanup.

### 3. Preserve output safety

Selected-output loading must be impossible for the managed pane even if stale
state, a malformed snapshot, or an event race attempts to select it. The
selection reducer and command boundary should enforce the same invariant as
normalization.

## Connected café world

The current café paints labels such as `CAFE WALL`, `CABLE RUN`, and `COUNTER`
onto an empty background, then lays dashboard cards over those labels. This
creates arbitrary lines and unused space rather than a coherent pixel world.

The replacement is a scene graph with explicit spatial layers:

```text
café
└── connected bays
    └── workspace
        └── authored room variant
            └── seated agent workstations
```

Every visible mark must represent one of:

- architecture: wall, doorway, counter, window, floor edge;
- furniture: desk, CRT, chair, lamp, shelf, bin;
- agent identity: seated silhouette, accessory, desk prop;
- state theatre: help card, update badge, screensaver, broken link;
- navigation: selected-bay cue, active workstation, doorway transition.

Decorative text is allowed only on an object that owns it, such as a sign,
CRT, counter placard, or guestbook board.

## Workspace and bay mapping

Each Herdr workspace maps to one café bay. Agents belonging to that workspace
are seated together. Workspace status becomes the bay's ambient signal, while
agent status remains local to the workstation.

As the herd grows, the café adds connected bays rather than compressing all
workstations into smaller cards. A bay has a bounded authored capacity; once it
is full, the renderer creates the next connected bay and preserves the existing
room geometry.

The selected agent's bay is the camera's dominant bay. Neighboring bays remain
visible through doorways, windows, or a simplified background silhouette so the
user can understand the larger herd.

## Deterministic room variants

Room variant selection is purely deterministic from workspace identity. It is
not user-configurable in this slice.

Initial authored variants:

1. **Wall row** — repeated desks against a shared back wall; strongest scan
   path for several contributors.
2. **Corner booth** — deeper perspective, aisle, and a more intimate selected
   workstation scene.
3. **Back-room lab** — denser equipment, secondary monitors, cables, and
   utility furniture.

The variant affects architecture and furniture placement only. It must not
change the meaning of presence, attention, focus, or accessibility signals.

Use a stable identity hash with explicit variant names. Do not use frame time,
iteration order, or pane id as the variant seed.

## Rendering rules

- Seated sprites remain full-body silhouettes anchored to chairs and desks.
- Workstations are placed in room coordinates, not independently bordered
  dashboard rectangles.
- Selection uses a lamp, corner marker, and clear focus cue while preserving
  the state pose and label.
- The selected-agent inspector is compact and subordinate to the scene.
- At 80x24, use one dominant bay plus a navigable bay strip or compact room
  fallback; do not revert to arbitrary shared lines.
- At narrow sizes, preserve the actionable compact list as a deliberate
  accessibility fallback.
- ASCII mode uses the same room topology with semantic glyphs and labels.
- Reduced-motion and no-motion modes keep the same geometry and state signals.

## Testing and invariants

Before upgrading or integrating 0.7.4 sidebar features, prove:

- the managed pane never appears in domain state;
- reconnect cannot reintroduce the managed pane;
- output, focus, reply, and attention commands reject the managed pane;
- synthetic manual reports use a dedicated plain pane;
- workspace-to-bay variant mapping is stable across restarts;
- bay ordering is stable and independent of map iteration order;
- seat positions are stable within a bay;
- every workstation is anchored to a room object;
- connected overflow bays remain selectable;
- blocked, done, idle, exited, and disconnected signals remain legible;
- 80x24, ASCII, reduced-motion, and zero-sized rendering remain safe.

## Post-fix Herdr 0.7.4 gate

After the above tests pass on the existing protocol, run the plugin against
Herdr 0.7.4. Only then add optional sidebar integrations, initially as small
ambient tokens or metadata rows. The sidebar should complement the café and
desk rather than duplicate their primary interaction model.

## Implementation sequence

1. Add managed-pane identity to the domain/runtime boundary and tests.
2. Add command-level output/focus/reply guards and reconnect coverage.
3. Update the manual source test guide for a dedicated plain pane.
4. Introduce room coordinates, bay topology, and deterministic variants.
5. Replace arbitrary café chrome with authored architecture and furniture.
6. Rework responsive rendering and golden tests.
7. Run the complete current-version suite.
8. Verify against Herdr 0.7.4.
9. Plan the smallest useful customizable-sidebar integration.
