mod audio;
mod audio_generator;
mod audio_modifiers;
mod backend;
mod caption;
mod graph_backend;
mod info;
mod item;
mod list;
mod project;
mod section;
mod selector;
mod track;
mod transition;
mod video;

use serde_json::Value;
use shrimply_cross_ui_core::editor::EditorSession;
use shrimply_inspector_core::{InspectorController, InspectorSnapshot, InspectorTarget};
use shrimply_project::project::ItemAddress;
use shrimply_state::{preferences::SharedPreferences, preview_focus::SharedPreviewFocus};
use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    sync::{Arc, mpsc},
    thread,
    time::{Duration, Instant},
};

use crate::item::{InspectorAction, ReloadKind};
use crate::list::{InspectorDocument, PreviewItem};

const VOICE_MODEL_RETRY_INTERVAL: Duration = Duration::from_secs(5);

thread_local! {
    static CONTROLLER: RefCell<Option<InspectorController>> = const { RefCell::new(None) };
    static PROPERTY_CLIPBOARD: RefCell<Option<shrimply_property_transfer::SharedClipboard>> = const { RefCell::new(None) };
    static PREFERENCES: RefCell<Option<SharedPreferences>> = const { RefCell::new(None) };
    static PREVIEW_FOCUS: RefCell<Option<SharedPreviewFocus>> = const { RefCell::new(None) };
    static PROJECT_FONTS: RefCell<Option<shrimply_inspector_core::font_cache::ProjectFontActivation>> = const { RefCell::new(None) };
    static MEDIA_METADATA: RefCell<MediaMetadata> = RefCell::new(MediaMetadata::default());
    static VOICE_MODELS: RefCell<VoiceModelCache> = RefCell::new(VoiceModelCache::default());
    static AUDIO_CACHE_STATUSES: RefCell<Vec<(uuid::Uuid, shrimply_inspector_core::AudioCacheStatus)>> = const { RefCell::new(Vec::new()) };
    static DIRTY: Cell<bool> = const { Cell::new(false) };
    static CACHE_DIRTY: Cell<bool> = const { Cell::new(false) };
    static EXPRESSION_DIRTY: Cell<bool> = const { Cell::new(false) };
    static PLAYHEAD_DIRTY: Cell<bool> = const { Cell::new(false) };
    static FOCUS_DIRTY: Cell<bool> = const { Cell::new(false) };
}

struct MediaMetadata {
    sender: mpsc::Sender<(MediaMetadataKey, Result<Arc<CachedMediaInfo>, String>)>,
    receiver: mpsc::Receiver<(MediaMetadataKey, Result<Arc<CachedMediaInfo>, String>)>,
    active: Option<MediaMetadataKey>,
    pending: Vec<MediaMetadataKey>,
    result: Option<(MediaMetadataKey, Result<Arc<CachedMediaInfo>, String>)>,
}

#[derive(Clone)]
pub(crate) enum MediaMetadataState {
    Loading,
    Ready(Arc<CachedMediaInfo>),
    Failed(String),
}

pub(crate) struct CachedMediaInfo {
    presentation: shrimply_inspector_core::info::MediaInfoPresentation,
    artwork_url: Option<String>,
    audio_stream_count: u32,
    video_stream_count: u32,
}

pub(crate) struct VoiceModels {
    pub(crate) values: Vec<String>,
    pub(crate) error: Option<String>,
    pub(crate) loading: bool,
}

pub(crate) struct AudioCacheControl {
    pub(crate) label: &'static str,
    pub(crate) progress: f64,
    pub(crate) tooltip: String,
    pub(crate) baking: bool,
}

#[derive(Clone, Eq, PartialEq)]
struct VoiceModelKey {
    server_url: String,
}

struct VoiceModelCache {
    sender: mpsc::Sender<(VoiceModelKey, Result<Vec<String>, String>)>,
    receiver: mpsc::Receiver<(VoiceModelKey, Result<Vec<String>, String>)>,
    pending: Vec<VoiceModelKey>,
    results: Vec<(VoiceModelKey, Result<Vec<String>, String>, Instant)>,
}

impl Default for VoiceModelCache {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver,
            pending: Vec::new(),
            results: Vec::new(),
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
struct MediaMetadataKey {
    path: PathBuf,
    revision: Option<u64>,
}

impl Default for MediaMetadata {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver,
            active: None,
            pending: Vec::new(),
            result: None,
        }
    }
}

pub fn init() {
    cxx_qt::init_crate!(shrimply_inspector_qt);
    cxx_qt::init_qml_module!("dev.shrimply.inspector");
}

pub fn install(session: &EditorSession) {
    CONTROLLER.with_borrow_mut(|controller| {
        assert!(controller.is_none(), "Qt inspector is already installed");
        *controller = Some(InspectorController::new(
            session.project.clone(),
            session.player_state.clone(),
            session.selection_state.clone(),
        ));
    });
    PROPERTY_CLIPBOARD.with_borrow_mut(|clipboard| {
        assert!(
            clipboard.is_none(),
            "Qt inspector clipboard is already installed"
        );
        *clipboard = Some(session.property_clipboard.clone());
    });
    PREFERENCES.with_borrow_mut(|preferences| {
        assert!(
            preferences.is_none(),
            "Qt inspector preferences are already installed"
        );
        *preferences = Some(session.preferences.clone());
    });
    PREVIEW_FOCUS.with_borrow_mut(|preview_focus| {
        assert!(
            preview_focus.is_none(),
            "Qt inspector preview focus is already installed"
        );
        *preview_focus = Some(session.preview_focus.clone());
    });
    let default_font =
        shrimply_state::preferences::snapshot(&session.preferences).default_text_font_family;
    PROJECT_FONTS.with_borrow_mut(|activation| {
        *activation = shrimply_inspector_core::font_cache::activate_project_google_fonts(
            &session.project.borrow(),
            &default_font,
        );
    });
    shrimply_timeline::selection_state::connect_named(
        &session.selection_state,
        "Qt inspector selection",
        mark_dirty,
    );
    shrimply_state::player_state::connect_named(
        &session.player_state,
        "Qt inspector project",
        |event| match event {
            shrimply_state::player_state::PlayerEvent::State(_) => PLAYHEAD_DIRTY.set(true),
            shrimply_state::player_state::PlayerEvent::Project(change) if change.inspector => {
                mark_dirty()
            }
            _ => {}
        },
    );
    shrimply_state::preview_focus::connect_named(
        &session.preview_focus,
        "Qt inspector preview focus",
        || FOCUS_DIRTY.set(true),
    );
    mark_dirty();
}

fn mark_dirty() {
    DIRTY.set(true);
}

fn target_change_pending(current: &InspectorTarget) -> bool {
    DIRTY.get()
        && CONTROLLER.with_borrow(|controller| {
            controller
                .as_ref()
                .expect("Qt inspector target requested before installation")
                .target()
                != *current
        })
}

fn take_document() -> Option<InspectorDocument> {
    audio::poll_tts_runtime();
    receive_project_fonts();
    receive_media_metadata();
    receive_voice_models();
    receive_audio_cache_statuses();
    if !DIRTY.replace(false) {
        return None;
    }
    CONTROLLER.with_borrow(|controller| {
        let snapshot = controller
            .as_ref()
            .expect("Qt inspector snapshot requested before installation")
            .snapshot();
        let document = document(snapshot);
        retain_audio_cache_statuses(&document);
        Some(document)
    })
}

fn receive_project_fonts() {
    let finished = PROJECT_FONTS.with_borrow(|activation| {
        activation
            .as_ref()
            .is_some_and(shrimply_inspector_core::font_cache::ProjectFontActivation::finished)
    });
    if !finished {
        return;
    }
    PROJECT_FONTS.with_borrow_mut(|activation| drop(activation.take()));
    CONTROLLER.with_borrow(|controller| {
        controller
            .as_ref()
            .expect("Qt inspector font activation completed before installation")
            .refresh_video();
    });
}

fn document(snapshot: InspectorSnapshot) -> InspectorDocument {
    let preview_item = match &snapshot.target {
        InspectorTarget::Item(address @ ItemAddress::Video { .. }) => Some(PreviewItem {
            address: address.clone(),
            id: snapshot
                .value
                .get("id")
                .and_then(Value::as_str)
                .expect("video inspector ID must be text")
                .parse()
                .expect("video inspector ID must be a UUID"),
        }),
        _ => None,
    };
    let metadata = media_metadata(snapshot.media.as_ref());
    let categories = match &snapshot.target {
        InspectorTarget::Project => project::categories(
            snapshot
                .project
                .as_ref()
                .expect("project inspector snapshot must include project presentation"),
        ),
        InspectorTarget::Track(_) => track::categories(
            snapshot
                .track
                .as_ref()
                .expect("track inspector snapshot must include track presentation"),
        ),
        InspectorTarget::Transition { .. } => transition::categories(
            snapshot
                .transition
                .as_ref()
                .expect("transition inspector snapshot must include transition presentation"),
        ),
        InspectorTarget::Item(address) => match address {
            ItemAddress::Caption { .. } => caption::categories(&snapshot.value, &snapshot.details),
            ItemAddress::Audio { .. } => audio::categories(
                &snapshot.value,
                &snapshot.details,
                metadata,
                audio_gain(&snapshot),
                snapshot.runtime,
                can_paste_audio_modifiers(&snapshot.target),
            ),
            ItemAddress::Video { .. } => video::categories(
                snapshot
                    .video
                    .as_ref()
                    .expect("video inspector snapshot must include video presentation"),
                &snapshot.details,
                metadata,
            ),
        },
    };
    let document = InspectorDocument {
        target: snapshot.target,
        title: snapshot.title,
        categories,
        preview_item,
    };
    validate_preview_focus(&document);
    document
}

fn validate_preview_focus(document: &InspectorDocument) {
    let focused = PREVIEW_FOCUS.with_borrow(|preview_focus| {
        preview_focus
            .as_ref()
            .and_then(shrimply_state::preview_focus::snapshot)
    });
    let valid = focused.as_ref().is_none_or(|focused| {
        document.preview_item.as_ref().is_some_and(|preview| {
            CONTROLLER.with_borrow(|controller| {
                controller
                    .as_ref()
                    .expect("Qt inspector preview validation requested before installation")
                    .valid_preview_focus(&focused.item, focused.target, &preview.address)
            })
        })
    });
    if !valid {
        PREVIEW_FOCUS.with_borrow(|preview_focus| {
            shrimply_state::preview_focus::clear(
                preview_focus
                    .as_ref()
                    .expect("Qt inspector preview focus requested before installation"),
            );
        });
    }
}

fn receive_media_metadata() {
    let changed = MEDIA_METADATA.with_borrow_mut(|metadata| {
        let results = metadata.receiver.try_iter().collect::<Vec<_>>();
        let mut changed = false;
        for (key, result) in results {
            metadata.pending.retain(|pending| pending != &key);
            if metadata.active.as_ref() == Some(&key) {
                metadata.result = Some((key, result));
                changed = true;
            }
        }
        changed
    });
    if changed {
        mark_dirty();
    }
}

fn receive_voice_models() {
    let changed = VOICE_MODELS.with_borrow_mut(|cache| {
        let results = cache.receiver.try_iter().collect::<Vec<_>>();
        for (key, result) in &results {
            cache.pending.retain(|pending| pending != key);
            if let Some((_, cached, completed)) = cache
                .results
                .iter_mut()
                .find(|(cached, _, _)| cached == key)
            {
                *cached = result.clone();
                *completed = Instant::now();
            } else {
                cache
                    .results
                    .push((key.clone(), result.clone(), Instant::now()));
            }
        }
        let mut changed = !results.is_empty();
        cache.results.retain(|(_, result, completed)| {
            let retry = result.is_err() && completed.elapsed() >= VOICE_MODEL_RETRY_INTERVAL;
            changed |= retry;
            !retry
        });
        changed
    });
    if changed {
        mark_dirty();
    }
}

pub(crate) fn voice_models(current: &str) -> VoiceModels {
    let server_url = PREFERENCES.with_borrow(|preferences| {
        shrimply_state::preferences::snapshot(
            preferences
                .as_ref()
                .expect("Qt inspector voice models requested before installation"),
        )
        .compute_server_url
    });
    let key = VoiceModelKey { server_url };
    VOICE_MODELS.with_borrow_mut(|cache| {
        if let Some(index) = cache
            .results
            .iter()
            .position(|(cached, _, _)| cached == &key)
        {
            let retry = cache.results[index].1.is_err()
                && cache.results[index].2.elapsed() >= VOICE_MODEL_RETRY_INTERVAL;
            if retry {
                drop(cache.results.remove(index));
            } else {
                return match &cache.results[index].1 {
                    Ok(values) => VoiceModels {
                        values: voice_models_with_current(values.clone(), current),
                        error: None,
                        loading: false,
                    },
                    Err(error) => VoiceModels {
                        values: vec![current.to_string()],
                        error: Some(error.clone()),
                        loading: false,
                    },
                };
            }
        }
        if cache.pending.iter().all(|pending| pending != &key) {
            cache.pending.push(key.clone());
            let sender = cache.sender.clone();
            let request = key.clone();
            thread::spawn(move || {
                let result =
                    shrimply_inspector_core::voice_change_model_catalog(&request.server_url);
                let _ = sender.send((request, result));
            });
        }
        VoiceModels {
            values: vec![current.to_string()],
            error: None,
            loading: true,
        }
    })
}

fn voice_models_with_current(mut values: Vec<String>, current: &str) -> Vec<String> {
    if !values.iter().any(|value| value == current) {
        values.insert(0, current.to_string());
    }
    values
}

pub(crate) fn audio_cache_status(id: uuid::Uuid) -> shrimply_inspector_core::AudioCacheStatus {
    let status = shrimply_inspector_core::audio_cache_status(id);
    AUDIO_CACHE_STATUSES.with_borrow_mut(|statuses| {
        if matches!(
            status,
            shrimply_inspector_core::AudioCacheStatus::Baking { .. }
        ) {
            if let Some((_, stored)) = statuses.iter_mut().find(|(stored, _)| *stored == id) {
                *stored = status.clone();
            } else {
                statuses.push((id, status.clone()));
            }
        } else {
            statuses.retain(|(stored, _)| *stored != id);
        }
    });
    status
}

pub(crate) fn audio_cache_control(id: uuid::Uuid) -> AudioCacheControl {
    cache_control(audio_cache_status(id))
}

pub(crate) fn tracked_audio_cache_control(id: uuid::Uuid) -> Option<AudioCacheControl> {
    AUDIO_CACHE_STATUSES.with_borrow(|statuses| {
        statuses
            .iter()
            .find(|(stored, _)| *stored == id)
            .map(|(_, status)| cache_control(status.clone()))
    })
}

fn cache_control(status: shrimply_inspector_core::AudioCacheStatus) -> AudioCacheControl {
    match status {
        shrimply_inspector_core::AudioCacheStatus::Missing => AudioCacheControl {
            label: "Bake",
            progress: -1.0,
            tooltip: String::new(),
            baking: false,
        },
        shrimply_inspector_core::AudioCacheStatus::Baking { completed, total } => {
            AudioCacheControl {
                label: "Baking…",
                progress: if total == 0 {
                    -1.0
                } else {
                    completed as f64 / total as f64
                },
                tooltip: "Cancel cache bake".to_string(),
                baking: true,
            }
        }
        shrimply_inspector_core::AudioCacheStatus::Ready => AudioCacheControl {
            label: "Rebake",
            progress: -1.0,
            tooltip: String::new(),
            baking: false,
        },
        shrimply_inspector_core::AudioCacheStatus::Failed(error) => AudioCacheControl {
            label: "Bake",
            progress: -1.0,
            tooltip: error,
            baking: false,
        },
    }
}

fn receive_audio_cache_statuses() {
    let (changed, terminal) = AUDIO_CACHE_STATUSES.with_borrow_mut(|statuses| {
        let mut changed = false;
        let mut terminal = false;
        statuses.retain_mut(|(id, stored)| {
            let current = shrimply_inspector_core::audio_cache_status(*id);
            if *stored != current {
                changed = true;
            }
            let baking = matches!(
                current,
                shrimply_inspector_core::AudioCacheStatus::Baking { .. }
            );
            terminal |= !baking;
            if baking {
                *stored = current;
            }
            baking
        });
        (changed, terminal)
    });
    if terminal {
        with_controller(|controller| {
            controller.refresh_audio_cache();
            Ok(())
        })
        .expect("Qt inspector cache refresh requested before installation");
    } else if changed {
        CACHE_DIRTY.set(true);
    }
}

fn retain_audio_cache_statuses(document: &InspectorDocument) {
    let ids = document
        .categories
        .iter()
        .flat_map(|category| &category.items)
        .flat_map(|item| match item {
            crate::item::InspectorListItem::Item(item) => item.section.controls.as_slice(),
            crate::item::InspectorListItem::Flat(section) => section.controls.as_slice(),
        })
        .filter(|control| control.kind == crate::section::ControlKind::AudioCache)
        .filter_map(|control| control.target_id)
        .collect::<Vec<_>>();
    AUDIO_CACHE_STATUSES.with_borrow_mut(|statuses| {
        statuses.retain(|(id, _)| ids.contains(id));
    });
}

fn take_cache_dirty() -> bool {
    CACHE_DIRTY.replace(false)
}

fn take_expression_dirty() -> bool {
    EXPRESSION_DIRTY.replace(false)
}

fn take_playhead_dirty() -> bool {
    PLAYHEAD_DIRTY.replace(false)
}

fn take_focus_dirty() -> bool {
    FOCUS_DIRTY.replace(false)
}

fn item_focused(document: &InspectorDocument, item: &crate::item::InspectorItem) -> bool {
    let Some(preview) = &document.preview_item else {
        return false;
    };
    PREVIEW_FOCUS.with_borrow(|preview_focus| {
        preview_focus
            .as_ref()
            .and_then(shrimply_state::preview_focus::snapshot)
            .is_some_and(|focused| {
                focused.item == preview.address && focused.card_key == item.presentation.key
            })
    })
}

fn focus_item(document: &InspectorDocument, item: &crate::item::InspectorItem) {
    let Some(preview) = &document.preview_item else {
        return;
    };
    PREVIEW_FOCUS.with_borrow(|preview_focus| {
        shrimply_state::preview_focus::set(
            preview_focus
                .as_ref()
                .expect("Qt inspector preview focus requested before installation"),
            shrimply_state::preview_focus::FocusedPreview {
                item: preview.address.clone(),
                card_key: item.presentation.key.clone(),
                target: item.presentation.preview_target.resolve(preview.id),
            },
        );
    });
}

fn keyframe_snapping() -> (bool, f64) {
    PREFERENCES.with_borrow(|preferences| {
        let snapshot = shrimply_state::preferences::snapshot(
            preferences
                .as_ref()
                .expect("Qt inspector keyframe preferences requested before installation"),
        );
        (
            snapshot.timeline_magnet == "true",
            f64::from(snapshot.timeline_snap_radius_px),
        )
    })
}

fn media_metadata(
    media: Option<&shrimply_inspector_core::InspectorMedia>,
) -> Option<MediaMetadataState> {
    MEDIA_METADATA.with_borrow_mut(|metadata| {
        let Some(media) = media else {
            metadata.active = None;
            metadata.result = None;
            return None;
        };
        let key = MediaMetadataKey {
            path: media.path.clone(),
            revision: media.revision,
        };
        if metadata.active.as_ref() != Some(&key) {
            metadata.active = Some(key.clone());
            metadata.result = None;
        }
        if let Some((_, result)) = metadata
            .result
            .as_ref()
            .filter(|(cached, _)| cached == &key)
        {
            return Some(match result {
                Ok(info) => MediaMetadataState::Ready(info.clone()),
                Err(error) => MediaMetadataState::Failed(error.clone()),
            });
        }
        if metadata.pending.iter().all(|pending| pending != &key) {
            metadata.pending.push(key.clone());
            let sender = metadata.sender.clone();
            thread::spawn(move || {
                let result = shrimply_media_info::inspect(&key.path, key.revision)
                    .map(info::cache_media_info);
                let _ = sender.send((key, result));
            });
        }
        Some(MediaMetadataState::Loading)
    })
}

fn audio_gain(snapshot: &InspectorSnapshot) -> f32 {
    let item: shrimply_project::project::AudioItem = serde_json::from_value(snapshot.value.clone())
        .expect("audio inspector value must be valid");
    item.gain.decibels.value_at(
        snapshot
            .runtime
            .local_time
            .unwrap_or(shrimply_project::project::Time::ZERO),
    )
}

fn set_field(target: &InspectorTarget, path: &str, value: &str) -> Result<(), String> {
    with_controller(|controller| controller.set_field(target, path, value))
}

fn set_transition_field(
    target: &InspectorTarget,
    path: &str,
    value: &str,
    commit_name: &str,
    commit_immediately: bool,
) -> Result<(), String> {
    with_controller(|controller| {
        controller.set_transition_field(target, path, value, commit_name, commit_immediately)
    })
}

fn set_transition_components(
    target: &InspectorTarget,
    path: &str,
    values: &[(usize, String)],
    commit_name: &str,
    commit_immediately: bool,
) -> Result<(), String> {
    with_controller(|controller| {
        controller.set_transition_components(target, path, values, commit_name, commit_immediately)
    })
}

fn commit_transition_field(target: &InspectorTarget, commit_name: &str) -> Result<(), String> {
    with_controller(|controller| controller.commit_transition_field(target, commit_name))
}

fn set_video_field(
    target: &InspectorTarget,
    path: &str,
    value: &str,
    commit_name: &str,
    commit_immediately: bool,
) -> Result<(), String> {
    with_controller(|controller| {
        controller.set_video_field(target, path, value, commit_name, commit_immediately)
    })
}

fn set_video_fraction(
    target: &InspectorTarget,
    path: &str,
    numerator: i64,
    denominator: i64,
    commit_name: &str,
) -> Result<(), String> {
    if denominator <= 0 {
        return Err("video fraction denominator must be positive".to_string());
    }
    with_controller(|controller| {
        controller.set_video_fraction(
            target,
            path,
            shrimply_math_core::fraction_new(numerator, denominator),
            commit_name,
        )
    })
}

fn trigger_video_control_action(
    target: &InspectorTarget,
    action: shrimply_inspector_core::InspectorControlAction,
) -> Result<(), String> {
    with_controller(|controller| controller.trigger_video_control_action(target, action))
}

fn video_stabilization_generating(target: &InspectorTarget) -> Option<bool> {
    CONTROLLER.with_borrow(|controller| {
        controller
            .as_ref()
            .expect("Qt inspector stabilization state requested before installation")
            .video_stabilization_generating(target)
    })
}

fn commit_control_edit(
    target: &InspectorTarget,
    control: &section::InspectorControl,
) -> Result<(), String> {
    if matches!(target, InspectorTarget::Transition { .. })
        && !control.commit_name.is_empty()
        && !control.commit_immediately
    {
        commit_transition_field(target, &control.commit_name)
    } else if matches!(target, InspectorTarget::Item(ItemAddress::Video { .. }))
        && !control.commit_name.is_empty()
        && !control.commit_immediately
    {
        with_controller(|controller| controller.commit_video_field(target, &control.commit_name))
    } else {
        finish_live_edit();
        Ok(())
    }
}

fn named_control_edit(
    target: &InspectorTarget,
    control: &section::InspectorControl,
    value: String,
) -> Option<Result<(), String>> {
    if control.commit_name.is_empty() {
        return None;
    }
    let value = if control.kind == section::ControlKind::Number && control.store_multiplier != 1.0 {
        value
            .parse::<f64>()
            .map(|value| (value * control.store_multiplier).to_string())
            .map_err(|_| format!("invalid numeric inspector value: {value}"))
    } else {
        Ok(value)
    };
    if matches!(target, InspectorTarget::Transition { .. }) {
        Some(value.and_then(|value| {
            set_transition_field(
                target,
                &control.path,
                &value,
                &control.commit_name,
                control.commit_immediately,
            )
        }))
    } else if matches!(target, InspectorTarget::Item(ItemAddress::Video { .. })) {
        Some(value.and_then(|value| {
            set_video_field(
                target,
                &control.path,
                &value,
                &control.commit_name,
                control.commit_immediately,
            )
        }))
    } else {
        None
    }
}

fn set_control_fraction(
    target: &InspectorTarget,
    control: &section::InspectorControl,
    numerator: i64,
    denominator: i64,
) -> Result<(), String> {
    if matches!(target, InspectorTarget::Item(ItemAddress::Video { .. }))
        && !control.commit_name.is_empty()
    {
        set_video_fraction(
            target,
            &control.path,
            numerator,
            denominator,
            &control.commit_name,
        )
    } else {
        set_fraction(target, &control.path, numerator, denominator)
    }
}

fn set_optional_field(
    target: &InspectorTarget,
    path: &str,
    value: Option<&str>,
) -> Result<(), String> {
    with_controller(|controller| controller.set_optional_field(target, path, value))
}

fn set_optional_number_field(
    target: &InspectorTarget,
    path: &str,
    value: Option<&str>,
) -> Result<(), String> {
    with_controller(|controller| controller.set_optional_number_field(target, path, value))
}

fn set_components(
    target: &InspectorTarget,
    path: &str,
    values: &[(usize, String)],
) -> Result<(), String> {
    with_controller(|controller| controller.set_components(target, path, values))
}

fn set_fraction(
    target: &InspectorTarget,
    path: &str,
    numerator: i64,
    denominator: i64,
) -> Result<(), String> {
    if denominator <= 0 {
        return Err("inspector fraction denominator must be positive".to_string());
    }
    with_controller(|controller| {
        controller.set_fraction(
            target,
            path,
            shrimply_math_core::fraction_new(numerator, denominator),
        )
    })
}

fn set_timeline_mode(
    target: &InspectorTarget,
    path: &str,
    keyframes: bool,
    enabled: bool,
    current: Value,
    default_expression: &str,
) -> Result<(), String> {
    with_controller(|controller| {
        controller.set_timeline_mode(
            target,
            path,
            keyframes,
            enabled,
            current,
            default_expression,
        )
    })
}

fn set_scalar_keyframes_enabled(
    target: &InspectorTarget,
    path: &str,
    enabled: bool,
) -> Result<(), String> {
    with_controller(|controller| controller.set_scalar_keyframes_enabled(target, path, enabled))
}

fn set_bool_value(target: &InspectorTarget, path: &str, value: bool) -> Result<(), String> {
    let result = with_controller(|controller| controller.set_bool_value(target, path, value));
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
    }
    result
}

fn set_bool_keyframes_enabled(
    target: &InspectorTarget,
    path: &str,
    enabled: bool,
) -> Result<(), String> {
    with_controller(|controller| controller.set_bool_keyframes_enabled(target, path, enabled))
}

fn set_step_keyframes_enabled(
    target: &InspectorTarget,
    path: &str,
    enabled: bool,
) -> Result<(), String> {
    with_controller(|controller| controller.set_step_keyframes_enabled(target, path, enabled))
}

fn set_expression_source(target: &InspectorTarget, path: &str, source: &str) -> Result<(), String> {
    let result =
        with_controller(|controller| controller.set_expression_source(target, path, source));
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
    }
    result
}

fn set_timeline_base(target: &InspectorTarget, path: &str, value: Value) -> Result<(), String> {
    let result = with_controller(|controller| controller.set_timeline_base(target, path, value));
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
    }
    result
}

fn set_audio_modifier_field(
    target: &InspectorTarget,
    id: uuid::Uuid,
    path: &str,
    value: &str,
) -> Result<(), String> {
    let result =
        with_controller(|controller| controller.set_audio_modifier_field(target, id, path, value));
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
    }
    result
}

fn set_audio_modifier_live_field(
    target: &InspectorTarget,
    id: uuid::Uuid,
    path: &str,
    value: &str,
) -> Result<(), String> {
    with_controller(|controller| controller.set_audio_modifier_live_field(target, id, path, value))
}

fn set_audio_modifier_timeline_base(
    target: &InspectorTarget,
    modifier_id: uuid::Uuid,
    timeline_id: uuid::Uuid,
    value: Value,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.set_audio_modifier_timeline_base(
            target,
            modifier_id,
            timeline_id,
            serde_json::from_value(value)
                .map_err(|error| format!("invalid audio modifier scalar: {error}"))?,
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
    }
    result
}

fn timeline_number_value(
    target: &InspectorTarget,
    target_id: Option<uuid::Uuid>,
    timeline_id: Option<uuid::Uuid>,
    path: &str,
) -> Result<f64, String> {
    with_controller(|controller| {
        target_id.map_or_else(
            || controller.timeline_number_value(target, path),
            |id| {
                controller.audio_modifier_number_value(
                    target,
                    id,
                    timeline_id.ok_or_else(|| {
                        "audio modifier timeline ID is no longer available".to_string()
                    })?,
                )
            },
        )
    })
}

fn scalar_expression_output(
    target: &InspectorTarget,
    path: &str,
) -> Result<shrimply_inspector_core::InspectorExpressionOutput, String> {
    with_controller(|controller| controller.scalar_expression_output(target, path))
}

fn bool_expression_output(
    target: &InspectorTarget,
    path: &str,
) -> Result<shrimply_inspector_core::InspectorExpressionOutput<bool>, String> {
    with_controller(|controller| controller.bool_expression_output(target, path))
}

fn step_expression_output(
    target: &InspectorTarget,
    path: &str,
) -> Result<shrimply_inspector_core::InspectorExpressionOutput<String>, String> {
    with_controller(|controller| controller.step_expression_output(target, path))
}

fn move_scalar_keyframe(
    target: &InspectorTarget,
    path: &str,
    change: shrimply_inspector_core::AudioModifierKeyframeMove,
) -> Result<(), String> {
    let result =
        with_controller(|controller| controller.move_scalar_keyframe(target, path, change));
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
    }
    result
}

fn delete_scalar_keyframe(
    target: &InspectorTarget,
    path: &str,
    time: shrimply_project::project::Time,
) -> Result<(), String> {
    let result =
        with_controller(|controller| controller.delete_scalar_keyframe(target, path, time));
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
    }
    result
}

fn add_scalar_keyframe(
    target: &InspectorTarget,
    path: &str,
    time: shrimply_project::project::Time,
) -> Result<(), String> {
    let result = with_controller(|controller| controller.add_scalar_keyframe(target, path, time));
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
    }
    result
}

fn set_scalar_keyframe_interpolation(
    target: &InspectorTarget,
    path: &str,
    owner_id: uuid::Uuid,
    interpolation: usize,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.set_scalar_keyframe_interpolation(target, path, owner_id, interpolation)
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
    }
    result
}

fn copy_scalar_keyframes(
    target: &InspectorTarget,
    path: &str,
    times: &[shrimply_project::project::Time],
) -> Result<usize, String> {
    with_controller(|controller| controller.copy_scalar_keyframes(target, path, times))
}

fn paste_scalar_keyframes(
    target: &InspectorTarget,
    path: &str,
    time: shrimply_project::project::Time,
) -> Result<usize, String> {
    let result =
        with_controller(|controller| controller.paste_scalar_keyframes(target, path, time));
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
    }
    result
}

fn seek_scalar_keyframe(
    target: &InspectorTarget,
    time: shrimply_project::project::Time,
) -> Result<(), String> {
    with_controller(|controller| controller.seek_scalar_keyframe(target, time))
}

fn move_bool_keyframes(
    target: &InspectorTarget,
    path: &str,
    moves: &[(
        shrimply_project::project::Time,
        shrimply_project::project::Time,
    )],
) -> Result<Vec<shrimply_project::project::Time>, String> {
    with_controller(|controller| controller.move_bool_keyframes(target, path, moves))
}

fn delete_bool_keyframe(
    target: &InspectorTarget,
    path: &str,
    time: shrimply_project::project::Time,
) -> Result<(), String> {
    with_controller(|controller| controller.delete_bool_keyframe(target, path, time))
}

fn add_bool_keyframe(
    target: &InspectorTarget,
    path: &str,
    time: shrimply_project::project::Time,
) -> Result<(), String> {
    with_controller(|controller| controller.add_bool_keyframe(target, path, time))
}

fn copy_bool_keyframes(
    target: &InspectorTarget,
    path: &str,
    times: &[shrimply_project::project::Time],
) -> Result<usize, String> {
    with_controller(|controller| controller.copy_bool_keyframes(target, path, times))
}

fn paste_bool_keyframes(
    target: &InspectorTarget,
    path: &str,
    time: shrimply_project::project::Time,
) -> Result<usize, String> {
    with_controller(|controller| controller.paste_bool_keyframes(target, path, time))
}

fn seek_discrete_keyframe(
    target: &InspectorTarget,
    time: shrimply_project::project::Time,
) -> Result<(), String> {
    with_controller(|controller| controller.seek_discrete_keyframe(target, time))
}

fn move_step_keyframes(
    target: &InspectorTarget,
    path: &str,
    moves: &[(
        shrimply_project::project::Time,
        shrimply_project::project::Time,
    )],
) -> Result<Vec<shrimply_project::project::Time>, String> {
    with_controller(|controller| controller.move_step_keyframes(target, path, moves))
}

fn delete_step_keyframe(
    target: &InspectorTarget,
    path: &str,
    time: shrimply_project::project::Time,
) -> Result<(), String> {
    with_controller(|controller| controller.delete_step_keyframe(target, path, time))
}

fn add_step_keyframe(
    target: &InspectorTarget,
    path: &str,
    time: shrimply_project::project::Time,
) -> Result<(), String> {
    with_controller(|controller| controller.add_step_keyframe(target, path, time))
}

fn copy_step_keyframes(
    target: &InspectorTarget,
    path: &str,
    times: &[shrimply_project::project::Time],
) -> Result<usize, String> {
    with_controller(|controller| controller.copy_step_keyframes(target, path, times))
}

fn paste_step_keyframes(
    target: &InspectorTarget,
    path: &str,
    time: shrimply_project::project::Time,
) -> Result<usize, String> {
    with_controller(|controller| controller.paste_step_keyframes(target, path, time))
}

fn current_keyframe_time(
    target: &InspectorTarget,
) -> Result<shrimply_project::project::Time, String> {
    with_controller(|controller| controller.current_keyframe_time(target))
}

fn audio_modifier_expression_output(
    target: &InspectorTarget,
    modifier_id: uuid::Uuid,
    timeline_id: uuid::Uuid,
) -> Result<shrimply_inspector_core::InspectorExpressionOutput, String> {
    with_controller(|controller| {
        controller.audio_modifier_expression_output(target, modifier_id, timeline_id)
    })
}

fn move_audio_modifier_keyframe(
    target: &InspectorTarget,
    modifier_id: uuid::Uuid,
    timeline_id: uuid::Uuid,
    change: shrimply_inspector_core::AudioModifierKeyframeMove,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.move_audio_modifier_keyframe(target, modifier_id, timeline_id, change)
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
    }
    result
}

fn delete_audio_modifier_keyframe(
    target: &InspectorTarget,
    modifier_id: uuid::Uuid,
    timeline_id: uuid::Uuid,
    time: shrimply_project::project::Time,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.delete_audio_modifier_keyframe(target, modifier_id, timeline_id, time)
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
    }
    result
}

fn add_audio_modifier_keyframe(
    target: &InspectorTarget,
    modifier_id: uuid::Uuid,
    timeline_id: uuid::Uuid,
    time: shrimply_project::project::Time,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.add_audio_modifier_keyframe(target, modifier_id, timeline_id, time)
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
    }
    result
}

fn set_audio_modifier_keyframe_interpolation(
    target: &InspectorTarget,
    modifier_id: uuid::Uuid,
    timeline_id: uuid::Uuid,
    owner_id: uuid::Uuid,
    interpolation: usize,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.set_audio_modifier_keyframe_interpolation(
            target,
            modifier_id,
            timeline_id,
            owner_id,
            interpolation,
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
    }
    result
}

fn seek_audio_modifier_keyframe(
    target: &InspectorTarget,
    time: shrimply_project::project::Time,
) -> Result<(), String> {
    with_controller(|controller| controller.seek_audio_modifier_keyframe(target, time))
}

fn toggle_keyframe_playback() {
    with_controller(|controller| {
        controller.toggle_keyframe_playback();
        Ok(())
    })
    .expect("Qt inspector controller must be installed before graph playback");
}

fn copy_audio_modifier_keyframes(
    target: &InspectorTarget,
    modifier_id: uuid::Uuid,
    timeline_id: uuid::Uuid,
    times: &[shrimply_project::project::Time],
) -> Result<usize, String> {
    with_controller(|controller| {
        controller.copy_audio_modifier_keyframes(target, modifier_id, timeline_id, times)
    })
}

fn paste_audio_modifier_keyframes(
    target: &InspectorTarget,
    modifier_id: uuid::Uuid,
    timeline_id: uuid::Uuid,
    time: shrimply_project::project::Time,
) -> Result<usize, String> {
    let result = with_controller(|controller| {
        controller.paste_audio_modifier_keyframes(target, modifier_id, timeline_id, time)
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
    }
    result
}

fn set_audio_modifier_timeline_mode(
    target: &InspectorTarget,
    modifier_id: uuid::Uuid,
    timeline_id: uuid::Uuid,
    path: &str,
    change: shrimply_inspector_core::TimelineModeChange<'_>,
) -> Result<(), String> {
    with_controller(|controller| {
        controller.set_audio_modifier_timeline_mode(target, modifier_id, timeline_id, path, change)
    })
}

fn set_audio_modifier_expression_source(
    target: &InspectorTarget,
    id: uuid::Uuid,
    path: &str,
    source: &str,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.set_audio_modifier_expression_source(target, id, path, source)
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
    }
    result
}

fn apply_project_settings(
    width: i32,
    height: i32,
    fps_numerator: &str,
    fps_denominator: &str,
) -> Result<(), String> {
    let dimensions = shrimply_project::project::CanvasSize {
        width: u32::try_from(width).map_err(|_| "project width must be positive".to_string())?,
        height: u32::try_from(height).map_err(|_| "project height must be positive".to_string())?,
    };
    let numerator = fps_numerator
        .parse::<i64>()
        .map_err(|_| format!("invalid project frame-rate numerator: {fps_numerator}"))?;
    let denominator = fps_denominator
        .parse::<i64>()
        .map_err(|_| format!("invalid project frame-rate denominator: {fps_denominator}"))?;
    if numerator <= 0 || denominator <= 0 {
        return Err("project frame rate must be positive".to_string());
    }
    let frame_rate = shrimply_math_core::fraction_new(numerator, denominator);
    with_controller(|controller| controller.apply_project_settings(dimensions, frame_rate))
}

fn can_paste_audio_modifiers(target: &InspectorTarget) -> bool {
    CONTROLLER.with_borrow(|controller| {
        PROPERTY_CLIPBOARD.with_borrow(|clipboard| {
            controller
                .as_ref()
                .expect("Qt inspector paste check requested before installation")
                .can_paste_audio_modifiers(
                    target,
                    clipboard
                        .as_ref()
                        .expect("Qt inspector paste check requested before installation"),
                )
        })
    })
}

fn add_audio_modifier(target: &InspectorTarget, kind: &str) -> Result<(), String> {
    with_controller(|controller| controller.add_audio_modifier(target, kind))
}

fn paste_audio_modifiers(target: &InspectorTarget) -> Result<usize, String> {
    with_controller(|controller| {
        PROPERTY_CLIPBOARD.with_borrow(|clipboard| {
            controller.paste_audio_modifiers(
                target,
                clipboard
                    .as_ref()
                    .expect("Qt inspector paste requested before installation"),
            )
        })
    })
}

fn set_audio_cache_preset(
    target: &InspectorTarget,
    id: uuid::Uuid,
    preset: &str,
) -> Result<(), String> {
    with_controller(|controller| controller.set_audio_cache_preset(target, id, preset))
}

fn toggle_audio_cache(target: &InspectorTarget, id: uuid::Uuid) -> Result<(), String> {
    with_controller(|controller| controller.toggle_audio_cache(target, id))
}

fn perform_action(
    target: &InspectorTarget,
    action: InspectorAction,
) -> Result<Option<String>, String> {
    let mut confirmation = None;
    let result = match action {
        InspectorAction::Reset { path, value } => {
            with_controller(|controller| controller.set_value(target, &path, value))
        }
        InspectorAction::ResetFields { values } => {
            with_controller(|controller| controller.set_values(target, &values))
        }
        InspectorAction::ResetVideo { reset } => {
            with_controller(|controller| controller.reset_video(target, &reset))
        }
        InspectorAction::SetBoolean { path, value } => {
            with_controller(|controller| controller.set_value(target, &path, Value::Bool(value)))
        }
        InspectorAction::SetOptional { path, value } => with_controller(|controller| {
            controller.set_value(target, &path, value.unwrap_or(Value::Null))
        }),
        InspectorAction::CopyArrayItem { path, index } => with_controller(|controller| {
            PROPERTY_CLIPBOARD.with_borrow(|clipboard| {
                controller.copy_array_item(
                    target,
                    &path,
                    index,
                    clipboard
                        .as_ref()
                        .expect("Qt inspector copy requested before installation"),
                )
            })
        }),
        InspectorAction::MoveArrayItem {
            path,
            index,
            offset,
        } => with_controller(|controller| controller.move_array_item(target, &path, index, offset)),
        InspectorAction::RemoveArrayItem { path, index } => {
            with_controller(|controller| controller.remove_array_item(target, &path, index))
        }
        InspectorAction::ResetAudioModifier { id, effect } => {
            with_controller(|controller| controller.reset_audio_modifier(target, id, effect))
        }
        InspectorAction::SetAudioModifierEnabled { id, enabled } => {
            with_controller(|controller| controller.set_audio_modifier_enabled(target, id, enabled))
        }
        InspectorAction::CopyAudioModifier { id } => with_controller(|controller| {
            PROPERTY_CLIPBOARD.with_borrow(|clipboard| {
                controller
                    .copy_audio_modifier(
                        target,
                        id,
                        clipboard
                            .as_ref()
                            .expect("Qt inspector copy requested before installation"),
                    )
                    .map(|name| confirmation = Some(format!("{name} copied")))
            })
        }),
        InspectorAction::MoveAudioModifier { id, offset } => {
            with_controller(|controller| controller.move_audio_modifier(target, id, offset))
        }
        InspectorAction::RemoveAudioModifier { id } => {
            with_controller(|controller| controller.remove_audio_modifier(target, id))
        }
        InspectorAction::ToggleAudioCache { id } => toggle_audio_cache(target, id),
        InspectorAction::ReloadAsset { asset, kind } => match kind {
            ReloadKind::Blender => video::reload_blender(&asset),
            ReloadKind::Manim => video::reload_manim(&asset),
        },
    };
    result.map(|()| confirmation)
}

fn finish_live_edit() {
    with_controller(|controller| {
        controller.finish_live_edit();
        Ok(())
    })
    .expect("Qt inspector live edit finished before installation");
}

fn with_controller<T>(
    operation: impl FnOnce(&InspectorController) -> Result<T, String>,
) -> Result<T, String> {
    CONTROLLER.with_borrow(|controller| {
        operation(
            controller
                .as_ref()
                .expect("Qt inspector edit requested before installation"),
        )
    })
}
