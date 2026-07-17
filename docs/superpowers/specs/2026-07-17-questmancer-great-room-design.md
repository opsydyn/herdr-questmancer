# Questmancer Great Room design

**Status:** Accepted on 2026-07-17

## Purpose

Replace the production Guild Hall's uniform dashboard panels with one coherent,
inhabited Great Room. The new projection must preserve every existing operational
fact and command while fulfilling Questmancer's approved cozy-fantasy direction.

The Great Room is the guild's shared home. It is not a decorative background
behind a dashboard. Campaigns, attention, selected output, history and optional
review actions belong to stable furniture and landmarks within the room.

## Evidence and problems addressed

The Herdr 0.7.4 manual test exposed two objective defects and several product
fidelity gaps:

- the header could say `CONNECTED` while the Scrying Table retained the startup
  message `connecting to Herdr`;
- the Delve's route-home overlay could leave the corrupted text `HOMET PATH`;
- the Guild Hall rendered as equally weighted bordered panels rather than the
  inhabited room required by the creative direction;
- large Quest Board, Scrying and Chronicle regions remained mostly empty;
- compact full-body adventurers were absent from the Guild Hall;
- generic workspace labels such as `~` were not useful campaign names;
- unavailable Reviewr occupied a permanent operational region.

The connected Delve sprite work is successful and remains outside this room
redesign, apart from the route-home regression fix.

## Approved direction

The accepted visual direction is:

1. **The Great Room** rather than a hybrid control rail or horizontal longhouse.
2. **Truthful Stations** rather than showing the whole party physically present.
3. **One Hall, Many Tables** rather than campaign annexes or one campaign at a
   time.
4. A responsive camera model: whole room, cropped room, then landmark pan.

The room must read as warm old stone and timber with rugs, maps, candles, books,
mugs, bedrolls and a communal hearth. ASCII and ANSI-16 modes may simplify the
art, but they must retain the same landmarks, hierarchy and facts.

## Stable landmarks

The Great Room owns these stable places:

- **Guild Door** — connection state, arrivals and departures;
- **Quest Wall** — campaign banners, rollups and global attention;
- **Campaign Tables** — one deterministic expedition table per workspace;
- **Counsel Bell** — blocked attention and projected adventurers;
- **Hearth** — idle adventurers and the room's persistent sense of home;
- **Chronicle Lectern** — bounded factual event history;
- **Scrying Alcove** — selected adventurer identity and recent output;
- **Spoils Desk** — completed unseen work and optional Reviewr action.

These are positions in one authored room, not independent card surfaces. Borders
belong to walls, furniture and architectural edges. They must not recreate the
current panel grid under fantasy labels.

## Truthful Stations

The room derives representation from live semantic state. It stores no separate
location state.

| Real state | Great Room representation |
| --- | --- |
| working | carved token at the owning campaign table |
| blocked | projected full-body likeness at the Counsel Bell |
| done and unseen | full-body return at the Spoils Desk |
| done and seen | calm token at the owning campaign table |
| idle | full-body adventurer resting at the Hearth |
| unknown | shrouded token at the owning campaign table |
| exited | no adventurer; departure remains at the Guild Door and Chronicle |
| focused | relevant table and Scrying Alcove illuminate without relocating anyone |

The projection must show every non-exited adventurer exactly once. Full-body art
therefore communicates literal theatrical presence. Tokens and projections keep
active expeditions visible without claiming that the adventurer is simultaneously
home and in the Delve.

## Multiple campaigns

All workspaces share one Great Room. Each campaign contributes:

- a banner and seal on the Quest Wall;
- a deterministic expedition-table identity;
- a stable table marker and party tokens;
- local attention cues at its table.

The selected campaign's table receives the bright lamp and more detail. Other
campaign tables remain visible rather than collapsing into a hidden selector.
The Counsel Bell identifies the owning campaign whenever it rings. The Hearth,
Chronicle, Scrying Alcove, Spoils Desk and Guild Door remain shared.

Campaign labels use the first meaningful value in this order:

1. a non-generic Herdr workspace label;
2. the checkout directory basename;
3. the workspace ID.

Blank labels and generic shell labels such as `~` do not qualify as meaningful.

## Responsive model

The room changes camera and detail level rather than identity.

### Wide: 120 columns or more

Render the whole Great Room. The Quest Wall, all visible campaign tables, Hearth,
Scrying Alcove and Spoils Desk coexist. The selected campaign receives stronger
lighting, not a separate panel.

### Medium: 80 through 119 columns

Crop the authored room around the selected campaign table. Preserve the Guild
Door, compact Quest Wall and Hearth. Combine the selected table with its Scrying
mirror so identity, attention and recent output remain readable. Other campaigns
remain as banners and table markers.

### Narrow: fewer than 80 columns

Show one landmark camera at a time. A compact room breadcrumb communicates the
current position and the other available landmarks. `tab` pans the room;
selection and agent actions retain their existing bindings. This is a camera
over one room, not a stack of dashboard panels.

Zero-sized and tiny areas must remain panic-free and may fall back to the existing
single-glyph behaviour.

## Projection architecture

The domain model remains authoritative. A pure Great Room projection consumes
the model, terminal area and current selection:

```text
Model + terminal area + selected adventurer
                    |
                    v
room mode + landmarks + campaign tables + representations
```

The implementation should introduce focused types along these lines:

```rust
enum GuildRoomMode {
    WholeRoom,
    CroppedRoom,
    LandmarkCamera,
}

enum GuildLandmark {
    Door,
    QuestWall,
    CampaignTable(WorkspaceId),
    CounselBell,
    Hearth,
    Chronicle,
    Scrying,
    Spoils,
}

enum AdventurerRepresentation {
    Physical { agent: AgentKey, station: GuildLandmark },
    Token { agent: AgentKey, table: WorkspaceId },
    Projection { agent: AgentKey, station: GuildLandmark },
}

struct GuildRoomProjection {
    mode: GuildRoomMode,
    landmarks: Vec<ProjectedLandmark>,
    campaigns: Vec<ProjectedCampaignTable>,
    adventurers: Vec<AdventurerRepresentation>,
}
```

Exact field names may follow existing crate conventions, but the separation is
required: projection computes geometry and representation; widgets only render
the projection.

Render in deterministic layers:

1. room architecture;
2. stable furniture and landmarks;
3. campaign banners and tables;
4. adventurer tokens, projections and full-body sprites;
5. semantic transition effects and selection lighting;
6. readable labels, diagnostics and footer.

The current `guild_hall.rs` should be divided into a pure projection module, a
Great Room scene renderer and small landmark widgets. Wide, medium and narrow
modes must consume the same projection boundary and may not own duplicate state.

## Interaction

Existing agent operations retain their semantics:

- `j/k` changes the selected adventurer and lights the owning campaign table;
- `/` searches adventurers and campaigns;
- `enter` observes the selected real Herdr pane;
- `r` issues counsel;
- `space` acknowledges unread Summons;
- `o` refreshes the selected output;
- `v` inspects spoils when Reviewr is available;
- `tab` pans landmarks only in landmark-camera mode and otherwise preserves the
  existing region-navigation role.

Switching between Guild Hall and Delve preserves selected adventurer and campaign.
Selection may alter illumination and responsive crop but must not perturb stable
campaign-table identity.

## Connection and notice model

Connection state belongs to the Guild Door:

| Connection | Door theatre |
| --- | --- |
| connecting | opening; joining paths |
| connected | open; ordinary room lighting |
| reconnecting | fogged or barred; last tales preserved |
| offline | closed; cached room remains readable |
| incompatible | sealed with exact protocol mismatch |

The stale startup message comes from storing unrelated diagnostics in one
untyped string. Replace that ambiguity with typed notices equivalent to:

```rust
enum Notice {
    ConnectionDiagnostic(String),
    ActionFeedback(String),
    PersistenceDiagnostic(String),
    IntegrationDiagnostic(String),
}
```

A successful connection clears its connection diagnostic without clearing
action feedback, persistence warnings or integration errors. Renderers must not
classify notices by matching string prefixes.

Output loading and failure cloud only the Scrying Alcove. Reconnect preserves
the last complete snapshot. An unavailable Reviewr leaves a quiet decorative
Spoils Desk and no large empty region. An empty guild remains furnished with a
lit Hearth and clear Quest Wall.

## Accessibility and performance

- No state may depend on colour, motion or sprite art alone.
- ASCII and ANSI-16 renderings retain every essential label and action.
- Reduced motion removes rapid effects; no motion eliminates animation without
  removing semantic furniture or state cues.
- The projection is pure and does not load output, spawn work or persist state.
- Static rooms render only on input, Herdr events or relevant semantic deadlines.
- Selection changes request at most one lazy output read under the existing
  command boundary.

## Verification

### Projection and property tests

- Identical model, area and clock produce identical geometry.
- Every non-exited adventurer has exactly one approved representation.
- Presence and attention combinations map to the required stations.
- Landmarks and campaign tables do not overlap or escape their render area.
- Arbitrary terminal sizes, campaign counts and agent mixtures never panic.
- Selection changes presentation without changing stable table identity.
- Campaign-label fallback follows the required order.

Use the existing `proptest` posture for arbitrary geometry and domain mixtures.

### Rendering and Storybook tests

Golden rendering covers:

- empty furnished hall;
- one and several campaigns;
- all Truthful Stations in one mixed scene;
- wide, medium, 80x24 and sub-80 camera modes;
- every connection state;
- Unicode, ASCII, ANSI-16, full motion, reduced motion and no motion;
- unavailable Reviewr and failed Scrying output.

Every new landmark, representation and responsive mode receives a fixed Storybook
story that calls production renderers. Storybook must continue to inventory every
authored production asset once.

### Interaction tests

- selection and search illuminate the correct campaign table;
- landmark-camera `tab` navigation is deterministic;
- observe, counsel, acknowledge, output refresh and Reviewr retain command parity;
- view switching preserves selection;
- no interaction causes duplicate output reads or Chronicle entries.

### Regression tests

- successful connection removes only the startup connection notice;
- connected and connecting messages cannot appear simultaneously;
- the Delve route-home overlay leaves no residual `HOMET PATH` text;
- unavailable integrations do not reserve a large empty operational region;
- the managed Questmancer pane is never recursively observed.

### Manual acceptance

Run a Herdr 0.7.4 synthetic-agent pass and capture wide and 80x24 screenshots.
Confirm blocked Summons, acknowledgement, search, output refresh, restart
persistence and singleton behaviour. A real agent or fixture remains necessary
for `done`, because Herdr 0.7.4 cannot synthesize that state.

## Scope

This slice includes:

- the two reproduced defects;
- the Great Room projection and production renderer;
- Truthful Stations and campaign-table scaling;
- responsive room cameras;
- typed transient notices;
- related Storybook, automated and manual-test documentation updates.

It excludes:

- a redesign of connected Delve architecture beyond the route-home fix;
- new agent states or Herdr protocol concepts;
- new persistence for derived room locations;
- multiple visual themes;
- sound, terminal image protocols or network services;
- the Herdr 0.7.4 customizable-sidebar integration, which remains a separate
  product slice.

## Acceptance criteria

The slice is accepted when:

1. the Guild Hall reads as one inhabited Great Room rather than bordered panels;
2. all stable landmarks are visible at wide width;
3. multiple campaigns coexist as banners and tables in one hall;
4. every non-exited agent has exactly one truthful representation;
5. full-body sprites appear only at semantically valid stations;
6. wide, medium and landmark-camera modes preserve the same room identity;
7. all existing agent commands remain available in their valid contexts;
8. connection theatre cannot contradict the typed connection state;
9. empty, disconnected and integration-unavailable states remain useful and cozy;
10. the `HOMET PATH` and stale startup-message regressions are covered;
11. Storybook exposes every new production asset and scene;
12. formatting, Clippy, all tests, release build and manual Herdr checks pass.
