# Sorcerer: Storm Channeler study

2026-11-17. The user selected **Storm Channeler** after reporting that the
current Sorcerer did not read as that class. This is a review-only matched world
sprite and native-card proposal. No production asset, persona, runtime or live
party was changed.

## Review images

- [World sprite and quiet poses](01-storm-world-study.png): current production
  master beside the proposed working, resting and unknown studies, at enlarged
  and native pixel sizes.
- [Guild Hall and Delve placement](02-storm-room-study.png): current and proposed
  masters on matching production backgrounds and actor coordinates.
- [Paired card comparison](03-storm-card-study.png): current native card beside
  the new card draft, with the proposed world master as an inset.
- [Unmodified generated card draft](storm-channeler-card-draft.png).

The proposed identity uses swept dark hair with a pale streak, a raised collar,
an asymmetric plum mantle over slate clothing, and a teal orb. The orb remains
visible while resting or unknown. Quiet poses do not assert successful work.
The card and world sprite share these cues; their resolutions remain independent.

## Method and provenance

`candidates.json` contains three hand-authored indexed 16×24 pixel studies.
`export.rs` decodes them through production asset code, checks the production
foot anchors, and uses the production room backgrounds, actor placement and
half-block adapter. Candidates are blitted **after** room lighting; this is a
static placement study, not a production animation or lighting acceptance run.
All three poses use the same working-station coordinates for comparison.

The native card was generated with the built-in `image_gen` tool and refined once
for a larger head and stockier proportions. The selected RGBA output is copied
without modification into this directory. See [generation provenance](card-generation.json),
[initial prompt](card-prompt.txt) and [refinement prompt](card-refinement-prompt.txt).
The original tool output paths are retained in the provenance record.

The card is 1086×1448 pixels. It has transparent padding around the visible
figure, but 34 pixels at the left canvas edge have alpha 1/255. The other three
edges are entirely transparent. Pixels with alpha greater than 8 occupy
`(79, 49, 1047, 1424)`. The faint edge residue is retained honestly in the draft;
production integration still needs an explicit asset-preparation decision.

To regenerate the review sheets with Pillow available:

```sh
REVIEW_PYTHON=python3.13 bash docs/design/reviews/2026-11-17-sorcerer-storm-study/regenerate.sh
```

This compiles the review exporter against the current checkout and rewrites this
study's outputs. It does not regenerate the AI card or replace production assets.
The local run used the existing cached Pillow package through `PYTHONPATH`.

## Evidence ledger

- **Result:** Complete for the review-study scope; visual approval pending.
- **Claim:** The paired proposal is reviewable in isolation, with a persistent
  orb in quiet world poses and unchanged production source.
- **Evidence:** [Verification receipt](verification.json), [source baseline](source-baseline.json),
  compiled exporter assertions, lossless half-block RGB conversion, unchanged
  pixels outside actor bounds in both rooms, and identical PNG hashes across
  two generations. All 230 baseline source/configuration files remained unchanged.
  Exporter formatting, shell syntax and `git diff --check` passed.
- **Remaining uncertainty:** User art approval, all eight production poses,
  saved-persona colour substitution, card fallback integration, production PNG
  preparation and native terminal appearance remain unperformed. The full app
  gate was not rerun for these review-only files. The preceding Librarian slice's
  793 Rust tests / 64 runs and 28 shell tests are separate evidence.
- **Next:** Review the matched world/card identity. If approved, integrate a
  bounded Sorcerer-only slice with all pose and fallback contracts, focused
  regression coverage, the full gate and a fresh native visual check. Keep the
  synthetic party open for that review.
