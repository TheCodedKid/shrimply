use shrimply_gtk_components::tr;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use glam::Vec2;
use gtk::prelude::*;
use shrimply_component_core::layered::{LayeredEdit, LayeredPropertyController};
use shrimply_inspector_core::{
    ControlKind, InspectorControl, NumberSpec,
    transform::{
        RESET_TRANSFORM_COMMIT, ScalarField, TRANSFORM_CARD_TITLE, TRANSFORM_KEYFRAME_COMMIT,
        TRANSFORM_KEYFRAMES_COMMIT, TRANSFORM_LIVE_COMMIT, TransformField, Vec2Field,
    },
};

use crate::InspectedItem as SelectedItem;
use crate::player_state::{self, ProjectChange, SharedPlayerState};
use crate::timeline_value::*;
use crate::transform_eval::{self, FrameAudioAnalysis, TransformExpressionCache};
use crate::ui::{
    FrameGraph, InspectorGraphProperty, Number2Picker, NumberPicker, NumberPickerHandle,
};
use shrimply_project::project::{Project, ResolvedTransform, Time, Transform};

use super::{
    Inspectable, InspectorContext,
    keyframe_editor::{self, KeyframeGraph},
    section::InspectorSection,
};

use crate::timeline_value::{
    scalar::{ScalarAccess, ScalarSpec, ScalarTarget, scalar_control},
    vector::vec2::{VecAccess, VecSpec, VecTarget, vec_control, vec_control_with_lock},
};

mod expressions;
mod keyframes;

use expressions::{scalar_expression_editor, set_expression_enabled, vec2_expression_editor};
use keyframes::{scalar_keyframe_body_editor, vec2_keyframe_body_editor};

const SLOW_TRANSFORM_LOG_THRESHOLD: Duration = Duration::from_millis(25);

pub(super) fn controls(transform: &Transform, context: &InspectorContext) -> Vec<gtk::Widget> {
    transform.controls(context)
}

pub(super) fn paint_stroke_controls(
    transform: &Transform,
    controls: &[InspectorControl],
    context: &InspectorContext,
) -> Vec<gtk::Widget> {
    let section = InspectorSection::controls();
    for control in controls {
        match control.kind {
            ControlKind::LayeredVector2 => {
                let (timeline, get, get_mut) = match control.timeline_id {
                    Some(id) if id == transform.position.id => (
                        &transform.position,
                        paint_position as PaintVecGet,
                        paint_position_mut as PaintVecGetMut,
                    ),
                    Some(id) if id == transform.anchor.id => (
                        &transform.anchor,
                        paint_anchor as PaintVecGet,
                        paint_anchor_mut as PaintVecGetMut,
                    ),
                    Some(id) if id == transform.scale.id => (
                        &transform.scale,
                        paint_scale as PaintVecGet,
                        paint_scale_mut as PaintVecGetMut,
                    ),
                    _ => panic!("shared paint stroke transform vector changed"),
                };
                let native = if control.lock {
                    vec_control_with_lock(
                        &control.label,
                        timeline,
                        context,
                        paint_vec_target(timeline.id, get, get_mut, paint_commit_name(control)),
                        paint_vec_spec(control),
                        true,
                    )
                } else {
                    vec_control(
                        &control.label,
                        timeline,
                        context,
                        paint_vec_target(timeline.id, get, get_mut, paint_commit_name(control)),
                        paint_vec_spec(control),
                    )
                };
                section.add_wide_control(&native);
            }
            ControlKind::LayeredNumber => {
                assert_eq!(control.timeline_id, Some(transform.rotation_degrees.id));
                section.add_wide_control(&scalar_control(
                    &control.label,
                    &transform.rotation_degrees,
                    context,
                    ScalarTarget {
                        access: ScalarAccess::ItemWithMutation {
                            get: paint_rotation,
                            get_mut: paint_rotation_mut,
                            mutated: crate::paint::bump_revision_for_key,
                        },
                        scope_id: Some(transform.rotation_degrees.id),
                        local_time: crate::video::visual_local_time,
                        duration: crate::video::visual_duration,
                        refresh: paint_refresh(),
                        commit_name: paint_commit_name(control),
                    },
                    paint_scalar_spec(control),
                ));
            }
            kind => panic!("unsupported shared paint stroke transform control: {kind:?}"),
        }
    }
    vec![section.into_widget()]
}

type PaintVecGet = for<'a> fn(&'a Project, SelectedItem) -> Option<&'a TimelineValue<glam::Vec2>>;
type PaintVecGetMut =
    for<'a> fn(&'a mut Project, SelectedItem) -> Option<&'a mut TimelineValue<glam::Vec2>>;

fn paint_vec_target(
    timeline_id: uuid::Uuid,
    get: PaintVecGet,
    get_mut: PaintVecGetMut,
    commit_name: &'static str,
) -> VecTarget {
    VecTarget {
        access: VecAccess::ItemWithMutation {
            get,
            get_mut,
            value_id: timeline_id,
            mutated: crate::paint::bump_revision_for_key,
        },
        scope_id: Some(timeline_id),
        local_time: crate::video::visual_local_time,
        duration: crate::video::visual_duration,
        refresh: paint_refresh(),
        commit_name,
    }
}

fn paint_vec_spec(control: &InspectorControl) -> VecSpec {
    let defaults = NumberSpec::default();
    assert_eq!(control.prefixes, ["X", "Y"]);
    VecSpec {
        first_prefix: "X",
        second_prefix: "Y",
        drag_step: control.number.drag_step,
        digits: usize::try_from(control.number.digits)
            .expect("paint transform vector digits must be nonnegative"),
        width_chars: control.width_characters,
        minimum: (control.number.minimum != defaults.minimum).then_some(control.number.minimum),
        maximum: (control.number.maximum != defaults.maximum).then_some(control.number.maximum),
        unit_name: control.number.unit,
    }
}

fn paint_scalar_spec(control: &InspectorControl) -> ScalarSpec {
    let defaults = NumberSpec::default();
    ScalarSpec {
        drag_step: control.number.drag_step,
        digits: usize::try_from(control.number.digits)
            .expect("paint transform scalar digits must be nonnegative"),
        integer: control.integer,
        width_chars: control.width_characters,
        minimum: (control.number.minimum != defaults.minimum).then_some(control.number.minimum),
        maximum: (control.number.maximum != defaults.maximum).then_some(control.number.maximum),
        unit_name: (!control.number.unit.is_empty()).then_some(control.number.unit),
        rotating_icon: control.prefix_icon_rotates.then_some((
            match control.prefix_icon.as_str() {
                "arrow3-up-symbolic" => "arrow3-up-symbolic",
                icon => panic!("unsupported shared paint rotating icon: {icon}"),
            },
            control.prefix_icon_rotation_offset_degrees,
        )),
        display: f64::from,
        store: |value| value as f32,
        clamp: crate::timeline_value::scalar::ScalarClamp::Function(|value| value),
    }
}

fn paint_commit_name(control: &InspectorControl) -> &'static str {
    let expected = shrimply_inspector_core::paint::STROKE_TRANSFORM_COMMIT;
    assert_eq!(control.commit_name, expected);
    assert_eq!(control.keyframe_commit_name, expected);
    assert_eq!(control.expression_commit_name, expected);
    expected
}

fn paint_refresh() -> ProjectChange {
    ProjectChange {
        video: true,
        inspector: true,
        ..ProjectChange::default()
    }
}

fn paint_position(project: &Project, key: SelectedItem) -> Option<&TimelineValue<Vec2>> {
    Some(
        &crate::paint::selected_paint(project, key)?
            .stroke_transform
            .position,
    )
}

fn paint_position_mut(
    project: &mut Project,
    key: SelectedItem,
) -> Option<&mut TimelineValue<Vec2>> {
    Some(
        &mut crate::paint::selected_paint_mut(project, key)?
            .stroke_transform
            .position,
    )
}

fn paint_anchor(project: &Project, key: SelectedItem) -> Option<&TimelineValue<Vec2>> {
    Some(
        &crate::paint::selected_paint(project, key)?
            .stroke_transform
            .anchor,
    )
}

fn paint_anchor_mut(project: &mut Project, key: SelectedItem) -> Option<&mut TimelineValue<Vec2>> {
    Some(
        &mut crate::paint::selected_paint_mut(project, key)?
            .stroke_transform
            .anchor,
    )
}

fn paint_scale(project: &Project, key: SelectedItem) -> Option<&TimelineValue<Vec2>> {
    Some(
        &crate::paint::selected_paint(project, key)?
            .stroke_transform
            .scale,
    )
}

fn paint_scale_mut(project: &mut Project, key: SelectedItem) -> Option<&mut TimelineValue<Vec2>> {
    Some(
        &mut crate::paint::selected_paint_mut(project, key)?
            .stroke_transform
            .scale,
    )
}

fn paint_rotation(project: &Project, key: SelectedItem) -> Option<&TimelineValue<f32>> {
    Some(
        &crate::paint::selected_paint(project, key)?
            .stroke_transform
            .rotation_degrees,
    )
}

fn paint_rotation_mut(project: &mut Project, key: SelectedItem) -> Option<&mut TimelineValue<f32>> {
    Some(
        &mut crate::paint::selected_paint_mut(project, key)?
            .stroke_transform
            .rotation_degrees,
    )
}

impl Inspectable for Transform {
    fn title(&self) -> &'static str {
        TRANSFORM_CARD_TITLE
    }

    fn default_action(&self, context: &InspectorContext) -> Option<Box<dyn Fn() + 'static>> {
        let key = context.selected_item.clone()?;
        let project = context.project.clone();
        let player_state = context.player_state.clone();
        let refresh = context.refresh.clone();
        Some(Box::new(move || {
            let mut project = project.borrow_mut();
            let canvas_size = project.canvas_size;
            let Some(item) = project.video_item_mut(&key) else {
                return;
            };
            item.transform = item.natural_transform(canvas_size);
            shrimply_project::project::commit_edit(&project, RESET_TRANSFORM_COMMIT);
            drop(project);
            player_state::refresh_project(
                &player_state,
                ProjectChange {
                    video: true,
                    inspector: true,
                    ..ProjectChange::default()
                },
            );
            refresh();
        }))
    }

    fn add_rows(&self, section: &InspectorSection, context: &InspectorContext) {
        let Some(key) = context.selected_item.clone() else {
            return;
        };
        let display = display_transform(context, key.clone()).unwrap_or_else(|| self.fallback());
        for field in TransformField::ALL {
            match field {
                TransformField::Vec2(field) => {
                    add_vec2_meta_control(section, context, key.clone(), field, display)
                }
                TransformField::Scalar(field) => {
                    add_scalar_meta_control(section, context, key.clone(), field, display)
                }
            }
        }
    }
}

fn add_vec2_meta_control(
    section: &InspectorSection,
    context: &InspectorContext,
    key: SelectedItem,
    field: Vec2Field,
    display: ResolvedTransform,
) {
    let current = current_vec2(context, key.clone(), field)
        .unwrap_or_else(|| TimelineValue::<glam::Vec2>::new_const(Vec2::ZERO));
    let keyframes_enabled = matches!(current.base, TimelineBase::Keyframes(_));
    let expression_enabled = {
        let project = context.project.borrow();
        project.video_item(&key).is_some_and(|item| {
            shrimply_inspector_core::transform::expressions::enabled(
                &item.transform,
                TransformField::Vec2(field),
            )
        })
    };
    let base_display = base_display_transform(context, key.clone()).unwrap_or(display);
    let controller = LayeredPropertyController::default();
    controller.set_keyframes(keyframes_enabled);
    controller.set_expression(expression_enabled);
    let editor = vec2_dynamic_editor(
        context,
        key.clone(),
        field,
        current.id,
        base_display,
        controller.clone(),
    );
    let keyframe_section = vec2_keyframe_body_editor(context, key.clone(), field);
    let expression_section = gtk::Box::new(gtk::Orientation::Vertical, 6);
    expression_section.append(&vec2_expression_editor(context, key.clone(), field));
    expression_section.append(&vec2_expression_output(
        context,
        key.clone(),
        field,
        controller.clone(),
    ));
    section.add_wide_control(&layered_control(
        context,
        key.clone(),
        LayeredControlInput {
            label: field.label(),
            editor,
            field: TransformField::Vec2(field),
            keyframe_section,
            expression_section: expression_section.upcast(),
            controller,
        },
    ));
}

fn add_scalar_meta_control(
    section: &InspectorSection,
    context: &InspectorContext,
    key: SelectedItem,
    field: ScalarField,
    display: ResolvedTransform,
) {
    let current = current_scalar(context, key.clone(), field)
        .unwrap_or_else(|| TimelineValue::<f32>::new_const(0.0));
    let keyframes_enabled = matches!(current.base, TimelineBase::Keyframes(_));
    let expression_enabled = {
        let project = context.project.borrow();
        project.video_item(&key).is_some_and(|item| {
            shrimply_inspector_core::transform::expressions::enabled(
                &item.transform,
                TransformField::Scalar(field),
            )
        })
    };
    let base_display = base_display_transform(context, key.clone()).unwrap_or(display);
    let controller = LayeredPropertyController::default();
    controller.set_keyframes(keyframes_enabled);
    controller.set_expression(expression_enabled);
    let editor = scalar_dynamic_editor(
        context,
        key.clone(),
        field,
        current.id,
        base_display,
        controller.clone(),
    );
    let keyframe_section = scalar_keyframe_body_editor(context, key.clone(), field, base_display);
    let expression_section = gtk::Box::new(gtk::Orientation::Vertical, 6);
    expression_section.append(&scalar_expression_editor(context, key.clone(), field));
    expression_section.append(&scalar_expression_output(
        context,
        key.clone(),
        field,
        controller.clone(),
    ));
    section.add_wide_control(&layered_control(
        context,
        key.clone(),
        LayeredControlInput {
            label: field.label(),
            editor,
            field: TransformField::Scalar(field),
            keyframe_section,
            expression_section: expression_section.upcast(),
            controller,
        },
    ));
}

struct LayeredControlInput<'a> {
    label: &'a str,
    editor: gtk::Widget,
    field: TransformField,
    keyframe_section: FrameGraph,
    expression_section: gtk::Widget,
    controller: LayeredPropertyController,
}

fn layered_control(
    context: &InspectorContext,
    key: SelectedItem,
    input: LayeredControlInput<'_>,
) -> gtk::Widget {
    let LayeredControlInput {
        label,
        editor,
        field,
        keyframe_section,
        expression_section,
        controller,
    } = input;
    let keyframe_context = context.clone();
    let expression_context = context.clone();
    let keyframe_key = key.clone();
    let expression_key = key;
    let property = InspectorGraphProperty::with_expression(
        label,
        &editor,
        &expression_section,
        keyframe_section,
        controller,
    );
    property.connect_keyframes_changed(move |enabled| {
        assert!(
            set_keyframes_enabled(&keyframe_context, keyframe_key.clone(), field, enabled),
            "transform keyframe mode could not be updated",
        );
    });
    property.connect_expression_changed(move |enabled| {
        assert!(
            set_expression_enabled(&expression_context, expression_key.clone(), field, enabled),
            "transform expression mode could not be updated",
        );
    });
    property.widget().clone()
}

fn vec2_dynamic_editor(
    context: &InspectorContext,
    key: SelectedItem,
    field: Vec2Field,
    timeline_id: uuid::Uuid,
    display: ResolvedTransform,
    controller: LayeredPropertyController,
) -> gtk::Widget {
    let value = field.value(display);
    let number = field.number();
    let prefixes = field.prefixes();
    let mut picker = Number2Picker::builder(value.x as f64, value.y as f64)
        .first_prefix(prefixes[0])
        .second_prefix(prefixes[1])
        .drag_step(number.drag_step)
        .digits(
            usize::try_from(number.digits).expect("transform vector digits must be nonnegative"),
        )
        .unit_name(number.unit)
        .width_chars(field.width_characters());
    if field.lock() {
        picker = picker.minimum(number.minimum).enable_lock();
    }

    let first_inspector_core = context.inspector_core.clone();
    let second_inspector_core = context.inspector_core.clone();
    let first_commit_project = context.project.clone();
    let second_commit_project = context.project.clone();
    let first_commit_player = context.player_state.clone();
    let second_commit_player = context.player_state.clone();
    let first_controller = controller.clone();
    let second_controller = controller.clone();
    let first_commit_controller = controller.clone();
    let second_commit_controller = controller.clone();
    let first_key = key.clone();
    let second_key = key.clone();
    let parts = picker
        .on_first_change(move |next| {
            update_vec2_dynamic(
                &first_inspector_core,
                first_key.clone(),
                field,
                timeline_id,
                0,
                next,
                &first_controller,
            );
        })
        .on_second_change(move |next| {
            update_vec2_dynamic(
                &second_inspector_core,
                second_key.clone(),
                field,
                timeline_id,
                1,
                next,
                &second_controller,
            );
        })
        .on_first_commit(move |_| {
            commit_dynamic_transform(
                &first_commit_project,
                &first_commit_player,
                &first_commit_controller,
            );
        })
        .on_second_commit(move |_| {
            commit_dynamic_transform(
                &second_commit_project,
                &second_commit_player,
                &second_commit_controller,
            );
        })
        .build_with_handles();
    let widget = parts.widget;
    connect_vec2_base_display(
        context,
        key.clone(),
        field,
        &widget,
        parts.first,
        parts.second,
        controller,
    );
    widget
}

fn scalar_dynamic_editor(
    context: &InspectorContext,
    key: SelectedItem,
    field: ScalarField,
    timeline_id: uuid::Uuid,
    display: ResolvedTransform,
    controller: LayeredPropertyController,
) -> gtk::Widget {
    let number = field.number();
    let picker = NumberPicker::builder(field.value(display))
        .drag_step(number.drag_step)
        .digits(
            usize::try_from(number.digits).expect("transform scalar digits must be nonnegative"),
        )
        .width_chars(field.width_characters())
        .unit_name(number.unit)
        .rotating_prefix_icon_name("arrow3-up-symbolic");

    let inspector_core = context.inspector_core.clone();
    let commit_project = context.project.clone();
    let commit_player = context.player_state.clone();
    let update_controller = controller.clone();
    let commit_controller = controller.clone();
    let update_key = key.clone();
    let parts = picker
        .on_change(move |next| {
            update_scalar_dynamic(
                &inspector_core,
                update_key.clone(),
                field,
                timeline_id,
                next,
                &update_controller,
            );
        })
        .on_commit(move |_| {
            commit_dynamic_transform(&commit_project, &commit_player, &commit_controller);
        })
        .build_with_handle();
    let widget = parts.widget;
    connect_scalar_base_display(
        context,
        key.clone(),
        field,
        &widget,
        parts.handle,
        controller,
    );
    widget
}

fn update_vec2_dynamic(
    inspector_core: &shrimply_inspector_core::InspectorController,
    key: SelectedItem,
    field: Vec2Field,
    timeline_id: uuid::Uuid,
    component: usize,
    value: f64,
    controller: &LayeredPropertyController,
) {
    let value = match controller.edit_component_value::<f64, 2>(component, value) {
        LayeredEdit::Base(value) | LayeredEdit::Keyframe(value) => value,
    };
    keyframes::update_vec2_keyframe(inspector_core, key, field, timeline_id, component, value);
}

fn update_scalar_dynamic(
    inspector_core: &shrimply_inspector_core::InspectorController,
    key: SelectedItem,
    field: ScalarField,
    timeline_id: uuid::Uuid,
    value: f64,
    controller: &LayeredPropertyController,
) {
    let value = match controller.edit(value) {
        LayeredEdit::Base(value) | LayeredEdit::Keyframe(value) => value,
    };
    keyframes::update_scalar_keyframe(inspector_core, key, field, timeline_id, value);
}

fn commit_dynamic_transform(
    project: &Rc<RefCell<Project>>,
    player_state: &SharedPlayerState,
    controller: &LayeredPropertyController,
) {
    match controller.edit(()) {
        LayeredEdit::Base(()) => {
            shrimply_project::project::commit_edit(&project.borrow(), TRANSFORM_LIVE_COMMIT);
        }
        LayeredEdit::Keyframe(()) => {
            shrimply_project::project::commit_edit(&project.borrow(), TRANSFORM_KEYFRAME_COMMIT);
            refresh_video(player_state);
        }
    }
}

fn set_keyframes_enabled(
    context: &InspectorContext,
    key: SelectedItem,
    field: TransformField,
    enabled: bool,
) -> bool {
    let target = shrimply_inspector_core::InspectorTarget::Item(key);
    let commit = shrimply_inspector_core::InspectorCommit::Immediate(TRANSFORM_KEYFRAMES_COMMIT);
    match field {
        TransformField::Vec2(field) => context.inspector_core.set_vector2_keyframes_enabled(
            &target,
            field.path(),
            enabled,
            commit,
        ),
        TransformField::Scalar(field) => context.inspector_core.set_scalar_keyframes_enabled(
            &target,
            field.path(),
            enabled,
            shrimply_inspector_core::NumberConstraint::default(),
            commit,
        ),
    }
    .is_ok()
}

fn display_transform(context: &InspectorContext, key: SelectedItem) -> Option<ResolvedTransform> {
    let position = player_state::snapshot(&context.player_state).position;
    let audio_analysis = context.audio_analysis_at(position);
    evaluated_transform(
        &context.project.borrow(),
        key.clone(),
        position,
        &audio_analysis,
    )
}

fn base_display_transform(
    context: &InspectorContext,
    key: SelectedItem,
) -> Option<ResolvedTransform> {
    let position = player_state::snapshot(&context.player_state).position;
    base_transform_at(&context.project.borrow(), key.clone(), position)
}

fn evaluated_transform(
    project: &Project,
    key: SelectedItem,
    position: Time,
    audio_analysis: &FrameAudioAnalysis,
) -> Option<ResolvedTransform> {
    let position = crate::video::visual_sequence_time(project, &key, position)?;
    let item = project.video_item(&key)?;
    let mut cache = TransformExpressionCache::default();
    Some(transform_eval::resolve_item_transform_with_audio(
        project,
        item,
        position,
        audio_analysis,
        &mut cache,
    ))
}

fn base_transform_at(
    project: &Project,
    key: SelectedItem,
    position: Time,
) -> Option<ResolvedTransform> {
    let position = crate::video::visual_sequence_time(project, &key, position)?;
    let item = project.video_item(&key)?;
    Some(transform_eval::resolve_item_base_transform(
        project, item, position,
    ))
}

fn selected_item_label(key: SelectedItem) -> String {
    format!("{:?}#{}:{}", key.kind(), key.track_id(), key.item_id())
}

fn connect_vec2_base_display(
    context: &InspectorContext,
    key: SelectedItem,
    field: Vec2Field,
    widget: &gtk::Widget,
    first: NumberPickerHandle,
    second: NumberPickerHandle,
    controller: LayeredPropertyController,
) {
    update_vec2_base_display(
        &context.project,
        &context.player_state,
        key.clone(),
        field,
        &first,
        &second,
    );
    widget.connect_map({
        let project = context.project.clone();
        let player_state = context.player_state.clone();
        let key = key.clone();
        let first = first.clone();
        let second = second.clone();
        let controller = controller.clone();
        move |_| {
            if controller.keyframes() || controller.expression() {
                update_vec2_base_display(
                    &project,
                    &player_state,
                    key.clone(),
                    field,
                    &first,
                    &second,
                );
            }
        }
    });
    let project = context.project.clone();
    let player_state = context.player_state.clone();
    let alive = Rc::downgrade(&context.listener_scope);
    let alive_for_prune = alive.clone();
    let first = first.downgrade();
    let second = second.downgrade();
    player_state::connect_while_alive_named(
        &context.player_state,
        "inspector transform vec2 base display",
        move || alive_for_prune.upgrade().is_some(),
        move |event| {
            if (!controller.keyframes() && !controller.expression())
                || !transform_display_refresh_event(event)
            {
                return;
            }
            shrimply_support::crash::set_context(format!(
                "inspector transform vec2 base display schedule key={} field={field:?} event={event:?}",
                selected_item_label(key.clone()),
            ));
            if alive.upgrade().is_none() {
                tracing::trace!(
                    "transform: skip vec2 base display key={} field={field:?} reason=stale-scope",
                    selected_item_label(key.clone()),
                );
                return;
            }
            let (Some(first), Some(second)) = (first.upgrade(), second.upgrade()) else {
                tracing::trace!(
                    "transform: skip vec2 base display key={} field={field:?} reason=widget-dropped",
                    selected_item_label(key.clone()),
                );
                return;
            };
            if !first.widget_is_mapped() || !second.widget_is_mapped() {
                return;
            }
            update_vec2_base_display(&project, &player_state, key.clone(), field, &first, &second);
        },
    );
}

fn connect_scalar_base_display(
    context: &InspectorContext,
    key: SelectedItem,
    field: ScalarField,
    widget: &gtk::Widget,
    handle: NumberPickerHandle,
    controller: LayeredPropertyController,
) {
    update_scalar_base_display(
        &context.project,
        &context.player_state,
        key.clone(),
        field,
        &handle,
    );
    widget.connect_map({
        let project = context.project.clone();
        let player_state = context.player_state.clone();
        let key = key.clone();
        let handle = handle.clone();
        let controller = controller.clone();
        move |_| {
            if controller.keyframes() || controller.expression() {
                update_scalar_base_display(&project, &player_state, key.clone(), field, &handle);
            }
        }
    });
    let project = context.project.clone();
    let player_state = context.player_state.clone();
    let alive = Rc::downgrade(&context.listener_scope);
    let alive_for_prune = alive.clone();
    let handle = handle.downgrade();
    player_state::connect_while_alive_named(
        &context.player_state,
        "inspector transform scalar base display",
        move || alive_for_prune.upgrade().is_some(),
        move |event| {
            if (!controller.keyframes() && !controller.expression())
                || !transform_display_refresh_event(event)
            {
                return;
            }
            shrimply_support::crash::set_context(format!(
                "inspector transform scalar base display schedule key={} field={field:?} event={event:?}",
                selected_item_label(key.clone()),
            ));
            if alive.upgrade().is_none() {
                tracing::trace!(
                    "transform: skip scalar base display key={} field={field:?} reason=stale-scope",
                    selected_item_label(key.clone()),
                );
                return;
            }
            let Some(handle) = handle.upgrade() else {
                tracing::trace!(
                    "transform: skip scalar base display key={} field={field:?} reason=widget-dropped",
                    selected_item_label(key.clone()),
                );
                return;
            };
            if !handle.widget_is_mapped() {
                return;
            }
            update_scalar_base_display(&project, &player_state, key.clone(), field, &handle);
        },
    );
}

fn update_vec2_base_display(
    project: &Rc<RefCell<Project>>,
    player_state: &SharedPlayerState,
    key: SelectedItem,
    field: Vec2Field,
    first: &NumberPickerHandle,
    second: &NumberPickerHandle,
) {
    let started = Instant::now();
    let position = player_state::snapshot(player_state).position;
    shrimply_support::crash::set_context(format!(
        "inspector transform vec2 base display lookup key={} field={field:?} position={}",
        selected_item_label(key.clone()),
        position.as_label(),
    ));
    let value = {
        let project = project.borrow();
        let Some(position) = crate::video::visual_sequence_time(&project, &key, position) else {
            return;
        };
        let Some(item) = project.video_item(&key) else {
            tracing::trace!(
                "transform: skip vec2 base display key={} field={field:?} position={} reason=missing-item",
                selected_item_label(key.clone()),
                position.as_label(),
            );
            return;
        };
        let eval = transform_eval::TransformEvaluation::for_item(&project, item, position);
        transform_eval::resolve_base(field.timeline(&item.transform), &eval)
    };
    let value = value;
    if !value.x.is_finite() || !value.y.is_finite() {
        tracing::debug!(
            "transform: skip vec2 base display key={} field={field:?} position={} reason=non-finite x={} y={}",
            selected_item_label(key.clone()),
            position.as_label(),
            value.x,
            value.y,
        );
        return;
    }
    shrimply_support::crash::set_context(format!(
        "inspector transform vec2 base display set first key={} field={field:?} value={}",
        selected_item_label(key.clone()),
        value.x,
    ));
    first.set_f64(value.x as f64);
    shrimply_support::crash::set_context(format!(
        "inspector transform vec2 base display set second key={} field={field:?} value={}",
        selected_item_label(key.clone()),
        value.y,
    ));
    second.set_f64(value.y as f64);
    let elapsed = started.elapsed();
    if elapsed >= SLOW_TRANSFORM_LOG_THRESHOLD {
        tracing::debug!(
            "transform: vec2 base display key={} field={field:?} position={} x={} y={} elapsed_us={}",
            selected_item_label(key.clone()),
            position.as_label(),
            value.x,
            value.y,
            elapsed.as_micros(),
        );
    }
}

fn update_scalar_base_display(
    project: &Rc<RefCell<Project>>,
    player_state: &SharedPlayerState,
    key: SelectedItem,
    field: ScalarField,
    handle: &NumberPickerHandle,
) {
    let started = Instant::now();
    let position = player_state::snapshot(player_state).position;
    shrimply_support::crash::set_context(format!(
        "inspector transform scalar base display lookup key={} field={field:?} position={}",
        selected_item_label(key.clone()),
        position.as_label(),
    ));
    let value = {
        let project = project.borrow();
        let Some(position) = crate::video::visual_sequence_time(&project, &key, position) else {
            return;
        };
        let Some(item) = project.video_item(&key) else {
            tracing::trace!(
                "transform: skip scalar base display key={} field={field:?} position={} reason=missing-item",
                selected_item_label(key.clone()),
                position.as_label(),
            );
            return;
        };
        let eval = transform_eval::TransformEvaluation::for_item(&project, item, position);
        transform_eval::resolve_base(field.timeline(&item.transform), &eval)
    };
    let value = f64::from(value);
    if !value.is_finite() {
        tracing::debug!(
            "transform: skip scalar base display key={} field={field:?} position={} reason=non-finite value={}",
            selected_item_label(key.clone()),
            position.as_label(),
            value,
        );
        return;
    }
    shrimply_support::crash::set_context(format!(
        "inspector transform scalar base display set key={} field={field:?} value={}",
        selected_item_label(key.clone()),
        value,
    ));
    handle.set_f64(value);
    let elapsed = started.elapsed();
    if elapsed >= SLOW_TRANSFORM_LOG_THRESHOLD {
        tracing::debug!(
            "transform: scalar base display key={} field={field:?} position={} value={} elapsed_us={}",
            selected_item_label(key.clone()),
            position.as_label(),
            value,
            elapsed.as_micros(),
        );
    }
}

fn vec2_expression_output(
    context: &InspectorContext,
    key: SelectedItem,
    field: Vec2Field,
    controller: LayeredPropertyController,
) -> gtk::Widget {
    let timeline_id = current_vec2(context, key.clone(), field)
        .expect("transform expression timeline must remain available")
        .id;
    let inspector = context.inspector_core.clone();
    let target = shrimply_inspector_core::InspectorTarget::Item(key);
    expression_output(context, controller, move |label| {
        update_vec2_expression_output(&inspector, &target, timeline_id, field, label)
    })
}

fn scalar_expression_output(
    context: &InspectorContext,
    key: SelectedItem,
    field: ScalarField,
    controller: LayeredPropertyController,
) -> gtk::Widget {
    let timeline_id = current_scalar(context, key.clone(), field)
        .expect("transform expression timeline must remain available")
        .id;
    let inspector = context.inspector_core.clone();
    let target = shrimply_inspector_core::InspectorTarget::Item(key);
    expression_output(context, controller, move |label| {
        update_scalar_expression_output(&inspector, &target, timeline_id, field, label)
    })
}

fn expression_output(
    context: &InspectorContext,
    controller: LayeredPropertyController,
    update: impl Fn(&gtk::Label) + 'static,
) -> gtk::Widget {
    let label = expression_output_value_label();
    let widget = expression_output_row(label.clone());
    let update: Rc<dyn Fn(&gtk::Label)> = Rc::new(update);
    if controller.expression() {
        update(&label);
    }
    widget.connect_map({
        let controller = controller.clone();
        let label = label.clone();
        let update = update.clone();
        move |_| {
            if controller.expression() {
                update(&label);
            }
        }
    });
    let alive = Rc::downgrade(&context.listener_scope);
    let label = label.downgrade();
    player_state::connect_while_alive_named(
        &context.player_state,
        "inspector transform expression output",
        move || alive.upgrade().is_some(),
        move |event| {
            if !controller.expression() || !transform_display_refresh_event(event) {
                return;
            }
            let Some(label) = label.upgrade() else {
                return;
            };
            if label.is_mapped() {
                update(&label);
            }
        },
    );
    widget
}

fn expression_output_row(value: gtk::Label) -> gtk::Widget {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    row.set_hexpand(true);
    let label = gtk::Label::new(Some(tr!("Output").as_ref()));
    label.add_css_class("dim-label");
    label.set_xalign(0.0);
    row.append(&label);
    row.append(&value);
    row.upcast()
}

fn expression_output_value_label() -> gtk::Label {
    let label = gtk::Label::new(None);
    label.set_hexpand(true);
    label.set_xalign(1.0);
    label.set_selectable(true);
    label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    label.add_css_class("numeric");
    label
}

fn update_vec2_expression_output(
    inspector: &shrimply_inspector_core::InspectorController,
    target: &shrimply_inspector_core::InspectorTarget,
    timeline_id: uuid::Uuid,
    field: Vec2Field,
    label: &gtk::Label,
) {
    let started = Instant::now();
    let outcome = match inspector.transform_vec2_expression_output(target, field, timeline_id) {
        Ok(Some(output)) if output.error.is_none() => {
            set_expression_output_value(
                label,
                &shrimply_inspector_core::transform::expressions::format_vec2(field, output.value),
            );
            "ok".to_string()
        }
        Ok(Some(output)) => {
            let error = output.error.expect("checked transform expression error");
            set_expression_output_error(label, &error);
            format!("error: {}", expression_error_summary(&error))
        }
        Ok(None) => "none".to_string(),
        Err(error) => {
            set_expression_output_error(label, &error);
            format!("error: {}", expression_error_summary(&error))
        }
    };
    let elapsed = started.elapsed();
    if (outcome != "ok" && outcome != "none") || elapsed >= SLOW_TRANSFORM_LOG_THRESHOLD {
        tracing::debug!(
            "transform: update_vec2_expression_output field={field:?} outcome={outcome} elapsed_us={}",
            elapsed.as_micros(),
        );
    }
}

fn update_scalar_expression_output(
    inspector: &shrimply_inspector_core::InspectorController,
    target: &shrimply_inspector_core::InspectorTarget,
    timeline_id: uuid::Uuid,
    field: ScalarField,
    label: &gtk::Label,
) {
    let started = Instant::now();
    let outcome = match inspector.transform_scalar_expression_output(target, field, timeline_id) {
        Ok(Some(output)) if output.error.is_none() => {
            set_expression_output_value(
                label,
                &shrimply_inspector_core::transform::expressions::format_scalar(
                    field,
                    output.value,
                ),
            );
            "ok".to_string()
        }
        Ok(Some(output)) => {
            let error = output.error.expect("checked transform expression error");
            set_expression_output_error(label, &error);
            format!("error: {}", expression_error_summary(&error))
        }
        Ok(None) => "none".to_string(),
        Err(error) => {
            set_expression_output_error(label, &error);
            format!("error: {}", expression_error_summary(&error))
        }
    };
    let elapsed = started.elapsed();
    if (outcome != "ok" && outcome != "none") || elapsed >= SLOW_TRANSFORM_LOG_THRESHOLD {
        tracing::debug!(
            "transform: update_scalar_expression_output field={field:?} outcome={outcome} elapsed_us={}",
            elapsed.as_micros(),
        );
    }
}

fn set_expression_output_value(label: &gtk::Label, value: &str) {
    label.remove_css_class("error");
    label.set_tooltip_text(None);
    label.set_text(value);
}

fn set_expression_output_error(label: &gtk::Label, error: &str) {
    label.add_css_class("error");
    label.set_tooltip_text(Some(error));
    label.set_text(&expression_error_summary(error));
}

fn expression_error_summary(error: &str) -> String {
    let message = error.lines().next().unwrap_or_default().trim();
    if message.is_empty() {
        "Invalid expression".to_string()
    } else {
        format!("Invalid expression: {message}")
    }
}

fn transform_display_refresh_event(event: player_state::PlayerEvent) -> bool {
    match event {
        player_state::PlayerEvent::State(_) => true,
        player_state::PlayerEvent::Project(change) => change.video || change.audio,
    }
}

fn current_vec2(
    context: &InspectorContext,
    key: SelectedItem,
    field: Vec2Field,
) -> Option<TimelineValue<glam::Vec2>> {
    let project = context.project.borrow();
    let transform = &project.video_item(&key)?.transform;
    Some(field.timeline(transform).clone())
}

fn current_scalar(
    context: &InspectorContext,
    key: SelectedItem,
    field: ScalarField,
) -> Option<TimelineValue<f32>> {
    let project = context.project.borrow();
    let transform = &project.video_item(&key)?.transform;
    Some(field.timeline(transform).clone())
}

fn refresh_video(player_state: &SharedPlayerState) {
    let started = Instant::now();
    player_state::refresh_project(
        player_state,
        ProjectChange {
            video: true,
            live_preview: true,
            ..ProjectChange::default()
        },
    );
    let elapsed = started.elapsed();
    if elapsed >= SLOW_TRANSFORM_LOG_THRESHOLD {
        tracing::debug!(
            "transform: refresh_video elapsed_us={}",
            elapsed.as_micros()
        );
    }
}
