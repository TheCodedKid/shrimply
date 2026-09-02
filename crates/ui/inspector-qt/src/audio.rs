use core::pin::Pin;
use std::{
    cell::RefCell,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QString, QStringList, QUrl};
use serde_json::{Value, json};
use shrimply_inspector_core::{
    InspectorDetail, InspectorRuntime, InspectorTarget,
    tts::{TtsEditorControl, TtsGeneration, TtsInputEdit},
};
use shrimply_math_core::fraction_as_f64;
use shrimply_project::project::{AudioItem, AudioSource};
use shrimply_tts::{TtsModel, TtsSettings};

use crate::item::{InspectorAction, InspectorItem, InspectorListItem};
use crate::list::InspectorCategory;
use crate::section::{ControlKind, InspectorControl, InspectorSection, LayeredState, NumberSpec};

const TTS_RETRY_INTERVAL: Duration = Duration::from_secs(5);
type TtsTable<'a> = (
    &'a str,
    &'a [shrimply_tts::TableColumn],
    &'a [std::collections::BTreeMap<String, String>],
);

thread_local! {
    static TTS_RUNTIME: RefCell<TtsRuntime> = RefCell::new(TtsRuntime::default());
}

#[cxx_qt::bridge]
pub mod tts_qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!("cxx-qt-lib/qurl.h");
        type QUrl = cxx_qt_lib::QUrl;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, audio_id, cxx_name = "audioId")]
        #[qproperty(i32, revision)]
        #[qproperty(bool, ready)]
        #[qproperty(bool, busy)]
        #[qproperty(bool, generating)]
        #[qproperty(QString, status)]
        #[qproperty(QString, status_tooltip, cxx_name = "statusTooltip")]
        #[qproperty(QString, generate_label, cxx_name = "generateLabel")]
        type TtsEditorBackend = super::TtsEditorBackendRust;

        #[qinvokable]
        fn refresh(self: Pin<&mut TtsEditorBackend>);
        #[qinvokable]
        fn poll(self: Pin<&mut TtsEditorBackend>);
        #[qinvokable]
        #[cxx_name = "modelValue"]
        fn model_value(self: &TtsEditorBackend) -> QString;
        #[qinvokable]
        #[cxx_name = "modelValues"]
        fn model_values(self: &TtsEditorBackend) -> QStringList;
        #[qinvokable]
        #[cxx_name = "modelLabels"]
        fn model_labels(self: &TtsEditorBackend) -> QStringList;
        #[qinvokable]
        #[cxx_name = "setModel"]
        fn set_model(self: Pin<&mut TtsEditorBackend>, value: &QString);
        #[qinvokable]
        #[cxx_name = "controlCount"]
        fn control_count(self: &TtsEditorBackend) -> i32;
        #[qinvokable]
        #[cxx_name = "controlKind"]
        fn control_kind(self: &TtsEditorBackend, control: i32) -> i32;
        #[qinvokable]
        #[cxx_name = "controlLabel"]
        fn control_label(self: &TtsEditorBackend, control: i32) -> QString;
        #[qinvokable]
        #[cxx_name = "controlValue"]
        fn control_value(self: &TtsEditorBackend, control: i32) -> QString;
        #[qinvokable]
        #[cxx_name = "controlMaximumLength"]
        fn control_maximum_length(self: &TtsEditorBackend, control: i32) -> i32;
        #[qinvokable]
        #[cxx_name = "controlChoiceValues"]
        fn control_choice_values(self: &TtsEditorBackend, control: i32) -> QStringList;
        #[qinvokable]
        #[cxx_name = "controlChoiceLabels"]
        fn control_choice_labels(self: &TtsEditorBackend, control: i32) -> QStringList;
        #[qinvokable]
        #[cxx_name = "controlMinimum"]
        fn control_minimum(self: &TtsEditorBackend, control: i32) -> f64;
        #[qinvokable]
        #[cxx_name = "controlMaximum"]
        fn control_maximum(self: &TtsEditorBackend, control: i32) -> f64;
        #[qinvokable]
        #[cxx_name = "controlStep"]
        fn control_step(self: &TtsEditorBackend, control: i32) -> f64;
        #[qinvokable]
        #[cxx_name = "controlDigits"]
        fn control_digits(self: &TtsEditorBackend, control: i32) -> i32;
        #[qinvokable]
        #[cxx_name = "setControlValue"]
        fn set_control_value(self: Pin<&mut TtsEditorBackend>, control: i32, value: &QString);
        #[qinvokable]
        #[cxx_name = "setControlToggle"]
        fn set_control_toggle(self: Pin<&mut TtsEditorBackend>, control: i32, value: bool);
        #[qinvokable]
        #[cxx_name = "setControlNumber"]
        fn set_control_number(
            self: Pin<&mut TtsEditorBackend>,
            control: i32,
            numerator: i64,
            denominator: i64,
        );
        #[qinvokable]
        #[cxx_name = "commitControl"]
        fn commit_control(self: Pin<&mut TtsEditorBackend>);
        #[qinvokable]
        #[cxx_name = "chooseControlAudio"]
        fn choose_control_audio(self: Pin<&mut TtsEditorBackend>, control: i32);
        #[qinvokable]
        #[cxx_name = "clearControlAudio"]
        fn clear_control_audio(self: Pin<&mut TtsEditorBackend>, control: i32);
        #[qinvokable]
        #[cxx_name = "showControlAudio"]
        fn show_control_audio(self: Pin<&mut TtsEditorBackend>, control: i32);
        #[qinvokable]
        #[cxx_name = "tableColumnCount"]
        fn table_column_count(self: &TtsEditorBackend, control: i32) -> i32;
        #[qinvokable]
        #[cxx_name = "tableRowCount"]
        fn table_row_count(self: &TtsEditorBackend, control: i32) -> i32;
        #[qinvokable]
        #[cxx_name = "tableColumnLabel"]
        fn table_column_label(self: &TtsEditorBackend, control: i32, column: i32) -> QString;
        #[qinvokable]
        #[cxx_name = "tableColumnMaximumLength"]
        fn table_column_maximum_length(self: &TtsEditorBackend, control: i32, column: i32) -> i32;
        #[qinvokable]
        #[cxx_name = "tableValue"]
        fn table_value(self: &TtsEditorBackend, control: i32, row: i32, column: i32) -> QString;
        #[qinvokable]
        #[cxx_name = "setTableValue"]
        fn set_table_value(
            self: Pin<&mut TtsEditorBackend>,
            control: i32,
            row: i32,
            column: i32,
            value: &QString,
        );
        #[qinvokable]
        #[cxx_name = "addTableRow"]
        fn add_table_row(self: Pin<&mut TtsEditorBackend>, control: i32);
        #[qinvokable]
        #[cxx_name = "removeTableRow"]
        fn remove_table_row(self: Pin<&mut TtsEditorBackend>, control: i32, row: i32);
        #[qinvokable]
        fn generate(self: Pin<&mut TtsEditorBackend>);
        #[qinvokable]
        fn cancel(self: Pin<&mut TtsEditorBackend>);

        #[qsignal]
        fn error(self: Pin<&mut TtsEditorBackend>, message: QString);
        #[qsignal]
        #[cxx_name = "openPath"]
        fn open_path(self: Pin<&mut TtsEditorBackend>, url: QUrl);
    }

    impl cxx_qt::Initialize for TtsEditorBackend {}
}

pub(crate) fn categories(
    audio: &Value,
    details: &[InspectorDetail],
    metadata: Option<crate::MediaMetadataState>,
    gain_value: f32,
    runtime: InspectorRuntime,
    can_paste_modifiers: bool,
) -> Vec<InspectorCategory> {
    let source = audio
        .get("source")
        .and_then(Value::as_object)
        .expect("audio source must be an object");
    let source_kind = source
        .get("kind")
        .and_then(Value::as_str)
        .expect("audio source kind must be text");

    let mut audio_items = vec![output_item(audio, gain_value, runtime).boxed()];
    if source_kind == "generator" {
        audio_items.push(crate::audio_generator::item(source, "/source", runtime).boxed());
    }
    if source_kind == "tts" {
        audio_items.push(tts_item(audio).boxed());
    }
    audio_items.extend(crate::audio_modifiers::items(
        audio
            .get("modifiers")
            .and_then(Value::as_array)
            .expect("audio modifiers must be an array"),
        runtime,
        can_paste_modifiers,
    ));

    let mut categories = vec![InspectorCategory {
        key: "audio",
        label: "Audio",
        icon: "sound-symbolic",
        items: audio_items,
    }];
    if source_kind != "generator" {
        categories.push(InspectorCategory {
            key: "playback",
            label: "Playback",
            icon: "playback-speed-symbolic",
            items: playback_items(audio),
        });
    }
    categories.push(InspectorCategory {
        key: "info",
        label: "Info",
        icon: "info-outline-symbolic",
        items: vec![info_item(audio, details, metadata.as_ref())],
    });
    categories
}

fn output_item(audio: &Value, gain_value: f32, runtime: InspectorRuntime) -> InspectorItem {
    let mut section = InspectorSection::default();
    section.add(
        InspectorControl::new(ControlKind::Boolean, "/enabled", "Enabled").value(
            audio
                .get("enabled")
                .and_then(Value::as_bool)
                .expect("audio enabled must be boolean")
                .to_string(),
        ),
    );
    section.add(timeline_number(
        audio,
        "/gain/decibels",
        "Level",
        gain_value,
        runtime,
        NumberSpec {
            minimum: -60.0,
            maximum: 36.0,
            drag_step: 0.01,
            digits: 2,
            unit: "dB",
        },
    ));

    InspectorItem::new("output", "Output", section).reset(InspectorAction::ResetFields {
        values: vec![
            ("/enabled".to_string(), Value::Bool(true)),
            (
                "/gain".to_string(),
                serde_json::to_value(shrimply_audio_modifiers::GainModifier::default())
                    .expect("default audio gain must serialize"),
            ),
        ],
    })
}

fn playback_items(audio: &Value) -> Vec<InspectorListItem> {
    let speed = audio
        .get("playback_speed")
        .and_then(Value::as_object)
        .expect("audio playback speed must be a fraction");
    let speed_control = InspectorControl::new(ControlKind::Fraction, "/playback_speed", "Value")
        .components(fraction_components(speed))
        .width_characters(9)
        .number(NumberSpec {
            drag_step: 0.05,
            digits: 2,
            unit: "x",
            ..NumberSpec::default()
        });
    let mut speed_section = InspectorSection::default();
    speed_section.add(speed_control);

    let mut pitch_section = InspectorSection::default();
    pitch_section.add(crate::selector::selector(
        "/speed_method",
        "Method",
        text(audio, "/speed_method"),
        [
            ("naive".to_string(), "Naive".to_string()),
            ("preserve_pitch".to_string(), "Preserve pitch".to_string()),
        ],
    ));

    let mut repeat_section = InspectorSection::default();
    repeat_section.add(crate::selector::selector(
        "/repeat_strategy",
        "Strategy",
        text(audio, "/repeat_strategy"),
        [
            ("repeat".to_string(), "Repeat".to_string()),
            ("ping_pong".to_string(), "Ping Pong".to_string()),
            ("hold".to_string(), "Hold".to_string()),
            ("empty".to_string(), "Empty".to_string()),
        ],
    ));

    vec![
        InspectorItem::new("speed", "Speed", speed_section)
            .reset(InspectorAction::Reset {
                path: "/playback_speed".to_string(),
                value: json!({ "numerator": 1, "denominator": 1 }),
            })
            .boxed(),
        InspectorItem::new("speed-method", "Speed method", pitch_section)
            .reset(InspectorAction::Reset {
                path: "/speed_method".to_string(),
                value: Value::String("preserve_pitch".to_string()),
            })
            .boxed(),
        InspectorItem::new("repeat", "Repeat", repeat_section)
            .reset(InspectorAction::Reset {
                path: "/repeat_strategy".to_string(),
                value: Value::String("hold".to_string()),
            })
            .boxed(),
    ]
}

fn info_item(
    audio: &Value,
    details: &[InspectorDetail],
    metadata: Option<&crate::MediaMetadataState>,
) -> InspectorListItem {
    let file_backed = metadata.is_some();
    let audio_stream_count = match metadata {
        Some(crate::MediaMetadataState::Ready(metadata)) => metadata.audio_stream_count,
        _ => 0,
    };
    let mut section = InspectorSection::default();
    if file_backed && audio_stream_count > 1 {
        section.add(crate::selector::selector(
            "/track_id",
            "Audio Stream",
            audio
                .get("track_id")
                .and_then(Value::as_u64)
                .expect("audio stream id must be an integer")
                .min(u64::from(audio_stream_count - 1))
                .to_string(),
            (0..audio_stream_count).map(|stream| {
                (
                    stream.to_string(),
                    shrimply_i18n_qt::text_args(
                        "Audio stream %{number}",
                        &[("number", (stream + 1).to_string())],
                    )
                    .to_string(),
                )
            }),
        ));
    }
    if file_backed {
        let id = audio
            .get("id")
            .and_then(Value::as_str)
            .and_then(|id| uuid::Uuid::parse_str(id).ok())
            .expect("audio id must be a UUID");
        section.add(
            InspectorControl::new(
                ControlKind::BeatDetection,
                "/beat_detection",
                "Beat Detection",
            )
            .target(id)
            .subtitle("Analyze this clip for beat-grid snapping")
            .value(
                audio
                    .get("beat_detection")
                    .and_then(Value::as_bool)
                    .expect("beat detection must be boolean")
                    .to_string(),
            ),
        );
    }
    crate::info::append(
        &mut section,
        details,
        metadata,
        shrimply_inspector_core::info::SourceMetadata::Audio(
            audio
                .get("track_id")
                .and_then(Value::as_u64)
                .and_then(|track| u32::try_from(track).ok())
                .unwrap_or_default(),
        ),
    );
    InspectorListItem::Flat(section)
}

fn tts_item(audio: &Value) -> InspectorItem {
    let id = audio
        .get("id")
        .and_then(Value::as_str)
        .and_then(|id| uuid::Uuid::parse_str(id).ok())
        .expect("audio id must be a UUID");
    let mut section = InspectorSection::default();
    section.add(
        InspectorControl::new(ControlKind::TtsEditor, "", "")
            .target(id)
            .value(id.to_string()),
    );
    InspectorItem::new("tts", "Text to Speech", section).reset(InspectorAction::Reset {
        path: "/source".to_string(),
        value: serde_json::to_value(shrimply_project::project::AudioSource::Tts(Box::default()))
            .expect("default TTS settings must serialize"),
    })
}

fn timeline_number(
    value: &Value,
    path: &str,
    label: &str,
    current: f32,
    runtime: InspectorRuntime,
    number: NumberSpec,
) -> InspectorControl {
    let timeline = value
        .pointer(path)
        .and_then(Value::as_object)
        .expect("audio timeline value must be an object");
    let base = timeline
        .get("base")
        .and_then(Value::as_object)
        .expect("audio timeline base must be an object");
    let expression = timeline.get("expression").and_then(Value::as_object);
    let timeline: shrimply_core::timeline_value::TimelineValue<f32> =
        serde_json::from_value(Value::Object(timeline.clone()))
            .expect("audio timeline value must be valid");
    InspectorControl::new(ControlKind::LayeredNumber, path, label)
        .value(current.to_string())
        .number(number)
        .layered(
            path,
            LayeredState {
                keyframes: base.contains_key("keyframes"),
                expression: expression
                    .and_then(|value| value.get("enabled"))
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                expression_source: expression
                    .and_then(|value| value.get("source"))
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
            },
        )
        .timeline(
            timeline.id,
            crate::audio_modifiers::scalar_graph(&timeline, runtime, f64::from),
        )
}

fn fraction_components(fraction: &serde_json::Map<String, Value>) -> Vec<String> {
    ["numerator", "denominator"]
        .map(|component| {
            fraction
                .get(component)
                .and_then(Value::as_i64)
                .expect("fraction component must be an integer")
                .to_string()
        })
        .to_vec()
}

fn text<'a>(value: &'a Value, path: &str) -> &'a str {
    value
        .pointer(path)
        .and_then(Value::as_str)
        .expect("audio selector value must be text")
}

enum TtsMessage {
    Models {
        server_url: String,
        result: Result<Vec<TtsModel>, String>,
    },
    Progress {
        audio_id: uuid::Uuid,
        status: String,
    },
    Generated {
        audio_id: uuid::Uuid,
        result: Result<TtsGeneration, String>,
    },
}

struct TtsModelResult {
    server_url: String,
    result: Result<Vec<TtsModel>, String>,
    completed: Instant,
}

struct TtsGenerationJob {
    audio_id: uuid::Uuid,
    target: InspectorTarget,
    model: TtsModel,
    cancellation: shrimply_server_client::CancellationToken,
    running: bool,
    status: String,
    error: Option<String>,
}

struct TtsRuntime {
    sender: mpsc::Sender<TtsMessage>,
    receiver: mpsc::Receiver<TtsMessage>,
    pending_models: Vec<String>,
    model_results: Vec<TtsModelResult>,
    generations: Vec<TtsGenerationJob>,
    view_revision: u64,
    status_revision: u64,
}

impl Default for TtsRuntime {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver,
            pending_models: Vec::new(),
            model_results: Vec::new(),
            generations: Vec::new(),
            view_revision: 0,
            status_revision: 0,
        }
    }
}

#[derive(Clone)]
struct TtsEditorView {
    target: InspectorTarget,
    audio_id: uuid::Uuid,
    server_url: String,
    settings: TtsSettings,
    model: Option<TtsModel>,
    models: Vec<TtsModel>,
    controls: Vec<TtsEditorControl>,
    generated: bool,
    model_status: TtsModelStatus,
}

#[derive(Clone)]
enum TtsModelStatus {
    Loading,
    Ready,
    Failed(String),
}

#[derive(Clone)]
struct TtsGenerationStatus {
    running: bool,
    status: String,
    error: Option<String>,
}

pub(crate) fn poll_tts_runtime() -> bool {
    TTS_RUNTIME.with_borrow_mut(|runtime| {
        let messages = runtime.receiver.try_iter().collect::<Vec<_>>();
        let changed = !messages.is_empty();
        for message in messages {
            match message {
                TtsMessage::Models { server_url, result } => {
                    runtime
                        .pending_models
                        .retain(|pending| pending != &server_url);
                    if let Some(cached) = runtime
                        .model_results
                        .iter_mut()
                        .find(|cached| cached.server_url == server_url)
                    {
                        cached.result = result;
                        cached.completed = Instant::now();
                    } else {
                        runtime.model_results.push(TtsModelResult {
                            server_url,
                            result,
                            completed: Instant::now(),
                        });
                    }
                    runtime.view_revision = runtime.view_revision.wrapping_add(1);
                }
                TtsMessage::Progress { audio_id, status } => {
                    if let Some(job) = runtime
                        .generations
                        .iter_mut()
                        .find(|job| job.audio_id == audio_id && job.running)
                        && job.status != status
                    {
                        job.status = status;
                        runtime.status_revision = runtime.status_revision.wrapping_add(1);
                    }
                }
                TtsMessage::Generated { audio_id, result } => {
                    finish_generation(runtime, audio_id, result);
                    runtime.view_revision = runtime.view_revision.wrapping_add(1);
                    runtime.status_revision = runtime.status_revision.wrapping_add(1);
                }
            }
        }
        changed
    })
}

fn finish_generation(
    runtime: &mut TtsRuntime,
    audio_id: uuid::Uuid,
    result: Result<TtsGeneration, String>,
) {
    let Some(index) = runtime
        .generations
        .iter()
        .position(|job| job.audio_id == audio_id && job.running)
    else {
        if let Ok(generation) = result {
            let _ = std::fs::remove_file(generation.path);
        }
        return;
    };
    let cancelled = runtime.generations[index].cancellation.is_cancelled();
    let target = runtime.generations[index].target.clone();
    let model = runtime.generations[index].model.clone();
    let outcome = match result {
        Ok(generation) if cancelled => {
            let _ = std::fs::remove_file(generation.path);
            Ok("Cancelled".to_string())
        }
        Ok(generation) => super::with_controller(|controller| {
            controller.apply_tts_generation(&target, audio_id, &model, generation)
        })
        .map(|()| "Generated".to_string()),
        Err(_) if cancelled => Ok("Cancelled".to_string()),
        Err(error) => Err(error),
    };
    let job = &mut runtime.generations[index];
    job.running = false;
    match outcome {
        Ok(status) => {
            job.status = status;
            job.error = None;
        }
        Err(error) => {
            job.status = if error.starts_with("Compute server connection failed") {
                "Compute server connection failed".to_string()
            } else {
                error.clone()
            };
            job.error = Some(error);
        }
    }
}

fn model_state(server_url: &str) -> (TtsModelStatus, Vec<TtsModel>) {
    TTS_RUNTIME.with_borrow_mut(|runtime| {
        if let Some(index) = runtime
            .model_results
            .iter()
            .position(|cached| cached.server_url == server_url)
        {
            let retry = runtime.model_results[index].result.is_err()
                && runtime.model_results[index].completed.elapsed() >= TTS_RETRY_INTERVAL;
            if retry {
                runtime.model_results.remove(index);
                runtime.view_revision = runtime.view_revision.wrapping_add(1);
            } else {
                return match &runtime.model_results[index].result {
                    Ok(models) => (TtsModelStatus::Ready, models.clone()),
                    Err(error) => (TtsModelStatus::Failed(error.clone()), Vec::new()),
                };
            }
        }
        if runtime
            .pending_models
            .iter()
            .all(|pending| pending != server_url)
        {
            runtime.pending_models.push(server_url.to_string());
            let sender = runtime.sender.clone();
            let request = server_url.to_string();
            thread::spawn(move || {
                let result = shrimply_inspector_core::tts::available_models(&request);
                let _ = sender.send(TtsMessage::Models {
                    server_url: request,
                    result,
                });
            });
        }
        (TtsModelStatus::Loading, Vec::new())
    })
}

fn model_retry_due(server_url: &str) -> bool {
    TTS_RUNTIME.with_borrow(|runtime| {
        runtime.model_results.iter().any(|cached| {
            cached.server_url == server_url
                && cached.result.is_err()
                && cached.completed.elapsed() >= TTS_RETRY_INTERVAL
        })
    })
}

fn generation_status(audio_id: uuid::Uuid) -> Option<TtsGenerationStatus> {
    TTS_RUNTIME.with_borrow(|runtime| {
        runtime
            .generations
            .iter()
            .find(|job| job.audio_id == audio_id)
            .map(|job| TtsGenerationStatus {
                running: job.running,
                status: job.status.clone(),
                error: job.error.clone(),
            })
    })
}

fn runtime_revisions() -> (u64, u64) {
    TTS_RUNTIME.with_borrow(|runtime| (runtime.view_revision, runtime.status_revision))
}

fn build_tts_view(audio_id: uuid::Uuid) -> Result<TtsEditorView, String> {
    TTS_RUNTIME.with_borrow_mut(|runtime| {
        runtime
            .generations
            .retain(|job| job.running || job.audio_id == audio_id);
    });
    let snapshot = super::CONTROLLER.with_borrow(|controller| {
        controller
            .as_ref()
            .expect("Qt TTS editor requested before inspector installation")
            .snapshot()
    });
    let audio: AudioItem = serde_json::from_value(snapshot.value.clone())
        .map_err(|error| format!("invalid TTS audio item: {error}"))?;
    if audio.id != audio_id {
        return Err("TTS editor selection changed".to_string());
    }
    let AudioSource::Tts(settings) = audio.source else {
        return Err("selected audio item no longer uses text to speech".to_string());
    };
    let preferences = super::PREFERENCES.with_borrow(|preferences| {
        shrimply_state::preferences::snapshot(
            preferences
                .as_ref()
                .expect("Qt TTS editor requested before preference installation"),
        )
    });
    let server_url = preferences.compute_server_url;
    let (model_status, models) = model_state(&server_url);
    let mut settings = *settings;
    let model = shrimply_inspector_core::tts::selected_model(
        &models,
        &settings,
        &preferences.last_tts_model,
    )
    .cloned();
    if let Some(model) = &model {
        let synchronized = shrimply_inspector_core::tts::synchronized_settings(&settings, model);
        if synchronized != settings {
            super::with_controller(|controller| {
                controller.set_tts_settings(&snapshot.target, synchronized.clone(), false)
            })?;
            settings = synchronized;
        }
        if preferences.last_tts_model != model.id {
            super::PREFERENCES.with_borrow(|preferences| {
                shrimply_state::preferences::set_last_tts_model(
                    preferences
                        .as_ref()
                        .expect("Qt TTS editor requested before preference installation"),
                    &model.id,
                );
            });
        }
    }
    let controls = model.as_ref().map_or_else(Vec::new, |model| {
        shrimply_inspector_core::tts::editor_controls(&settings, model)
    });
    Ok(TtsEditorView {
        target: snapshot.target,
        audio_id,
        server_url,
        settings,
        model,
        models,
        controls,
        generated: !audio.file.as_os_str().is_empty(),
        model_status,
    })
}

fn start_generation(view: &TtsEditorView) -> Result<(), String> {
    let model = view
        .model
        .clone()
        .ok_or_else(|| "Select a text-to-speech model".to_string())?;
    let cancellation = shrimply_server_client::CancellationToken::new(&view.server_url)?;
    TTS_RUNTIME.with_borrow_mut(|runtime| {
        if runtime
            .generations
            .iter()
            .any(|job| job.audio_id == view.audio_id && job.running)
        {
            return Err("Text-to-speech generation is already running".to_string());
        }
        let job = TtsGenerationJob {
            audio_id: view.audio_id,
            target: view.target.clone(),
            model: model.clone(),
            cancellation: cancellation.clone(),
            running: true,
            status: "Sending request…".to_string(),
            error: None,
        };
        if let Some(current) = runtime
            .generations
            .iter_mut()
            .find(|job| job.audio_id == view.audio_id)
        {
            *current = job;
        } else {
            runtime.generations.push(job);
        }
        runtime.status_revision = runtime.status_revision.wrapping_add(1);
        let sender = runtime.sender.clone();
        let server_url = view.server_url.clone();
        let settings = view.settings.clone();
        let audio_id = view.audio_id;
        thread::spawn(move || {
            let progress_sender = sender.clone();
            let result = shrimply_inspector_core::tts::generate(
                &server_url,
                &cancellation,
                &model,
                &settings,
                |status| {
                    let _ = progress_sender.send(TtsMessage::Progress {
                        audio_id,
                        status: status.to_string(),
                    });
                    !cancellation.is_cancelled()
                },
            );
            let _ = sender.send(TtsMessage::Generated { audio_id, result });
        });
        Ok(())
    })
}

fn cancel_generation(audio_id: uuid::Uuid) -> Result<(), String> {
    TTS_RUNTIME.with_borrow_mut(|runtime| {
        let job = runtime
            .generations
            .iter_mut()
            .find(|job| job.audio_id == audio_id && job.running)
            .ok_or_else(|| "Text-to-speech generation is not running".to_string())?;
        job.cancellation.cancel();
        job.status = "Cancelling…".to_string();
        runtime.status_revision = runtime.status_revision.wrapping_add(1);
        Ok(())
    })
}

pub struct TtsEditorBackendRust {
    audio_id: QString,
    revision: i32,
    ready: bool,
    busy: bool,
    generating: bool,
    status: QString,
    status_tooltip: QString,
    generate_label: QString,
    view: Option<TtsEditorView>,
    runtime_view_revision: u64,
    runtime_status_revision: u64,
}

impl Default for TtsEditorBackendRust {
    fn default() -> Self {
        Self {
            audio_id: QString::default(),
            revision: 0,
            ready: false,
            busy: true,
            generating: false,
            status: QString::from("Connecting to server…"),
            status_tooltip: QString::default(),
            generate_label: QString::from("Generate"),
            view: None,
            runtime_view_revision: 0,
            runtime_status_revision: 0,
        }
    }
}

impl cxx_qt::Initialize for tts_qobject::TtsEditorBackend {
    fn initialize(self: Pin<&mut Self>) {}
}

impl tts_qobject::TtsEditorBackend {
    pub fn refresh(mut self: Pin<&mut Self>) {
        poll_tts_runtime();
        let view = uuid::Uuid::parse_str(&self.audio_id().to_string())
            .map_err(|_| "TTS editor audio ID is invalid".to_string())
            .and_then(build_tts_view);
        let (ready, busy, generating, status, tooltip, label) = match &view {
            Ok(view) => {
                let generation = generation_status(view.audio_id);
                let generating = generation.as_ref().is_some_and(|state| state.running);
                let (mut busy, mut status, mut tooltip) = match &view.model_status {
                    TtsModelStatus::Loading => {
                        (true, "Connecting to server…".to_string(), String::new())
                    }
                    TtsModelStatus::Ready => (false, "Ready".to_string(), String::new()),
                    TtsModelStatus::Failed(error) => (false, error.clone(), error.clone()),
                };
                if let Some(generation) = generation {
                    busy |= generation.running;
                    status = generation.status;
                    tooltip = generation.error.unwrap_or_default();
                }
                (
                    view.model.is_some(),
                    busy,
                    generating,
                    status,
                    tooltip,
                    if view.generated {
                        "Regenerate"
                    } else {
                        "Generate"
                    },
                )
            }
            Err(error) => (
                false,
                false,
                false,
                error.clone(),
                error.clone(),
                "Generate",
            ),
        };
        self.as_mut().rust_mut().view = view.ok();
        let (view_revision, status_revision) = runtime_revisions();
        self.as_mut().rust_mut().runtime_view_revision = view_revision;
        self.as_mut().rust_mut().runtime_status_revision = status_revision;
        self.as_mut().set_ready(ready);
        self.as_mut().set_busy(busy);
        self.as_mut().set_generating(generating);
        self.as_mut().set_status(QString::from(status));
        self.as_mut().set_status_tooltip(QString::from(tooltip));
        self.as_mut().set_generate_label(QString::from(label));
        let revision = self.revision().wrapping_add(1);
        self.as_mut().set_revision(revision);
    }

    pub fn poll(mut self: Pin<&mut Self>) {
        poll_tts_runtime();
        let retry_due = self.rust().view.as_ref().is_some_and(|view| {
            matches!(&view.model_status, TtsModelStatus::Failed(_))
                && model_retry_due(&view.server_url)
        });
        let (view_revision, status_revision) = runtime_revisions();
        if self.rust().runtime_view_revision != view_revision || retry_due {
            self.as_mut().refresh();
        } else if self.rust().runtime_status_revision != status_revision {
            self.as_mut().refresh_status();
        }
    }

    fn refresh_status(mut self: Pin<&mut Self>) {
        let Some(view) = self.rust().view.as_ref() else {
            self.as_mut().refresh();
            return;
        };
        let generation = generation_status(view.audio_id);
        let generating = generation.as_ref().is_some_and(|state| state.running);
        let (mut busy, mut status, mut tooltip) = match &view.model_status {
            TtsModelStatus::Loading => (true, "Connecting to server…".to_string(), String::new()),
            TtsModelStatus::Ready => (false, "Ready".to_string(), String::new()),
            TtsModelStatus::Failed(error) => (false, error.clone(), error.clone()),
        };
        if let Some(generation) = generation {
            busy |= generation.running;
            status = generation.status;
            tooltip = generation.error.unwrap_or_default();
        }
        self.as_mut().rust_mut().runtime_status_revision = runtime_revisions().1;
        self.as_mut().set_busy(busy);
        self.as_mut().set_generating(generating);
        self.as_mut().set_status(QString::from(status));
        self.as_mut().set_status_tooltip(QString::from(tooltip));
    }

    pub fn model_value(&self) -> QString {
        self.rust()
            .view
            .as_ref()
            .and_then(|view| view.model.as_ref())
            .map_or_else(QString::default, |model| QString::from(model.id.as_str()))
    }

    pub fn model_values(&self) -> QStringList {
        self.rust()
            .view
            .as_ref()
            .map(|view| {
                view.models
                    .iter()
                    .map(|model| QString::from(&model.id))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn model_labels(&self) -> QStringList {
        self.rust()
            .view
            .as_ref()
            .map(|view| {
                view.models
                    .iter()
                    .map(|model| QString::from(&model.label))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn set_model(mut self: Pin<&mut Self>, value: &QString) {
        let value = value.to_string();
        let result = self
            .rust()
            .view
            .clone()
            .ok_or_else(|| "TTS editor is not ready".to_string())
            .and_then(|view| {
                let model = view
                    .models
                    .iter()
                    .find(|model| model.id == value)
                    .ok_or_else(|| "Selected TTS model is unavailable".to_string())?;
                let settings =
                    shrimply_inspector_core::tts::synchronized_settings(&view.settings, model);
                super::with_controller(|controller| {
                    controller.set_tts_settings(&view.target, settings, false)
                })?;
                super::PREFERENCES.with_borrow(|preferences| {
                    shrimply_state::preferences::set_last_tts_model(
                        preferences
                            .as_ref()
                            .expect("Qt TTS editor requested before preference installation"),
                        &model.id,
                    );
                });
                Ok(())
            });
        self.as_mut().finish(result);
    }

    pub fn control_count(&self) -> i32 {
        self.rust()
            .view
            .as_ref()
            .and_then(|view| i32::try_from(view.controls.len()).ok())
            .unwrap_or_default()
    }

    pub fn control_kind(&self, control: i32) -> i32 {
        self.control(control).map_or(-1, |control| match control {
            TtsEditorControl::Text {
                multiline: false, ..
            } => 0,
            TtsEditorControl::Text {
                multiline: true, ..
            } => 1,
            TtsEditorControl::Select { .. } => 2,
            TtsEditorControl::Audio { .. } => 3,
            TtsEditorControl::Toggle { .. } => 4,
            TtsEditorControl::Number { .. } => 5,
            TtsEditorControl::Table { .. } => 6,
        })
    }

    pub fn control_label(&self, control: i32) -> QString {
        self.control(control)
            .map_or_else(QString::default, |control| {
                QString::from(control_label(control))
            })
    }

    pub fn control_value(&self, control: i32) -> QString {
        self.control(control)
            .map_or_else(QString::default, |control| {
                QString::from(match control {
                    TtsEditorControl::Text { value, .. }
                    | TtsEditorControl::Select { value, .. } => value.clone(),
                    TtsEditorControl::Audio { path, .. } => path
                        .as_ref()
                        .map(|path| path.to_string_lossy().into_owned())
                        .unwrap_or_default(),
                    TtsEditorControl::Toggle { value, .. } => value.to_string(),
                    TtsEditorControl::Number { value, .. } => fraction_as_f64(*value).to_string(),
                    TtsEditorControl::Table { .. } => String::new(),
                })
            })
    }

    pub fn control_maximum_length(&self, control: i32) -> i32 {
        self.control(control)
            .and_then(|control| match control {
                TtsEditorControl::Text { max_length, .. } => i32::try_from(*max_length).ok(),
                _ => None,
            })
            .unwrap_or_default()
    }

    pub fn control_choice_values(&self, control: i32) -> QStringList {
        self.control(control)
            .and_then(|control| match control {
                TtsEditorControl::Select { choices, .. } => Some(
                    choices
                        .iter()
                        .map(|choice| QString::from(&choice.value))
                        .collect(),
                ),
                _ => None,
            })
            .unwrap_or_default()
    }

    pub fn control_choice_labels(&self, control: i32) -> QStringList {
        self.control(control)
            .and_then(|control| match control {
                TtsEditorControl::Select { choices, .. } => Some(
                    choices
                        .iter()
                        .map(|choice| QString::from(&choice.label))
                        .collect(),
                ),
                _ => None,
            })
            .unwrap_or_default()
    }

    pub fn control_minimum(&self, control: i32) -> f64 {
        self.number(control)
            .map_or(0.0, |(_, minimum, _, _)| fraction_as_f64(minimum))
    }

    pub fn control_maximum(&self, control: i32) -> f64 {
        self.number(control)
            .map_or(0.0, |(_, _, maximum, _)| fraction_as_f64(maximum))
    }

    pub fn control_step(&self, control: i32) -> f64 {
        self.number(control)
            .map_or(1.0, |(_, _, _, step)| fraction_as_f64(step))
    }

    pub fn control_digits(&self, control: i32) -> i32 {
        self.control(control)
            .and_then(|control| match control {
                TtsEditorControl::Number { digits, .. } => i32::try_from(*digits).ok(),
                _ => None,
            })
            .unwrap_or_default()
    }

    pub fn set_control_value(mut self: Pin<&mut Self>, control: i32, value: &QString) {
        let value = value.to_string();
        let result = match self.control(control) {
            Some(TtsEditorControl::Text { .. }) => {
                self.edit_settings(control, TtsInputEdit::Text(value))
            }
            Some(TtsEditorControl::Select { .. }) => {
                self.edit_settings(control, TtsInputEdit::Select(value))
            }
            _ => Err("TTS control does not accept text".to_string()),
        };
        self.as_mut().finish(result);
    }

    pub fn set_control_toggle(mut self: Pin<&mut Self>, control: i32, value: bool) {
        let result = self.edit_settings(control, TtsInputEdit::Toggle(value));
        self.as_mut().finish(result);
    }

    pub fn set_control_number(
        mut self: Pin<&mut Self>,
        control: i32,
        numerator: i64,
        denominator: i64,
    ) {
        let result = if denominator <= 0 {
            Err("TTS number denominator must be positive".to_string())
        } else {
            self.edit_settings(
                control,
                TtsInputEdit::Number(shrimply_math_core::fraction_new(numerator, denominator)),
            )
        };
        self.as_mut().finish(result);
    }

    pub fn commit_control(mut self: Pin<&mut Self>) {
        super::finish_live_edit();
        self.as_mut().refresh();
    }

    pub fn choose_control_audio(mut self: Pin<&mut Self>, control: i32) {
        let Some(TtsEditorControl::Audio { key, label, .. }) = self.control(control).cloned()
        else {
            self.as_mut()
                .error(QString::from("TTS control is not an audio input"));
            return;
        };
        let Some(path) = shrimply_qt_components::file_picker::open(
            &format!("tts-{key}"),
            &label,
            "All files (*)",
        ) else {
            return;
        };
        let result = self.edit_settings(control, TtsInputEdit::Audio(Some(path)));
        self.as_mut().finish(result);
    }

    pub fn clear_control_audio(mut self: Pin<&mut Self>, control: i32) {
        let result = self.edit_settings(control, TtsInputEdit::Audio(None));
        self.as_mut().finish(result);
    }

    pub fn show_control_audio(mut self: Pin<&mut Self>, control: i32) {
        let result = self
            .control(control)
            .and_then(|control| match control {
                TtsEditorControl::Audio { path, .. } => path.clone(),
                _ => None,
            })
            .ok_or_else(|| "No TTS reference audio is selected".to_string())
            .and_then(|path| shrimply_qt_components::desktop_open::prepare(&path, None));
        match result {
            Ok(shrimply_qt_components::desktop_open::Action::Open(path)) => self
                .as_mut()
                .open_path(QUrl::from_local_file(&QString::from(
                    path.to_string_lossy().as_ref(),
                ))),
            Ok(shrimply_qt_components::desktop_open::Action::FocusRevealed(_)) => {}
            Err(error) => self.as_mut().error(QString::from(error)),
        }
    }

    pub fn table_column_count(&self, control: i32) -> i32 {
        self.table(control)
            .and_then(|(_, columns, _)| i32::try_from(columns.len()).ok())
            .unwrap_or_default()
    }

    pub fn table_row_count(&self, control: i32) -> i32 {
        self.table(control)
            .and_then(|(_, _, rows)| i32::try_from(rows.len()).ok())
            .unwrap_or_default()
    }

    pub fn table_column_label(&self, control: i32, column: i32) -> QString {
        self.table_column(control, column)
            .map_or_else(QString::default, |column| QString::from(&column.label))
    }

    pub fn table_column_maximum_length(&self, control: i32, column: i32) -> i32 {
        self.table_column(control, column)
            .and_then(|column| i32::try_from(column.max_length).ok())
            .unwrap_or_default()
    }

    pub fn table_value(&self, control: i32, row: i32, column: i32) -> QString {
        self.table(control)
            .and_then(|(_, columns, rows)| {
                rows.get(usize::try_from(row).ok()?)?
                    .get(&columns.get(usize::try_from(column).ok()?)?.key)
            })
            .map_or_else(QString::default, QString::from)
    }

    pub fn set_table_value(
        mut self: Pin<&mut Self>,
        control: i32,
        row: i32,
        column: i32,
        value: &QString,
    ) {
        let result = usize::try_from(row)
            .map_err(|_| "TTS table row is invalid".to_string())
            .and_then(|row| {
                usize::try_from(column)
                    .map_err(|_| "TTS table column is invalid".to_string())
                    .and_then(|column| {
                        self.edit_settings(
                            control,
                            TtsInputEdit::TableCell {
                                row,
                                column,
                                value: value.to_string(),
                            },
                        )
                    })
            });
        self.as_mut().finish(result);
    }

    pub fn add_table_row(mut self: Pin<&mut Self>, control: i32) {
        let result = self.edit_settings(control, TtsInputEdit::AddTableRow);
        self.as_mut().finish(result);
    }

    pub fn remove_table_row(mut self: Pin<&mut Self>, control: i32, row: i32) {
        let result = usize::try_from(row)
            .map_err(|_| "TTS table row is invalid".to_string())
            .and_then(|row| self.edit_settings(control, TtsInputEdit::RemoveTableRow(row)));
        self.as_mut().finish(result);
    }

    pub fn generate(mut self: Pin<&mut Self>) {
        let result = self
            .rust()
            .view
            .as_ref()
            .ok_or_else(|| "TTS editor is not ready".to_string())
            .and_then(start_generation);
        self.as_mut().finish(result);
    }

    pub fn cancel(mut self: Pin<&mut Self>) {
        let result = self
            .rust()
            .view
            .as_ref()
            .map(|view| view.audio_id)
            .ok_or_else(|| "TTS editor is not ready".to_string())
            .and_then(cancel_generation);
        self.as_mut().finish(result);
    }

    fn control(&self, control: i32) -> Option<&TtsEditorControl> {
        self.rust()
            .view
            .as_ref()?
            .controls
            .get(usize::try_from(control).ok()?)
    }

    fn number(
        &self,
        control: i32,
    ) -> Option<(
        shrimply_math_core::Fraction,
        shrimply_math_core::Fraction,
        shrimply_math_core::Fraction,
        shrimply_math_core::Fraction,
    )> {
        match self.control(control)? {
            TtsEditorControl::Number {
                value,
                minimum,
                maximum,
                step,
                ..
            } => Some((*value, *minimum, *maximum, *step)),
            _ => None,
        }
    }

    fn table(&self, control: i32) -> Option<TtsTable<'_>> {
        match self.control(control)? {
            TtsEditorControl::Table {
                key, columns, rows, ..
            } => Some((key, columns, rows)),
            _ => None,
        }
    }

    fn table_column(&self, control: i32, column: i32) -> Option<&shrimply_tts::TableColumn> {
        self.table(control)?.1.get(usize::try_from(column).ok()?)
    }

    fn edit_settings(&self, control: i32, edit: TtsInputEdit) -> Result<(), String> {
        let view = self
            .rust()
            .view
            .as_ref()
            .ok_or_else(|| "TTS editor is not ready".to_string())?;
        let control = view
            .controls
            .get(usize::try_from(control).map_err(|_| "TTS control index is invalid".to_string())?)
            .ok_or_else(|| "TTS control is unavailable".to_string())?;
        let mut settings = view.settings.clone();
        let live = shrimply_inspector_core::tts::edit_input(
            &mut settings,
            view.model
                .as_ref()
                .ok_or_else(|| "TTS model is unavailable".to_string())?,
            control_key(control),
            edit,
        )?;
        super::with_controller(|controller| {
            controller.set_tts_settings(&view.target, settings, live)
        })
    }

    fn finish(mut self: Pin<&mut Self>, result: Result<(), String>) {
        match result {
            Ok(()) => self.as_mut().refresh(),
            Err(error) => self.as_mut().error(QString::from(error)),
        }
    }
}

fn control_label(control: &TtsEditorControl) -> &str {
    match control {
        TtsEditorControl::Text { label, .. }
        | TtsEditorControl::Select { label, .. }
        | TtsEditorControl::Audio { label, .. }
        | TtsEditorControl::Toggle { label, .. }
        | TtsEditorControl::Number { label, .. }
        | TtsEditorControl::Table { label, .. } => label,
    }
}

fn control_key(control: &TtsEditorControl) -> &str {
    match control {
        TtsEditorControl::Text { key, .. }
        | TtsEditorControl::Select { key, .. }
        | TtsEditorControl::Audio { key, .. }
        | TtsEditorControl::Toggle { key, .. }
        | TtsEditorControl::Number { key, .. }
        | TtsEditorControl::Table { key, .. } => key,
    }
}
