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

- [ ] Deliver the existing shared frame graph as a complete reusable inspector keyframe-editor component, not a reduced graph-only showcase or static preview.
  - [ ] Replace the production inspector's duplicate interaction controller with the shared Rust controller, keeping GTK and Qt limited to native event/rendering adapters while project time mapping, persistent per-property view state, typed clipboard actions, and preferences remain explicit inputs.
  - [ ] Keep Qt keyboard focus and an explicit mouse grab for the full graph drag so parent scroll/focus handlers cannot steal it.
  - [ ] Match inspector keyboard behavior including playback, navigation, delete/backspace, copy, and paste through reusable action callbacks.
  - [ ] Make graph mutations request changes from the authoritative model and refresh from it; expose live graph/range/frame-step/playhead/snapping inputs and component-identified mutation/playhead/interpolation outputs from both wrappers, with no discarded Qt actions, sample-data-only adapter, rebuilt UUIDs, or forced interpolation modes.
  - [ ] Initialize each demonstrated graph from that property's actual value/model; do not reuse one unrelated sample curve for every Transform property.
  - [ ] Preserve the full typed value when a graph shows a projection such as Vec2 speed; editing either vector axis must not silently replace or ignore the other axis.
  - [ ] Match discrete step graph height, minimum zoom, and half-frame-cell drag positioning, and support speed/text interpolation context actions at release coordinates.
- [ ] Run `make components-check`, launch both showcases, run `make check`, and obtain a final parity review.
