use shrimply_3dgs::{
    AnimatedTransform3d, Camera3d, CameraProjection, GaussianScene, RotationOrder,
};
use shrimply_core::timeline_value::TimelineValue;
use shrimply_project::project::{ItemAddress, Project, Time};

use crate::{
    ControlKind, InspectorControl, InspectorRuntime, InspectorSection, LayeredState, NumberMapping,
    NumberSpec, VideoCard, VideoReset,
};

pub const MODEL_POSITION_PATH: &str = "/content/model/position";
pub const MODEL_ANCHOR_PATH: &str = "/content/model/anchor";
pub const MODEL_ROTATION_PATH: &str = "/content/model/rotation_degrees";
pub const MODEL_ROTATION_ORDER_PATH: &str = "/content/model/rotation_order";
pub const MODEL_SCALE_PATH: &str = "/content/model/scale";
pub const CAMERA_PROJECTION_PATH: &str = "/content/camera/projection";
pub const CAMERA_POSITION_PATH: &str = "/content/camera/position";
pub const CAMERA_ROTATION_PATH: &str = "/content/camera/rotation_degrees";
pub const CAMERA_FOV_PATH: &str = "/content/camera/vertical_fov_degrees";
pub const CAMERA_FOCUS_DISTANCE_PATH: &str = "/content/camera/focus_distance";
pub const CAMERA_F_STOP_PATH: &str = "/content/camera/f_stop";
pub const CAMERA_ORTHOGRAPHIC_HEIGHT_PATH: &str = "/content/camera/orthographic_height";
pub const CAMERA_EXPOSURE_PATH: &str = "/content/camera/exposure_ev";

pub const VECTOR_COMMIT: &str = "edit-scene-3d-vec3";
pub const VECTOR_EXPRESSION_COMMIT: &str = "edit-scene-3d-vec3-expression";
pub const ROTATION_ORDER_COMMIT: &str = "edit-gaussian-rotation-order";
pub const PROJECTION_COMMIT: &str = "edit-gaussian-projection";
pub const SCALAR_COMMIT: &str = "edit-gaussian-scalar";

pub fn cards(
    project: &Project,
    address: &ItemAddress,
    scene: &GaussianScene,
    runtime: InspectorRuntime,
    camera_models: Option<&Result<Vec<String>, String>>,
) -> [VideoCard; 2] {
    [
        model_card(&scene.model, runtime),
        camera_card(project, address, &scene.camera, runtime, camera_models),
    ]
}

pub fn model_card(model: &AnimatedTransform3d, runtime: InspectorRuntime) -> VideoCard {
    let mut section = InspectorSection::default();
    section.add(vector(
        MODEL_POSITION_PATH,
        "Position",
        &model.position,
        runtime,
        false,
        false,
    ));
    section.add(vector(
        MODEL_ANCHOR_PATH,
        "Anchor",
        &model.anchor,
        runtime,
        false,
        false,
    ));
    section.add(vector(
        MODEL_ROTATION_PATH,
        "Rotation",
        &model.rotation_degrees,
        runtime,
        false,
        true,
    ));
    section.add(
        crate::selector::layered_step_selector(
            MODEL_ROTATION_ORDER_PATH,
            "Rotation order",
            &model.rotation_order,
            runtime,
        )
        .live_commit(ROTATION_ORDER_COMMIT)
        .timeline_commits(ROTATION_ORDER_COMMIT, ROTATION_ORDER_COMMIT),
    );
    section.add(vector(
        MODEL_SCALE_PATH,
        "Scale",
        &model.scale,
        runtime,
        true,
        false,
    ));
    card("gaussian-model", "Model", section, model_reset())
}

pub fn camera_card(
    project: &Project,
    address: &ItemAddress,
    camera: &Camera3d,
    runtime: InspectorRuntime,
    camera_models: Option<&Result<Vec<String>, String>>,
) -> VideoCard {
    let source =
        crate::camera_source::presentation(project, address, &camera.source, camera_models);
    let custom_source = source.custom;
    let mut section = source.section;
    if custom_source {
        section.add(
            crate::selector::selector(
                CAMERA_PROJECTION_PATH,
                "Projection",
                projection_key(camera.projection),
                [
                    (CameraProjection::Perspective, "Perspective"),
                    (CameraProjection::Orthographic, "Orthographic"),
                    (CameraProjection::Equirectangular, "Equirectangular"),
                    (CameraProjection::Cylindrical, "Cylindrical"),
                    (CameraProjection::Fisheye, "Fisheye"),
                ]
                .map(|(projection, label)| (projection_key(projection), label.to_string())),
            )
            .immediate_commit(PROJECTION_COMMIT),
        );
    }
    section.add(vector(
        CAMERA_POSITION_PATH,
        "Position",
        &camera.position,
        runtime,
        false,
        false,
    ));
    section.add(vector(
        CAMERA_ROTATION_PATH,
        "Rotation",
        &camera.rotation_degrees,
        runtime,
        false,
        true,
    ));
    if custom_source {
        match camera.projection {
            CameraProjection::Perspective => {
                section.add(scalar(
                    CAMERA_FOV_PATH,
                    "Focal length",
                    &camera.vertical_fov_degrees,
                    runtime,
                    ScalarKind::FocalLength,
                ));
                section.add(scalar(
                    CAMERA_FOCUS_DISTANCE_PATH,
                    "Focus distance (0 = off)",
                    &camera.focus_distance,
                    runtime,
                    ScalarKind::Nonnegative,
                ));
                section.add(scalar(
                    CAMERA_F_STOP_PATH,
                    "Aperture",
                    &camera.f_stop,
                    runtime,
                    ScalarKind::FStop,
                ));
            }
            CameraProjection::Orthographic => section.add(scalar(
                CAMERA_ORTHOGRAPHIC_HEIGHT_PATH,
                "Orthographic height",
                &camera.orthographic_height,
                runtime,
                ScalarKind::Positive,
            )),
            CameraProjection::Cylindrical => section.add(scalar(
                CAMERA_FOV_PATH,
                "Vertical FOV",
                &camera.vertical_fov_degrees,
                runtime,
                ScalarKind::Fov,
            )),
            CameraProjection::Equirectangular => {}
            CameraProjection::Fisheye => section.add(scalar(
                CAMERA_FOV_PATH,
                "FOV",
                &camera.vertical_fov_degrees,
                runtime,
                ScalarKind::FisheyeFov,
            )),
        }
        section.add(scalar(
            CAMERA_EXPOSURE_PATH,
            "Exposure",
            &camera.exposure_ev,
            runtime,
            ScalarKind::Exposure,
        ));
    }
    card("gaussian-camera", "Camera", section, camera_reset())
}

pub fn model_reset() -> VideoReset {
    reset(
        "/content/model",
        serde_json::to_value(AnimatedTransform3d::default())
            .expect("default Gaussian model must serialize"),
        "reset-gaussian-model",
    )
}

pub fn camera_reset() -> VideoReset {
    reset(
        "/content/camera",
        serde_json::to_value(Camera3d::default()).expect("default Gaussian camera must serialize"),
        "reset-gaussian-camera",
    )
}

pub fn clamp_scalar(path: &str, value: f32, projection: CameraProjection) -> f32 {
    match path {
        CAMERA_FOCUS_DISTANCE_PATH => value.max(0.0),
        CAMERA_ORTHOGRAPHIC_HEIGHT_PATH => value.max(0.001),
        CAMERA_FOV_PATH if projection == CameraProjection::Fisheye => value.clamp(1.0, 360.0),
        CAMERA_FOV_PATH => value.clamp(1.0, 179.0),
        CAMERA_F_STOP_PATH => value.clamp(shrimply_3dgs::MIN_F_STOP, shrimply_3dgs::MAX_F_STOP),
        CAMERA_EXPOSURE_PATH => value.clamp(
            shrimply_3dgs::MIN_EXPOSURE_EV,
            shrimply_3dgs::MAX_EXPOSURE_EV,
        ),
        _ => value,
    }
}

pub fn rotation_order_timeline(scene: &GaussianScene) -> &TimelineValue<RotationOrder> {
    &scene.model.rotation_order
}

fn card(
    key: &'static str,
    title: &'static str,
    section: InspectorSection,
    reset: VideoReset,
) -> VideoCard {
    VideoCard {
        key,
        title,
        section,
        reset: Some(reset),
        alpha_mask: None,
        preview_facet: None,
        actions: Vec::new(),
    }
}

fn reset(path: &str, value: serde_json::Value, commit_name: &'static str) -> VideoReset {
    VideoReset {
        values: vec![(path.to_string(), value)],
        fraction: None,
        commit_name,
        cancel_stabilization: false,
        paint_palette: false,
    }
}

fn projection_key(projection: CameraProjection) -> String {
    serde_json::to_value(projection)
        .expect("Gaussian camera projection must serialize")
        .as_str()
        .expect("Gaussian camera projection must serialize as text")
        .to_string()
}

fn vector(
    path: &'static str,
    label: &'static str,
    value: &TimelineValue<glam::Vec3>,
    runtime: InspectorRuntime,
    lock: bool,
    degrees: bool,
) -> InspectorControl {
    let current = value.value_at(runtime.local_time.unwrap_or(Time::ZERO));
    let control = InspectorControl::new(ControlKind::LayeredVector3, path, label)
        .components(vec![
            current.x.to_string(),
            current.y.to_string(),
            current.z.to_string(),
        ])
        .number(NumberSpec {
            minimum: if lock {
                0.0
            } else {
                NumberSpec::default().minimum
            },
            drag_step: if degrees { 1.0 } else { 0.1 },
            digits: 2,
            unit: if degrees { "°" } else { "" },
            ..NumberSpec::default()
        })
        .width_characters(5)
        .prefixes(["X", "Y", "Z"])
        .layered(path, LayeredState::from(value))
        .timeline(
            value.id,
            crate::visual_modifiers::vector3_speed_graph(value, runtime),
        )
        .live_commit(VECTOR_COMMIT)
        .timeline_commits(VECTOR_COMMIT, VECTOR_EXPRESSION_COMMIT);
    if lock { control.lock() } else { control }
}

#[derive(Clone, Copy)]
enum ScalarKind {
    Nonnegative,
    Positive,
    FocalLength,
    Fov,
    FisheyeFov,
    FStop,
    Exposure,
}

fn scalar(
    path: &'static str,
    label: &'static str,
    value: &TimelineValue<f32>,
    runtime: InspectorRuntime,
    kind: ScalarKind,
) -> InspectorControl {
    let stored = f64::from(value.value_at(runtime.local_time.unwrap_or(Time::ZERO)));
    let mapping = if matches!(kind, ScalarKind::FocalLength) {
        NumberMapping::FocalLengthMillimeters
    } else {
        NumberMapping::Linear
    };
    InspectorControl::new(ControlKind::LayeredNumber, path, label)
        .value(mapping.display(stored, 1.0).to_string())
        .number(NumberSpec {
            drag_step: 0.1,
            digits: 2,
            minimum: match kind {
                ScalarKind::Nonnegative => 0.0,
                ScalarKind::Positive => 0.001,
                ScalarKind::FocalLength | ScalarKind::Fov | ScalarKind::FisheyeFov => 1.0,
                ScalarKind::FStop => f64::from(shrimply_3dgs::MIN_F_STOP),
                ScalarKind::Exposure => f64::from(shrimply_3dgs::MIN_EXPOSURE_EV),
            },
            maximum: match kind {
                ScalarKind::FocalLength => shrimply_3dgs::focal_length_mm(1.0),
                ScalarKind::Fov => 179.0,
                ScalarKind::FisheyeFov => 360.0,
                ScalarKind::FStop => f64::from(shrimply_3dgs::MAX_F_STOP),
                ScalarKind::Exposure => f64::from(shrimply_3dgs::MAX_EXPOSURE_EV),
                _ => NumberSpec::default().maximum,
            },
            unit: match kind {
                ScalarKind::FocalLength => "mm",
                ScalarKind::Fov | ScalarKind::FisheyeFov => "deg",
                _ => "",
            },
        })
        .width_characters(9)
        .number_mapping(mapping)
        .layered(path, LayeredState::from(value))
        .timeline(value.id, scalar_graph(value, runtime, mapping))
        .live_commit(SCALAR_COMMIT)
        .timeline_commits(SCALAR_COMMIT, SCALAR_COMMIT)
}

fn scalar_graph(
    value: &TimelineValue<f32>,
    runtime: InspectorRuntime,
    mapping: NumberMapping,
) -> Option<crate::ScalarGraph> {
    let mut graph = crate::transform::scalar_graph(
        value,
        value.value_at(runtime.local_time.unwrap_or(Time::ZERO)),
        runtime,
    )?;
    graph
        .points
        .iter_mut()
        .for_each(|point| point.value = mapping.display(point.value, 1.0));
    graph.segments.iter_mut().for_each(|segment| {
        segment.start_value = mapping.display(segment.start_value, 1.0);
        segment.end_value = mapping.display(segment.end_value, 1.0);
    });
    Some(graph)
}
