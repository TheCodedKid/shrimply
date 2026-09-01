# Component parity TODO

## Source-parity rule

- [ ] Audit each remaining item one-for-one against its concrete project reference before declaring parity, including native event phases, focus/grab behavior, modifier mapping, selection changes, scrolling limits, callbacks, and resize behavior; do not infer behavior from screenshots or generic toolkit conventions.
- [ ] Never author replacement SVG icons; copy the exact asset from Adwaita or `../breeze-icons` and record the source.
- Concrete references:
  - GTK component behavior and layout: the corresponding implementation in `crates/ui/gtk-components/src/ui/` moved from `ui-foundation`.
  - Inspector category tabs and property mode layout: `crates/ui/inspector-ui/src/list.rs` and the relevant `timeline_value` inspector.
  - Frame graph interaction/state: `crates/ui/inspector-ui/src/keyframe_editor.rs` and `crates/ui/inspector-ui/src/keyframe_editor/graph.rs`.
  - Frame graph drawing: `crates/timeline/keyframe-graph-ui/src/lib.rs`.
  - GTK expression editor language/diagnostics: `crates/ui/inspector-ui/src/rhai_editor/`.
  - Qt Wayland popup anchoring: the working timeline popup implementation in `crates/apps/editor-qt-ui/` and its native surface path.

- [ ] Add reusable, functional Transform inspector cards and property rows to both component libraries, then demonstrate them in both showcases.
  - [ ] Match the real inspector's expandable card, reset action, Position/Anchor/Scale/Shear/Rotation rows, and per-property Keyframe/Expression mode controls.
  - [ ] Match the inspector's internal card geometry: 12 px horizontal/body-bottom insets, 4 px body-top inset, and 8 px between property blocks; remove only the unintended parent gap between the card and Add Modifier action.
  - [ ] Keep the fake Transform as showcase composition of reusable library chunks—expandable card, property row/mode controls, keyframe section, expression editor/output, and modifier menu—while shared Rust owns non-presentation behavior; do not build a monolithic demo component or copy lifecycle logic between toolkits.
  - [ ] Use the same graph-backed reusable property chunk for Position, Anchor, Scale, Shear, and Rotation; every row's Keyframe and Expression buttons must reveal a real frame graph and separate code editor/output section directly below the property.
  - [ ] Route every editable axis through the shared layered value controller: pair edits, including locked cascades, must update every affected component graph and activate the edited component even when Keyframe mode is currently disabled.
  - [ ] Make the layered property abstraction generic over its value editor and injected keyframe/expression bodies, then route every inspector value kind that supports Keyframe/Expression (numbers, vectors, colors, booleans, steps, and text) through it; do not create type-specific copies of the row/mode lifecycle.
  - [ ] Use the reusable controller for production timeline-value edits: Keyframe mode updates/inserts the playhead key, base mode updates the base value, and scalar/Vec2/Vec3/color/boolean/step/text inspectors do not retain per-type copies of that routing.
- [ ] Demonstrate the reusable live-performance component on both toolkits with updating sample metrics rather than static text.
- [ ] Deliver the existing shared frame graph as a complete reusable inspector keyframe-editor component, not a reduced graph-only showcase or static preview.
  - [ ] Replace the production inspector's duplicate interaction controller with the shared Rust controller, keeping GTK and Qt limited to native event/rendering adapters while project time mapping, persistent per-property view state, typed clipboard actions, and preferences remain explicit inputs.
  - [ ] Keep Qt keyboard focus and an explicit mouse grab for the full graph drag so parent scroll/focus handlers cannot steal it.
  - [ ] Match inspector keyboard behavior including playback, navigation, delete/backspace, copy, and paste through reusable action callbacks.
  - [ ] Make graph mutations request changes from the authoritative model and refresh from it; expose live graph/range/frame-step/playhead/snapping inputs and component-identified mutation/playhead/interpolation outputs from both wrappers, with no discarded Qt actions, sample-data-only adapter, rebuilt UUIDs, or forced interpolation modes.
  - [ ] Initialize each demonstrated graph from that property's actual value/model; do not reuse one unrelated sample curve for every Transform property.
  - [ ] Preserve the full typed value when a graph shows a projection such as Vec2 speed; editing either vector axis must not silently replace or ignore the other axis.
  - [ ] Match discrete step graph height, minimum zoom, and half-frame-cell drag positioning, and support speed/text interpolation context actions at release coordinates.
- [ ] Run `make components-check`, launch both showcases, run `make check`, and obtain a final parity review.
