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

## Current file gate

- [ ] Finish and obtain fresh acceptance for the active modifier ports.

## Qt files not yet accepted

- [ ] `video.rs`
- [ ] Qt-only native bridge: `backend.rs`
- [ ] Qt-only view composition: `qml/InspectorView.qml`

## Top-level and shared source ports remaining

- [ ] `alpha_mask.rs`
- [ ] `audio_cache.rs`
- [ ] `background.rs`
- [ ] `benchmarking.rs`
- [ ] `camera_source.rs`
- [ ] `font_selector.rs`
- [ ] `gaussian_3d.rs`
- [ ] `generated/common.rs`
- [ ] `generated/mod.rs`
- [ ] `generated/shape.rs`
- [ ] `generated/text.rs`
- [ ] `keyframe_editor.rs` parity/extraction gate
- [ ] `keyframe_graph.rs` parity/extraction gate
- [ ] `paint.rs`
- [ ] `rhai_editor/mod.rs`
- [ ] `scene_3d.rs`
- [ ] `timeline_value/color.rs`
- [ ] `timeline_value/mod.rs`
- [ ] `timeline_value/scalar.rs`
- [ ] `timeline_value/step.rs`
- [ ] `timeline_value/text.rs`
- [ ] `timeline_value/vector/mod.rs`
- [ ] `timeline_value/vector/vec2.rs`
- [ ] `timeline_value/vector/vec3.rs`
- [ ] `transform/expressions.rs`
- [ ] `transform/keyframes.rs`
- [ ] `transform/mod.rs`
- [ ] `video/blender.rs`
- [ ] `video/manim_parameters.rs`
- [ ] `video/pdf.rs`
- [ ] `video/playback.rs`

## Visual modifier ports remaining

- [ ] `modifiers/dithering.rs`
- [ ] `modifiers/halftone.rs`
- [ ] `modifiers/kaleidoscope.rs`
- [ ] `modifiers/kuwahara.rs`
- [ ] `modifiers/mask.rs`
- [ ] `modifiers/object_3d.rs`
- [ ] `modifiers/path_offset.rs`
- [ ] `modifiers/point_light.rs`
- [ ] `modifiers/rasterize.rs`
- [ ] `modifiers/repeat.rs`
- [ ] `modifiers/sam2.rs`
- [ ] `modifiers/sampling.rs`
- [ ] `modifiers/scanlines_crt.rs`
- [ ] `modifiers/shaky_path.rs`
- [ ] `modifiers/shape_3d.rs`
- [ ] `modifiers/sun_light.rs`
- [ ] `modifiers/text_3d.rs`
- [ ] `modifiers/texture_bounds.rs`
- [ ] `modifiers/transparent_fill.rs`

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
