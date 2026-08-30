use shrimply_math_core::Fraction;
use shrimply_project::project::{self, CanvasSize, Project, ProjectPreparation};
use shrimply_state::{player_state, preferences, preview_focus};
use shrimply_timeline::selection_state;
use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

const KDENLIVE_READING: &str = "Reading and converting Kdenlive timeline…";
const OTIO_READING: &str = "Reading and converting OTIO timeline…";
const PROJECT_READING: &str = "Loading project…";
const PROJECT_WRITING: &str = "Writing Shrimply project…";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImportKind {
    Kdenlive,
    Otio,
}

#[derive(Debug)]
pub enum LoadEvent {
    ConfirmKdenlive,
    ChooseOtioSettings,
    Progress(&'static str),
    ConfirmFrameGridRepair,
    ChooseDestination {
        title: &'static str,
        suggested_name: String,
    },
    ImportWarnings(Vec<String>),
    LockedByOtherInstance(u32),
    Ready {
        path: PathBuf,
        project: Box<Project>,
    },
    Error {
        heading: &'static str,
        body: String,
    },
    Canceled,
}

pub enum SessionEvent {
    AudioPlaybackStopped(String),
}

pub struct EditorSession {
    pub project: Rc<RefCell<Project>>,
    pub player_state: player_state::SharedPlayerState,
    pub playback_performance: shrimply_playback_performance::SharedCollector,
    pub selection_state: selection_state::SharedSelectionState,
    pub preview_focus: preview_focus::SharedPreviewFocus,
    pub preferences: preferences::SharedPreferences,
    pub audio_levels: shrimply_audio::SharedAudioLevels,
    pub audio_player: Rc<shrimply_audio::AudioPlayer>,
    pub property_clipboard: shrimply_property_transfer::SharedClipboard,
    asset_changes: async_channel::Receiver<shrimply_asset::AssetChange>,
    pending_cursor: Rc<Cell<Option<project::Time>>>,
}

impl EditorSession {
    pub fn new(project: Project) -> Result<Self, String> {
        let playback_performance = shrimply_playback_performance::open(Arc::new(project.clone()));
        let project = Rc::new(RefCell::new(project));
        project.borrow().watch_assets()?;
        let (duration, frame_rate, cursor_position) = {
            let project = project.borrow();
            (project.duration(), project.fps, project.cursor_position)
        };
        let player_state = player_state::new(duration, frame_rate);
        if let Some(position) = cursor_position {
            player_state::seek_time(&player_state, position.max(project::Time::ZERO));
        }
        let pending_cursor = Rc::new(Cell::new(None));
        let cursor_target = pending_cursor.clone();
        let cursor_player = player_state.clone();
        player_state::connect_named(&player_state, "persist project cursor", move |event| {
            if matches!(event, player_state::PlayerEvent::State(_)) {
                cursor_target.set(Some(
                    player_state::snapshot(&cursor_player)
                        .position
                        .max(project::Time::ZERO),
                ));
            }
        });
        let watched_project = project.clone();
        player_state::connect_named(&player_state, "watch project assets", move |event| {
            if matches!(event, player_state::PlayerEvent::Project(_))
                && let Err(error) = watched_project.borrow().watch_assets()
            {
                tracing::error!(%error, "could not watch project assets");
            }
        });
        let preferences = preferences::open_with_defaults();
        let preference = preferences::snapshot(&preferences);
        shrimply_blender::set_binary(preference.blender_binary);
        shrimply_audio::pneuma::set_server_url(&preference.compute_server_url);
        let audio_levels = Arc::new(shrimply_audio::AudioLevels::default());
        let audio_player = Rc::new(shrimply_audio::AudioPlayer::new(
            &project.borrow(),
            audio_levels.clone(),
        )?);
        connect_audio_playback(&project, &player_state, &audio_player);
        Ok(Self {
            project,
            player_state,
            playback_performance,
            selection_state: selection_state::new(),
            preview_focus: preview_focus::new(),
            preferences,
            audio_levels,
            audio_player,
            property_clipboard: shrimply_property_transfer::new_clipboard(),
            asset_changes: shrimply_asset::subscribe(),
            pending_cursor,
        })
    }

    pub fn poll(&self) -> Option<SessionEvent> {
        player_state::tick(&self.player_state);
        let event = self
            .audio_player
            .take_failure()
            .map(SessionEvent::AudioPlaybackStopped);
        if event.is_some() {
            player_state::set_playing(&self.player_state, false);
        }
        while let Ok(change) = self.asset_changes.try_recv() {
            let (audio, video) = {
                let project = self.project.borrow();
                (
                    project.uses_audio_asset(&change.path),
                    project.uses_video_asset(&change.path),
                )
            };
            if audio || video {
                tracing::info!(
                    path = %change.path.display(),
                    revision = change.revision,
                    audio,
                    video,
                    "project asset changed"
                );
                player_state::refresh_project(
                    &self.player_state,
                    player_state::ProjectChange {
                        audio,
                        audio_beats: audio,
                        audio_waveforms: audio,
                        video,
                        ..Default::default()
                    },
                );
            }
        }
        let Some(position) = self.pending_cursor.take() else {
            return event;
        };
        let mut project = self.project.borrow_mut();
        if project.cursor_position != Some(position) {
            project.cursor_position = Some(position);
            project::save_view_state(&project);
        }
        event
    }
}

impl Drop for EditorSession {
    fn drop(&mut self) {
        self.audio_player.stop();
    }
}

fn connect_audio_playback(
    project: &Rc<RefCell<Project>>,
    player_state: &player_state::SharedPlayerState,
    audio_player: &Rc<shrimply_audio::AudioPlayer>,
) {
    let project = project.clone();
    let media_player = audio_player.clone();
    let media_state = player_state.clone();
    let last_snapshot = player_state::snapshot(player_state);
    let last_position = Rc::new(Cell::new(last_snapshot.position));
    let last_playing = Rc::new(Cell::new(last_snapshot.playing));
    let last_playback_speed = Rc::new(Cell::new(last_snapshot.playback_speed));
    player_state::connect_named(player_state, "editor audio sync", move |event| {
        let snapshot = player_state::snapshot(&media_state);
        let position_changed = last_position.get() != snapshot.position;
        let playing_changed = last_playing.get() != snapshot.playing;
        let playback_speed_changed = last_playback_speed.get() != snapshot.playback_speed;
        let project_audio_changed = matches!(
            event,
            player_state::PlayerEvent::Project(change) if change.audio
        );
        let natural_playback = matches!(
            event,
            player_state::PlayerEvent::State(player_state::StateChange {
                position: Some(player_state::PositionChange::Playback),
                ..
            })
        );
        if project_audio_changed {
            media_player.set_project(&project.borrow());
        }
        if (position_changed && !natural_playback)
            || playing_changed
            || playback_speed_changed
            || project_audio_changed
        {
            if (position_changed && !natural_playback)
                || (playing_changed && snapshot.playing)
                || project_audio_changed
            {
                media_player.seek(snapshot.position);
            }
            media_player.set_playback_speed(snapshot.playback_speed);
            media_player.set_playing(snapshot.playing);
            if (position_changed || project_audio_changed) && !snapshot.playing {
                media_player.preview_from(snapshot.position);
            }
        }
        if position_changed || playing_changed || playback_speed_changed {
            last_position.set(snapshot.position);
            last_playing.set(snapshot.playing);
            last_playback_speed.set(snapshot.playback_speed);
        }
    });
}

enum AfterSave {
    Load,
    Warnings(Vec<String>),
}

enum Pending {
    Load(Result<ProjectPreparation, project::ProjectLoadError>),
    Import {
        kind: ImportKind,
        source: PathBuf,
        result: Result<(project::ProjectValidationOutcome, Vec<String>), String>,
    },
    Save {
        result: Result<PathBuf, String>,
        error_heading: &'static str,
        after_save: AfterSave,
    },
}

enum State {
    Idle,
    WaitingForEditorLock,
    WaitingForKdenlive,
    WaitingForOtioSettings,
    Working(Receiver<Pending>),
    WaitingForRepair {
        source: PathBuf,
        project: Project,
        error_heading: &'static str,
        after_save: AfterSave,
    },
    WaitingForDestination {
        project: Project,
        error_heading: &'static str,
        after_save: AfterSave,
    },
    WaitingForWarnings {
        path: PathBuf,
    },
    WaitingForLock,
    Finished,
}

pub struct ProjectLoader {
    path: PathBuf,
    state: State,
}

impl ProjectLoader {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            state: State::Idle,
        }
    }

    pub fn begin(&mut self) -> LoadEvent {
        assert!(
            matches!(self.state, State::Idle),
            "project loader already started"
        );
        match acquire_editor_lock() {
            Ok(()) => self.begin_after_editor_lock(),
            Err(project::ProjectLockError::AlreadyLockedByOtherInstance { pid }) => {
                self.state = State::WaitingForEditorLock;
                LoadEvent::LockedByOtherInstance(pid)
            }
            Err(error) => {
                self.state = State::Finished;
                LoadEvent::Error {
                    heading: "Could not start editor",
                    body: editor_lock_error(error),
                }
            }
        }
    }

    fn begin_after_editor_lock(&mut self) -> LoadEvent {
        if has_extension(&self.path, "kdenlive") {
            self.state = State::WaitingForKdenlive;
            LoadEvent::ConfirmKdenlive
        } else if has_extension(&self.path, "otio") {
            self.state = State::WaitingForOtioSettings;
            LoadEvent::ChooseOtioSettings
        } else {
            self.start_native_load();
            LoadEvent::Progress(PROJECT_READING)
        }
    }

    pub fn confirm_kdenlive(&mut self, convert: bool) -> LoadEvent {
        assert!(
            matches!(self.state, State::WaitingForKdenlive),
            "Kdenlive response without a pending confirmation"
        );
        if !convert {
            self.state = State::Finished;
            return LoadEvent::Canceled;
        }
        let source = self.path.clone();
        self.start_worker(move || {
            let result = (|| {
                let import =
                    shrimply_kdenlive::from_file(&source).map_err(|error| error.to_string())?;
                let native = project::from_json_value_with_frame_grid_repair(import.project)
                    .map_err(|error| error.to_string())?;
                Ok((native, import.warnings))
            })();
            Pending::Import {
                kind: ImportKind::Kdenlive,
                source,
                result,
            }
        });
        LoadEvent::Progress(OTIO_READING)
    }

    pub fn choose_otio_settings(&mut self, settings: Option<(CanvasSize, Fraction)>) -> LoadEvent {
        assert!(
            matches!(self.state, State::WaitingForOtioSettings),
            "OTIO settings without a pending request"
        );
        let Some((canvas_size, fps)) = settings else {
            self.state = State::Finished;
            return LoadEvent::Canceled;
        };
        let source = self.path.clone();
        self.start_worker(move || {
            let result = shrimply_otio::from_file(&source, canvas_size, fps).and_then(|import| {
                let native = project::from_json_value_with_frame_grid_repair(import.project)?;
                Ok((native, import.warnings))
            });
            Pending::Import {
                kind: ImportKind::Otio,
                source,
                result,
            }
        });
        LoadEvent::Progress(KDENLIVE_READING)
    }

    pub fn confirm_frame_grid_repair(&mut self, repair: bool) -> LoadEvent {
        let state = std::mem::replace(&mut self.state, State::Finished);
        let State::WaitingForRepair {
            source,
            project,
            error_heading,
            after_save,
        } = state
        else {
            panic!("repair response without a pending confirmation");
        };
        if !repair {
            return LoadEvent::Canceled;
        }
        let title = if matches!(&after_save, AfterSave::Load) {
            "Save Fixed Project"
        } else {
            match error_heading {
                "Could not import Kdenlive project" => "Import Kdenlive as Shrimply Project",
                _ => "Import OTIO as Shrimply Project",
            }
        };
        let suggested_name = fixed_project_filename(&source);
        self.state = State::WaitingForDestination {
            project,
            error_heading,
            after_save,
        };
        LoadEvent::ChooseDestination {
            title,
            suggested_name,
        }
    }

    pub fn choose_destination(&mut self, destination: Option<PathBuf>) -> LoadEvent {
        let state = std::mem::replace(&mut self.state, State::Finished);
        let State::WaitingForDestination {
            project,
            error_heading,
            after_save,
        } = state
        else {
            panic!("destination response without a pending request");
        };
        let Some(mut destination) = destination else {
            return LoadEvent::Canceled;
        };
        if !has_extension(&destination, "shrimp") {
            destination.set_extension("shrimp");
        }
        self.start_worker(move || {
            let result = project::create_project_file(&destination, &project).map(|()| destination);
            Pending::Save {
                result,
                error_heading,
                after_save,
            }
        });
        LoadEvent::Progress(PROJECT_WRITING)
    }

    pub fn acknowledge_warnings(&mut self) -> LoadEvent {
        let state = std::mem::replace(&mut self.state, State::Finished);
        let State::WaitingForWarnings { path } = state else {
            panic!("warning acknowledgement without pending warnings");
        };
        self.path = path;
        self.start_native_load();
        LoadEvent::Progress(PROJECT_READING)
    }

    pub fn retry_locked_project(&mut self, stop_other: bool, pid: u32) -> LoadEvent {
        assert!(
            matches!(
                self.state,
                State::WaitingForEditorLock | State::WaitingForLock
            ),
            "lock response without a pending lock"
        );
        if stop_other && !project::terminate_project_process(pid) {
            return LoadEvent::Error {
                heading: "Could not stop other editor",
                body: "Shrimply could not signal the other process.".to_string(),
            };
        }
        if matches!(self.state, State::WaitingForEditorLock) {
            return match acquire_editor_lock() {
                Ok(()) => self.begin_after_editor_lock(),
                Err(project::ProjectLockError::AlreadyLockedByOtherInstance { pid }) => {
                    LoadEvent::LockedByOtherInstance(pid)
                }
                Err(error) => LoadEvent::Error {
                    heading: "Could not start editor",
                    body: editor_lock_error(error),
                },
            };
        }
        self.start_native_load();
        LoadEvent::Progress(PROJECT_READING)
    }

    pub fn cancel(&mut self) -> LoadEvent {
        self.state = State::Finished;
        LoadEvent::Canceled
    }

    pub fn poll(&mut self) -> Option<LoadEvent> {
        let result = match &self.state {
            State::Working(receiver) => match receiver.try_recv() {
                Ok(result) => result,
                Err(TryRecvError::Empty) => return None,
                Err(TryRecvError::Disconnected) => {
                    self.state = State::Finished;
                    return Some(LoadEvent::Error {
                        heading: "Could not open project",
                        body: "The project worker stopped unexpectedly.".to_string(),
                    });
                }
            },
            _ => return None,
        };
        Some(match result {
            Pending::Load(result) => self.finish_native_load(result),
            Pending::Import {
                kind,
                source,
                result,
            } => self.finish_import(kind, source, result),
            Pending::Save {
                result,
                error_heading,
                after_save,
            } => self.finish_save(result, error_heading, after_save),
        })
    }

    fn start_native_load(&mut self) {
        let path = self.path.clone();
        self.start_worker(move || {
            Pending::Load(project::prepare_project_with_frame_grid_repair(&path))
        });
    }

    fn start_worker(&mut self, work: impl FnOnce() -> Pending + Send + 'static) {
        let (sender, receiver) = mpsc::sync_channel(1);
        thread::spawn(move || {
            let _ = sender.send(work());
        });
        self.state = State::Working(receiver);
    }

    fn finish_native_load(
        &mut self,
        result: Result<ProjectPreparation, project::ProjectLoadError>,
    ) -> LoadEvent {
        match result {
            Ok(ProjectPreparation::Ready(prepared)) => {
                let path = self.path.clone();
                let project = project::activate_project(prepared);
                self.state = State::Finished;
                LoadEvent::Ready {
                    path,
                    project: Box::new(project),
                }
            }
            Ok(ProjectPreparation::FrameGridRepair(project)) => {
                self.state = State::WaitingForRepair {
                    source: self.path.clone(),
                    project,
                    error_heading: "Could not fix project",
                    after_save: AfterSave::Load,
                };
                LoadEvent::ConfirmFrameGridRepair
            }
            Err(project::ProjectLoadError::LockedByOtherInstance { pid }) => {
                self.state = State::WaitingForLock;
                LoadEvent::LockedByOtherInstance(pid)
            }
            Err(project::ProjectLoadError::Other(body)) => {
                self.state = State::Finished;
                LoadEvent::Error {
                    heading: "Could not open project",
                    body,
                }
            }
        }
    }

    fn finish_import(
        &mut self,
        kind: ImportKind,
        source: PathBuf,
        result: Result<(project::ProjectValidationOutcome, Vec<String>), String>,
    ) -> LoadEvent {
        let error_heading = match kind {
            ImportKind::Kdenlive => "Could not import Kdenlive project",
            ImportKind::Otio => "Could not import OTIO",
        };
        let (outcome, warnings) = match result {
            Ok(result) => result,
            Err(body) => {
                self.state = State::Finished;
                return LoadEvent::Error {
                    heading: error_heading,
                    body,
                };
            }
        };
        let after_save = match kind {
            ImportKind::Kdenlive => AfterSave::Load,
            ImportKind::Otio if warnings.is_empty() => AfterSave::Load,
            ImportKind::Otio => AfterSave::Warnings(warnings),
        };
        match outcome {
            project::ProjectValidationOutcome::Valid(project) => {
                self.state = State::WaitingForDestination {
                    project,
                    error_heading,
                    after_save,
                };
                LoadEvent::ChooseDestination {
                    title: match kind {
                        ImportKind::Kdenlive => "Import Kdenlive as Shrimply Project",
                        ImportKind::Otio => "Import OTIO as Shrimply Project",
                    },
                    suggested_name: imported_project_filename(&source),
                }
            }
            project::ProjectValidationOutcome::FrameGridRepair(project) => {
                self.state = State::WaitingForRepair {
                    source,
                    project,
                    error_heading,
                    after_save,
                };
                LoadEvent::ConfirmFrameGridRepair
            }
        }
    }

    fn finish_save(
        &mut self,
        result: Result<PathBuf, String>,
        error_heading: &'static str,
        after_save: AfterSave,
    ) -> LoadEvent {
        match result {
            Ok(path) => match after_save {
                AfterSave::Load => {
                    self.path = path;
                    self.start_native_load();
                    LoadEvent::Progress(PROJECT_READING)
                }
                AfterSave::Warnings(warnings) => {
                    self.state = State::WaitingForWarnings { path };
                    LoadEvent::ImportWarnings(warnings)
                }
            },
            Err(body) => {
                self.state = State::Finished;
                LoadEvent::Error {
                    heading: error_heading,
                    body,
                }
            }
        }
    }
}

fn acquire_editor_lock() -> Result<(), project::ProjectLockError> {
    let runtime = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    project::acquire_project_lock(&runtime.join("shrimply-editor-instance.shrimp"))
}

fn editor_lock_error(error: project::ProjectLockError) -> String {
    match error {
        project::ProjectLockError::AlreadyLockedByThisInstance => {
            "Shrimply editor is already running in this process.".to_string()
        }
        project::ProjectLockError::AlreadyLockedByOtherInstance { pid } => {
            format!("Shrimply editor is already running (PID {pid}).")
        }
        project::ProjectLockError::RegistryUnavailable => {
            "could not access the editor lock registry".to_string()
        }
        project::ProjectLockError::CouldNotCreate(error) => {
            format!("could not lock the editor process: {error}")
        }
    }
}

fn has_extension(path: &Path, expected: &str) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case(expected))
}

fn imported_project_filename(source: &Path) -> String {
    source
        .file_stem()
        .map(|name| format!("{}.shrimp", name.to_string_lossy()))
        .unwrap_or_else(|| "imported.shrimp".to_string())
}

fn fixed_project_filename(source: &Path) -> String {
    let stem = source
        .file_stem()
        .map(|name| name.to_string_lossy())
        .unwrap_or_else(|| "project".into());
    let timestamp = glib::DateTime::now_local()
        .and_then(|time| time.format("%Y-%m-%d_%H-%M-%S"))
        .map(|value| value.to_string())
        .unwrap_or_else(|_| "unknown-time".to_string());
    format!("{stem}_{timestamp}-fix.shrimp")
}
