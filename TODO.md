# Qt inspector parity TODO

Active goal: complete a one-for-one Qt port of the GTK inspector, move all
backend-independent behavior into `shrimply-inspector-core`, support both UI
backends in the pipeline, and pass the required fresh reviews.

## Rules for this checklist

- A checked source file means its GTK behavior and structure were mirrored in
  Qt, its shared logic was extracted into `inspector-core`, relevant checks
  passed, and a fresh per-file reviewer accepted it.
- An existing Qt file is not considered ported until it passes that gate.
- Work in this order: find one GTK file, extract shared logic, port it to Qt,
  run checks, get a fresh review, then select the next file.
- Preserve type-specific frame graphs. Only their keyframe/expression
  show/hide and caching lifecycle is shared.
- Use exact fractions for time, native Qt controls, cards, and vector locks.
- Do not mark the whole port complete until two additional fresh sequential
  adversarial integration reviews accept it.

## Foundation implemented

- [x] Rename `shrimply-inspector-ui` to `shrimply-inspector-gtk` with `git mv`.
- [x] Create `shrimply-inspector-core` and move shared font/keyframe logic into it.
- [x] Add the Qt inspector package and connect it to the Qt editor application.
- [x] Update workspace manifests and Makefile targets to include GTK and Qt.
- [x] Lazily instantiate collapsed Qt card bodies.
- [x] Cache loaded keyframe and expression sections while their card remains alive.
- [x] Add a shared graph gesture-finished action for GTK and Qt undo boundaries.
- [x] Cache successful Pneuma voice-model catalogs by server URL in inspector-core.
- [x] Share preview-provider geometry/evaluation preparation between GTK and Qt,
  including audio-driven expressions, text sizing, tracked cameras, provider
  extensions, snapping, native pointer interaction, compositor exclusion,
  retiring-overlay handoff, coalesced live rendering, and cached Qt overlay
  drawing. Accepted by fresh adversarial review 80 after reviews 72-79 found
  extension, interaction, invalidation, snapping, and lifecycle gaps.

These foundation items remain subject to the per-file and final integration
reviews below.

## Accepted file gates

- [x] `audio_modifiers.rs`
  - GTK: `crates/ui/inspector-gtk/src/audio_modifiers.rs`
  - Qt: `crates/ui/inspector-qt/src/audio_modifiers.rs` plus the native QML controls
  - Shared: `crates/ui/inspector-core/src/audio_modifiers.rs` and controller methods
  - Status: accepted by fresh adversarial review 12.
  - Fresh reviews 6 through 11 rejected it; their concrete findings were fixed.
  - Review 10's zero-digit number mismatch was fixed in shared component-core,
    so GTK and Qt now persist the same quantized value their native picker displays.
  - Review 11's card expansion binding loss was fixed by keeping expansion
    backend-owned and using an explicit request signal plus revision invalidation.

- [x] `info.rs`
  - GTK: `crates/ui/inspector-gtk/src/info.rs`
  - Qt: `crates/ui/inspector-qt/src/info.rs` plus audio/video metadata integration
  - Shared: structured media presentation, stream selection, and persistent
    asset revision lookup in `crates/ui/inspector-core/src/info.rs`
  - Status: accepted by fresh adversarial review 18.
  - Reviews 16 and 17 rejected missing video controls, reveal behavior,
    UI-thread work, stale/unbounded caching, stringly selection, locale dates,
    external asset invalidation, and boolean expression feedback; those findings
    were fixed before review 18.

- [x] `timeline_value/boolean.rs`
  - GTK: `crates/ui/inspector-gtk/src/timeline_value/boolean.rs`
  - Qt: typed layered boolean control, Step graph, and complete graph actions
  - Shared: typed `TimelineBool` edits, discrete keyframe operations, expression
    evaluation, cached audio sampling, and unified layer visibility behavior
  - Status: accepted as part of fresh adversarial review 18.

- [x] `audio.rs`
  - GTK: `crates/ui/inspector-gtk/src/audio.rs`
  - Qt: `crates/ui/inspector-qt/src/audio.rs` plus its native QML/backend support
  - Shared: audio/TTS mutation, model, generation, refresh, and timeline logic in
    `crates/ui/inspector-core`
  - Includes the GTK-inline TTS editor behavior without an unmatched Qt-only
    source split.
  - Status: accepted by fresh adversarial reviews 19 and 20; review 20 covers
    the final bounded TTS generation-status cache.

- [x] `audio_generator.rs`
  - GTK: `crates/ui/inspector-gtk/src/audio_generator.rs`
  - Qt: `crates/ui/inspector-qt/src/audio_generator.rs`
  - Shared: ordered waveform/number controls, visibility, choices, exact numeric
    presentation, conversions, and refresh routing in `crates/ui/inspector-core`
  - Status: accepted by fresh adversarial review 22 after review 21's stale
    waveform-dependent rows were fixed with structural invalidation.

- [x] `caption.rs`
  - GTK: `crates/ui/inspector-gtk/src/caption.rs`
  - Qt: `crates/ui/inspector-qt/src/caption.rs` plus native selector rendering
  - Shared: typed caption card state, defaults, ordered choices, numeric
    presentations, markup help, and native icon mappings in
    `crates/ui/inspector-core/src/caption.rs`
  - Includes atomic card-local resets, disabled-card sensitivity, native Qt
    alignment button groups, dropdown selectors, integer percent inputs, and
    the GTK text-edit/commit lifecycle.
  - Status: accepted by fresh adversarial review 23.

- [x] `item.rs`
  - GTK: `crates/ui/inspector-gtk/src/item.rs`
  - Qt: `crates/ui/inspector-qt/src/item.rs` plus the reusable native card
    integration in `InspectorCard.qml` and `InspectorView.qml`
  - Shared: header action/toggle models, item presentation, preview target
    resolution, and stale-focus validation in
    `crates/ui/inspector-core/src/item.rs`
  - Includes boxed card/flat parity, GTK header ordering, remembered expansion,
    passive pointer and descendant keyboard preview focus, cached accent updates,
    standalone/controlled card expansion, and per-feature sensitivity.
  - Status: accepted by fresh adversarial review 25 after review 24's generic
    toggle-sensitivity and standalone expansion findings were fixed.

- [x] `lib.rs`
  - GTK: `crates/ui/inspector-gtk/src/lib.rs`
  - Qt: `crates/ui/inspector-qt/src/lib.rs`, its backend lifecycle, and native
    view integration
  - Shared: authoritative target resolution, exact item/transition keyframe
    timing, audio sampling, font activation, and refresh routing in
    `crates/ui/inspector-core`
  - Includes address-based transition/item/nested-track precedence, stale-target
    fallback, coalesced listeners, per-target category/expansion/scroll state,
    live boolean playback, asynchronous cached Google-font activation, and
    preview-focus validation.
  - Status: accepted by fresh adversarial review 28 after reviews 26 and 27's
    video keyframe-time and live scroll-restoration findings were fixed.

- [x] `list.rs`
  - GTK: `crates/ui/inspector-gtk/src/list.rs`
  - Qt: `crates/ui/inspector-qt/src/list.rs` plus native cached list/card
    composition in `qml/InspectorView.qml`
  - Shared: normalized per-target category, expansion, default-expansion, and
    scroll state in `crates/ui/inspector-core/src/list.rs`
  - Includes GTK-equivalent card/header ordering, margins, sensitivity,
    translation, modifier auto-expansion, lazy bodies, live cached-delegate
    addressing, and focus retirement before a target swap.
  - Status: accepted by fresh adversarial review 30 after review 29's cached
    delegate addressing and stale-focused-editor findings were fixed.

- [x] `project.rs`
  - GTK: `crates/ui/inspector-gtk/src/project.rs`
  - Qt: `crates/ui/inspector-qt/src/project.rs` plus reusable native project
    settings, locked number-pair, and live-performance components
  - Shared: typed project presentation, direct coalesced name mutation, exact
    settings validation/application, and domain dimension bounds in
    `crates/ui/inspector-core` and project/component core
  - Includes exact custom FPS fractions, staged discard/destructive apply,
    translated track plurals, duration/file info, lazy collapsed performance,
    source-aware draft caching, and the shared 320 px content minimum.
  - Status: accepted by fresh adversarial review 33 after review 32's edit
    coalescing, performance laziness, sizing, and negative-fraction findings
    were fixed. Interrupted review 31 did not count.

- [x] `section.rs`
  - GTK: `crates/ui/inspector-gtk/src/section.rs`
  - Qt: thin `crates/ui/inspector-qt/src/section.rs` boundary plus the reusable
    native control-row/card composition in `InspectorView.qml` and Qt components
  - Shared: ordered control/section presentation, numeric specifications, and
    graph presentation in `crates/ui/inspector-core/src/section.rs`
  - Includes GTK-equivalent row/card spacing and margins, selectable read-only
    fields, interactive performance views, and lazy card bodies retained only
    for the same stable target/category/card identity.
  - Status: accepted by fresh adversarial review 38 after reviews 34-37 found
    read-only interaction, action spacing, performance interaction, body
  retention, cache identity, and Qt binding-order issues.

- [x] `selector.rs`
  - GTK: `crates/ui/inspector-gtk/src/selector.rs`
  - Qt: thin `crates/ui/inspector-qt/src/selector.rs` boundary plus native
    reusable dropdown, button-selector, search-menu, and Step graph components
  - Shared: selector construction, validation, stabilization mutation, typed
    boolean/selector keyframes, exact timing, typed clipboard, and atomic
    discrete graph edits in `crates/ui/inspector-core`
  - Includes translated parameterized choices, required/optional validation,
    alpha-mask sensitivity, fresh value-at-playhead collapse, offset/trim graph
    ranges, exact Fraction frame canonicalization, and success/error graph
    reconciliation for continuous and multi-key drags.
  - Status: accepted by fresh adversarial review 55 after reviews 39-54 found
    selector validation/translation, stabilization, undo, Step semantics,
    clipboard typing, visible-range, stale-value, timing, drag identity,
  collision atomicity, rollback, and snapshot-lifecycle defects.

- [x] `track.rs`
  - GTK: `crates/ui/inspector-gtk/src/track.rs`
  - Qt: `crates/ui/inspector-qt/src/track.rs` using the native reusable switch
    and selector controls
  - Shared: typed presentation, supported-language normalization, Info rows,
    exact commit names, mutation, and media/duration refresh routing in
    `crates/ui/inspector-core/src/track.rs`
  - Includes all three track kinds, the GTK Enabled description, caption-only
    language selection, and one cached presentation per inspector snapshot.
  - Status: accepted by fresh adversarial review 57 after review 56 found and
    the implementation removed full-track serialization on every refresh.

- [x] `transition.rs`
  - GTK: `crates/ui/inspector-gtk/src/transition.rs`
  - Qt: `crates/ui/inspector-qt/src/transition.rs` using native reusable
    selector, number, switch, and color controls
  - Shared: typed target resolution, cached presentation, ordered conditional
    controls, dependent kind defaults, mutation, exact commit names, and
    media/inspector refresh routing in
    `crates/ui/inspector-core/src/transition.rs`
  - Includes visual/audio item and clip transitions, all GTK choices and
    conditional fields, exact ranges/precision, and matching live versus
    immediate undo boundaries.
  - Status: accepted by fresh adversarial review 58.

- [x] `video/playback.rs`
  - GTK: `crates/ui/inspector-gtk/src/video/playback.rs`
  - Qt: `crates/ui/inspector-qt/src/video/playback.rs` using the native reusable
    fraction, selector, switch, and card controls
  - Shared: typed exact-fraction presentation, validation, reset, mutation, and
    refresh behavior in `crates/ui/inspector-core/src/video/playback.rs`
  - Status: accepted by fresh adversarial review 64 after review 61 found stale
    resets, approximate fraction initialization, and duplicated FPS mutation.

## Accepted transform file gate

- [x] `transform.rs`
  - GTK: `crates/ui/inspector-gtk/src/transform.rs`
  - Qt: mirror the complete Transform card with native layered scalar/vector
    controls, Scale lock, expressions, and type-specific frame graphs.
  - Shared: typed transform presentation, reset, values, expressions, and
    keyframe operations in inspector-core.
  - Status: accepted by fresh adversarial review 80 after reviews 73, 75-79
    found cache scoping, interaction, snapping, exclusion-handoff, live-render,
    and pointer lifecycle gaps.

## Accepted modifier file gates

- [x] `modifiers.rs`
  - GTK: `crates/ui/inspector-gtk/src/modifiers.rs`
  - Qt: `crates/ui/inspector-qt/src/modifiers.rs` plus native card/header and
    modifier alpha-mask controls.
  - Shared: modifier presentations and defaults, chain validation and edits,
    controller mutations, alpha-mask controls/keyframes, and preview targeting
    in `crates/ui/inspector-core/src/visual_modifiers.rs`.
  - Includes visible modifier cards, enable/reset/copy/reorder/remove behavior,
    cache-safe removal, modifier/mask preview focus, and the centered add row.
  - Status: accepted by fresh adversarial review 84 after reviews 81-83 found
    reset-default, adapter-only default, alpha-mask, duplicated mutation, and
    strict-lint defects.

- [x] `modifiers/add_menu.rs`
  - GTK: `crates/ui/inspector-gtk/src/modifiers/add_menu.rs` plus the modifier
    chain add/paste row in `modifiers.rs`
  - Qt: use the reusable native `ModifierMenuButton` at the bottom center of
    the Visual category, with the same searchable choices and paste action.
  - Shared: output-state adaptation, catalog/search data, centered defaults,
    chain validation, add, and paste behavior in inspector-core.
  - Status: accepted by fresh adversarial review 65.

- [x] `modifiers/opacity.rs`
  - GTK: `crates/ui/inspector-gtk/src/modifiers/opacity.rs`
  - Qt: `crates/ui/inspector-qt/src/modifiers/opacity.rs`
  - Shared: typed vector/raster opacity presentation and scalar control behavior
    in `crates/ui/inspector-core/src/visual_modifiers/opacity.rs`.
  - Includes the 0–1 range, 0.01 step, two digits, live named undo boundary,
    expression evaluation, and raw-value keyframe graph.
  - Status: accepted by fresh adversarial review 86 after review 85 found the
    named plain-field route intercepting layered values.

- [x] `modifiers/transform.rs`
  - Port the GTK transform modifier body one-for-one with native Qt controls.
  - Reuse the shared scalar/vector keyframe, expression, graph, and lock logic.
  - Includes cached live vector/rotation expressions, dynamic modifier paths,
    complete graph actions, modifier preview focus, and HiDPI-aligned shared
    preview-provider drawing.
  - Status: implementation closed after fresh adversarial reviews 87 and 88
    found and the implementation fixed path, cache, revision, and preview-focus
  integration defects; the user closed the subsequent re-review loop.

- [x] `modifiers/bulge_pinch.rs`
  - Mirrors GTK Center, Radius, and Strength ordering and numeric specifications.
  - Uses shared typed modifier timeline IDs for cached scalar/vector values,
    expressions, graphs, and complete keyframe actions.
  - Status: accepted by a fresh final per-file adversarial review.

- [x] `modifiers/fisheye.rs`
  - Mirrors GTK Intensity and Center controls, including exact ranges, precision,
    units, and unlocked vector behavior.
  - Uses the shared typed modifier timeline and graph-revision paths.
  - Status: accepted by a fresh final per-file adversarial review.

- [x] `modifiers/gaussian_blur.rs`
  - Mirrors GTK's locked Radius Vec2 with `0..100 px`, step `1`, and zero digits.
  - Includes direct scrub values, live speed-graph/expression invalidation, and
    GTK-equivalent final-key deletion.
  - Status: accepted by a fresh final per-file adversarial review.

- [x] `modifiers/lens_distortion.rs`
  - Mirrors GTK Distortion and Center controls with direct typed scalar/vector
    reads, expressions, graphs, and complete keyframe actions.
  - Status: accepted by a fresh per-file adversarial review.

- [x] `modifiers/pixelate_mosaic.rs`
  - Mirrors GTK Block width and Block height integer-style scalar controls,
    including the `1..512` bounds, integer display, and shared timeline behavior.
  - Status: accepted by a fresh per-file adversarial review.

- [x] `modifiers/chromatic_aberration.rs`
  - Mirrors GTK Red X, Red Y, Blue X, and Blue Y ordering and pixel inputs.
  - Uses shared typed scalar timelines, graphs, expressions, and cached refreshes.
  - Status: accepted by a fresh per-file adversarial review.

- [x] `modifiers/vignette.rs`
  - Mirrors GTK Amount, Midpoint, and Softness ordering and `0..1` inputs.
  - Uses shared typed scalar timelines, graphs, expressions, and cached refreshes.
  - Status: accepted by a fresh per-file adversarial review.

- [x] `modifiers/emboss.rs`
  - Mirrors GTK Direction, Depth, and Amount ordering, numeric constraints, and
    rotating direction icon.
  - Uses the shared typed scalar timeline, graph, expression, and cache paths.
  - Status: accepted by a fresh per-file adversarial review.

- [x] `modifiers/directional_blur.rs`
  - Mirrors GTK Radius and rotating Angle controls, including native picker
    defaults, precision, and units.
  - Every indexed scalar mutation validates the timeline UUID first.
  - Status: accepted by a fresh per-file adversarial re-review.

- [x] `modifiers/edge_detection.rs`
  - Mirrors GTK Amount, Edge color, and Background color ordering and specs.
  - Uses shared typed scalar/color timeline lookup, graphs, expressions, and
    UUID validation for every mutation.
  - Status: accepted by a fresh per-file adversarial re-review.

- [x] `modifiers/alpha_outline.rs`
  - Mirrors GTK Width and alpha-enabled Color controls.
  - Uses shared typed scalar/color timelines, UUID guards, color speed graphs,
    unified visibility, and cached graph refresh.
  - Status: accepted by a fresh final per-file adversarial review.

- [x] `modifiers/erode_dilate.rs`
  - Mirrors GTK Operation and integer Radius controls.
  - Uses the shared typed layered-selector expression, UUID, exact step-graph,
    clipboard, and cached graph-revision paths.
  - Status: accepted by a fresh final per-file adversarial re-review.

- [x] `modifiers/sharpen.rs`
  - Mirrors GTK Amount and Radius ordering, ranges, precision, and units.
  - Uses shared UUID-validated scalar timelines and cached graph revisions.
  - Status: accepted by a fresh per-file adversarial review.

- [x] `modifiers/threshold.rs`
  - Mirrors GTK Threshold, Low color, and High color ordering and specs.
  - Uses shared typed scalar/color timelines and generalized color graph logic.
  - Status: accepted by a fresh per-file adversarial review.

- [x] `modifiers/invert.rs`
  - Mirrors GTK's single Amount control and `0..1` numeric behavior.
  - Uses the shared UUID-validated scalar timeline and graph paths.
  - Status: accepted by a fresh per-file adversarial review.

- [x] `modifiers/posterize.rs`
  - Mirrors GTK's integer Levels control with the `2..256` range.
  - Uses the shared UUID-validated scalar timeline and graph paths.
  - Status: accepted by a fresh per-file adversarial review.

- [x] `modifiers/luma_key.rs`
  - Mirrors GTK Threshold, Softness, and Invert ordering and specs.
  - Scalar timelines and the plain Invert switch validate stable UUID identity
    before any indexed mutation.
  - Status: accepted by a fresh per-file adversarial re-review.

- [x] `modifiers/mirror.rs`
  - Mirrors GTK Horizontal and Vertical native switches.
  - Both plain controls carry and validate the modifier UUID before indexed edits.
  - Status: accepted by a fresh per-file adversarial review.

- [x] `modifiers/colorize_duotone.rs`
  - Mirrors GTK Shadow color and Highlight color ordering with alpha.
  - Uses generalized typed color timelines, UUID guards, and cached color graphs.
  - Status: accepted by a fresh per-file adversarial review.

- [x] `modifiers/chroma_key.rs`
  - Mirrors GTK Key color, Similarity, Softness, and Spill ordering and specs.
  - Uses typed UUID-validated color/scalar timelines and cached graph revisions.
  - Status: accepted by a fresh per-file adversarial review.

- [x] `modifiers/drop_shadow.rs`
  - Mirrors GTK Offset, Blur, and alpha Color controls, including unlocked Vec2
    pixel input behavior.
  - Uses shared typed vector/scalar/color timeline and graph paths.
  - Status: accepted by a fresh per-file adversarial review.

- [x] `modifiers/hsv.rs`
  - Mirrors GTK rotating Hue, Saturation, and Value controls and vector-effect dispatch.
  - Uses shared UUID-validated scalar timelines and cached graph revisions.
  - Status: accepted by a fresh per-file adversarial review.

- [x] `modifiers/channel_mixer.rs`
  - Mirrors all nine GTK channel-coefficient rows, labels, order, and `-2..2` specs.
  - Uses shared UUID-validated scalar timelines and cached graph revisions.
  - Status: accepted by a fresh per-file adversarial review.

- [x] `modifiers/color_correction.rs`
  - Mirrors all nine GTK correction rows, including units and rotating Hue.
  - Uses shared UUID-validated scalar timelines and cached graph revisions.
  - Status: accepted by a fresh per-file adversarial review.

- [x] `modifiers/glow_bloom.rs`
  - Mirrors GTK Threshold, Radius, and Intensity ordering and specs.
  - Uses shared UUID-validated scalar timelines and cached graph revisions.
  - Status: accepted by a fresh per-file adversarial review.

- [x] `modifiers/twirl.rs`
  - Mirrors GTK Center, Radius, and rotating Angle ordering and specs.
  - Uses shared UUID-validated scalar/vector timelines and cached graphs.
  - Status: accepted by a fresh per-file adversarial review.

- [x] `modifiers/radial_blur.rs`
  - Mirrors GTK Center, rotating Angle, and integer Samples controls.
  - Explicit integer metadata keeps both native input and graph edits integral.
  - Status: accepted by a fresh final per-file adversarial re-review.

- [x] `modifiers/zoom_blur.rs`
  - Mirrors GTK Center, Strength, and integer Samples controls.
  - Explicit integer metadata applies only to Samples, including graph edits.
  - Status: accepted by a fresh per-file adversarial re-review.

- [x] `modifiers/film_grain.rs`
  - Mirrors GTK Amount, Size, Color, and integer Seed controls.
  - Explicit integer metadata covers both Seed input and graph edits.
  - Status: accepted by a fresh per-file adversarial review.

- [x] `modifiers/displacement_map.rs`
  - Mirrors GTK Amount, Scale, and rotating Phase controls.
  - Uses shared UUID-validated scalar timelines and cached graphs.
  - Status: accepted by a fresh per-file adversarial review.

- [x] `modifiers/wave_ripple.rs`
  - Mirrors GTK Amplitude, Wavelength, rotating Angle, and rotating Phase controls.
  - Uses shared UUID-validated scalar timelines and cached graphs.
  - Status: accepted by a fresh per-file adversarial review.

## Current file gate

- [ ] Select the next GTK modifier file, port it through shared core and native
  Qt controls, then obtain a fresh per-file adversarial review.

## Qt files present but not yet accepted

These files exist, but must be revisited one by one against their GTK source.

- [ ] `video.rs`
- [ ] Qt-only native bridge: `backend.rs`
- [ ] Qt-only view composition: `qml/InspectorView.qml`

## Missing top-level and shared source ports

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

## Missing visual modifier ports

- [ ] `modifiers/cache.rs`
- [ ] `modifiers/corner_pin.rs`
- [ ] `modifiers/crop.rs`
- [ ] `modifiers/dithering.rs`
- [ ] `modifiers/ground.rs`
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
- [ ] `modifiers/text_mask.rs`
- [ ] `modifiers/texture_bounds.rs`
- [ ] `modifiers/transparent_fill.rs`
- [ ] `modifiers/vectorize.rs`

## Cross-cutting parity still required

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
