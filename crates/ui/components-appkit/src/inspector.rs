use crate::{
    FrameGraph, MultilineTextInput, action, column_append, column_stack,
    control_row_with_suffix, row_stack,
};
use block2::RcBlock;
use objc2::MainThreadOnly;
use objc2::rc::Retained;
use objc2_app_kit::{
    NSAnimationContext, NSButton, NSButtonType, NSColor, NSControlStateValueOn, NSGlassEffectView,
    NSGlassEffectViewStyle, NSImage, NSLayoutConstraint, NSStackView, NSTextAlignment, NSTextField,
    NSView,
};
use objc2_foundation::{MainThreadMarker, NSRect, NSString};
use shrimply_component_core::layered::LayeredPropertyController;
use std::{cell::RefCell, rc::Rc};

const CARD_GAP: f64 = 8.0;
const CARD_ANIMATION_SECONDS: f64 = 0.18;
type ExpansionHandlers = Rc<RefCell<Vec<Box<dyn Fn(bool, bool)>>>>;

pub struct InspectorCard {
    root: Retained<NSGlassEffectView>,
    controls: Retained<NSStackView>,
    expansion_handlers: ExpansionHandlers,
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
        let vertical = column_stack(0.0, mtm);
        let header = row_stack(4.0, mtm);
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
        let controls = column_stack(CARD_GAP, mtm);
        let controls_container = inset_view(&controls, 12.0, 12.0, 4.0, 12.0, mtm);
        controls_container.setHidden(!expanded);
        let animation_height = controls_container
            .heightAnchor()
            .constraintEqualToConstant(0.0);
        let callback_container = controls_container.clone();
        let callback_root = root.clone();
        let callback_height = animation_height.clone();
        let expansion_handlers = Rc::new(RefCell::new(Vec::<Box<dyn Fn(bool, bool)>>::new()));
        let callback_handlers = expansion_handlers.clone();
        action::attach(
            &disclosure,
            move |control| {
                let expanding = callback_container.isHidden();
                for handler in callback_handlers.borrow().iter() {
                    handler(expanding, false);
                }
                animate_card_content(
                    callback_root.clone(),
                    callback_container.clone(),
                    callback_height.clone(),
                    expanding,
                    callback_handlers.clone(),
                );
                control
                    .downcast_ref::<NSButton>()
                    .expect("disclosure sender")
                    .setImage(Some(&symbol(
                        if expanding {
                            "chevron.down"
                        } else {
                            "chevron.right"
                        },
                        "Expand",
                    )));
            },
            mtm,
        );
        column_append(&vertical, &inset_view(&header, 8.0, 8.0, 6.0, 6.0, mtm));
        column_append(&vertical, &controls_container);
        root.setContentView(Some(&vertical));
        Self {
            root,
            controls,
            expansion_handlers,
        }
    }

    pub fn append(&self, child: &NSView) {
        column_append(&self.controls, child);
    }
    pub fn view(&self) -> &NSGlassEffectView {
        &self.root
    }

    pub fn connect_expansion(&self, handler: impl Fn(bool, bool) + 'static) {
        self.expansion_handlers.borrow_mut().push(Box::new(handler));
    }
}

fn animate_card_content(
    root: Retained<NSGlassEffectView>,
    container: Retained<NSView>,
    height: Retained<NSLayoutConstraint>,
    expanding: bool,
    handlers: ExpansionHandlers,
) {
    root.layoutSubtreeIfNeeded();
    if expanding {
        container.setHidden(false);
        height.setActive(false);
        root.layoutSubtreeIfNeeded();
        let target = container.fittingSize().height;
        height.setConstant(0.0);
        height.setActive(true);
        root.layoutSubtreeIfNeeded();
        animate_height(root, container, height, target, true, handlers);
    } else {
        height.setConstant(container.frame().size.height);
        height.setActive(true);
        root.layoutSubtreeIfNeeded();
        animate_height(root, container, height, 0.0, false, handlers);
    }
}

fn animate_height(
    root: Retained<NSGlassEffectView>,
    container: Retained<NSView>,
    height: Retained<NSLayoutConstraint>,
    target: f64,
    expanding: bool,
    handlers: ExpansionHandlers,
) {
    let changes_root = root.clone();
    let changes_height = height.clone();
    let changes = RcBlock::new(move |context: std::ptr::NonNull<NSAnimationContext>| {
        unsafe { context.as_ref() }.setDuration(CARD_ANIMATION_SECONDS);
        unsafe { context.as_ref() }.setAllowsImplicitAnimation(true);
        changes_height.setConstant(target);
        changes_root.layoutSubtreeIfNeeded();
    });
    let completion_root = root;
    let completion_container = container;
    let completion_height = height;
    let completion = RcBlock::new(move || {
        completion_container.setHidden(!expanding);
        completion_height.setActive(false);
        completion_root.layoutSubtreeIfNeeded();
        for handler in handlers.borrow().iter() {
            handler(expanding, true);
        }
    });
    NSAnimationContext::runAnimationGroup_completionHandler(&changes, Some(&completion));
}

fn inset_view(
    child: &NSView,
    left: f64,
    right: f64,
    top: f64,
    bottom: f64,
    mtm: MainThreadMarker,
) -> Retained<NSView> {
    let wrapper = NSView::new(mtm);
    child.setTranslatesAutoresizingMaskIntoConstraints(false);
    wrapper.addSubview(child);
    for constraint in [
        child.leadingAnchor().constraintEqualToAnchor_constant(&wrapper.leadingAnchor(), left),
        child.trailingAnchor().constraintEqualToAnchor_constant(&wrapper.trailingAnchor(), -right),
        child.topAnchor().constraintEqualToAnchor_constant(&wrapper.topAnchor(), top),
        child.bottomAnchor().constraintEqualToAnchor_constant(&wrapper.bottomAnchor(), -bottom),
    ] {
        constraint.setActive(true);
    }
    wrapper
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
        let root = column_stack(4.0, mtm);
        let output_field = NSTextField::labelWithString(&NSString::from_str(output), mtm);
        output_field.setAlignment(NSTextAlignment::Right);
        output_field.setTextColor(Some(&objc2_app_kit::NSColor::secondaryLabelColor()));
        let callback_output = output_field.clone();
        let editor = MultilineTextInput::code(
            source,
            180.0,
            move |source| {
                callback_output.setStringValue(&NSString::from_str(&on_edit(source)));
                true
            },
            || {},
            mtm,
        );
        column_append(&root, editor.view());
        column_append(&root, &output_field);
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
    controller: LayeredPropertyController,
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
        let root = column_stack(6.0, mtm);
        let keyframes = unsafe {
            NSButton::buttonWithImage_target_action(
                &symbol("stopwatch", "Toggle keyframes"),
                None,
                None,
                mtm,
            )
        };
        keyframes.setButtonType(NSButtonType::PushOnPushOff);
        keyframes.setBordered(false);
        keyframes.setToolTip(Some(&NSString::from_str("Toggle keyframes")));
        set_toggle_tint(&keyframes, false);
        let expression = unsafe {
            NSButton::buttonWithImage_target_action(
                &symbol("chevron.left.forwardslash.chevron.right", "Toggle expression"),
                None,
                None,
                mtm,
            )
        };
        expression.setButtonType(NSButtonType::PushOnPushOff);
        expression.setBordered(false);
        expression.setToolTip(Some(&NSString::from_str("Toggle expression")));
        set_toggle_tint(&expression, false);
        let suffix = row_stack(4.0, mtm);
        suffix.addArrangedSubview(&keyframes);
        suffix.addArrangedSubview(&expression);
        column_append(
            &root,
            &control_row_with_suffix(label, editor, Some(&suffix), mtm),
        );
        graph.view().setHidden(true);
        expression_editor.view().setHidden(true);
        column_append(&root, graph.view());
        column_append(&root, expression_editor.view());
        graph
            .view()
            .widthAnchor()
            .constraintEqualToAnchor(&root.widthAnchor())
            .setActive(true);
        expression_editor
            .view()
            .widthAnchor()
            .constraintEqualToAnchor(&root.widthAnchor())
            .setActive(true);
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
                set_toggle_tint(control.downcast_ref::<NSButton>().expect("keyframe toggle sender"), active);
                graph_view.setHidden(!active);
                invalidate_ancestor_layout(&graph_view);
            },
            mtm,
        );
        let expression_view = expression_editor.root.clone();
        let expression_controller = controller.clone();
        action::attach(
            &expression,
            move |control| {
                let active = control
                    .downcast_ref::<NSButton>()
                    .expect("expression toggle sender")
                    .state()
                    == NSControlStateValueOn;
                expression_controller.set_expression(active);
                set_toggle_tint(control.downcast_ref::<NSButton>().expect("expression toggle sender"), active);
                expression_view.setHidden(!active);
                invalidate_ancestor_layout(&expression_view);
            },
            mtm,
        );
        Self {
            root,
            keyframes,
            expression,
            graph,
            controller,
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
        self.controller.set_keyframes(active);
        self.keyframes
            .setState(if active { NSControlStateValueOn } else { 0 });
        set_toggle_tint(&self.keyframes, active);
        self.graph.view().setHidden(!active);
        invalidate_ancestor_layout(self.graph.view());
    }
    pub fn set_expression_active(&self, active: bool) {
        self.controller.set_expression(active);
        self.expression
            .setState(if active { NSControlStateValueOn } else { 0 });
        set_toggle_tint(&self.expression, active);
        self._expression_editor.view().setHidden(!active);
        invalidate_ancestor_layout(self._expression_editor.view());
    }
}

fn set_toggle_tint(button: &NSButton, active: bool) {
    button.setContentTintColor(Some(if active {
        &NSColor::controlAccentColor()
    } else {
        &NSColor::secondaryLabelColor()
    }));
}

fn invalidate_ancestor_layout(view: &NSView) {
    let mut ancestor = unsafe { view.superview() };
    while let Some(view) = ancestor {
        view.invalidateIntrinsicContentSize();
        view.setNeedsLayout(true);
        ancestor = unsafe { view.superview() };
    }
}

fn symbol(name: &str, label: &str) -> Retained<NSImage> {
    NSImage::imageWithSystemSymbolName_accessibilityDescription(
        &NSString::from_str(name),
        Some(&NSString::from_str(label)),
    )
    .unwrap_or_else(|| panic!("macOS must provide the {name} system symbol"))
}
