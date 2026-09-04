# Qt inspector parity TODO

Active goal: complete a one-for-one Qt port of the GTK inspector, move all
backend-independent behavior into `shrimply-inspector-core`, support both UI
backends in the pipeline, and pass the required fresh reviews.

## Rules

- Find one GTK file, extract shared logic, port it to Qt, obtain a fresh
  per-file adversarial review, then select the next file.
- Remove accepted items from this file; it tracks remaining work only.
- Preserve type-specific frame graphs. Share their keyframe/expression
  show/hide and caching lifecycle.
- Use exact fractions for time, native Qt controls, cards, and vector locks.
- Do not complete the port until two fresh sequential integration reviews pass.

## Qt files not yet accepted

- [ ] `video.rs`
- [ ] Qt-only native bridge: `backend.rs`
- [ ] Qt-only view composition: `qml/InspectorView.qml`

## Top-level and shared source ports remaining

- [ ] `transform/expressions.rs`
- [ ] `transform/keyframes.rs`
- [ ] `transform/mod.rs`
- [ ] `video/blender.rs`
- [ ] `video/manim_parameters.rs`
- [ ] `video/pdf.rs`
- [ ] `video/playback.rs`

## Cross-cutting parity remaining

- [ ] Populate and edit GTK-equivalent raw scalar graphs in every relevant Qt file.
- [ ] Populate and edit GTK-equivalent step graphs for boolean/selector values.
- [ ] Populate and edit GTK-equivalent speed graphs for vector values.
- [ ] Preserve every graph component for vector controls.
- [ ] Port all vector lock behavior one-for-one.
- [ ] Verify every time field uses exact fraction storage and the correct native input.
- [ ] Verify naming/renaming cannot leave an invalid inspector state or crash.
- [ ] Verify collapsed cards and hidden sections do no unnecessary hot-path work.
- [ ] Remove remaining backend-independent duplication from GTK and Qt.
- [ ] Verify undo/redo and external model updates keep keyframe/expression visibility synchronized.

## Required checks and acceptance

- [ ] `make components-check`
- [ ] `make source-size-check`
- [ ] `make cargo-check` with no missing-file stop
- [ ] `make lint` with no missing-file stop
- [ ] `make check` after the complete large change
- [ ] Verify the CI pipeline builds/supports both GTK and Qt variants
- [ ] Fresh adversarial integration review 1: accepted
- [ ] Fresh adversarial integration review 2: accepted
