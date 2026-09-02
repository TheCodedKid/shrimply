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

## Current file gate

- [ ] `video/playback.rs`
  - GTK: `crates/ui/inspector-gtk/src/video/playback.rs`
  - Qt: mirror the GTK playback cards and exact fraction/undo behavior with
    native reusable Qt controls while extracting backend-independent behavior
    into inspector-core.
  - Do not start the next source-file port until a fresh reviewer accepts it.

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
- [ ] `modifiers.rs`
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

- [ ] `modifiers/add_menu.rs`
- [ ] `modifiers/alpha_outline.rs`
- [ ] `modifiers/bulge_pinch.rs`
- [ ] `modifiers/cache.rs`
- [ ] `modifiers/channel_mixer.rs`
- [ ] `modifiers/chroma_key.rs`
- [ ] `modifiers/chromatic_aberration.rs`
- [ ] `modifiers/color_correction.rs`
- [ ] `modifiers/colorize_duotone.rs`
- [ ] `modifiers/corner_pin.rs`
- [ ] `modifiers/crop.rs`
- [ ] `modifiers/directional_blur.rs`
- [ ] `modifiers/displacement_map.rs`
- [ ] `modifiers/dithering.rs`
- [ ] `modifiers/drop_shadow.rs`
- [ ] `modifiers/edge_detection.rs`
- [ ] `modifiers/emboss.rs`
- [ ] `modifiers/erode_dilate.rs`
- [ ] `modifiers/film_grain.rs`
- [ ] `modifiers/fisheye.rs`
- [ ] `modifiers/gaussian_blur.rs`
- [ ] `modifiers/glow_bloom.rs`
- [ ] `modifiers/ground.rs`
- [ ] `modifiers/halftone.rs`
- [ ] `modifiers/hsv.rs`
- [ ] `modifiers/invert.rs`
- [ ] `modifiers/kaleidoscope.rs`
- [ ] `modifiers/kuwahara.rs`
- [ ] `modifiers/lens_distortion.rs`
- [ ] `modifiers/luma_key.rs`
- [ ] `modifiers/mask.rs`
- [ ] `modifiers/mirror.rs`
- [ ] `modifiers/object_3d.rs`
- [ ] `modifiers/opacity.rs`
- [ ] `modifiers/path_offset.rs`
- [ ] `modifiers/pixelate_mosaic.rs`
- [ ] `modifiers/point_light.rs`
- [ ] `modifiers/posterize.rs`
- [ ] `modifiers/radial_blur.rs`
- [ ] `modifiers/rasterize.rs`
- [ ] `modifiers/repeat.rs`
- [ ] `modifiers/sam2.rs`
- [ ] `modifiers/sampling.rs`
- [ ] `modifiers/scanlines_crt.rs`
- [ ] `modifiers/shaky_path.rs`
- [ ] `modifiers/shape_3d.rs`
- [ ] `modifiers/sharpen.rs`
- [ ] `modifiers/sun_light.rs`
- [ ] `modifiers/text_3d.rs`
- [ ] `modifiers/text_mask.rs`
- [ ] `modifiers/texture_bounds.rs`
- [ ] `modifiers/threshold.rs`
- [ ] `modifiers/transform.rs`
- [ ] `modifiers/transparent_fill.rs`
- [ ] `modifiers/twirl.rs`
- [ ] `modifiers/vectorize.rs`
- [ ] `modifiers/vignette.rs`
- [ ] `modifiers/wave_ripple.rs`
- [ ] `modifiers/zoom_blur.rs`

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
