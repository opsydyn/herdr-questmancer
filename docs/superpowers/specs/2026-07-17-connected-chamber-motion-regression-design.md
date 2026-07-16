# Connected Chamber and Motion Regression Design

## Problem

Production connected-Delve geometry caps authored chambers at six rows. The
crop-honest chamber renderer requires eight rows for one name row, six complete
packed persona rows, and one state row. Consequently normal connected chambers
fall back to text, and Storybook motion stories at their declared 130x36
reference viewport do not display the animation frames they calculate.

## Design

Raise the production authored chamber maximum from six rows to eight. The
existing layout already clamps the requested chamber height to each partition,
so connected rooms with adequate vertical space receive complete eight-row
compact scenes while genuinely constrained cells remain shorter and truthfully
use the textual presentation. Do not add a separate six-row composition or
relax persona projection thresholds.

The renderer continues to consume the production structural projection without
recomputing geometry. Persona ownership remains derived from the resulting
chamber rectangle and pose, so Departed and short textual chambers claim no
persona art.

Motion fixtures continue to share one deterministic Working/Idle baseline and
differ only by `Motion`. At each story's declared reference viewport, the new
eight-row connected chambers make the Working Full-motion frame and Idle
Reduced-motion frame visible, producing pairwise-distinct production buffers.

## Verification

- A feature-off structural test requires representative connected geometry to
  allocate eight-row chambers when its partitions permit it.
- A feature-off production buffer test changes top and bottom persona traits in
  a representative connected chamber and requires visible buffer differences.
- The Storybook motion test reads reference dimensions from each story's
  `Viewport`, confirms the stories share a comparable viewport, and requires
  Full, Reduced, and None production buffers to be pairwise distinct.
- Existing exact chamber threshold and Departed tests continue to prove crop
  honesty at constrained sizes.
- Run the complete final-review verification matrix and repeat the Herdr-free
  PTY smoke because production runtime output changes.

## Scope

No Storybook viewport, story ordering, story ownership, persona composition,
connection behavior, dependency, or terminal lifecycle change is included.
