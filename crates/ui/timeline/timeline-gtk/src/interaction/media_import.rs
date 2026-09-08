use super::*;
use shrimply_components_gtk::tr;

const MEDIA_INSPECTION_DELIVERY_INTERVAL: Duration = Duration::from_millis(16);

fn deliver_media_inspection(
    area: &gtk::GLArea,
    runtime: &Rc<RefCell<TimelineRuntime>>,
    subscription: shrimply_resource_pipeline::Subscription<
        import::InspectionKey,
        (),
        import::MediaInfo,
    >,
    on_ready: impl FnOnce(&gtk::GLArea, import::MediaInfo) + 'static,
) {
    let mut on_ready = Some(on_ready);
    let handle = shrimply_components_gtk::resource_pipeline::deliver(
        area.downgrade(),
        subscription,
        MEDIA_INSPECTION_DELIVERY_INTERVAL,
        move |area, event| match event {
            shrimply_resource_pipeline::Event::Finished(info) => {
                if let Err(error) = info.snapshot.ensure_current() {
                    show_error_dialog(area, "Could not import file", &error);
                } else if let Some(on_ready) = on_ready.take() {
                    on_ready(area, (*info).clone());
                }
            }
            shrimply_resource_pipeline::Event::Failed(error) => {
                show_error_dialog(area, "Could not import file", &error);
            }
            shrimply_resource_pipeline::Event::Progress(_)
            | shrimply_resource_pipeline::Event::Cancelled => {}
        },
    );
    let mut runtime = runtime.borrow_mut();
    runtime.resource_jobs.retain(|job| job.is_active());
    runtime.resource_jobs.push(handle);
}

pub(crate) fn open_track_import_dialog(
    area: &gtk::GLArea,
    project: &Rc<RefCell<Project>>,
    player_state: &SharedPlayerState,
    selection_state: &SharedSelectionState,
    runtime: &Rc<RefCell<TimelineRuntime>>,
    targets: Vec<TrackKey>,
) {
    if targets.is_empty() {
        return;
    }
    let kind = targets[0].kind;
    if !targets.iter().all(|target| target.kind == kind) {
        show_error_dialog(
            area,
            "Could not import file",
            "Selected tracks must have the same type",
        );
        return;
    }

    let label = "Import to Track";
    let dialog = gtk::FileDialog::builder()
        .title(tr!(label).as_ref())
        .build();
    let area = area.clone();
    let project = project.clone();
    let player_state = player_state.clone();
    let selection_state = selection_state.clone();
    let runtime = runtime.clone();
    shrimply_components_gtk::file_picker::open(
        label,
        &dialog,
        None::<&gtk::Window>,
        move |result| {
            let Some(path) = result.ok().and_then(|file| file.path()) else {
                return;
            };
            import_path_to_tracks(
                &area,
                &project,
                &player_state,
                &selection_state,
                &runtime,
                path,
                kind,
                targets.iter().map(|target| target.track_index).collect(),
            );
        },
    );
}

#[allow(clippy::too_many_arguments)]
fn import_path_to_tracks(
    area: &gtk::GLArea,
    project: &Rc<RefCell<Project>>,
    player_state: &SharedPlayerState,
    selection_state: &SharedSelectionState,
    runtime: &Rc<RefCell<TimelineRuntime>>,
    path: PathBuf,
    target_kind: TrackKind,
    track_indices: Vec<usize>,
) {
    let start = player_state::snapshot(player_state).position;
    let default_visual_duration = runtime.borrow().scene.default_visual_duration;
    let started = import::start_track_import(
        &mut project.borrow_mut(),
        path,
        target_kind,
        track_indices,
        start,
        default_visual_duration,
    );
    let started = match started {
        Ok(started) => started,
        Err(error) => {
            finish_track_import(area, player_state, selection_state, Err(error));
            return;
        }
    };
    match started {
        import::TrackImportStart::Complete(result) => {
            finish_track_import(area, player_state, selection_state, Ok(result));
        }
        import::TrackImportStart::Inspect(inspection) => {
            let import::TrackImportInspection {
                subscription,
                context,
            } = inspection;
            let project = project.clone();
            let player_state = player_state.clone();
            let selection_state = selection_state.clone();
            deliver_media_inspection(area, runtime, subscription, move |area, info| {
                let result = import::finish_track_import_inspection(
                    &mut project.borrow_mut(),
                    context,
                    &info,
                );
                finish_track_import(area, &player_state, &selection_state, result);
            });
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn finish_track_import(
    area: &gtk::GLArea,
    player_state: &SharedPlayerState,
    selection_state: &SharedSelectionState,
    result: Result<(import::ImportResult, Time), String>,
) {
    if let Err(error) = import::finish_track_import(player_state, selection_state, result) {
        show_error_dialog(area, "Could not import file", &error);
        return;
    }
    area.queue_render();
}

pub(crate) fn show_error_dialog(area: &gtk::GLArea, heading: &str, body: &str) {
    let dialog = adw::AlertDialog::new(Some(heading), Some(body));
    dialog.add_response("close", "Close");
    dialog.set_close_response("close");
    dialog.set_default_response(Some("close"));
    dialog.choose(
        Some(area.upcast_ref::<gtk::Widget>()),
        None::<&gio::Cancellable>,
        |_| {},
    );
}
