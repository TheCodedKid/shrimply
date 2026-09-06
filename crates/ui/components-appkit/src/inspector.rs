use crate::{FrameGraph, MultilineTextInput, action, control_row, stack};
use objc2::MainThreadOnly;
use objc2::rc::Retained;
use objc2_app_kit::{
    NSButton, NSControlStateValueOn, NSGlassEffectView, NSGlassEffectViewStyle, NSImage,
    NSStackView, NSTextAlignment, NSTextField, NSView,
};
use objc2_foundation::{MainThreadMarker, NSEdgeInsets, NSRect, NSString};
use shrimply_component_core::layered::LayeredPropertyController;

const CARD_GAP: f64 = 8.0;
const CARD_INSET: f64 = 10.0;

pub struct InspectorCard {
    root: Retained<NSGlassEffectView>,
    controls: Retained<NSStackView>,
}

impl InspectorCard {
    pub fn new(
        title: &str,
        expanded: bool,
        on_reset: impl Fn() + 'static,
        mtm: MainThreadMarker,
    ) -> Self {
        let root = NSGlassEffectView::initWithFrame(NSGlassEffectView::alloc(mtm), NSRect::ZERO);
        root.setStyle(NSGlassEffectViewStyle::Regular);
        root.setCornerRadius(8.0);
        let vertical = stack(true, CARD_GAP, mtm);
        vertical.setEdgeInsets(NSEdgeInsets {
            top: CARD_INSET,
            left: CARD_INSET,
            bottom: CARD_INSET,
            right: CARD_INSET,
        });
        let header = stack(false, 4.0, mtm);
        let disclosure = unsafe {
            NSButton::buttonWithImage_target_action(
                &symbol(
                    if expanded {
                        "chevron.down"
                    } else {
                        "chevron.right"
                    },
                    "Expand",
                ),
                None,
                None,
                mtm,
            )
        };
        disclosure.setBordered(false);
        let label = NSTextField::labelWithString(&NSString::from_str(title), mtm);
        label.setFont(Some(&objc2_app_kit::NSFont::boldSystemFontOfSize(
            objc2_app_kit::NSFont::systemFontSize(),
        )));
        let spacer = NSView::new(mtm);
        let reset = unsafe {
            NSButton::buttonWithImage_target_action(
                &symbol("arrow.counterclockwise", "Reset"),
                None,
                None,
                mtm,
            )
        };
        reset.setBordered(false);
        reset.setToolTip(Some(&NSString::from_str("Reset")));
        action::attach(&reset, move |_| on_reset(), mtm);
        header.addArrangedSubview(&disclosure);
        header.addArrangedSubview(&label);
        header.addArrangedSubview(&spacer);
        header.addArrangedSubview(&reset);
        let controls = stack(true, CARD_GAP, mtm);
        controls.setHidden(!expanded);
        let callback_controls = controls.clone();
        action::attach(
            &disclosure,
            move |control| {
                let hidden = !callback_controls.isHidden();
                callback_controls.setHidden(hidden);
                control
                    .downcast_ref::<NSButton>()
                    .expect("disclosure sender")
                    .setImage(Some(&symbol(
                        if hidden {
                            "chevron.right"
                        } else {
                            "chevron.down"
                        },
                        "Expand",
                    )));
            },
            mtm,
        );
        vertical.addArrangedSubview(&header);
        vertical.addArrangedSubview(&controls);
        root.setContentView(Some(&vertical));
        Self { root, controls }
    }

    pub fn append(&self, child: &NSView) {
        self.controls.addArrangedSubview(child);
    }
    pub fn view(&self) -> &NSGlassEffectView {
        &self.root
    }
}

pub struct ExpressionEditor {
    root: Retained<NSStackView>,
    output: Retained<NSTextField>,
    _editor: MultilineTextInput,
}

impl ExpressionEditor {
    pub fn new(
        source: &str,
        output: &str,
        on_edit: impl Fn(String) -> String + 'static,
        mtm: MainThreadMarker,
    ) -> Self {
        let root = stack(true, 4.0, mtm);
        let output_field = NSTextField::labelWithString(&NSString::from_str(output), mtm);
        output_field.setAlignment(NSTextAlignment::Right);
        output_field.setTextColor(Some(&objc2_app_kit::NSColor::secondaryLabelColor()));
        let callback_output = output_field.clone();
        let editor = MultilineTextInput::new(
            source,
            72.0,
            None,
            move |source| {
                callback_output.setStringValue(&NSString::from_str(&on_edit(source)));
                true
            },
            || {},
            mtm,
        );
        root.addArrangedSubview(editor.view());
        root.addArrangedSubview(&output_field);
        Self {
            root,
            output: output_field,
            _editor: editor,
        }
    }

    pub fn view(&self) -> &NSStackView {
        &self.root
    }
    pub fn set_output(&self, value: &str) {
        self.output.setStringValue(&NSString::from_str(value));
    }
}

pub struct InspectorGraphProperty {
    root: Retained<NSStackView>,
    keyframes: Retained<NSButton>,
    expression: Retained<NSButton>,
    graph: FrameGraph,
    _expression_editor: ExpressionEditor,
}

impl InspectorGraphProperty {
    pub fn new(
        label: &str,
        editor: &NSView,
        graph: FrameGraph,
        expression_editor: ExpressionEditor,
        controller: LayeredPropertyController,
        mtm: MainThreadMarker,
    ) -> Self {
        let root = stack(true, 6.0, mtm);
        let controls = stack(false, 4.0, mtm);
        controls.addArrangedSubview(editor);
        let keyframes = unsafe {
            NSButton::checkboxWithTitle_target_action(&NSString::from_str("Keys"), None, None, mtm)
        };
        let expression = unsafe {
            NSButton::checkboxWithTitle_target_action(&NSString::from_str("Code"), None, None, mtm)
        };
        controls.addArrangedSubview(&keyframes);
        controls.addArrangedSubview(&expression);
        root.addArrangedSubview(&control_row(label, &controls, mtm));
        graph.view().setHidden(true);
        expression_editor.view().setHidden(true);
        root.addArrangedSubview(graph.view());
        root.addArrangedSubview(expression_editor.view());
        let graph_view = graph.retained_view();
        let graph_controller = controller.clone();
        action::attach(
            &keyframes,
            move |control| {
                let active = control
                    .downcast_ref::<NSButton>()
                    .expect("keyframe toggle sender")
                    .state()
                    == NSControlStateValueOn;
                graph_controller.set_keyframes(active);
                graph_view.setHidden(!active);
            },
            mtm,
        );
        let expression_view = expression_editor.root.clone();
        action::attach(
            &expression,
            move |control| {
                let active = control
                    .downcast_ref::<NSButton>()
                    .expect("expression toggle sender")
                    .state()
                    == NSControlStateValueOn;
                controller.set_expression(active);
                expression_view.setHidden(!active);
            },
            mtm,
        );
        Self {
            root,
            keyframes,
            expression,
            graph,
            _expression_editor: expression_editor,
        }
    }

    pub fn view(&self) -> &NSStackView {
        &self.root
    }

    pub fn into_view(self) -> Retained<NSStackView> {
        self.root
    }

    pub fn graph(&self) -> &FrameGraph {
        &self.graph
    }
    pub fn set_keyframes_active(&self, active: bool) {
        self.keyframes
            .setState(if active { NSControlStateValueOn } else { 0 });
        self.graph.view().setHidden(!active);
    }
    pub fn set_expression_active(&self, active: bool) {
        self.expression
            .setState(if active { NSControlStateValueOn } else { 0 });
        self._expression_editor.view().setHidden(!active);
    }
}

fn symbol(name: &str, label: &str) -> Retained<NSImage> {
    NSImage::imageWithSystemSymbolName_accessibilityDescription(
        &NSString::from_str(name),
        Some(&NSString::from_str(label)),
    )
    .unwrap_or_else(|| panic!("macOS must provide the {name} system symbol"))
}
