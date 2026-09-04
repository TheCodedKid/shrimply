use super::*;

pub(super) fn scalar_expression_editor(
    context: &InspectorContext,
    key: SelectedItem,
    field: ScalarField,
) -> gtk::Widget {
    let (source, timeline_id) = {
        let project = context.project.borrow();
        let transform = &project
            .video_item(&key)
            .expect("transform expression item must remain available")
            .transform;
        (
            shrimply_inspector_core::transform::expressions::source(
                transform,
                TransformField::Scalar(field),
            )
            .map(str::to_string),
            field.timeline(transform).id,
        )
    };
    expression_editor(source, crate::rhai_editor::ExpressionValue::Scalar, {
        let controller = context.inspector_core.clone();
        move |source| {
            controller
                .set_transform_expression_source(
                    &shrimply_inspector_core::InspectorTarget::Item(key.clone()),
                    TransformField::Scalar(field),
                    timeline_id,
                    source,
                    shrimply_inspector_core::InspectorCommit::Coalesced(
                        shrimply_inspector_core::transform::expressions::SOURCE_COMMIT,
                    ),
                )
                .expect("transform expression source could not be updated")
        }
    })
}

pub(super) fn vec2_expression_editor(
    context: &InspectorContext,
    key: SelectedItem,
    field: Vec2Field,
) -> gtk::Widget {
    let (source, timeline_id) = {
        let project = context.project.borrow();
        let transform = &project
            .video_item(&key)
            .expect("transform expression item must remain available")
            .transform;
        (
            shrimply_inspector_core::transform::expressions::source(
                transform,
                TransformField::Vec2(field),
            )
            .map(str::to_string),
            field.timeline(transform).id,
        )
    };
    expression_editor(source, crate::rhai_editor::ExpressionValue::Vec2, {
        let controller = context.inspector_core.clone();
        move |source| {
            controller
                .set_transform_expression_source(
                    &shrimply_inspector_core::InspectorTarget::Item(key.clone()),
                    TransformField::Vec2(field),
                    timeline_id,
                    source,
                    shrimply_inspector_core::InspectorCommit::Coalesced(
                        shrimply_inspector_core::transform::expressions::SOURCE_COMMIT,
                    ),
                )
                .expect("transform expression source could not be updated")
        }
    })
}

fn expression_editor(
    mut source: Option<String>,
    value: crate::rhai_editor::ExpressionValue,
    update: impl Fn(String) + 'static,
) -> gtk::Widget {
    source.get_or_insert_with(|| match value {
        crate::rhai_editor::ExpressionValue::Scalar => {
            shrimply_inspector_core::timeline_value::SCALAR_EXPRESSION_DEFAULT.to_string()
        }
        crate::rhai_editor::ExpressionValue::Vec2 => {
            shrimply_inspector_core::timeline_value::VECTOR2_EXPRESSION_DEFAULT.to_string()
        }
        _ => unreachable!("transform expression editor received an unsupported value type"),
    });
    crate::rhai_editor::editor(source, value, update)
}

pub(super) fn set_expression_enabled(
    context: &InspectorContext,
    key: SelectedItem,
    field: TransformField,
    enabled: bool,
) -> bool {
    let timeline_id = {
        let project = context.project.borrow();
        let Some(transform) = project.video_item(&key).map(|item| &item.transform) else {
            return false;
        };
        if shrimply_inspector_core::transform::expressions::enabled(transform, field) == enabled {
            return false;
        }
        match field {
            TransformField::Vec2(field) => field.timeline(transform).id,
            TransformField::Scalar(field) => field.timeline(transform).id,
        }
    };
    context
        .inspector_core
        .set_transform_expression_enabled(
            &shrimply_inspector_core::InspectorTarget::Item(key),
            field,
            timeline_id,
            enabled,
            shrimply_inspector_core::InspectorCommit::Immediate(
                shrimply_inspector_core::transform::expressions::TOGGLE_COMMIT,
            ),
        )
        .is_ok()
}
