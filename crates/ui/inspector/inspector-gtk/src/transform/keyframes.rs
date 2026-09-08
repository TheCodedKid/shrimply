use super::*;
use shrimply_inspector_core::{
    AudioModifierKeyframeMove, InspectorCommit, InspectorTarget, NumberConstraint,
    transform::TRANSFORM_KEYFRAME_COMMITS,
};

struct TransformKeyframeEditorInput {
    graph_data: KeyframeGraph,
    refresh_graph: Rc<dyn Fn() -> KeyframeGraph>,
    visible_area: (Time, Time),
    view_state_scope: &'static str,
    actions: keyframe_editor::KeyframeEditorActions,
}

pub(super) fn vec2_keyframe_body_editor(
    context: &InspectorContext,
    key: SelectedItem,
    field: Vec2Field,
) -> FrameGraph {
    let graph_project = context.project.clone();
    let graph_key = key.clone();
    let transform_field = TransformField::Vec2(field);
    let timeline_id = transform_field.timeline_id(
        &context
            .project
            .borrow()
            .video_item(&key)
            .expect("selected transform item must remain available")
            .transform,
    );
    build_transform_keyframe_editor(
        context,
        TransformKeyframeEditorInput {
            graph_data: vec2_speed_graph(&context.project.borrow(), &key, field),
            refresh_graph: Rc::new(move || {
                vec2_speed_graph(&graph_project.borrow(), &graph_key, field)
            }),
            visible_area: selected_video_visible_area(context, key.clone()),
            view_state_scope: field.path(),
            actions: transform_keyframe_actions(
                context,
                InspectorTarget::Item(key),
                transform_field,
                timeline_id,
            ),
        },
    )
}

pub(super) fn scalar_keyframe_body_editor(
    context: &InspectorContext,
    key: SelectedItem,
    field: ScalarField,
    display: ResolvedTransform,
) -> FrameGraph {
    let graph_project = context.project.clone();
    let graph_key = key.clone();
    let transform_field = TransformField::Scalar(field);
    let timeline_id = transform_field.timeline_id(
        &context
            .project
            .borrow()
            .video_item(&key)
            .expect("selected transform item must remain available")
            .transform,
    );
    build_transform_keyframe_editor(
        context,
        TransformKeyframeEditorInput {
            graph_data: scalar_value_graph(&context.project.borrow(), &key, field, display),
            refresh_graph: Rc::new(move || {
                scalar_value_graph(&graph_project.borrow(), &graph_key, field, display)
            }),
            visible_area: selected_video_visible_area(context, key.clone()),
            view_state_scope: field.path(),
            actions: transform_keyframe_actions(
                context,
                InspectorTarget::Item(key),
                transform_field,
                timeline_id,
            ),
        },
    )
}

fn build_transform_keyframe_editor(
    context: &InspectorContext,
    input: TransformKeyframeEditorInput,
) -> FrameGraph {
    let refresh_graph = input.refresh_graph.clone();
    let built = keyframe_editor::build(
        context,
        input.graph_data,
        input.visible_area,
        input.view_state_scope,
        input.actions,
    );
    keyframe_editor::connect_graph_refresh(
        context,
        "inspector transform keyframe graph refresh",
        &built,
        move || Some(refresh_graph()),
    );
    built.frame_graph
}

fn transform_keyframe_actions(
    context: &InspectorContext,
    target: InspectorTarget,
    field: TransformField,
    timeline_id: uuid::Uuid,
) -> keyframe_editor::KeyframeEditorActions {
    let path = field.path();
    let controller = context.inspector_core.clone();
    let add_controller = controller.clone();
    let add_target = target.clone();
    let delete_controller = controller.clone();
    let delete_target = target.clone();
    let move_controller = controller.clone();
    let move_target = target.clone();
    let copy_controller = controller.clone();
    let copy_target = target.clone();
    let paste_controller = controller.clone();
    let paste_target = target.clone();
    let interpolation_controller = controller.clone();
    let interpolation_target = target.clone();
    keyframe_editor::KeyframeEditorActions {
        add_at_time: Rc::new(move |time| {
            warned(
                "add GTK transform keyframe",
                add_controller
                    .ensure_timeline(&add_target, path, timeline_id)
                    .and_then(|()| match field {
                        TransformField::Vec2(_) => add_controller.add_vector2_keyframe(
                            &add_target,
                            path,
                            time,
                            InspectorCommit::Immediate(TRANSFORM_KEYFRAME_COMMITS.add),
                        ),
                        TransformField::Scalar(_) => add_controller.add_scalar_keyframe(
                            &add_target,
                            path,
                            time,
                            NumberConstraint::default(),
                            InspectorCommit::Immediate(TRANSFORM_KEYFRAME_COMMITS.add),
                        ),
                    }),
            );
        }),
        delete_at_time: Rc::new(move |time| {
            warned(
                "delete GTK transform keyframe",
                delete_controller
                    .ensure_timeline(&delete_target, path, timeline_id)
                    .and_then(|()| match field {
                        TransformField::Vec2(_) => delete_controller.delete_vector2_keyframe(
                            &delete_target,
                            path,
                            time,
                            InspectorCommit::Immediate(TRANSFORM_KEYFRAME_COMMITS.delete),
                        ),
                        TransformField::Scalar(_) => delete_controller.delete_scalar_keyframe(
                            &delete_target,
                            path,
                            time,
                            InspectorCommit::Immediate(TRANSFORM_KEYFRAME_COMMITS.delete),
                        ),
                    }),
            );
        }),
        update_point: Rc::new(move |old_time, time, displayed_value| {
            warned(
                "move GTK transform keyframe",
                move_controller
                    .ensure_timeline(&move_target, path, timeline_id)
                    .and_then(|()| match field {
                        TransformField::Vec2(_) => move_controller
                            .move_vector2_keyframes(
                                &move_target,
                                path,
                                &[(old_time, time)],
                                InspectorCommit::Coalesced(
                                    TRANSFORM_KEYFRAME_COMMITS.move_keyframe,
                                ),
                            )
                            .map(|_| ()),
                        TransformField::Scalar(_) => move_controller.move_scalar_keyframe(
                            &move_target,
                            path,
                            AudioModifierKeyframeMove {
                                old_time,
                                time,
                                displayed_value,
                                store_multiplier: 1.0,
                            },
                            NumberConstraint::default(),
                            InspectorCommit::Coalesced(TRANSFORM_KEYFRAME_COMMITS.move_keyframe),
                        ),
                    }),
            );
        }),
        clipboard: keyframe_editor::KeyframeClipboardActions::Managed {
            copy: Rc::new(move |times| {
                warned(
                    "copy GTK transform keyframes",
                    copy_controller
                        .ensure_timeline(&copy_target, path, timeline_id)
                        .and_then(|()| match field {
                            TransformField::Vec2(_) => {
                                copy_controller.copy_vector2_keyframes(&copy_target, path, times)
                            }
                            TransformField::Scalar(_) => {
                                copy_controller.copy_scalar_keyframes(&copy_target, path, times)
                            }
                        }),
                )
            }),
            paste: Rc::new(move |time| {
                warned(
                    "paste GTK transform keyframes",
                    paste_controller
                        .ensure_timeline(&paste_target, path, timeline_id)
                        .and_then(|()| match field {
                            TransformField::Vec2(_) => paste_controller.paste_vector2_keyframes(
                                &paste_target,
                                path,
                                time,
                                InspectorCommit::Immediate(TRANSFORM_KEYFRAME_COMMITS.paste),
                            ),
                            TransformField::Scalar(_) => paste_controller.paste_scalar_keyframes(
                                &paste_target,
                                path,
                                time,
                                NumberConstraint::default(),
                                InspectorCommit::Immediate(TRANSFORM_KEYFRAME_COMMITS.paste),
                            ),
                        }),
                )
            }),
        },
        set_interpolation: Some(Rc::new(move |owner_id, interpolation| {
            let interpolation = Interpolation::KEYFRAME
                .iter()
                .position(|candidate| *candidate == interpolation)
                .expect("transform interpolation must be available");
            warned(
                "change GTK transform keyframe interpolation",
                interpolation_controller
                    .ensure_timeline(&interpolation_target, path, timeline_id)
                    .and_then(|()| match field {
                        TransformField::Vec2(_) => interpolation_controller
                            .set_vector2_interpolation(
                                &interpolation_target,
                                path,
                                owner_id,
                                interpolation,
                                InspectorCommit::Immediate(
                                    TRANSFORM_KEYFRAME_COMMITS.interpolation,
                                ),
                            ),
                        TransformField::Scalar(_) => interpolation_controller
                            .set_scalar_keyframe_interpolation(
                                &interpolation_target,
                                path,
                                owner_id,
                                interpolation,
                                InspectorCommit::Immediate(
                                    TRANSFORM_KEYFRAME_COMMITS.interpolation,
                                ),
                            ),
                    }),
            );
        })),
        text_interpolation: None,
        toggle_playback: Rc::new(move || controller.toggle_keyframe_playback()),
    }
}

fn warned<T>(operation: &str, result: Result<T, String>) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(error) => {
            tracing::warn!("Could not {operation}: {error}");
            None
        }
    }
}

pub(super) fn update_vec2_keyframe(
    controller: &shrimply_inspector_core::InspectorController,
    key: SelectedItem,
    field: Vec2Field,
    timeline_id: uuid::Uuid,
    component: usize,
    value: f64,
) {
    warned(
        "edit GTK transform vector keyframe",
        controller.set_vector2_component_value(
            &InspectorTarget::Item(key),
            field,
            timeline_id,
            component,
            value,
            InspectorCommit::Deferred,
        ),
    );
}

pub(super) fn update_scalar_keyframe(
    controller: &shrimply_inspector_core::InspectorController,
    key: SelectedItem,
    field: ScalarField,
    timeline_id: uuid::Uuid,
    value: f64,
) {
    warned(
        "edit GTK transform scalar keyframe",
        controller.set_transform_scalar_value(
            &InspectorTarget::Item(key),
            field,
            timeline_id,
            value,
            InspectorCommit::Deferred,
        ),
    );
}

fn scalar_value_graph(
    project: &Project,
    key: &SelectedItem,
    field: ScalarField,
    display: ResolvedTransform,
) -> KeyframeGraph {
    project.video_item(key).map_or_else(
        || KeyframeGraph::RawValue {
            points: Vec::new(),
            segments: Vec::new(),
            static_value: field.value(display),
        },
        |item| {
            shrimply_inspector_core::keyframe_model::scalar_graph(
                field.timeline(&item.transform),
                field.value(display),
                f64::from,
            )
        },
    )
}

fn vec2_speed_graph(project: &Project, key: &SelectedItem, field: Vec2Field) -> KeyframeGraph {
    project.video_item(key).map_or_else(
        || KeyframeGraph::Speed {
            segments: Vec::new(),
            keys: Vec::new(),
            static_value: 0.0,
        },
        |item| {
            shrimply_inspector_core::timeline_value::vector::speed_graph(
                field.timeline(&item.transform),
            )
        },
    )
}

fn selected_video_visible_area(context: &InspectorContext, key: SelectedItem) -> (Time, Time) {
    crate::video::visual_visible_area(&context.project.borrow(), key)
        .unwrap_or((Time::ZERO, Time::ZERO))
}
