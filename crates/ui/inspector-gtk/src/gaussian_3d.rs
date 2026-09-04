use shrimply_3dgs::{
    AnimatedTransform3d as Transform3d, Camera3d, CameraProjection, GaussianScene,
};
use shrimply_core::timeline_value::TimelineValue;
use shrimply_inspector_core::{
    ControlKind, InspectorControl, InspectorTarget, NumberMapping, NumberSpec,
    gaussian_3d as shared,
};
use shrimply_project::project::{Project, Time, VideoItemContent, generated_item_keyframe_span};

use crate::InspectedItem as SelectedItem;
use crate::InspectorContext;
use crate::item::{DefaultInspectorItem, InspectorListItem};
use crate::player_state::ProjectChange;
use crate::section::InspectorSection;
use crate::selector::enum_selector;
use crate::timeline_value::scalar::{ScalarAccess, ScalarSpec, ScalarTarget, scalar_control};
use crate::timeline_value::step::{StepTarget, step_control};
use crate::timeline_value::vector::vec3::{Vec3Target, control as vec3_control};

pub(super) fn items(scene: &GaussianScene, _context: &InspectorContext) -> Vec<InspectorListItem> {
    vec![
        DefaultInspectorItem::new(
            "gaussian-model",
            "Model",
            scene.model.clone(),
            model_controls,
            |context, _| reset(context, shared::model_reset()),
        )
        .boxed(),
        DefaultInspectorItem::new(
            "gaussian-camera",
            "Camera",
            scene.camera.clone(),
            camera_controls,
            |context, _| reset(context, shared::camera_reset()),
        )
        .boxed(),
    ]
}

fn model_controls(value: &Transform3d, context: &InspectorContext) -> Vec<gtk::Widget> {
    let presentation = shared::model_card(value, context.inspector_core.snapshot().runtime);
    let mut presentation_controls = presentation.section.controls.into_iter();
    controls(|section| {
        let control = next(&mut presentation_controls, shared::MODEL_POSITION_PATH);
        add_vec3(
            section,
            &control,
            &value.position,
            context,
            Vec3Target::item_builder(model_position, model_position_mut).build(),
        );
        let control = next(&mut presentation_controls, shared::MODEL_ANCHOR_PATH);
        add_vec3(
            section,
            &control,
            &value.anchor,
            context,
            Vec3Target::item_builder(model_anchor, model_anchor_mut).build(),
        );
        let control = next(&mut presentation_controls, shared::MODEL_ROTATION_PATH);
        add_vec3(
            section,
            &control,
            &value.rotation_degrees,
            context,
            Vec3Target::item_builder(model_rotation, model_rotation_mut)
                .degrees()
                .build(),
        );
        let control = next(
            &mut presentation_controls,
            shared::MODEL_ROTATION_ORDER_PATH,
        );
        assert_eq!(control.kind, ControlKind::LayeredSelector);
        assert_eq!(control.timeline_id, Some(value.rotation_order.id));
        let rotation_id = value.rotation_order.id;
        section.add_wide_control(&step_control(
            &control.label,
            &value.rotation_order,
            context,
            StepTarget::new(
                move |project, key| {
                    let value = &selected_scene(project, key)?.model.rotation_order;
                    (value.id == rotation_id).then_some(value)
                },
                move |project, key| {
                    let value = &mut selected_scene_mut(project, key)?.model.rotation_order;
                    (value.id == rotation_id).then_some(value)
                },
                shared::ROTATION_ORDER_COMMIT,
                video_change(),
            ),
        ));
        let control = next(&mut presentation_controls, shared::MODEL_SCALE_PATH);
        add_vec3(
            section,
            &control,
            &value.scale,
            context,
            Vec3Target::item_builder(model_scale, model_scale_mut)
                .minimum(0.0)
                .lock()
                .build(),
        );
        assert!(presentation_controls.next().is_none());
    })
}

fn camera_controls(value: &Camera3d, context: &InspectorContext) -> Vec<gtk::Widget> {
    let key = context
        .selected_item
        .clone()
        .expect("Gaussian camera controls require a selected item");
    let server_url = shrimply_state::preferences::snapshot(&context.preferences).compute_server_url;
    let models = shrimply_inspector_core::camera_source::cached_tracking_models(&server_url);
    let presentation = shared::camera_card(
        &context.project.borrow(),
        &key,
        value,
        context.inspector_core.snapshot().runtime,
        models.as_ref(),
    );
    let mut presentation_controls = presentation.section.controls.into_iter().filter(|control| {
        !control
            .path
            .starts_with(shrimply_inspector_core::camera_source::SOURCE_PATH)
    });
    controls(|section| {
        let custom_source = crate::camera_source::add_controls(section, &value.source, context);
        if custom_source {
            let control = next(&mut presentation_controls, shared::CAMERA_PROJECTION_PATH);
            section.add_wide_control(&projection_selector(&control, value.projection, context));
        }
        let control = next(&mut presentation_controls, shared::CAMERA_POSITION_PATH);
        add_vec3(
            section,
            &control,
            &value.position,
            context,
            Vec3Target::item_builder(camera_position, camera_position_mut).build(),
        );
        let control = next(&mut presentation_controls, shared::CAMERA_ROTATION_PATH);
        add_vec3(
            section,
            &control,
            &value.rotation_degrees,
            context,
            Vec3Target::item_builder(camera_rotation, camera_rotation_mut)
                .degrees()
                .build(),
        );
        if !custom_source {
            assert!(presentation_controls.next().is_none());
            return;
        }
        match value.projection {
            CameraProjection::Perspective => {
                let control = next(&mut presentation_controls, shared::CAMERA_FOV_PATH);
                add_scalar(
                    section,
                    &control,
                    &value.vertical_fov_degrees,
                    context,
                    camera_fov,
                    camera_fov_mut,
                );
                let control = next(
                    &mut presentation_controls,
                    shared::CAMERA_FOCUS_DISTANCE_PATH,
                );
                add_scalar(
                    section,
                    &control,
                    &value.focus_distance,
                    context,
                    camera_focus_distance,
                    camera_focus_distance_mut,
                );
                let control = next(&mut presentation_controls, shared::CAMERA_F_STOP_PATH);
                add_scalar(
                    section,
                    &control,
                    &value.f_stop,
                    context,
                    camera_f_stop,
                    camera_f_stop_mut,
                );
            }
            CameraProjection::Orthographic => {
                let control = next(
                    &mut presentation_controls,
                    shared::CAMERA_ORTHOGRAPHIC_HEIGHT_PATH,
                );
                add_scalar(
                    section,
                    &control,
                    &value.orthographic_height,
                    context,
                    camera_orthographic_height,
                    camera_orthographic_height_mut,
                );
            }
            CameraProjection::Cylindrical => {
                let control = next(&mut presentation_controls, shared::CAMERA_FOV_PATH);
                add_scalar(
                    section,
                    &control,
                    &value.vertical_fov_degrees,
                    context,
                    camera_fov,
                    camera_fov_mut,
                );
            }
            CameraProjection::Equirectangular => {}
            CameraProjection::Fisheye => {
                let control = next(&mut presentation_controls, shared::CAMERA_FOV_PATH);
                add_scalar(
                    section,
                    &control,
                    &value.vertical_fov_degrees,
                    context,
                    camera_fov,
                    camera_fov_mut,
                );
            }
        }
        let control = next(&mut presentation_controls, shared::CAMERA_EXPOSURE_PATH);
        add_scalar(
            section,
            &control,
            &value.exposure_ev,
            context,
            camera_exposure,
            camera_exposure_mut,
        );
        assert!(presentation_controls.next().is_none());
    })
}

fn controls(add: impl FnOnce(&InspectorSection)) -> Vec<gtk::Widget> {
    let section = InspectorSection::controls();
    add(&section);
    vec![section.into_widget()]
}

fn add_vec3(
    section: &InspectorSection,
    control: &InspectorControl,
    value: &TimelineValue<glam::Vec3>,
    context: &InspectorContext,
    target: Vec3Target,
) {
    assert_eq!(control.kind, ControlKind::LayeredVector3);
    assert_eq!(control.timeline_id, Some(value.id));
    assert_eq!(control.prefixes, ["X", "Y", "Z"]);
    assert_eq!(control.width_characters, 5);
    let target = target.presentation(
        control,
        shared::VECTOR_COMMIT,
        shared::VECTOR_EXPRESSION_COMMIT,
    );
    section.add_wide_control(&vec3_control(&control.label, value, context, target));
}

fn next(controls: &mut impl Iterator<Item = InspectorControl>, path: &str) -> InspectorControl {
    let control = controls
        .next()
        .unwrap_or_else(|| panic!("shared Gaussian control is missing: {path}"));
    assert_eq!(control.path, path);
    control
}

type ScalarGet = for<'a> fn(&'a Project, SelectedItem) -> Option<&'a TimelineValue<f32>>;
type ScalarGetMut = for<'a> fn(&'a mut Project, SelectedItem) -> Option<&'a mut TimelineValue<f32>>;

fn add_scalar(
    section: &InspectorSection,
    control: &InspectorControl,
    value: &TimelineValue<f32>,
    context: &InspectorContext,
    get: ScalarGet,
    get_mut: ScalarGetMut,
) {
    assert_eq!(control.kind, ControlKind::LayeredNumber);
    assert_eq!(control.timeline_id, Some(value.id));
    assert_eq!(control.commit_name, shared::SCALAR_COMMIT);
    section.add_wide_control(&scalar_control(
        &control.label,
        value,
        context,
        ScalarTarget {
            access: ScalarAccess::ItemScoped {
                get,
                get_mut,
                value_id: value.id,
            },
            scope_id: Some(value.id),
            local_time: crate::video::visual_local_time,
            duration: scene_duration,
            refresh: video_change(),
            commit_name: shared::SCALAR_COMMIT,
        },
        scalar_spec(control),
    ));
}

fn scalar_spec(control: &InspectorControl) -> ScalarSpec {
    let defaults = NumberSpec::default();
    ScalarSpec {
        drag_step: control.number.drag_step,
        digits: usize::try_from(control.number.digits)
            .expect("Gaussian scalar digits must be nonnegative"),
        integer: control.integer,
        width_chars: control.width_characters,
        minimum: (control.number.minimum != defaults.minimum).then_some(control.number.minimum),
        maximum: (control.number.maximum != defaults.maximum).then_some(control.number.maximum),
        unit_name: (!control.number.unit.is_empty()).then_some(control.number.unit),
        rotating_icon: None,
        display: match control.number_mapping {
            NumberMapping::Linear => linear_display,
            NumberMapping::FocalLengthMillimeters => focal_display,
        },
        store: match control.number_mapping {
            NumberMapping::Linear => linear_store,
            NumberMapping::FocalLengthMillimeters => focal_store,
        },
        clamp: crate::timeline_value::scalar::ScalarClamp::Function(match control.path.as_str() {
            shared::CAMERA_FOCUS_DISTANCE_PATH => clamp_nonnegative,
            shared::CAMERA_ORTHOGRAPHIC_HEIGHT_PATH => clamp_positive,
            shared::CAMERA_FOV_PATH if control.number.maximum == 360.0 => clamp_fisheye_fov,
            shared::CAMERA_FOV_PATH => clamp_fov,
            shared::CAMERA_F_STOP_PATH => clamp_f_stop,
            shared::CAMERA_EXPOSURE_PATH => clamp_exposure,
            path => panic!("unsupported shared Gaussian scalar: {path}"),
        }),
    }
}

fn projection_selector(
    control: &InspectorControl,
    value: CameraProjection,
    context: &InspectorContext,
) -> gtk::Widget {
    assert_eq!(control.kind, ControlKind::Selector);
    assert_eq!(control.commit_name, shared::PROJECTION_COMMIT);
    assert!(control.commit_immediately);
    let target = InspectorTarget::Item(
        context
            .selected_item
            .clone()
            .expect("Gaussian projection requires a selected item"),
    );
    let controller = context.inspector_core.clone();
    enum_selector(&control.label, value, move |projection| {
        let value = serde_json::to_value(projection)
            .expect("Gaussian projection must serialize")
            .as_str()
            .expect("Gaussian projection must serialize as text")
            .to_string();
        if let Err(error) = controller.set_video_field(
            &target,
            shared::CAMERA_PROJECTION_PATH,
            &value,
            shared::PROJECTION_COMMIT,
            true,
        ) {
            tracing::error!(%error, "Could not update GTK Gaussian projection");
        }
    })
}

fn reset(context: &InspectorContext, reset: shrimply_inspector_core::VideoReset) {
    let Some(key) = context.selected_item.clone() else {
        return;
    };
    if let Err(error) = context
        .inspector_core
        .reset_video(&InspectorTarget::Item(key), &reset)
    {
        tracing::error!(%error, "Could not reset GTK Gaussian inspector card");
    }
}

fn linear_display(value: f32) -> f64 {
    f64::from(value)
}

fn linear_store(value: f64) -> f32 {
    value as f32
}

fn focal_display(value: f32) -> f64 {
    NumberMapping::FocalLengthMillimeters.display(f64::from(value), 1.0)
}

fn focal_store(value: f64) -> f32 {
    NumberMapping::FocalLengthMillimeters.store(value, 1.0) as f32
}

fn clamp_nonnegative(value: f32) -> f32 {
    shared::clamp_scalar(
        shared::CAMERA_FOCUS_DISTANCE_PATH,
        value,
        CameraProjection::Perspective,
    )
}

fn clamp_positive(value: f32) -> f32 {
    shared::clamp_scalar(
        shared::CAMERA_ORTHOGRAPHIC_HEIGHT_PATH,
        value,
        CameraProjection::Orthographic,
    )
}

fn clamp_fov(value: f32) -> f32 {
    shared::clamp_scalar(
        shared::CAMERA_FOV_PATH,
        value,
        CameraProjection::Perspective,
    )
}

fn clamp_fisheye_fov(value: f32) -> f32 {
    shared::clamp_scalar(shared::CAMERA_FOV_PATH, value, CameraProjection::Fisheye)
}

fn clamp_f_stop(value: f32) -> f32 {
    shared::clamp_scalar(
        shared::CAMERA_F_STOP_PATH,
        value,
        CameraProjection::Perspective,
    )
}

fn clamp_exposure(value: f32) -> f32 {
    shared::clamp_scalar(
        shared::CAMERA_EXPOSURE_PATH,
        value,
        CameraProjection::Perspective,
    )
}

fn selected_scene(project: &Project, key: SelectedItem) -> Option<&GaussianScene> {
    let item = project.video_item(&key)?;
    let VideoItemContent::Gaussian(scene) = &item.content else {
        return None;
    };
    Some(scene)
}

fn selected_scene_mut(project: &mut Project, key: SelectedItem) -> Option<&mut GaussianScene> {
    let item = project.video_item_mut(&key)?;
    let VideoItemContent::Gaussian(scene) = &mut item.content else {
        return None;
    };
    Some(scene)
}

fn scene_duration(project: &Project, key: SelectedItem) -> Option<Time> {
    let item = project.video_item(&key)?;
    generated_item_keyframe_span(item)
        .map(|(start, end)| end.saturating_sub(start))
        .or_else(|| crate::video::visual_duration(project, key))
}

fn video_change() -> ProjectChange {
    ProjectChange {
        video: true,
        ..ProjectChange::default()
    }
}

macro_rules! vec3_accessors {
    ($get:ident, $get_mut:ident, $field:ident.$value:ident) => {
        fn $get(project: &Project, key: SelectedItem) -> Option<&TimelineValue<glam::Vec3>> {
            Some(&selected_scene(project, key.clone())?.$field.$value)
        }

        fn $get_mut(
            project: &mut Project,
            key: SelectedItem,
        ) -> Option<&mut TimelineValue<glam::Vec3>> {
            Some(&mut selected_scene_mut(project, key.clone())?.$field.$value)
        }
    };
}

macro_rules! scalar_accessors {
    ($get:ident, $get_mut:ident, $value:ident) => {
        fn $get(project: &Project, key: SelectedItem) -> Option<&TimelineValue<f32>> {
            Some(&selected_scene(project, key.clone())?.camera.$value)
        }

        fn $get_mut(project: &mut Project, key: SelectedItem) -> Option<&mut TimelineValue<f32>> {
            Some(&mut selected_scene_mut(project, key.clone())?.camera.$value)
        }
    };
}

vec3_accessors!(model_position, model_position_mut, model.position);
vec3_accessors!(model_anchor, model_anchor_mut, model.anchor);
vec3_accessors!(model_rotation, model_rotation_mut, model.rotation_degrees);
vec3_accessors!(model_scale, model_scale_mut, model.scale);
vec3_accessors!(camera_position, camera_position_mut, camera.position);
vec3_accessors!(
    camera_rotation,
    camera_rotation_mut,
    camera.rotation_degrees
);
scalar_accessors!(camera_fov, camera_fov_mut, vertical_fov_degrees);
scalar_accessors!(
    camera_focus_distance,
    camera_focus_distance_mut,
    focus_distance
);
scalar_accessors!(camera_f_stop, camera_f_stop_mut, f_stop);
scalar_accessors!(
    camera_orthographic_height,
    camera_orthographic_height_mut,
    orthographic_height
);
scalar_accessors!(camera_exposure, camera_exposure_mut, exposure_ev);
