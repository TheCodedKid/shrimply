use std::{cell::RefCell, collections::HashSet, thread, time::Duration};

use adw::prelude::*;
use gtk::glib;
use shrimply_3dgs::CameraSource;
use shrimply_inspector_core::{
    AnalysisControlPresentation, ControlKind, InspectorControl, InspectorTarget,
    camera_source::{self, CameraSourcePresentation},
};

use crate::{InspectorContext, section::InspectorSection, ui::dropdown};

thread_local! {
    static MODEL_REQUESTS: RefCell<HashSet<String>> = RefCell::default();
}

pub(super) fn add_controls(
    section: &InspectorSection,
    source: &CameraSource,
    context: &InspectorContext,
) -> bool {
    let key = context
        .selected_item
        .clone()
        .expect("camera controls require a selected item");
    let server_url = shrimply_state::preferences::snapshot(&context.preferences).compute_server_url;
    let models = camera_source::cached_tracking_models(&server_url);
    let presentation =
        camera_source::presentation(&context.project.borrow(), &key, source, models.as_ref());
    if models.is_none() {
        request_models(&server_url, context);
    }
    add_presentation(section, presentation, context, server_url);
    matches!(source, CameraSource::Custom)
}

fn add_presentation(
    section: &InspectorSection,
    presentation: CameraSourcePresentation,
    context: &InspectorContext,
    server_url: String,
) {
    let key = context
        .selected_item
        .clone()
        .expect("camera controls require a selected item");
    let target = InspectorTarget::Item(key);
    let mut controls = presentation.section.controls.into_iter();

    let source = controls.next().expect("camera source selector is missing");
    section.add_control_row(&source.label, &selector(&source, &target, context));
    if presentation.custom {
        assert!(controls.next().is_none());
        return;
    }

    let method = controls
        .next()
        .expect("camera tracking method selector is missing");
    section.add_control_row(&method.label, &selector(&method, &target, context));

    let mut next = controls
        .next()
        .expect("camera tracking controls are incomplete");
    if next.path == camera_source::QUALITY_PATH {
        section.add_control_row(&next.label, &selector(&next, &target, context));
        next = controls.next().expect("camera model selector is missing");
        assert_eq!(next.path, camera_source::CAMERA_MODEL_PATH);
        section.add_control_row(&next.label, &selector(&next, &target, context));
        next = controls
            .next()
            .expect("camera analysis FPS control is missing");
    }

    assert_eq!(next.kind, ControlKind::Number);
    assert_eq!(next.path, camera_source::ANALYSIS_FPS_PATH);
    let fps = gtk::SpinButton::with_range(next.number.minimum, next.number.maximum, 1.0);
    fps.set_value(
        next.value
            .parse()
            .expect("shared camera analysis FPS must be numeric"),
    );
    fps.set_digits(0);
    fps.set_sensitive(next.sensitive);
    let fps_controller = context.inspector_core.clone();
    let fps_target = target.clone();
    let fps_path = next.path.clone();
    fps.connect_value_changed(move |spin| {
        let value = spin.value_as_int().clamp(1, 60).to_string();
        if let Some(Err(error)) =
            fps_controller.set_camera_source_field(&fps_target, &fps_path, &value)
        {
            tracing::error!(%error, "Could not update GTK camera analysis FPS");
        }
    });
    section.add_control_row(&next.label, &fps);

    let status = controls.next().expect("camera analysis status is missing");
    assert_eq!(status.kind, ControlKind::ReadOnly);
    let status_row = adw::ActionRow::builder()
        .title(shrimply_gtk_components::tr!(&status.label).as_ref())
        .subtitle(shrimply_gtk_components::tr!(&status.value).as_ref())
        .build();
    let spinner = adw::Spinner::new();
    spinner.set_size_request(18, 18);
    spinner.set_visible(status.busy);
    status_row.add_suffix(&spinner);
    section.add_wide_control(&status_row);

    let analysis = controls.next().expect("camera analysis action is missing");
    assert_eq!(analysis.kind, ControlKind::Analysis);
    assert_eq!(
        analysis.action,
        Some(shrimply_inspector_core::InspectorControlAction::ToggleCameraAnalysis)
    );
    assert!(controls.next().is_none());
    let analysis_state = analysis
        .analysis
        .expect("camera analysis action has no shared state");
    let button = gtk::Button::builder()
        .label(shrimply_gtk_components::tr!(&analysis_state.label).as_ref())
        .sensitive(analysis_state.sensitive)
        .halign(gtk::Align::Fill)
        .build();
    let action_controller = context.inspector_core.clone();
    let action_target = target.clone();
    let action_url = server_url.clone();
    button.connect_clicked(move |_| {
        if let Err(error) =
            action_controller.toggle_camera_analysis(&action_target, action_url.clone())
        {
            tracing::error!(%error, "Could not toggle GTK camera tracking analysis");
        }
    });
    section.add_wide_control(&button);

    if analysis_state.active() {
        let poll_context = context.detached();
        let poll_controller = context.inspector_core.clone();
        glib::timeout_add_local(Duration::from_millis(150), move || {
            if button.parent().is_none() {
                return glib::ControlFlow::Break;
            }
            let Ok((state, status)) = poll_controller.camera_analysis_state(&target, &server_url)
            else {
                return glib::ControlFlow::Break;
            };
            update_analysis_controls(&status_row, &spinner, &button, &state, &status);
            if state.active() {
                glib::ControlFlow::Continue
            } else {
                refresh(&poll_context);
                glib::ControlFlow::Break
            }
        });
    }
}

fn selector(
    control: &InspectorControl,
    target: &InspectorTarget,
    context: &InspectorContext,
) -> gtk::DropDown {
    assert_eq!(control.kind, ControlKind::Selector);
    let selected = control
        .values
        .iter()
        .position(|value| value == &control.value)
        .expect("shared camera selector value must be listed");
    let values = control.values.clone();
    let controller = context.inspector_core.clone();
    let target = target.clone();
    let path = control.path.clone();
    let widget = dropdown(
        selected,
        control.labels.clone().into_iter().enumerate(),
        move |index| {
            if let Some(Err(error)) =
                controller.set_camera_source_field(&target, &path, &values[index])
            {
                tracing::error!(%error, "Could not update GTK camera source");
            }
        },
    );
    widget.set_sensitive(control.sensitive);
    widget
}

fn update_analysis_controls(
    status_row: &adw::ActionRow,
    spinner: &adw::Spinner,
    button: &gtk::Button,
    state: &AnalysisControlPresentation,
    status: &str,
) {
    status_row.set_subtitle(status);
    spinner.set_visible(state.active());
    button.set_label(shrimply_gtk_components::tr!(&state.label).as_ref());
    button.set_sensitive(state.sensitive);
}

fn request_models(server_url: &str, context: &InspectorContext) {
    if !MODEL_REQUESTS.with(|requests| requests.borrow_mut().insert(server_url.to_string())) {
        return;
    }
    let url = server_url.to_string();
    let request_url = url.clone();
    let refresh_context = context.detached();
    let (sender, receiver) = async_channel::bounded(1);
    thread::spawn(move || {
        let result = camera_source::tracking_models(&request_url);
        let _ = sender.send_blocking(result);
    });
    glib::spawn_future_local(async move {
        let _ = receiver.recv().await;
        MODEL_REQUESTS.with(|requests| {
            requests.borrow_mut().remove(&url);
        });
        refresh(&refresh_context);
    });
}

fn refresh(context: &InspectorContext) {
    context.inspector_core.refresh_analysis_output();
}
