use gtk::prelude::*;
use shrimply_component_core::layered::LayeredPropertyController;

use super::{ExpressionEditor, FrameGraph, InspectorLayeredProperty};

pub struct InspectorGraphProperty {
    property: InspectorLayeredProperty,
    graph: FrameGraph,
}

impl InspectorGraphProperty {
    pub fn new(
        label: &str,
        editor: &impl IsA<gtk::Widget>,
        graph: FrameGraph,
        expression: ExpressionEditor,
        controller: LayeredPropertyController,
    ) -> Self {
        Self::with_expression(label, editor, expression.widget(), graph, controller)
    }

    pub fn with_expression(
        label: &str,
        editor: &impl IsA<gtk::Widget>,
        expression: &impl IsA<gtk::Widget>,
        graph: FrameGraph,
        controller: LayeredPropertyController,
    ) -> Self {
        let property =
            InspectorLayeredProperty::new(label, editor, graph.widget(), expression, controller);
        Self { property, graph }
    }

    pub fn with_expression_wide(
        label: &str,
        editor: &impl IsA<gtk::Widget>,
        expression: &impl IsA<gtk::Widget>,
        graph: FrameGraph,
        controller: LayeredPropertyController,
    ) -> Self {
        let property = InspectorLayeredProperty::new_wide(
            label,
            editor,
            graph.widget(),
            expression,
            controller,
        );
        Self { property, graph }
    }

    pub fn widget(&self) -> &gtk::Widget {
        self.property.widget()
    }

    pub fn graph(&self) -> &FrameGraph {
        &self.graph
    }

    pub fn connect_keyframes_changed(&self, changed: impl Fn(bool) + 'static) {
        self.property.connect_keyframes_changed(changed);
    }

    pub fn connect_expression_changed(&self, changed: impl Fn(bool) + 'static) {
        self.property.connect_expression_changed(changed);
    }

    pub fn set_keyframes_active(&self, active: bool) {
        self.property.set_keyframes_active(active);
    }

    pub fn set_expression_active(&self, active: bool) {
        self.property.set_expression_active(active);
    }
}
