use gtk::prelude::*;
use shrimply_component_core::layered::LayeredPropertyController;

use super::{ExpressionEditor, FrameGraph, InspectorPropertyRow};

pub struct InspectorGraphProperty {
    row: InspectorPropertyRow,
    graph: FrameGraph,
    controller: LayeredPropertyController,
}

impl InspectorGraphProperty {
    pub fn new(
        label: &str,
        editor: &impl IsA<gtk::Widget>,
        graph: FrameGraph,
        expression: ExpressionEditor,
        controller: LayeredPropertyController,
    ) -> Self {
        let row = InspectorPropertyRow::new(label, editor);
        row.set_keyframe_section(graph.widget());
        row.set_expression_section(expression.widget());
        row.connect_keyframes_changed({
            let controller = controller.clone();
            move |enabled| controller.set_keyframes(enabled)
        });
        row.connect_expression_changed({
            let controller = controller.clone();
            move |enabled| controller.set_expression(enabled)
        });
        Self {
            row,
            graph,
            controller,
        }
    }

    pub fn widget(&self) -> &gtk::Widget {
        self.row.widget()
    }

    pub fn graph(&self) -> &FrameGraph {
        &self.graph
    }

    pub fn set_keyframes_active(&self, active: bool) {
        self.controller.set_keyframes(active);
        self.row.set_keyframes_active(active);
    }

    pub fn set_expression_active(&self, active: bool) {
        self.controller.set_expression(active);
        self.row.set_expression_active(active);
    }
}
