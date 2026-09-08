# Questmancer roadmap assessment — 2026-09-08

The Questmancer requested a local candidate commit and a roadmap assessment.
The promoted product work is complete. The next milestone is release acceptance
and distribution. Further feature expansion should follow that milestone.

This assessment checks the current source and tests against `PLAN.md`, the
party-delight plan and the complete candidate review. It does not promote or
implement another feature.

## Roadmap position

| Area | Assessment |
| --- | --- |
| v0.1 engineering | Complete: scene-first runtime, explicit counsel/output effects, persistence, responsive rooms, sidebar and standing |
| Party identity and rituals | Complete and visually approved: all fourteen classes, native-size matching card fallbacks, heraldry and keepsakes |
| Room reaction and history | Complete and visually approved: bounded cat reaction and on-demand factual Chronicle chapters |
| Review tooling | 45 production Storybook stories; current review packs approved |
| Candidate preparation | Full verification, release and packaged-source builds passed; package-size blocker fixed; isolated runtime and cleanup passed |
| Clean candidate | Local commit authorised in this turn; commit-specific qualification must be recorded after that commit |
| Native/live acceptance | Current terminal room/graphics captures and real-agent resting/completion remain outstanding |
| Distribution | Matching 0.1.9 archives, checksums, published installer and live workflow dispatch remain outstanding; publication needs separate authorisation |

The full preparation receipt is [here](2026-09-08-party-candidate/README.md).
Clean qualification logs and `verification.json` are written after the local
commit under `/tmp/questmancer-019-party-clean-qualification/`, outside the
checkout. Check that receipt's commit hash and result before calling this
candidate clean-qualified. It does not replace native or distribution evidence.

## Recommended order

1. **Qualify the local commit.** Run the complete gate, release build, clean
   `cargo package --locked --offline`, and fresh isolated Herdr integration.
   Keep receipts outside the checkout and confirm it remains clean.
2. **Close native/live acceptance.** Capture the current Hall and Delve through
   actual terminal transport, including fallback/native cards and small sizes.
   Observe real-agent resting/completion separately. Automated synthetic reports
   cannot express explicit done; headless action success proves invocation only.
   Respect the existing Computer Use boundary and shared-server ownership rules.
3. **Publish the approved release when authorised.** Check version/tag agreement,
   workflow completion, four target archives and SHA256SUMS, then run the actual
   published installer. Treat the optional registry path as a separate gate.
4. **Choose one subsequent correctness slice.** Chronicle capture semantics are
   the strongest candidate: snapshots currently retain the old Chronicle without
   appending observed transitions, the joined enum also represents unknown
   whereabouts, and workspace removal does not emit campaign closure. Define
   reconnect baselines, what is actually observed and event deduplication before
   changing capture. Preserve honest gaps rather than inferring success.

## Keep deferred

Durable mementos/trophy shelves require event identity, replay, migration and
retention design; the bounded Chronicle cannot reconstruct permanent awards.
They should follow trustworthy capture semantics. Ancestry silhouettes and
unused appearance attributes need a demonstrated recognition benefit. Additional
Hall layout changes and cross-category Storybook navigation are optional polish.
None of these is required to finish the approved party-delight sequence.

Old dashboard, cybercafe and alternate-renderer milestones remain retired.
The historical handoff in the party-delight plan is now explicitly labelled;
it must not reopen class art, keepsakes or chapters that are already approved.

## Subsequent native regression

The Questmancer then accepted the current sidebar/Hall/Delve screenshots while
reporting missing native card and Librarian illustrations in Ghostty. The
[regression repair](2026-09-08-native-portrait-regression/README.md) is the next
release prerequisite. Its new source is outside `9ea8501` qualification; its
record owns verification and user visual confirmation. Native Artificer, Bard
and Librarian restoration was subsequently confirmed with Ghostty screenshots.
