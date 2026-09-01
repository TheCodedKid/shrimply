# Component parity TODO

## Source-parity rule

- [ ] Audit each remaining item one-for-one against its concrete project reference before declaring parity, including native event phases, focus/grab behavior, modifier mapping, selection changes, scrolling limits, callbacks, and resize behavior; do not infer behavior from screenshots or generic toolkit conventions.
- [ ] Never author replacement SVG icons; copy the exact asset from Adwaita or `../breeze-icons` and record the source.
- Concrete references:
  - GTK component behavior and layout: the corresponding implementation in `crates/ui/gtk-components/src/ui/` moved from `ui-foundation`.
  - Inspector category tabs and property mode layout: `crates/ui/inspector-ui/src/list.rs` and the relevant `timeline_value` inspector.
  - Frame graph interaction/state: `crates/ui/inspector-ui/src/keyframe_editor.rs` and `crates/timeline/keyframe-graph-ui/src/controller.rs`.
  - Frame graph drawing: `crates/timeline/keyframe-graph-ui/src/lib.rs`.
  - GTK expression editor language/diagnostics: `crates/ui/inspector-ui/src/rhai_editor/`.
  - Qt Wayland popup anchoring: the working timeline popup implementation in `crates/apps/editor-qt-ui/` and its native surface path.

- [ ] Deliver the existing shared frame graph as a complete reusable inspector keyframe-editor component, not a reduced graph-only showcase or static preview.
- [ ] Unify every layered inspector value behind one reusable path for numbers, vectors, text, booleans, colors, and step values; remove the parallel `timeline_value/layered.rs` assembly path.
  - [ ] Preserve the generic evaluation order `const or interpolated keyframe base -> typed expression input -> final result`; expression variables such as `value`, `x`, `y`, `z`, and color channels must come from the current base value and every expression must run with the current exact fractional frame and dynamic context, including implicit time helpers such as `sin()`.
  - [ ] Make inspector output use the same invalid-expression fallback semantics as rendering.
  - [ ] Route base edits, automatic keyframe insertion, expression enable/source changes, output display, and renderer refresh through shared typed logic instead of per-property mode checks or copied callbacks.
  - [ ] Keep disabled, collapsed, or unmapped graph and expression UI free of playback-frame evaluation while refreshing from the authoritative value when mapped again.
- [ ] Run `make components-check`, launch both showcases, run `make check`, and obtain a final parity review.
