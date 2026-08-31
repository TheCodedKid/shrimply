use std::{cell::Cell, rc::Rc};

use shrimply_component_core::layered::{LayeredEdit, LayeredPropertyController};
use shrimply_gtk_components::ui::{
    ExpressionEditor, FrameGraph, InspectorGraphProperty, NumberPickerHandle,
};

pub fn graph(label: &str, value: f64, log: Rc<dyn Fn(String)>) -> FrameGraph {
    let label = label.to_string();
    FrameGraph::with_actions(
        shrimply_keyframe_graph_ui::FrameGraphState::sample_for_value(value),
        move |_| log(format!("{label} keyframe action")),
    )
}

pub fn edit_handler(
    controller: LayeredPropertyController,
    graph: FrameGraph,
) -> impl Fn(f64) + 'static {
    move |value| {
        if let LayeredEdit::Keyframe(value) = controller.edit(value) {
            graph.edit_value(value);
        }
    }
}

pub fn pair_edit_handler(
    controller: LayeredPropertyController,
    graph: FrameGraph,
    value: Rc<Cell<[f64; 2]>>,
    axis: usize,
) -> impl Fn(f64) + 'static {
    move |axis_value| {
        let mut next = value.get();
        next[axis] = axis_value;
        value.set(next);
        if let LayeredEdit::Keyframe((_, graph_value)) = controller.edit_component(next, axis) {
            graph.edit_value(graph_value);
        }
    }
}

pub struct PropertyConfig<'a> {
    pub label: &'a str,
    pub initial_value: f64,
    pub modes: (bool, bool),
}

pub fn property(
    config: PropertyConfig<'_>,
    editor: &gtk::Widget,
    handle: &NumberPickerHandle,
    graph: FrameGraph,
    controller: LayeredPropertyController,
    log: Rc<dyn Fn(String)>,
    on_graph_value: impl Fn(f64) + 'static,
) -> InspectorGraphProperty {
    handle.set_f64(config.initial_value);
    on_graph_value(config.initial_value);
    graph.connect_status({
        let handle = handle.clone();
        move |status| {
            handle.set_f64(status.value);
            on_graph_value(status.value);
        }
    });
    let expression_label = config.label.to_string();
    let expression = ExpressionEditor::new(
        shrimply_components_demo_core::EXPRESSION_SOURCE,
        Some("rhai"),
        &shrimply_components_demo_core::expression_output(
            shrimply_components_demo_core::EXPRESSION_SOURCE,
        ),
        {
            move |value| {
                log(format!(
                    "{expression_label} expression edited ({} chars)",
                    value.len()
                ));
                shrimply_components_demo_core::expression_output(&value)
            }
        },
    );
    let property = InspectorGraphProperty::new(config.label, editor, graph, expression, controller);
    property.set_keyframes_active(config.modes.0);
    property.set_expression_active(config.modes.1);
    property
}
