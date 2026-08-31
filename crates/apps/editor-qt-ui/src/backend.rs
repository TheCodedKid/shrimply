use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QString, QUrl};
use shrimply_cross_ui_core::editor::{EditorSession, LoadEvent, ProjectLoader, SessionEvent};
use shrimply_math_core::{Fraction, frame_count, frame_index, time_from_signed_frame};
use shrimply_project::project::{self, CanvasSize};
use shrimply_state::player_state;
use std::path::PathBuf;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("gpu_surface.h");
        #[namespace = "shrimply"]
        fn force_opengl();
        #[namespace = "shrimply"]
        fn configure_icons();
        #[namespace = "shrimply"]
        fn fixed_font_family() -> QString;
        #[namespace = "shrimply"]
        fn register_gpu_surfaces();

        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qurl.h");
        type QUrl = cxx_qt_lib::QUrl;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(bool, ready)]
        #[qproperty(QString, loading_text, cxx_name = "loadingText")]
        #[qproperty(QString, project_title, cxx_name = "projectTitle")]
        #[qproperty(bool, playing)]
        #[qproperty(i64, position_frame, cxx_name = "positionFrame")]
        #[qproperty(i64, duration_frame, cxx_name = "durationFrame")]
        #[qproperty(QString, time_label, cxx_name = "timeLabel")]
        #[qproperty(QString, frame_rate_label, cxx_name = "frameRateLabel")]
        #[qproperty(QString, playback_speed_label, cxx_name = "playbackSpeedLabel")]
        #[qproperty(QString, fixed_font_family, cxx_name = "fixedFontFamily")]
        type EditorBackend = super::EditorBackendRust;

        #[qinvokable]
        fn begin(self: Pin<&mut EditorBackend>);
        #[qinvokable]
        fn poll(self: Pin<&mut EditorBackend>);
        #[qinvokable]
        #[cxx_name = "confirmKdenlive"]
        fn confirm_kdenlive(self: Pin<&mut EditorBackend>, convert: bool);
        #[qinvokable]
        #[cxx_name = "chooseOtio"]
        fn choose_otio(
            self: Pin<&mut EditorBackend>,
            accepted: bool,
            width: i32,
            height: i32,
            fps_numerator: i32,
            fps_denominator: i32,
        );
        #[qinvokable]
        #[cxx_name = "confirmRepair"]
        fn confirm_repair(self: Pin<&mut EditorBackend>, repair: bool);
        #[qinvokable]
        #[cxx_name = "chooseDestination"]
        fn choose_destination(self: Pin<&mut EditorBackend>, accepted: bool, url: &QUrl);
        #[qinvokable]
        #[cxx_name = "acknowledgeWarnings"]
        fn acknowledge_warnings(self: Pin<&mut EditorBackend>);
        #[qinvokable]
        #[cxx_name = "resolveLock"]
        fn resolve_lock(self: Pin<&mut EditorBackend>, action: i32);
        #[qinvokable]
        #[cxx_name = "togglePlaying"]
        fn toggle_playing(self: Pin<&mut EditorBackend>);
        #[qinvokable]
        #[cxx_name = "stepFrame"]
        fn step_frame(self: Pin<&mut EditorBackend>, delta: i32);
        #[qinvokable]
        #[cxx_name = "seekFrame"]
        fn seek_frame(self: Pin<&mut EditorBackend>, frame: i64);
        #[qinvokable]
        fn save(self: Pin<&mut EditorBackend>);
        #[qinvokable]
        fn undo(self: Pin<&mut EditorBackend>);
        #[qinvokable]
        fn redo(self: Pin<&mut EditorBackend>);

        #[qsignal]
        #[cxx_name = "requestKdenlive"]
        fn request_kdenlive(self: Pin<&mut EditorBackend>);
        #[qsignal]
        #[cxx_name = "requestOtio"]
        fn request_otio(self: Pin<&mut EditorBackend>);
        #[qsignal]
        #[cxx_name = "requestRepair"]
        fn request_repair(self: Pin<&mut EditorBackend>);
        #[qsignal]
        #[cxx_name = "requestDestination"]
        fn request_destination(
            self: Pin<&mut EditorBackend>,
            title: QString,
            suggested_name: QString,
        );
        #[qsignal]
        #[cxx_name = "requestWarnings"]
        fn request_warnings(self: Pin<&mut EditorBackend>, body: QString);
        #[qsignal]
        #[cxx_name = "requestLock"]
        fn request_lock(self: Pin<&mut EditorBackend>, pid: i64);
        #[qsignal]
        #[cxx_name = "showError"]
        fn show_error(self: Pin<&mut EditorBackend>, heading: QString, body: QString);
        #[qsignal]
        #[cxx_name = "showPlaybackError"]
        fn show_playback_error(self: Pin<&mut EditorBackend>, body: QString);
        #[qsignal]
        fn canceled(self: Pin<&mut EditorBackend>);
    }

    impl cxx_qt::Initialize for EditorBackend {}
}

pub struct EditorBackendRust {
    ready: bool,
    loading_text: QString,
    project_title: QString,
    playing: bool,
    position_frame: i64,
    duration_frame: i64,
    time_label: QString,
    frame_rate_label: QString,
    playback_speed_label: QString,
    fixed_font_family: QString,
    loader: Option<ProjectLoader>,
    session: Option<Pin<Box<EditorSession>>>,
    pending_lock_pid: Option<u32>,
}

impl cxx_qt::Initialize for qobject::EditorBackend {
    fn initialize(self: Pin<&mut Self>) {}
}

impl Default for EditorBackendRust {
    fn default() -> Self {
        Self {
            ready: false,
            loading_text: QString::from("Loading project…"),
            project_title: QString::from("Shrimply"),
            playing: false,
            position_frame: 0,
            duration_frame: 0,
            time_label: QString::default(),
            frame_rate_label: QString::from("--"),
            playback_speed_label: QString::from("x1"),
            fixed_font_family: qobject::fixed_font_family(),
            loader: None,
            session: None,
            pending_lock_pid: None,
        }
    }
}

impl qobject::EditorBackend {
    pub fn begin(mut self: Pin<&mut Self>) {
        assert!(
            self.loader.is_none(),
            "Qt editor project loader already started"
        );
        let path = std::env::args_os()
            .nth(1)
            .map(PathBuf::from)
            .expect("shrimply-editor-qt requires a project path");
        let mut loader = ProjectLoader::new(path);
        let event = loader.begin();
        self.as_mut().rust_mut().get_mut().loader = Some(loader);
        self.handle_event(event);
    }

    pub fn poll(mut self: Pin<&mut Self>) {
        let session_event = self
            .as_ref()
            .rust()
            .session
            .as_deref()
            .and_then(EditorSession::poll);
        if let Some(SessionEvent::AudioPlaybackStopped(error)) = session_event {
            self.as_mut().show_playback_error(QString::from(error));
        }
        let event = self
            .as_mut()
            .rust_mut()
            .get_mut()
            .loader
            .as_mut()
            .and_then(ProjectLoader::poll);
        if let Some(event) = event {
            self.as_mut().handle_event(event);
        }
        self.as_mut().update_player_properties();
    }

    pub fn confirm_kdenlive(mut self: Pin<&mut Self>, convert: bool) {
        let event = self.as_mut().loader_mut().confirm_kdenlive(convert);
        self.as_mut().handle_event(event);
    }

    pub fn choose_otio(
        mut self: Pin<&mut Self>,
        accepted: bool,
        width: i32,
        height: i32,
        fps_numerator: i32,
        fps_denominator: i32,
    ) {
        let settings = accepted.then(|| {
            let width = u32::try_from(width).expect("OTIO width must be positive");
            let height = u32::try_from(height).expect("OTIO height must be positive");
            assert!(
                fps_numerator > 0 && fps_denominator > 0,
                "invalid OTIO frame rate"
            );
            (
                CanvasSize { width, height },
                Fraction::new(fps_numerator as u64, fps_denominator as u64),
            )
        });
        let event = self.as_mut().loader_mut().choose_otio_settings(settings);
        self.as_mut().handle_event(event);
    }

    pub fn confirm_repair(mut self: Pin<&mut Self>, repair: bool) {
        let event = self.as_mut().loader_mut().confirm_frame_grid_repair(repair);
        self.as_mut().handle_event(event);
    }

    pub fn choose_destination(mut self: Pin<&mut Self>, accepted: bool, url: &QUrl) {
        let destination = accepted.then(|| local_path(url)).flatten();
        let event = self.as_mut().loader_mut().choose_destination(destination);
        self.as_mut().handle_event(event);
    }

    pub fn acknowledge_warnings(mut self: Pin<&mut Self>) {
        let event = self.as_mut().loader_mut().acknowledge_warnings();
        self.as_mut().handle_event(event);
    }

    pub fn resolve_lock(mut self: Pin<&mut Self>, action: i32) {
        let pid = self
            .as_mut()
            .rust_mut()
            .get_mut()
            .pending_lock_pid
            .take()
            .expect("Qt lock response without a pending lock");
        let event = match action {
            1 => self.as_mut().loader_mut().retry_locked_project(false, pid),
            2 => self.as_mut().loader_mut().retry_locked_project(true, pid),
            _ => self.as_mut().loader_mut().cancel(),
        };
        self.as_mut().handle_event(event);
    }

    pub fn toggle_playing(self: Pin<&mut Self>) {
        if let Some(session) = self.session.as_deref() {
            player_state::toggle_playing(&session.player_state);
        }
    }

    pub fn step_frame(self: Pin<&mut Self>, delta: i32) {
        let Some(session) = self.session.as_deref() else {
            return;
        };
        let snapshot = player_state::snapshot(&session.player_state);
        let current = frame_index(snapshot.position, snapshot.frame_rate).unwrap_or(0);
        let target = current.saturating_add(i64::from(delta)).max(0);
        if let Some(position) = time_from_signed_frame(target, snapshot.frame_rate) {
            shrimply_preview_qt::mark_preview_step(delta);
            player_state::seek_time(&session.player_state, position);
        }
    }

    pub fn seek_frame(self: Pin<&mut Self>, frame: i64) {
        let Some(session) = self.session.as_deref() else {
            return;
        };
        let fps = player_state::snapshot(&session.player_state).frame_rate;
        if let Some(position) = time_from_signed_frame(frame.max(0), fps) {
            player_state::seek_time(&session.player_state, position);
        }
    }

    pub fn save(self: Pin<&mut Self>) {
        if let Err(error) = project::save() {
            self.emit_error("Could not save project", &error);
        }
    }

    pub fn undo(self: Pin<&mut Self>) {
        self.history_action(project::undo);
    }

    pub fn redo(self: Pin<&mut Self>) {
        self.history_action(project::redo);
    }

    fn history_action(self: Pin<&mut Self>, action: fn(&mut project::Project) -> bool) {
        let Some(session) = self.session.as_deref() else {
            return;
        };
        if action(&mut session.project.borrow_mut()) {
            let duration = session.project.borrow().duration();
            player_state::refresh_project(
                &session.player_state,
                player_state::ProjectChange {
                    duration: Some(duration),
                    audio: true,
                    audio_beats: true,
                    audio_waveforms: true,
                    video: true,
                    captions: true,
                    inspector: true,
                    ..Default::default()
                },
            );
        }
    }

    fn loader_mut(self: Pin<&mut Self>) -> &mut ProjectLoader {
        self.rust_mut()
            .get_mut()
            .loader
            .as_mut()
            .expect("Qt editor project loader is not active")
    }

    fn handle_event(mut self: Pin<&mut Self>, event: LoadEvent) {
        match event {
            LoadEvent::ConfirmKdenlive => self.as_mut().request_kdenlive(),
            LoadEvent::ChooseOtioSettings => self.as_mut().request_otio(),
            LoadEvent::Progress(text) => self.set_loading_text(QString::from(text)),
            LoadEvent::ConfirmFrameGridRepair => self.as_mut().request_repair(),
            LoadEvent::ChooseDestination {
                title,
                suggested_name,
            } => self.as_mut().request_destination(
                QString::from(shrimply_i18n::text(title).as_ref()),
                QString::from(suggested_name),
            ),
            LoadEvent::ImportWarnings(warnings) => self
                .as_mut()
                .request_warnings(QString::from(warnings.join("\n"))),
            LoadEvent::LockedByOtherInstance(pid) => {
                self.as_mut().rust_mut().get_mut().pending_lock_pid = Some(pid);
                self.as_mut().request_lock(i64::from(pid));
            }
            LoadEvent::Ready { path, project } => {
                if let Err(error) = shrimply_support::recent_projects::touch(&path, &project.name) {
                    tracing::warn!(%error, "could not update recent projects");
                }
                ffmpeg_next::init().expect("FFmpeg should initialize");
                ffmpeg_next::util::log::set_level(ffmpeg_next::util::log::Level::Error);
                let session = EditorSession::new(*project)
                    .unwrap_or_else(|error| panic!("could not initialize Qt editor: {error}"));
                shrimply_preview_qt::install(&session).unwrap_or_else(|error| {
                    panic!("could not initialize Qt GPU surfaces: {error}")
                });
                self.as_mut()
                    .set_project_title(QString::from(session.project.borrow().name.as_str()));
                self.as_mut().rust_mut().get_mut().session = Some(Box::pin(session));
                self.as_mut().set_ready(true);
                self.as_mut().update_player_properties();
            }
            LoadEvent::Error { heading, body } => self.emit_error(heading, &body),
            LoadEvent::Canceled => self.as_mut().canceled(),
        }
    }

    fn update_player_properties(mut self: Pin<&mut Self>) {
        let this = self.as_ref();
        let Some(session) = this.rust().session.as_deref() else {
            return;
        };
        let snapshot = player_state::snapshot(&session.player_state);
        let playing = snapshot.playing;
        let position_frame = frame_index(snapshot.position, snapshot.frame_rate).unwrap_or(0);
        let duration_frame = frame_count(snapshot.duration, snapshot.frame_rate)
            .and_then(|frame| i64::try_from(frame).ok())
            .unwrap_or(i64::MAX);
        let time_label = QString::from(shrimply_preview_runtime::playback_time_label(
            snapshot.position,
            snapshot.duration,
        ));
        let frame_rate_label = QString::from(shrimply_preview_qt::preview_frame_rate_label());
        let playback_speed_label = QString::from(shrimply_preview_runtime::playback_speed_label(
            snapshot.playback_speed,
        ));
        self.as_mut().set_playing(playing);
        self.as_mut().set_position_frame(position_frame);
        self.as_mut().set_duration_frame(duration_frame);
        self.as_mut().set_time_label(time_label);
        self.as_mut().set_frame_rate_label(frame_rate_label);
        self.as_mut().set_playback_speed_label(playback_speed_label);
    }

    fn emit_error(mut self: Pin<&mut Self>, heading: &str, body: &str) {
        self.as_mut().show_error(
            QString::from(shrimply_i18n::text(heading).as_ref()),
            QString::from(body),
        );
    }
}

fn local_path(url: &QUrl) -> Option<PathBuf> {
    url.is_local_file()
        .then(|| PathBuf::from(url.to_local_file_or_default().to_string()))
}
