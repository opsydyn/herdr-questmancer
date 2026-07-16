# Questmancer Storybook final review fix report

Base: `abf03b1`

Implementation commit: `98746cd` (`fix: close Storybook final review findings`)

## Outcome

All eight findings in the consolidated final-review brief are fixed. The
Storybook still has the prescribed 44 story IDs and order, uses the existing
`questmancer-storybook` binary and `storybook` feature, adds no dependency or
unsafe code, and retains feature-off production behavior.

## RED and GREEN evidence

### 1. Head shape and face detail were inert

RED:

```text
cargo test --test persona_art every_head_shape_has_distinct_profile_and_chamber_geometry -- --exact
assertion failed: left == right (1 distinct canvas, expected 4)

cargo test --test persona_art every_face_detail_has_distinct_profile_and_chamber_canvas -- --exact
assertion failed: left == right (1 distinct canvas, expected 6)
```

GREEN:

- The production profile and chamber composers now consume every `HeadShape`
  and `FaceDetail` variant with original pixel treatments.
- The focused tests collect canonical production canvases and prove complete,
  pairwise-distinct families at both claimed resolutions.
- The plan and design contracts now require renderer-consumption evidence and
  intentional visual distinctness rather than non-empty output.

```text
cargo test --test persona_art
18 passed; 0 failed
```

### 2. Chamber catalogue was not the approved matrix

RED:

```text
cargo test --features storybook --test storybook_catalogue chamber_atlas_is_the_complete_pose_by_production_layout_matrix -- --exact
assertion failed: 2 tiles, expected 14
```

GREEN:

- `widgets.chambers` now contains every one of the seven production
  `TheatrePose` values in both full and compact layouts.
- All 14 tiles use production `render_chamber` content, exact matrix labels,
  the production threshold dimensions, and the intended selected state.

```text
cargo test --features storybook --test storybook_catalogue chamber_atlas_is_the_complete_pose_by_production_layout_matrix -- --exact
1 passed; 0 failed
```

### 3. `shows` was not a truthful responsive visibility projection

RED:

```text
cargo test --features storybook --test render_projection
compile errors: unresolved ChamberPresentation, GuildRegion,
PersonaRenderMode, render_projection_for, and chamber_presentation
```

GREEN:

- Production now owns a Storybook-independent semantic `RenderProjection` for
  visible agents, poses, persona mode, chamber presentation, Guild regions and
  selected profile, Delve variants, and connected architecture.
- The production Guild and Delve render paths share their responsive
  structural decisions with that projection.
- Storybook maps projections to the full visible asset set and unions every
  width/height from each story's declared minimum through reference viewport.
  This captures the `60x36` full-chamber branch missed by endpoint sampling.
- Unicode full personas report class, derived gear, ancestry, and every
  appearance trait. ASCII silhouettes do not claim persona-specific art.

```text
cargo test --features storybook --test render_projection
4 passed; 0 failed

cargo test --features storybook --test storybook_catalogue
21 passed; 0 failed
```

### 4. Motion compatibility stories were visually identical

RED:

```text
cargo test --features storybook --test storybook_rendering motion_stories_share_one_phased_baseline_and_only_change_motion -- --exact
failed: deterministic fixture contained no Idle agent

cargo test --features storybook --test storybook_rendering motion_story_production_buffers_are_pairwise_distinct -- --exact
failed: Full == Reduced and Reduced == None
```

GREEN:

- All three stories clone one semantic baseline containing deliberately phased
  Working and Idle adventurers and differ only in `Motion`.
- Full moves the Working and Idle cues, Reduced freezes rapid Working motion
  but retains the slow Resting cue, and None freezes both.
- Production `frame_for` results and complete production buffers are pairwise
  distinguishable.

```text
cargo test --features storybook --test storybook_rendering
29 passed; 0 failed
```

### 5. Panic restoration was not exactly once

RED:

```text
cargo test terminal::tests::panic_restore_and_guard_drop_share_one_exactly_once_gate -- --exact
compile error: no restore_for_panic coordination on TerminalGuard
```

GREEN:

- The guard and panic hook now share an `Arc<RestoreGate>` with an atomic
  exactly-once transition.
- The gate is registered immediately after raw mode succeeds, so partial setup
  failure is still restored by the guard.
- A weak active registration lets the panic hook restore the current session;
  guard drop is then inert. Matching cleanup preserves sequential sessions.

```text
cargo test terminal::tests
6 passed; 0 failed
```

### 6. Party-order regression was weak

RED:

```text
cargo test --features storybook --test storybook_fixtures campaign_fixture_preserves_the_authored_party_order -- --exact
failed: lexical fixture input equalled its sorted form
```

GREEN:

- The fixture now uses deliberately reversed `[zeta, alpha]` input, first
  proves it is non-lexical, and then asserts exact preservation.

```text
cargo test --features storybook --test storybook_fixtures campaign_fixture_preserves_the_authored_party_order -- --exact
1 passed; 0 failed
```

### 7. Literal glyph branch detection was brittle

RED:

- The original catalogue test inferred branches from `. --RUNE--.`-style and
  `====CHAMBER====` glyph observations and could not express the intermediate
  responsive branch structurally.
- The projection RED in finding 3 failed before the shared structural API
  existed.

GREEN:

- Catalogue tests no longer inspect literal rendering glyphs or use
  `reference_has_persona_sprites`.
- Boundary tests assert the shared `ChamberPresentation` and
  `PersonaRenderMode` structure directly.

```text
rg -n 'reference_has_persona_sprites|RUNE--|CHAMBER====' src/storybook tests/storybook_catalogue.rs
no matches
```

### 8. Asset inventory duplicated enum variants manually

RED:

```text
cargo test --features storybook --test storybook_catalogue production_and_storybook_asset_families_expose_exhaustive_collections -- --exact
compile errors: 21 production/Storybook enum families had no exhaustive ALL collection
```

GREEN:

- Production persona, pose, Delve, goblin, and colour-role enums expose
  one-source `ALL` collections.
- Storybook widget, scene, and compatibility enums use the same pattern.
- `AssetId` family slices are generated from those collections. A newly added
  variant is forced through labelling, inventory, and ownership coverage by
  exhaustive matches and catalogue validation.

```text
cargo test --features storybook --test storybook_catalogue production_and_storybook_asset_families_expose_exhaustive_collections -- --exact
1 passed; 0 failed
```

## Final verification matrix

Every required command was rerun after the final implementation changes:

```text
cargo fmt --all --check
PASS

cargo clippy --all-targets --all-features -- -D warnings
PASS

cargo test --all-targets --all-features
PASS

PROPTEST_CASES=1024 cargo test --features storybook --test storybook_properties
4 passed; 0 failed

cargo test --all-targets
PASS (feature-off; Storybook-gated integration targets ran 0 tests as expected)

bash tests/scripts.sh
workflow contracts: valid
scripts: 20 passed

bash -n tests/scripts.sh herdr/install.sh herdr/run.sh herdr/control.sh
PASS

cargo build --release
PASS

git diff --check
PASS
```

## Herdr-free PTY smoke

The production rendering and terminal restoration paths changed, so the smoke
was repeated in a real PTY with all discovered `HERDR_*` variables explicitly
unset:

```text
env -u HERDR_BIN_PATH -u HERDR_PANE_ID -u HERDR_PLUGIN_ID \
  -u HERDR_PLUGIN_STATE_DIR -u HERDR_PLUGIN_ROOT \
  -u HERDR_PLUGIN_CONFIG_DIR -u HERDR_SOCKET_PATH \
  cargo run --features storybook --bin questmancer-storybook
```

Observed actions:

1. The offline fixture catalogue launched and reported `validation: PASS`.
2. `l` moved from the asset atlas to the Widgets category.
3. Enter opened the adventurer-card inspection and rendered production full
   and compact cards.
4. `?` opened and closed the complete Storybook help overlay.
5. Escape returned to the catalogue.
6. `q` exited with status 0; the PTY emitted cursor, mouse-capture, and
   alternate-screen restoration sequences.

No Herdr connection, configuration, persistence, network, or file-write path
was required.

## Self-review

Reviewed the complete `abf03b1..98746cd` diff after the matrix:

- confirmed production owns responsive decisions and has no dependency on
  Storybook `AssetId`;
- confirmed Guild profile projection reports its actual textual class and
  ancestry surface, while full appearance ownership remains tied to the
  production persona composers used by profile cards and chambers;
- confirmed Delve chamber geometry is derived from the same layout result as
  the renderer and covers compact list, connected, and intermediate branches;
- confirmed panic and drop share one restore gate, partial setup remains
  guarded, and sequential sessions cannot clear a newer active gate;
- confirmed no new dependency, unsafe code, story ID/order change, or literal
  glyph observer was introduced;
- confirmed `git diff --check abf03b1..98746cd` passes.

Remaining concern: none known. The semantic projection is intentionally a
production contract now, so future renderer branches must extend it and their
exhaustive boundary tests together.

Delivery state: after committing this report, the final worktree status was
rechecked clean.

## Second final-review wave

Implementation commit: `9274bf2` (`fix: make Delve projection render-authoritative`).

### Finding 1: chamber persona projection overstated visible art

RED:

- `cargo test --test delve_widgets compact_scene_preserves_top_and_bottom_persona_rows -- --exact`
  failed because the Boots and Sabatons 20x8 buffers were identical. The old
  compact chamber gave the packed sprite only five rows, then overwrote its top
  row with the name and clipped its bottom footwear row.
- The production persona projection was size-aware but not pose-aware, so a
  Departed chamber could still claim full or silhouette persona assets even
  though the renderer deliberately replaces the adventurer with departure art.

GREEN:

- `CompactScene` now begins at 14x8 and reserves one name row, all six packed
  sprite rows, and one state row. Smaller chambers use `Text` honestly.
- Persona projection now accepts the production `TheatrePose`; Departed always
  reports `PersonaRenderMode::None` in Unicode and ASCII.
- Full-buffer tests prove Departed output is persona-independent, footwear and
  hair traits survive at the bottom and top of a compact scene, and exact
  14x7/14x8 and full-size thresholds are reported correctly.
- Storybook ownership uses the same pose-aware production projection, and the
  intermediate catalogue assertions now report their actual textual chambers.

```text
cargo test --test delve_widgets
20 passed; 0 failed

cargo test --features storybook --test render_projection
8 passed; 0 failed

cargo test --features storybook --test storybook_catalogue
21 passed; 0 failed
```

### Finding 2: Delve renderer recomputed structural layout

RED:

`cargo test --features storybook --test render_projection` failed to compile
the new structural assertions with six `E0609` errors because `ProjectedAgent`
had no `chamber_area` and `RenderProjection` had no `delve_regions`. The public
semantic projection could not describe the architecture and chamber rectangles
actually consumed by the renderer.

GREEN:

- The production Delve projection now owns body/footer geometry, responsive
  content mode, exact chamber rectangles, exact Delve rectangles and variants,
  active selection, compact paging, campaign-strip geometry, and connection
  overlay geometry.
- `render_with_projection` creates that structure once, maps semantic metadata
  from it, and passes the same projection to the Delve renderer.
- `src/ui/views/delve.rs` consumes projected structures and contains no
  independent campaign derivation, `layout_delves` call, active-Delve remap,
  compact paging, or chamber-rectangle calculation.
- Buffer-backed structural tests cover compact, intermediate, connected,
  multi-Delve, and changed-selection branches. They prove each projected
  chamber contains its rendered adventurer name and each projected Delve region
  contains its production variant architecture marker.
- The motion compatibility fixture was narrowed to a deterministic two-agent
  production layout, and its buffer distinction is tested at a supported
  compact size where the honest full chambers can render motion cues.

```text
cargo test --test delve_rendering
29 passed; 0 failed

cargo test --test persona_art
18 passed; 0 failed

cargo test --features storybook --test storybook_fixtures
6 passed; 0 failed

cargo test --features storybook --test storybook_rendering
29 passed; 0 failed
```

## Second-wave final verification matrix

All required automated checks were rerun after the production changes:

```text
cargo fmt --all --check
PASS

cargo clippy --all-targets --all-features -- -D warnings
PASS

cargo test --all-targets --all-features
PASS

PROPTEST_CASES=1024 cargo test --features storybook --test storybook_properties
4 passed; 0 failed

cargo test --all-targets
PASS (feature-off; Storybook-gated integration targets ran 0 tests as expected)

bash tests/scripts.sh
workflow contracts: valid
scripts: 20 passed

bash -n tests/scripts.sh herdr/install.sh herdr/run.sh herdr/control.sh
PASS

cargo build --release
PASS

git diff --check
PASS
```

After the matrix, the Departed regression assertion was strengthened from
glyph-string equality to complete terminal-buffer equality. The affected check
was rerun:

```text
cargo fmt --all --check
PASS

cargo test --test delve_widgets departed_chambers_are_persona_independent_at_full_and_compact_scene_sizes -- --exact
1 passed; 0 failed

git diff --check
PASS
```

## Second-wave Herdr-free PTY smoke

The real PTY smoke used the Storybook binary with every discovered Herdr
variable explicitly unset:

```text
env -u HERDR_BIN_PATH -u HERDR_PANE_ID -u HERDR_PLUGIN_ID \
  -u HERDR_PLUGIN_STATE_DIR -u HERDR_PLUGIN_ROOT \
  -u HERDR_PLUGIN_CONFIG_DIR -u HERDR_SOCKET_PATH \
  cargo run --features storybook --bin questmancer-storybook
```

Observed actions:

1. The offline catalogue launched with `validation: PASS` and 158 owned assets.
2. `lljjjjj` navigated from the Atlas through Widgets to Full Scenes and selected
   the named Forgotten Library Delve story.
3. Enter inspected the production Delve at the actual 80x24 PTY size. The
   connected architecture, two honest textual chambers, agent states, and
   footer rendered correctly; the height-six connected anchors did not claim
   cropped persona art.
4. `?` opened and closed the help overlay.
5. Escape returned to the catalogue.
6. `q` exited with status 0 and emitted cursor, mouse-capture, and
   alternate-screen restoration sequences.

No Herdr service, Herdr environment, persistence, network, or filesystem write
was used by the Storybook session.

## Second-wave self-review

Reviewed the complete second-wave implementation before committing:

- confirmed Delve architecture, chamber, selection, strip, and overlay geometry
  are calculated only in `src/ui/delve_projection.rs` and consumed directly by
  the renderer;
- confirmed compact, active intermediate, and multi-Delve rendering retain the
  previous draw ordering for architecture, fog, routes, chambers, strip, and
  connection overlays;
- confirmed high-level semantic projection is derived from the same internal
  structure used for the actual frame render;
- confirmed Departed never claims or leaks persona traits, while compact
  Unicode scenes dedicate all six packed rows and ASCII projection remains an
  honest silhouette claim;
- confirmed no Storybook story ID/order or total ownership change was
  introduced; the validated totals remain 44 stories and 158 owned assets;
- confirmed no new dependency, unsafe code, or Storybook-to-production
  dependency was introduced;
- confirmed `git diff --check` and the focused post-matrix regression rerun
  pass.

Remaining concern: none known. The chamber atlas's existing 26x9 outer tile has
a 24x7 inner chamber and now truthfully presents text rather than claiming a
cropped compact persona scene; exact compact persona coverage is exercised by
the dedicated 20x8 production-buffer tests.

## Third final-review wave: connected persona and reference motion

Design commit: `834c98a` (`docs: design connected chamber regression fix`).
Plan commit: `44546fa` (`docs: plan connected chamber regression fix`).
Implementation commit: `1c9509e` (`fix: restore connected Delve persona motion`).

### Correction to the second-wave conclusion

The second-wave report correctly required `CompactScene` to reserve one name
row, all six packed persona rows, and one state row, but it incorrectly treated
the resulting textual height-six connected chambers as an acceptable final
production result. The normal production connected layout itself authored every
chamber with a maximum height of six, so raising the renderer threshold to eight
removed persona and motion art from the entire connected branch. Crop honesty
was preserved, but a feature-off production capability regressed.

The third wave fixes the production geometry rather than weakening the honest
rendering contract: authored connected chambers now use up to eight rows when
their partition permits it. Truly constrained cells retain their actual smaller
height and remain on the textual fallback.

### Finding 1: normal connected Delves lost complete persona scenes

RED:

```text
cargo test --test delve_scene connected_layout_allocates_complete_compact_chambers_when_room_allows -- --exact
FAILED: representative connected anchors were
[ChamberAnchor { width: 36, height: 6 }, ChamberAnchor { width: 36, height: 6 }]

cargo test --test delve_rendering connected_persona_art_preserves_top_and_bottom_rows -- --exact
FAILED: Boots and Sabatons produced identical 120x30 production buffers
("the production connected layout hid the bottom persona row")
```

GREEN:

- `authored_chambers` raises only its production maximum from six to eight:
  `(height / rows).clamp(1, 8)`.
- The existing cell partition remains authoritative, so a cell with fewer than
  eight available rows is not enlarged and cannot claim persona art.
- A feature-off structural test proves a representative 120x30 connected room
  now allocates two complete 36x8 chambers.
- A feature-off full-buffer test proves both bottom footwear and top hair
  changes survive through production connected layout and rendering.
- Exact widget tests continue proving the six persona rows are neither cropped
  nor overwritten, and Departed remains persona-independent.

```text
cargo test --test delve_scene
6 passed; 0 failed

cargo test --test delve_rendering
30 passed; 0 failed

cargo test --test delve_widgets
20 passed; 0 failed

cargo test --features storybook --test render_projection
8 passed; 0 failed
```

### Finding 2: motion stories hid motion at their declared viewport

RED:

```text
cargo test --features storybook --test storybook_rendering motion_story_production_buffers_are_pairwise_distinct -- --exact
FAILED: Full and Reduced production buffers were identical at the declared
130x36 reference viewport ("working motion must distinguish full")
```

The earlier 60x34 assertion exercised a fallback size rather than the story
contract. It passed because compact-list geometry happened to expose full
chambers there, while the declared 130x36 connected canvas remained textual and
ignored each calculated `animation_frame`.

GREEN:

- The motion test now resolves each catalogue `Story`, reads its own
  `Viewport.reference_width` and `Viewport.reference_height`, and renders the
  production application at those exact dimensions.
- It first proves the three motion stories share a comparable declared
  viewport, then requires Full, Reduced, and None buffers to be pairwise
  distinct.
- Restored eight-row connected chambers make the Working Full-motion cue and
  Idle Reduced-motion cue visible without changing the deterministic fixture,
  story metadata, or motion semantics.
- Catalogue projection expectations now truthfully report `CompactScene` for
  the restored 130x36 connected reference branch; minimum textual and
  intermediate full branches remain distinct.

```text
cargo test --features storybook --test storybook_catalogue
21 passed; 0 failed

cargo test --features storybook --test storybook_fixtures
6 passed; 0 failed

cargo test --features storybook --test storybook_rendering
29 passed; 0 failed
```

## Third-wave final verification matrix

Every required command was rerun after implementation commit `1c9509e`:

```text
cargo fmt --all --check
PASS

cargo clippy --all-targets --all-features -- -D warnings
PASS

cargo test --all-targets --all-features
PASS

PROPTEST_CASES=1024 cargo test --features storybook --test storybook_properties
4 passed; 0 failed

cargo test --all-targets
PASS (feature-off; Storybook-gated integration targets ran 0 tests as expected)

bash tests/scripts.sh
workflow contracts: valid
scripts: 20 passed

bash -n tests/scripts.sh herdr/install.sh herdr/run.sh herdr/control.sh
PASS

cargo build --release
PASS

git diff --check
PASS
```

## Third-wave Herdr-free PTY smoke

The changed production connected output was repeated in a real 80x24 PTY with
all discovered Herdr variables explicitly unset:

```text
env -u HERDR_BIN_PATH -u HERDR_PANE_ID -u HERDR_PLUGIN_ID \
  -u HERDR_PLUGIN_STATE_DIR -u HERDR_PLUGIN_ROOT \
  -u HERDR_PLUGIN_CONFIG_DIR -u HERDR_SOCKET_PATH \
  cargo run --features storybook --bin questmancer-storybook
```

Observed actions:

1. The offline catalogue launched with `validation: PASS` and 158 owned assets.
2. The named Forgotten Library story opened through the production renderer at
   the actual 80x24 PTY size.
3. Both connected chambers visibly contained a name row, six complete packed
   persona/scene rows, and a state row; footwear and head rows were present.
4. Compatibility navigation opened Reduced and Full Motion stories whose
   header reported their declared `ref 130x36` metadata. Their connected
   production canvases displayed packed rune/persona motion cues rather than
   textual chambers.
5. `?` opened and closed the complete help overlay, Escape returned to the
   catalogue between inspections, and `q` exited with status 0.
6. Exit emitted cursor, mouse-capture, and alternate-screen restoration
   sequences.

No Herdr service, Herdr environment, persistence, network, or filesystem write
was used by the Storybook session.

## Third-wave self-review

Reviewed the full third-wave implementation and its interaction with every
production chamber height class:

- zero width or height remains `Hidden` and claims no persona;
- nonzero chambers below 14x8 remain `Text` and claim no persona;
- chambers from 14x8 use `CompactScene`, with exactly one name row, six complete
  scene/persona rows, and one state row;
- chambers from 28x10 remain `Full`; their scene is still capped at the same six
  complete persona rows;
- production authored geometry now requests at most eight rows and never
  exceeds its partition, so constrained connected cells remain honest Text;
- Departed overrides every otherwise-visible chamber presentation to
  `PersonaRenderMode::None` in Unicode and ASCII, and complete-buffer tests show
  that changing its persona cannot affect output;
- the Delve renderer still receives the same render-authoritative structural
  projection; no layout or chamber calculation returned to the view;
- motion buffers are rendered from each story's declared reference dimensions,
  not a hardcoded fallback, and the test guards viewport equality before
  comparing buffers;
- no viewport, story ID/order, ownership count, dependency, unsafe code,
  terminal lifecycle, or Storybook-to-production dependency changed. The
  validated totals remain 44 stories and 158 owned assets.

Remaining concern: none known. The feature-off structural/property suites keep
all authored chambers inside their partitions, while the exact presentation and
full-buffer tests jointly guard both constrained honesty and complete persona
visibility.
