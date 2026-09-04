mod audio;
mod audio_generator;
mod audio_modifiers;
mod backend;
mod caption;
mod graph_backend;
mod info;
mod item;
mod list;
mod modifiers;
mod project;
mod section;
mod selector;
mod track;
mod transition;
mod value_backend;
mod video;

use serde_json::Value;
use shrimply_cross_ui_core::editor::EditorSession;
use shrimply_inspector_core::{
    ControlKind, InspectorController, InspectorSnapshot, InspectorTarget,
};
use shrimply_project::project::{ItemAddress, Time};
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
    static FONT_BROWSER: RefCell<shrimply_inspector_core::font_selector::Browser> = RefCell::new(shrimply_inspector_core::font_selector::Browser::default());
    static PENDING_FONT_EDIT: RefCell<Option<PendingFontEdit>> = const { RefCell::new(None) };
    static MEDIA_METADATA: RefCell<MediaMetadata> = RefCell::new(MediaMetadata::default());
    static VOICE_MODELS: RefCell<VoiceModelCache> = RefCell::new(VoiceModelCache::default());
    static CAMERA_MODELS: RefCell<CameraModelRequests> = RefCell::new(CameraModelRequests::default());
    static CACHE_STATUSES: RefCell<shrimply_inspector_core::CacheStatusTracker<CacheKind>> = RefCell::new(shrimply_inspector_core::CacheStatusTracker::default());
    static DIRTY: Cell<bool> = const { Cell::new(false) };
    static CACHE_DIRTY: Cell<bool> = const { Cell::new(false) };
    static EXPRESSION_DIRTY: Cell<bool> = const { Cell::new(false) };
    static GRAPH_DIRTY: Cell<bool> = const { Cell::new(false) };
    static PLAYHEAD_DIRTY: Cell<bool> = const { Cell::new(false) };
    static TRANSFORM_DIRTY: Cell<bool> = const { Cell::new(false) };
    static FOCUS_DIRTY: Cell<bool> = const { Cell::new(false) };
}

#[derive(Clone)]
struct PendingFontEdit {
    target: InspectorTarget,
    modifier_id: Option<uuid::Uuid>,
    path: String,
    commit_name: String,
    source_value: String,
    value: String,
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

#[derive(Clone, Copy, Eq, PartialEq)]
enum CacheKind {
    Audio,
    Visual,
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

struct CameraModelRequests {
    sender: mpsc::Sender<String>,
    receiver: mpsc::Receiver<String>,
    pending: Vec<String>,
}

impl Default for CameraModelRequests {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver,
            pending: Vec::new(),
        }
    }
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
    let default_font =
        shrimply_state::preferences::snapshot(&session.preferences).default_text_font_family;
    CONTROLLER.with_borrow_mut(|controller| {
        assert!(controller.is_none(), "Qt inspector is already installed");
        *controller = Some(
            InspectorController::new(
                session.project.clone(),
                session.player_state.clone(),
                session.selection_state.clone(),
            )
            .with_default_text_font(default_font.clone()),
        );
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
            shrimply_state::player_state::PlayerEvent::Project(change)
                if change.video && change.live_preview =>
            {
                TRANSFORM_DIRTY.set(true)
            }
            shrimply_state::player_state::PlayerEvent::Project(change) if change.audio => {
                TRANSFORM_DIRTY.set(true);
                EXPRESSION_DIRTY.set(true);
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
    CONTROLLER.with_borrow(|controller| {
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
    receive_camera_models();
    receive_cache_statuses();
    if !DIRTY.replace(false) {
        return None;
    }
    CONTROLLER.with_borrow(|controller| {
        let controller = controller
            .as_ref()
            .expect("Qt inspector snapshot requested before installation");
        controller.retain_analysis_transitions();
        let (server_url, camera_models) = PREFERENCES.with_borrow(|preferences| {
            let server_url = shrimply_state::preferences::snapshot(
                preferences
                    .as_ref()
                    .expect("Qt inspector preferences requested before installation"),
            )
            .compute_server_url;
            let models =
                shrimply_inspector_core::camera_source::cached_tracking_models(&server_url);
            (server_url, models)
        });
        let snapshot = controller.snapshot_with_camera_models(camera_models.as_ref());
        if camera_models.is_none()
            && snapshot.video.as_ref().is_some_and(|video| {
                video.visual.iter().any(|card| {
                    card.section.controls.iter().any(|control| {
                        control.path == shrimply_inspector_core::camera_source::SOURCE_PATH
                    })
                })
            })
        {
            request_camera_models(&server_url);
        }
        let document = document(snapshot);
        retain_cache_statuses(&document);
        Some(document)
    })
}

fn receive_camera_models() {
    CAMERA_MODELS.with_borrow_mut(|requests| {
        while let Ok(server_url) = requests.receiver.try_recv() {
            requests.pending.retain(|pending| pending != &server_url);
            mark_dirty();
        }
    });
}

fn request_camera_models(server_url: &str) {
    if shrimply_inspector_core::camera_source::cached_tracking_models(server_url).is_some() {
        return;
    }
    CAMERA_MODELS.with_borrow_mut(|requests| {
        if requests.pending.iter().any(|pending| pending == server_url) {
            return;
        }
        requests.pending.push(server_url.to_string());
        let sender = requests.sender.clone();
        let request_url = server_url.to_string();
        thread::spawn(move || {
            let _ = shrimply_inspector_core::camera_source::tracking_models(&request_url);
            let _ = sender.send(request_url);
        });
    });
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

fn open_font_browser() -> bool {
    FONT_BROWSER.with_borrow_mut(|browser| browser.open())
}

fn search_font_browser(query: String) -> bool {
    FONT_BROWSER.with_borrow_mut(|browser| browser.search(query))
}

fn request_font_browser_previews(range: std::ops::Range<usize>) {
    FONT_BROWSER.with_borrow_mut(|browser| browser.request_previews(range));
}

fn font_browser_count() -> usize {
    FONT_BROWSER.with_borrow(|browser| browser.visible().len())
}

fn font_browser_choice(index: usize) -> Option<shrimply_inspector_core::font_cache::FontFamily> {
    FONT_BROWSER.with_borrow(|browser| browser.visible().get(index).cloned())
}

fn find_font_browser_choice(
    name: &str,
    source: shrimply_inspector_core::font_cache::FontSource,
) -> Option<shrimply_inspector_core::font_cache::FontFamily> {
    FONT_BROWSER.with_borrow(|browser| {
        browser
            .visible()
            .iter()
            .find(|family| family.source == source && family.name.eq_ignore_ascii_case(name))
            .cloned()
    })
}

fn font_browser_lookup() -> Option<shrimply_inspector_core::font_cache::GoogleFamily> {
    FONT_BROWSER.with_borrow(|browser| browser.lookup().cloned())
}

fn font_browser_status() -> String {
    FONT_BROWSER.with_borrow(|browser| browser.status().to_string())
}

fn font_browser_busy() -> bool {
    FONT_BROWSER.with_borrow(shrimply_inspector_core::font_selector::Browser::busy)
}

fn activate_font_browser_family(
    family: shrimply_inspector_core::font_cache::FontFamily,
    edit: PendingFontEdit,
) -> Result<(), String> {
    if PENDING_FONT_EDIT.with_borrow(Option::is_some) {
        return Err("A font is already being activated".to_string());
    }
    FONT_BROWSER.with_borrow_mut(|browser| browser.activate(family))?;
    PENDING_FONT_EDIT.with_borrow_mut(|pending| *pending = Some(edit));
    Ok(())
}

fn ensure_font_control(
    controller: &InspectorController,
    target: &InspectorTarget,
    path: &str,
    modifier_id: Option<uuid::Uuid>,
) -> Result<(), String> {
    let available = if let Some(modifier_id) = modifier_id {
        controller
            .text_3d_presentation(target, modifier_id)?
            .controls
            .iter()
            .any(|control| {
                control.kind == ControlKind::FontFamilies
                    && control.target_id == Some(modifier_id)
                    && control.path == path
            })
    } else {
        let snapshot = controller.snapshot();
        (&snapshot.target == target)
            && snapshot.video.is_some_and(|video| {
                video.visual.iter().any(|card| {
                    card.section.controls.iter().any(|control| {
                        control.kind == ControlKind::FontFamilies
                            && control.target_id.is_none()
                            && control.path == path
                    })
                })
            })
    };
    available
        .then_some(())
        .ok_or_else(|| "text font control is no longer available".to_string())
}

fn apply_font_browser_edit(edit: &PendingFontEdit) -> Result<(), String> {
    with_controller(|controller| {
        if controller.target() != edit.target {
            return Ok(());
        }
        ensure_font_control(controller, &edit.target, &edit.path, edit.modifier_id)?;
        let unchanged = if let Some(modifier_id) = edit.modifier_id {
            controller
                .text_3d_presentation(&edit.target, modifier_id)?
                .controls
                .iter()
                .any(|control| control.path == edit.path && control.value == edit.source_value)
        } else {
            controller.snapshot().video.is_some_and(|video| {
                video.visual.iter().any(|card| {
                    card.section.controls.iter().any(|control| {
                        control.path == edit.path && control.value == edit.source_value
                    })
                })
            })
        };
        if !unchanged {
            return Ok(());
        }
        controller.set_video_field(
            &edit.target,
            &edit.path,
            &edit.value,
            &edit.commit_name,
            true,
        )
    })
}

fn receive_font_browser() -> (bool, Vec<(PendingFontEdit, Result<(), String>)>) {
    let poll = FONT_BROWSER.with_borrow_mut(|browser| browser.poll());
    let edits = poll
        .activations
        .into_iter()
        .filter_map(|result| {
            PENDING_FONT_EDIT
                .with_borrow_mut(Option::take)
                .map(|edit| (edit, result.map(drop)))
        })
        .collect();
    (poll.changed, edits)
}

fn cancel_font_browser_edit() {
    PENDING_FONT_EDIT.with_borrow_mut(|pending| drop(pending.take()));
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
                can_paste_visual_modifiers(&snapshot.target),
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

fn cache_status(kind: CacheKind, id: uuid::Uuid) -> shrimply_inspector_core::CacheStatus {
    let status = current_cache_status(kind, id);
    CACHE_STATUSES.with_borrow_mut(|statuses| statuses.observe(kind, id, status))
}

fn current_cache_status(kind: CacheKind, id: uuid::Uuid) -> shrimply_inspector_core::CacheStatus {
    match kind {
        CacheKind::Audio => shrimply_inspector_core::audio_cache_status(id),
        CacheKind::Visual => shrimply_inspector_core::visual_cache_status(id),
    }
}

fn cache_kind(control: crate::section::ControlKind) -> Option<CacheKind> {
    match control {
        crate::section::ControlKind::AudioCache | crate::section::ControlKind::AudioCachePreset => {
            Some(CacheKind::Audio)
        }
        crate::section::ControlKind::VisualCache
        | crate::section::ControlKind::VisualCacheQuality => Some(CacheKind::Visual),
        _ => None,
    }
}

pub(crate) fn tracked_cache_control(
    control: crate::section::ControlKind,
    id: uuid::Uuid,
) -> Option<shrimply_inspector_core::CacheControlPresentation> {
    let kind = cache_kind(control)?;
    let tracked = CACHE_STATUSES.with_borrow(|statuses| statuses.tracked(kind, id).cloned());
    let status = tracked.unwrap_or_else(|| cache_status(kind, id));
    Some(match kind {
        CacheKind::Audio => shrimply_inspector_core::audio_cache_control(status),
        CacheKind::Visual => shrimply_inspector_core::cache_control_presentation(status, ""),
    })
}

fn receive_cache_statuses() {
    let poll = CACHE_STATUSES.with_borrow_mut(|statuses| statuses.poll(current_cache_status));
    let audio_terminal = poll.finished.contains(&CacheKind::Audio);
    let visual_terminal = poll.finished.contains(&CacheKind::Visual);
    if audio_terminal || visual_terminal {
        with_controller(|controller| {
            if audio_terminal {
                controller.refresh_audio_cache();
            }
            if visual_terminal {
                controller.refresh_visual_cache();
            }
            Ok(())
        })
        .expect("Qt inspector cache refresh requested before installation");
    } else if poll.changed {
        CACHE_DIRTY.set(true);
    }
}

fn retain_cache_statuses(document: &InspectorDocument) {
    let mut ids = document
        .categories
        .iter()
        .flat_map(|category| &category.items)
        .flat_map(|item| match item {
            crate::item::InspectorListItem::Item(item) => item.section.controls.as_slice(),
            crate::item::InspectorListItem::Flat(section) => section.controls.as_slice(),
        })
        .filter_map(|control| Some((cache_kind(control.kind)?, control.target_id?)))
        .fold(Vec::new(), |mut ids, entry| {
            if !ids.contains(&entry) {
                ids.push(entry);
            }
            ids
        });
    CACHE_STATUSES.with_borrow_mut(|statuses| {
        statuses.retain(|kind, id| ids.contains(&(kind, id)));
        for (kind, id) in ids.drain(..) {
            statuses.observe(kind, id, current_cache_status(kind, id));
        }
    });
}

fn take_cache_dirty() -> bool {
    CACHE_DIRTY.replace(false)
}

fn take_expression_dirty() -> bool {
    EXPRESSION_DIRTY.replace(false)
}

fn take_graph_dirty() -> bool {
    GRAPH_DIRTY.replace(false)
}

fn take_playhead_dirty() -> bool {
    PLAYHEAD_DIRTY.replace(false)
}

fn take_transform_dirty() -> bool {
    TRANSFORM_DIRTY.replace(false)
}

fn mark_transform_dirty() {
    TRANSFORM_DIRTY.set(true);
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

fn focus_control(
    document: &InspectorDocument,
    item: &crate::item::InspectorItem,
    control: &section::InspectorControl,
) {
    let Some(focus) = &control.preview_focus else {
        focus_item(document, item);
        return;
    };
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
                card_key: focus.card_key.clone(),
                target: focus.target.resolve(preview.id),
            },
        );
    });
}

fn focus_alpha_mask(
    target: &InspectorTarget,
    mask_target: shrimply_project::project::VisualAlphaMaskTarget,
    enabled: bool,
) {
    let InspectorTarget::Item(address @ shrimply_project::project::ItemAddress::Video { .. }) =
        target
    else {
        return;
    };
    let item_id = CONTROLLER.with_borrow(|controller| {
        controller
            .as_ref()
            .expect("Qt inspector alpha-mask focus requested before installation")
            .snapshot()
            .video
            .expect("video inspector snapshot must include video presentation")
            .item_id
    });
    let focus = shrimply_inspector_core::alpha_mask::preview_focus(item_id, mask_target, enabled);
    PREVIEW_FOCUS.with_borrow(|preview_focus| {
        shrimply_state::preview_focus::set(
            preview_focus
                .as_ref()
                .expect("Qt inspector preview focus requested before installation"),
            shrimply_state::preview_focus::FocusedPreview {
                item: address.clone(),
                card_key: focus.card_key,
                target: focus.target.resolve(item_id),
            },
        );
    });
}

fn keyframe_snapping() -> (bool, f64) {
    PREFERENCES.with_borrow(|preferences| {
        shrimply_inspector_core::keyframe_model::graph_snapping(
            preferences
                .as_ref()
                .expect("Qt inspector keyframe preferences requested before installation"),
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

fn select_object_3d_model(target: &InspectorTarget, modifier_id: uuid::Uuid) -> Result<(), String> {
    let Some(path) = shrimply_qt_components::file_picker::open(
        "select-3d-object-model",
        "Select 3D model",
        "3D models (*.obj *.glb)",
    ) else {
        return Ok(());
    };
    with_controller(|controller| controller.set_object_3d_model(target, modifier_id, &path))
}

fn select_scene_3d_environment(target: &InspectorTarget) -> Result<(), String> {
    let Some(path) = shrimply_qt_components::file_picker::open(
        "select-scene-3d-environment",
        "Select environment image",
        "Environment images (*.png *.jpg *.jpeg *.webp *.avif *.hdr *.exr)",
    ) else {
        return Ok(());
    };
    with_controller(|controller| controller.set_scene_3d_environment(target, &path))
}

fn select_paint_texture(target: &InspectorTarget, color_id: uuid::Uuid) -> Result<(), String> {
    let Some(path) = shrimply_qt_components::file_picker::open(
        "select-paint-texture",
        "Select paint texture",
        "Images (*.png *.jpg *.jpeg *.webp *.gif *.bmp *.tif *.tiff)",
    ) else {
        return Ok(());
    };
    with_controller(|controller| controller.set_paint_texture(target, color_id, &path))
}

fn paint_drawing_expression_output(
    target: &InspectorTarget,
    timeline_id: uuid::Uuid,
) -> Result<shrimply_inspector_core::InspectorExpressionOutput<String>, String> {
    with_controller(|controller| controller.paint_drawing_expression_output(target, timeline_id))
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
    if crate::graph_backend::background_integer(control) {
        let timeline_id = control
            .timeline_id
            .ok_or_else(|| "background integer timeline ID is unavailable".to_string())?;
        let path = control.timeline_path.as_deref().unwrap_or(&control.path);
        commit_background_integer_value(target, path, timeline_id)
    } else if matches!(
        control.kind,
        section::ControlKind::LayeredNumber
            | section::ControlKind::LayeredSelector
            | section::ControlKind::LayeredVector2
            | section::ControlKind::LayeredVector3
            | section::ControlKind::LayeredText
            | section::ControlKind::LayeredDrawing
    ) {
        with_controller(|controller| controller.finish_live_inspector_edit(target))
    } else if matches!(target, InspectorTarget::Transition { .. })
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
    if control.commit_name.is_empty()
        || matches!(
            control.kind,
            section::ControlKind::LayeredNumber
                | section::ControlKind::LayeredBoolean
                | section::ControlKind::LayeredSelector
        )
    {
        return None;
    }
    if let Some(result) = alpha_mask_control_edit(target, control, &value) {
        return Some(result);
    }
    let value = if control.kind == section::ControlKind::Number
        && (control.store_multiplier != 1.0
            || control.number_mapping != shrimply_inspector_core::NumberMapping::Linear)
    {
        value
            .parse::<f64>()
            .map(|value| control.store_number(value).to_string())
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

fn alpha_mask_control_edit(
    target: &InspectorTarget,
    control: &section::InspectorControl,
    value: &str,
) -> Option<Result<(), String>> {
    let field = control.path.rsplit_once("/alpha_mask/")?.1;
    let mask_target = if control.path.starts_with("/compositing/alpha_mask/") {
        shrimply_project::project::VisualAlphaMaskTarget::Compositing
    } else if control.path.starts_with("/modifiers/") {
        let Some(id) = control.target_id else {
            return Some(Err("alpha-mask modifier target is unavailable".to_string()));
        };
        shrimply_project::project::VisualAlphaMaskTarget::Modifier(id)
    } else {
        return None;
    };
    match field {
        "shape" => {
            let Some(shape) = shrimply_inspector_core::alpha_mask::SHAPE_CHOICES
                .iter()
                .find_map(|(shape, key, _)| (*key == value).then_some(*shape))
            else {
                return Some(Err(format!("invalid alpha-mask shape: {value}")));
            };
            Some(with_controller(|controller| {
                controller.set_alpha_mask_shape(target, mask_target, shape)
            }))
        }
        "invert" => Some(
            value
                .parse::<bool>()
                .map_err(|_| format!("invalid alpha-mask inversion: {value}"))
                .and_then(|invert| {
                    with_controller(|controller| {
                        controller.set_alpha_mask_inverted(target, mask_target, invert)
                    })
                }),
        ),
        _ => None,
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

fn ensure_control_timeline(
    target: &InspectorTarget,
    control: &section::InspectorControl,
) -> Result<(), String> {
    let path = control.timeline_path.as_deref().unwrap_or(&control.path);
    let layered = matches!(
        control.kind,
        section::ControlKind::LayeredNumber
            | section::ControlKind::LayeredSelector
            | section::ControlKind::LayeredVector2
            | section::ControlKind::LayeredVector3
            | section::ControlKind::LayeredColor
            | section::ControlKind::LayeredText
            | section::ControlKind::LayeredDrawing
    );
    if path.starts_with("/content/generator/") {
        if path == "/content/generator/kind" && !layered {
            return Ok(());
        }
        return control
            .timeline_id
            .ok_or_else(|| "background timeline ID is unavailable".to_string())
            .and_then(|timeline_id| {
                with_controller(|controller| controller.ensure_timeline(target, path, timeline_id))
            });
    }
    if path.contains("/alpha_mask/") && layered {
        if let Some(modifier_id) = control.target_id {
            with_controller(|controller| {
                controller.ensure_visual_modifier(target, path, modifier_id)
            })?;
        }
        return control
            .timeline_id
            .ok_or_else(|| "alpha-mask timeline ID is unavailable".to_string())
            .and_then(|timeline_id| {
                with_controller(|controller| controller.ensure_timeline(target, path, timeline_id))
            });
    }
    if !path.starts_with("/modifiers/") {
        if shrimply_inspector_core::paint::is_timeline_path(path) && layered {
            return control
                .timeline_id
                .ok_or_else(|| "paint timeline ID is unavailable".to_string())
                .and_then(|timeline_id| {
                    with_controller(|controller| {
                        controller.ensure_timeline(target, path, timeline_id)
                    })
                });
        }
        if shrimply_inspector_core::generated::is_timeline_control(control) {
            return control
                .timeline_id
                .ok_or_else(|| "generated timeline ID is unavailable".to_string())
                .and_then(|timeline_id| {
                    with_controller(|controller| {
                        controller.ensure_timeline(target, path, timeline_id)
                    })
                });
        }
        if (path.starts_with("/content/model/") || path.starts_with("/content/camera/")) && layered
        {
            return control
                .timeline_id
                .ok_or_else(|| "Gaussian timeline ID is unavailable".to_string())
                .and_then(|timeline_id| {
                    with_controller(|controller| {
                        controller.ensure_timeline(target, path, timeline_id)
                    })
                });
        }
        if shrimply_inspector_core::scene_3d::is_timeline_path(path) && layered {
            return control
                .timeline_id
                .ok_or_else(|| "scene 3D timeline ID is unavailable".to_string())
                .and_then(|timeline_id| {
                    with_controller(|controller| {
                        controller.ensure_timeline(target, path, timeline_id)
                    })
                });
        }
        if path.starts_with("/transform/")
            && matches!(control.kind, section::ControlKind::LayeredVector2)
        {
            return control
                .timeline_id
                .ok_or_else(|| "transform timeline ID is unavailable".to_string())
                .and_then(|timeline_id| {
                    with_controller(|controller| {
                        controller.ensure_timeline(target, path, timeline_id)
                    })
                });
        }
        return Ok(());
    }
    if let Some(modifier_id) = control.target_id {
        with_controller(|controller| controller.ensure_visual_modifier(target, path, modifier_id))?;
    }
    if !layered {
        return Ok(());
    }
    let timeline_id = control
        .timeline_id
        .ok_or_else(|| "visual modifier timeline ID is unavailable".to_string())?;
    with_controller(|controller| match control.kind {
        section::ControlKind::LayeredNumber => {
            controller.ensure_visual_modifier_number(target, path, timeline_id)
        }
        section::ControlKind::LayeredSelector => {
            controller.ensure_timeline(target, path, timeline_id)
        }
        section::ControlKind::LayeredColor => controller.ensure_timeline(target, path, timeline_id),
        section::ControlKind::LayeredText => {
            controller.ensure_visual_modifier_text(target, path, timeline_id)
        }
        section::ControlKind::LayeredVector2 => {
            controller.ensure_visual_modifier_vector2(target, path, timeline_id)
        }
        section::ControlKind::LayeredVector3 => {
            controller.ensure_visual_modifier_vector3(target, path, timeline_id)
        }
        _ => Ok(()),
    })
}

fn set_vector2_value(
    target: &InspectorTarget,
    path: &str,
    first: f64,
    second: f64,
    commit_name: &str,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.set_vector2_value(
            target,
            path,
            first,
            second,
            shrimply_inspector_core::InspectorCommit::Coalesced(commit_name),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
}

fn set_vector3_value(
    target: &InspectorTarget,
    path: &str,
    first: f64,
    second: f64,
    third: f64,
    commit_name: &str,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.set_vector3_value(
            target,
            path,
            first,
            second,
            third,
            shrimply_inspector_core::InspectorCommit::Coalesced(commit_name),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
}

fn control_graph(
    target: &InspectorTarget,
    control: &shrimply_inspector_core::InspectorControl,
) -> Result<Option<shrimply_inspector_core::ScalarGraph>, String> {
    with_controller(|controller| controller.control_graph(target, control))
}

fn control_graph_source(
    target: &InspectorTarget,
) -> Result<(serde_json::Value, shrimply_inspector_core::InspectorRuntime), String> {
    with_controller(|controller| controller.control_graph_source(target))
}

fn set_paint_drawing_keyframes_enabled(
    target: &InspectorTarget,
    timeline_id: uuid::Uuid,
    enabled: bool,
) -> Result<(), String> {
    with_controller(|controller| {
        controller.set_paint_drawing_keyframes_enabled(target, timeline_id, enabled)
    })
}

fn move_paint_drawing_keyframes(
    target: &InspectorTarget,
    timeline_id: uuid::Uuid,
    moves: &[(Time, Time)],
) -> Result<Vec<Time>, String> {
    with_controller(|controller| {
        controller.move_paint_drawing_keyframes(target, timeline_id, moves)
    })
}

fn delete_paint_drawing_keyframe(
    target: &InspectorTarget,
    timeline_id: uuid::Uuid,
    time: Time,
) -> Result<(), String> {
    with_controller(|controller| {
        controller.delete_paint_drawing_keyframe(target, timeline_id, time)
    })
}

fn add_paint_drawing_keyframe(
    target: &InspectorTarget,
    timeline_id: uuid::Uuid,
    time: Time,
) -> Result<(), String> {
    with_controller(|controller| controller.add_paint_drawing_keyframe(target, timeline_id, time))
}

fn copy_paint_drawing_keyframes(
    target: &InspectorTarget,
    timeline_id: uuid::Uuid,
    times: &[Time],
) -> Result<usize, String> {
    with_controller(|controller| {
        controller.copy_paint_drawing_keyframes(target, timeline_id, times)
    })
}

fn paste_paint_drawing_keyframes(
    target: &InspectorTarget,
    timeline_id: uuid::Uuid,
    time: Time,
) -> Result<usize, String> {
    with_controller(|controller| {
        controller.paste_paint_drawing_keyframes(target, timeline_id, time)
    })
}

fn set_paint_drawing_interpolation(
    target: &InspectorTarget,
    timeline_id: uuid::Uuid,
    owner_id: uuid::Uuid,
    interpolation: usize,
) -> Result<(), String> {
    with_controller(|controller| {
        controller.set_paint_drawing_interpolation(target, timeline_id, owner_id, interpolation)
    })
}

fn transform_live_presentation(
    target: &InspectorTarget,
) -> Option<shrimply_inspector_core::transform::TransformLivePresentation> {
    CONTROLLER.with_borrow(|controller| {
        controller
            .as_ref()
            .expect("Qt inspector transform requested before installation")
            .transform_live_presentation(target)
    })
}

fn resolved_transform(
    target: &InspectorTarget,
) -> Option<shrimply_project::project::ResolvedTransform> {
    CONTROLLER.with_borrow(|controller| {
        controller
            .as_ref()
            .expect("Qt inspector transform requested before installation")
            .resolved_transform(target)
    })
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
    commit_name: &str,
) -> Result<(), String> {
    with_controller(|controller| {
        controller.set_timeline_mode_with_commit(
            target,
            path,
            shrimply_inspector_core::TimelineModeChange {
                keyframes,
                enabled,
                current,
                default_expression,
            },
            shrimply_inspector_core::InspectorCommit::Immediate(commit_name),
        )
    })
}

fn set_scalar_keyframes_enabled(
    target: &InspectorTarget,
    path: &str,
    enabled: bool,
    constraint: shrimply_inspector_core::NumberConstraint,
    commit_name: &str,
) -> Result<(), String> {
    with_controller(|controller| {
        controller.set_scalar_keyframes_enabled(
            target,
            path,
            enabled,
            constraint,
            shrimply_inspector_core::InspectorCommit::Immediate(commit_name),
        )
    })
}

fn set_background_integer_keyframes_enabled(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
    enabled: bool,
) -> Result<(), String> {
    with_controller(|controller| {
        controller.set_background_integer_keyframes_enabled(target, path, timeline_id, enabled)
    })
}

fn set_background_integer_value(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
    value: u32,
) -> Result<(), String> {
    with_controller(|controller| {
        controller.set_background_integer_value(target, path, timeline_id, value)
    })
}

fn commit_background_integer_value(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
) -> Result<(), String> {
    with_controller(|controller| {
        controller.commit_background_integer_value(target, path, timeline_id)
    })
}

fn set_background_integer_expression_enabled(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
    enabled: bool,
) -> Result<(), String> {
    with_controller(|controller| {
        controller.set_background_integer_expression_enabled(target, path, timeline_id, enabled)
    })
}

fn set_background_integer_expression_source(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
    source: String,
) -> Result<(), String> {
    with_controller(|controller| {
        controller.set_background_integer_expression_source(target, path, timeline_id, source)
    })
}

fn set_vector2_keyframes_enabled(
    target: &InspectorTarget,
    path: &str,
    enabled: bool,
    commit_name: &str,
) -> Result<(), String> {
    with_controller(|controller| {
        controller.set_vector2_keyframes_enabled(
            target,
            path,
            enabled,
            shrimply_inspector_core::InspectorCommit::Immediate(commit_name),
        )
    })
}

fn set_vector2_expression_enabled(
    target: &InspectorTarget,
    path: &str,
    enabled: bool,
    commit_name: &str,
) -> Result<(), String> {
    with_controller(|controller| {
        controller.set_vector2_expression_enabled(
            target,
            path,
            enabled,
            shrimply_inspector_core::InspectorCommit::Immediate(commit_name),
        )
    })
}

fn set_transform_expression_enabled(
    target: &InspectorTarget,
    field: shrimply_inspector_core::transform::TransformField,
    timeline_id: uuid::Uuid,
    enabled: bool,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.set_transform_expression_enabled(
            target,
            field,
            timeline_id,
            enabled,
            shrimply_inspector_core::InspectorCommit::Immediate(
                shrimply_inspector_core::transform::expressions::TOGGLE_COMMIT,
            ),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
    }
    result
}

fn set_vector3_keyframes_enabled(
    target: &InspectorTarget,
    path: &str,
    enabled: bool,
    commit_name: &str,
) -> Result<(), String> {
    with_controller(|controller| {
        controller.set_vector3_keyframes_enabled(
            target,
            path,
            enabled,
            shrimply_inspector_core::InspectorCommit::Immediate(commit_name),
        )
    })
}

fn vector2_expression_output(
    target: &InspectorTarget,
    path: &str,
    timeline_id: Option<uuid::Uuid>,
) -> Result<shrimply_inspector_core::InspectorExpressionOutput<glam::Vec2>, String> {
    with_controller(|controller| controller.vector2_expression_output(target, path, timeline_id))
}

fn transform_vec2_expression_output(
    target: &InspectorTarget,
    field: shrimply_inspector_core::transform::Vec2Field,
    timeline_id: uuid::Uuid,
) -> Result<Option<shrimply_inspector_core::InspectorExpressionOutput<glam::Vec2>>, String> {
    with_controller(|controller| {
        controller.transform_vec2_expression_output(target, field, timeline_id)
    })
}

fn vector3_expression_output(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
) -> Result<shrimply_inspector_core::InspectorExpressionOutput<glam::Vec3>, String> {
    with_controller(|controller| controller.vector3_expression_output(target, path, timeline_id))
}

fn set_color_value(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
    value: shrimply_core::Color<u8>,
    commit_name: &str,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.set_color_value(
            target,
            path,
            timeline_id,
            value,
            shrimply_inspector_core::InspectorCommit::Coalesced(commit_name),
        )
    });
    if result.is_ok() {
        mark_dirty();
    }
    result
}

fn set_color_keyframes_enabled(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
    enabled: bool,
    commit_name: &str,
) -> Result<(), String> {
    with_controller(|controller| {
        controller.set_color_keyframes_enabled(
            target,
            path,
            timeline_id,
            enabled,
            shrimply_inspector_core::InspectorCommit::Immediate(commit_name),
        )
    })
}

fn color_expression_output(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
) -> Result<shrimply_inspector_core::InspectorExpressionOutput<shrimply_core::Color<u8>>, String> {
    with_controller(|controller| controller.color_expression_output(target, path, timeline_id))
}

fn color_value(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
) -> Result<shrimply_core::Color<u8>, String> {
    with_controller(|controller| controller.color_value(target, path, timeline_id))
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
    commit_name: &str,
) -> Result<(), String> {
    with_controller(|controller| {
        controller.set_bool_keyframes_enabled(
            target,
            path,
            enabled,
            shrimply_inspector_core::InspectorCommit::Immediate(commit_name),
        )
    })
}

fn set_text_value(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
    value: String,
    commit_name: &str,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.set_text_value(
            target,
            path,
            timeline_id,
            value,
            shrimply_inspector_core::InspectorCommit::Coalesced(commit_name),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
}

fn set_text_keyframes_enabled(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
    enabled: bool,
    commit_name: &str,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.set_text_keyframes_enabled(
            target,
            path,
            timeline_id,
            enabled,
            shrimply_inspector_core::InspectorCommit::Immediate(commit_name),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
}

fn set_step_keyframes_enabled(
    target: &InspectorTarget,
    path: &str,
    enabled: bool,
    commit_name: &str,
) -> Result<(), String> {
    with_controller(|controller| {
        controller.set_step_keyframes_enabled_with_commit(
            target,
            path,
            enabled,
            shrimply_inspector_core::InspectorCommit::Immediate(commit_name),
        )
    })
}

fn set_expression_source(
    target: &InspectorTarget,
    path: &str,
    source: &str,
    commit_name: &str,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.set_expression_source_with_commit(
            target,
            path,
            source,
            shrimply_inspector_core::InspectorCommit::Coalesced(commit_name),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
    }
    result
}

fn set_vector2_expression_source(
    target: &InspectorTarget,
    path: &str,
    source: String,
    commit_name: &str,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.set_vector2_expression_source(
            target,
            path,
            source,
            shrimply_inspector_core::InspectorCommit::Coalesced(commit_name),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
    }
    result
}

fn set_transform_expression_source(
    target: &InspectorTarget,
    field: shrimply_inspector_core::transform::TransformField,
    timeline_id: uuid::Uuid,
    source: String,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.set_transform_expression_source(
            target,
            field,
            timeline_id,
            source,
            shrimply_inspector_core::InspectorCommit::Coalesced(
                shrimply_inspector_core::transform::expressions::SOURCE_COMMIT,
            ),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
    }
    result
}

fn set_text_expression_source(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
    source: &str,
    commit_name: &str,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.set_text_expression_source(
            target,
            path,
            timeline_id,
            source.to_string(),
            shrimply_inspector_core::InspectorCommit::Coalesced(commit_name),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
    }
    result
}

fn set_text_expression_enabled(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
    enabled: bool,
    commit_name: &str,
) -> Result<(), String> {
    with_controller(|controller| {
        controller.set_text_expression_enabled(
            target,
            path,
            timeline_id,
            enabled,
            shrimply_inspector_core::InspectorCommit::Immediate(commit_name),
        )
    })
}

fn set_timeline_base(
    target: &InspectorTarget,
    path: &str,
    value: Value,
    commit_name: &str,
    commit_immediately: bool,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        let commit = if commit_immediately {
            shrimply_inspector_core::InspectorCommit::Immediate(commit_name)
        } else {
            shrimply_inspector_core::InspectorCommit::Coalesced(commit_name)
        };
        controller.set_timeline_base_with_commit(target, path, value, commit)
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
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
    audio_modifier: bool,
    target_id: Option<uuid::Uuid>,
    timeline_id: Option<uuid::Uuid>,
    path: &str,
) -> Result<f64, String> {
    with_controller(|controller| {
        if audio_modifier {
            controller.audio_modifier_number_value(
                target,
                target_id
                    .ok_or_else(|| "audio modifier target is no longer available".to_string())?,
                timeline_id.ok_or_else(|| {
                    "audio modifier timeline ID is no longer available".to_string()
                })?,
            )
        } else {
            controller.timeline_number_value(target, path, timeline_id)
        }
    })
}

fn timeline_vector2_value(
    target: &InspectorTarget,
    timeline_id: Option<uuid::Uuid>,
    path: &str,
) -> Result<glam::Vec2, String> {
    with_controller(|controller| controller.timeline_vector2_value(target, path, timeline_id))
}

fn timeline_vector3_value(
    target: &InspectorTarget,
    timeline_id: uuid::Uuid,
    path: &str,
) -> Result<glam::Vec3, String> {
    with_controller(|controller| controller.timeline_vector3_value(target, path, timeline_id))
}

fn scalar_expression_output(
    target: &InspectorTarget,
    path: &str,
    timeline_id: Option<uuid::Uuid>,
) -> Result<shrimply_inspector_core::InspectorExpressionOutput, String> {
    with_controller(|controller| controller.scalar_expression_output(target, path, timeline_id))
}

fn transform_scalar_expression_output(
    target: &InspectorTarget,
    field: shrimply_inspector_core::transform::ScalarField,
    timeline_id: uuid::Uuid,
) -> Result<Option<shrimply_inspector_core::InspectorExpressionOutput>, String> {
    with_controller(|controller| {
        controller.transform_scalar_expression_output(target, field, timeline_id)
    })
}

fn background_integer_expression_output(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
) -> Result<shrimply_inspector_core::InspectorExpressionOutput<u32>, String> {
    with_controller(|controller| {
        controller.background_integer_expression_output(target, path, timeline_id)
    })
}

fn bool_expression_output(
    target: &InspectorTarget,
    path: &str,
) -> Result<shrimply_inspector_core::InspectorExpressionOutput<bool>, String> {
    with_controller(|controller| controller.bool_expression_output(target, path))
}

fn text_expression_output(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
) -> Result<shrimply_inspector_core::InspectorExpressionOutput<String>, String> {
    with_controller(|controller| controller.text_expression_output(target, path, timeline_id))
}

fn step_expression_output(
    target: &InspectorTarget,
    path: &str,
    timeline_id: Option<uuid::Uuid>,
) -> Result<shrimply_inspector_core::InspectorExpressionOutput<String>, String> {
    with_controller(|controller| controller.step_expression_output(target, path, timeline_id))
}

fn move_scalar_keyframe(
    target: &InspectorTarget,
    path: &str,
    change: shrimply_inspector_core::AudioModifierKeyframeMove,
    constraint: shrimply_inspector_core::NumberConstraint,
    commit_name: &str,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.move_scalar_keyframe(
            target,
            path,
            change,
            constraint,
            shrimply_inspector_core::InspectorCommit::Coalesced(commit_name),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
}

fn move_background_integer_keyframe(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
    change: shrimply_inspector_core::AudioModifierKeyframeMove,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.move_background_integer_keyframe(target, path, timeline_id, change)
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
}

fn delete_scalar_keyframe(
    target: &InspectorTarget,
    path: &str,
    time: shrimply_project::project::Time,
    commit_name: &str,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.delete_scalar_keyframe(
            target,
            path,
            time,
            shrimply_inspector_core::InspectorCommit::Immediate(commit_name),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
}

fn delete_background_integer_keyframe(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
    time: shrimply_project::project::Time,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.delete_background_integer_keyframe(target, path, timeline_id, time)
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
}

fn add_scalar_keyframe(
    target: &InspectorTarget,
    path: &str,
    time: shrimply_project::project::Time,
    constraint: shrimply_inspector_core::NumberConstraint,
    commit_name: &str,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.add_scalar_keyframe(
            target,
            path,
            time,
            constraint,
            shrimply_inspector_core::InspectorCommit::Immediate(commit_name),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
}

fn add_background_integer_keyframe(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
    time: shrimply_project::project::Time,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.add_background_integer_keyframe(target, path, timeline_id, time)
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
}

fn set_scalar_keyframe_interpolation(
    target: &InspectorTarget,
    path: &str,
    owner_id: uuid::Uuid,
    interpolation: usize,
    commit_name: &str,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.set_scalar_keyframe_interpolation(
            target,
            path,
            owner_id,
            interpolation,
            shrimply_inspector_core::InspectorCommit::Immediate(commit_name),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
}

fn set_background_integer_interpolation(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
    owner_id: uuid::Uuid,
    interpolation: usize,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.set_background_integer_interpolation(
            target,
            path,
            timeline_id,
            owner_id,
            interpolation,
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
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

fn copy_background_integer_keyframes(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
    times: &[shrimply_project::project::Time],
) -> Result<usize, String> {
    with_controller(|controller| {
        controller.copy_background_integer_keyframes(target, path, timeline_id, times)
    })
}

fn paste_scalar_keyframes(
    target: &InspectorTarget,
    path: &str,
    time: shrimply_project::project::Time,
    constraint: shrimply_inspector_core::NumberConstraint,
    commit_name: &str,
) -> Result<usize, String> {
    let result = with_controller(|controller| {
        controller.paste_scalar_keyframes(
            target,
            path,
            time,
            constraint,
            shrimply_inspector_core::InspectorCommit::Immediate(commit_name),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
}

fn paste_background_integer_keyframes(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
    time: shrimply_project::project::Time,
) -> Result<usize, String> {
    let result = with_controller(|controller| {
        controller.paste_background_integer_keyframes(target, path, timeline_id, time)
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
}

fn seek_scalar_keyframe(
    target: &InspectorTarget,
    time: shrimply_project::project::Time,
) -> Result<(), String> {
    with_controller(|controller| controller.seek_scalar_keyframe(target, time))
}

fn move_vector2_keyframes(
    target: &InspectorTarget,
    path: &str,
    moves: &[(
        shrimply_project::project::Time,
        shrimply_project::project::Time,
    )],
    commit_name: &str,
) -> Result<Vec<shrimply_project::project::Time>, String> {
    let result = with_controller(|controller| {
        controller.move_vector2_keyframes(
            target,
            path,
            moves,
            shrimply_inspector_core::InspectorCommit::Coalesced(commit_name),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
}

fn delete_vector2_keyframe(
    target: &InspectorTarget,
    path: &str,
    time: shrimply_project::project::Time,
    commit_name: &str,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.delete_vector2_keyframe(
            target,
            path,
            time,
            shrimply_inspector_core::InspectorCommit::Immediate(commit_name),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
}

fn delete_transform_scalar_keyframe(
    target: &InspectorTarget,
    path: &str,
    time: shrimply_project::project::Time,
    commit_name: &str,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.delete_transform_scalar_keyframe(
            target,
            path,
            time,
            shrimply_inspector_core::InspectorCommit::Immediate(commit_name),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
}

fn add_vector2_keyframe(
    target: &InspectorTarget,
    path: &str,
    time: shrimply_project::project::Time,
    commit_name: &str,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.add_vector2_keyframe(
            target,
            path,
            time,
            shrimply_inspector_core::InspectorCommit::Immediate(commit_name),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
}

fn copy_vector2_keyframes(
    target: &InspectorTarget,
    path: &str,
    times: &[shrimply_project::project::Time],
) -> Result<usize, String> {
    with_controller(|controller| controller.copy_vector2_keyframes(target, path, times))
}

fn paste_vector2_keyframes(
    target: &InspectorTarget,
    path: &str,
    time: shrimply_project::project::Time,
    commit_name: &str,
) -> Result<usize, String> {
    let result = with_controller(|controller| {
        controller.paste_vector2_keyframes(
            target,
            path,
            time,
            shrimply_inspector_core::InspectorCommit::Immediate(commit_name),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
}

fn set_vector2_interpolation(
    target: &InspectorTarget,
    path: &str,
    owner_id: uuid::Uuid,
    interpolation: usize,
    commit_name: &str,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.set_vector2_interpolation(
            target,
            path,
            owner_id,
            interpolation,
            shrimply_inspector_core::InspectorCommit::Immediate(commit_name),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
}

fn move_vector3_keyframes(
    target: &InspectorTarget,
    path: &str,
    moves: &[(
        shrimply_project::project::Time,
        shrimply_project::project::Time,
    )],
    commit_name: &str,
) -> Result<Vec<shrimply_project::project::Time>, String> {
    let result = with_controller(|controller| {
        controller.move_vector3_keyframes(
            target,
            path,
            moves,
            shrimply_inspector_core::InspectorCommit::Coalesced(commit_name),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
}

fn delete_vector3_keyframe(
    target: &InspectorTarget,
    path: &str,
    time: shrimply_project::project::Time,
    commit_name: &str,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.delete_vector3_keyframe(
            target,
            path,
            time,
            shrimply_inspector_core::InspectorCommit::Immediate(commit_name),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
}

fn add_vector3_keyframe(
    target: &InspectorTarget,
    path: &str,
    time: shrimply_project::project::Time,
    commit_name: &str,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.add_vector3_keyframe(
            target,
            path,
            time,
            shrimply_inspector_core::InspectorCommit::Immediate(commit_name),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
}

fn copy_vector3_keyframes(
    target: &InspectorTarget,
    path: &str,
    times: &[shrimply_project::project::Time],
) -> Result<usize, String> {
    with_controller(|controller| controller.copy_vector3_keyframes(target, path, times))
}

fn paste_vector3_keyframes(
    target: &InspectorTarget,
    path: &str,
    time: shrimply_project::project::Time,
    commit_name: &str,
) -> Result<usize, String> {
    let result = with_controller(|controller| {
        controller.paste_vector3_keyframes(
            target,
            path,
            time,
            shrimply_inspector_core::InspectorCommit::Immediate(commit_name),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
}

fn set_vector3_interpolation(
    target: &InspectorTarget,
    path: &str,
    owner_id: uuid::Uuid,
    interpolation: usize,
    commit_name: &str,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.set_vector3_interpolation(
            target,
            path,
            owner_id,
            interpolation,
            shrimply_inspector_core::InspectorCommit::Immediate(commit_name),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
}

fn move_color_keyframes(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
    moves: &[(
        shrimply_project::project::Time,
        shrimply_project::project::Time,
    )],
    commit_name: &str,
) -> Result<Vec<shrimply_project::project::Time>, String> {
    let result = with_controller(|controller| {
        controller.move_color_keyframes(
            target,
            path,
            timeline_id,
            moves,
            shrimply_inspector_core::InspectorCommit::Coalesced(commit_name),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
}

fn delete_color_keyframe(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
    time: shrimply_project::project::Time,
    commit_name: &str,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.delete_color_keyframe(
            target,
            path,
            timeline_id,
            time,
            shrimply_inspector_core::InspectorCommit::Immediate(commit_name),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
}

fn add_color_keyframe(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
    time: shrimply_project::project::Time,
    commit_name: &str,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.add_color_keyframe(
            target,
            path,
            timeline_id,
            time,
            shrimply_inspector_core::InspectorCommit::Immediate(commit_name),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
}

fn copy_color_keyframes(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
    times: &[shrimply_project::project::Time],
) -> Result<usize, String> {
    with_controller(|controller| controller.copy_color_keyframes(target, path, timeline_id, times))
}

fn paste_color_keyframes(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
    time: shrimply_project::project::Time,
    commit_name: &str,
) -> Result<usize, String> {
    let result = with_controller(|controller| {
        controller.paste_color_keyframes(
            target,
            path,
            timeline_id,
            time,
            shrimply_inspector_core::InspectorCommit::Immediate(commit_name),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
}

fn set_color_interpolation(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
    owner_id: uuid::Uuid,
    interpolation: usize,
    commit_name: &str,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.set_color_interpolation(
            target,
            path,
            timeline_id,
            owner_id,
            interpolation,
            shrimply_inspector_core::InspectorCommit::Immediate(commit_name),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
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

fn move_text_keyframes(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
    moves: &[(
        shrimply_project::project::Time,
        shrimply_project::project::Time,
    )],
    commits: shrimply_inspector_core::TextKeyframeCommits,
) -> Result<Vec<shrimply_project::project::Time>, String> {
    let result = with_controller(|controller| {
        controller.move_text_keyframes(target, path, timeline_id, moves, commits)
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
}

fn delete_text_keyframe(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
    time: shrimply_project::project::Time,
    commits: shrimply_inspector_core::TextKeyframeCommits,
) -> Result<(), String> {
    with_controller(|controller| {
        controller.delete_text_keyframe(target, path, timeline_id, time, commits)
    })
}

fn add_text_keyframe(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
    time: shrimply_project::project::Time,
    commits: shrimply_inspector_core::TextKeyframeCommits,
) -> Result<(), String> {
    with_controller(|controller| {
        controller.add_text_keyframe(target, path, timeline_id, time, commits)
    })
}

fn copy_text_keyframes(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
    times: &[shrimply_project::project::Time],
) -> Result<usize, String> {
    with_controller(|controller| controller.copy_text_keyframes(target, path, timeline_id, times))
}

fn paste_text_keyframes(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
    time: shrimply_project::project::Time,
    commits: shrimply_inspector_core::TextKeyframeCommits,
) -> Result<usize, String> {
    with_controller(|controller| {
        controller.paste_text_keyframes(target, path, timeline_id, time, commits)
    })
}

fn set_text_keyframe_interpolation(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
    owner_id: uuid::Uuid,
    interpolation: usize,
    commit_name: &str,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.set_text_keyframe_interpolation(
            target,
            path,
            timeline_id,
            owner_id,
            interpolation,
            commit_name,
        )
    });
    if result.is_ok() {
        GRAPH_DIRTY.set(true);
    }
    result
}

fn text_keyframe_text_interpolation(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
    owner_id: uuid::Uuid,
) -> Result<usize, String> {
    with_controller(|controller| {
        controller.text_keyframe_text_interpolation(target, path, timeline_id, owner_id)
    })
}

fn set_text_keyframe_text_interpolation(
    target: &InspectorTarget,
    path: &str,
    timeline_id: uuid::Uuid,
    owner_id: uuid::Uuid,
    interpolation: usize,
    commit_name: &str,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.set_text_keyframe_text_interpolation(
            target,
            path,
            timeline_id,
            owner_id,
            interpolation,
            commit_name,
        )
    });
    if result.is_ok() {
        GRAPH_DIRTY.set(true);
    }
    result
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
    commit_name: &str,
) -> Result<Vec<shrimply_project::project::Time>, String> {
    let result = with_controller(|controller| {
        controller.move_step_keyframes(
            target,
            path,
            moves,
            shrimply_inspector_core::InspectorCommit::Coalesced(commit_name),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
}

fn delete_step_keyframe(
    target: &InspectorTarget,
    path: &str,
    time: shrimply_project::project::Time,
    commit_name: &str,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.delete_step_keyframe(
            target,
            path,
            time,
            shrimply_inspector_core::InspectorCommit::Immediate(commit_name),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
}

fn add_step_keyframe(
    target: &InspectorTarget,
    path: &str,
    time: shrimply_project::project::Time,
    commit_name: &str,
) -> Result<(), String> {
    let result = with_controller(|controller| {
        controller.add_step_keyframe(
            target,
            path,
            time,
            shrimply_inspector_core::InspectorCommit::Immediate(commit_name),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
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
    commit_name: &str,
) -> Result<usize, String> {
    let result = with_controller(|controller| {
        controller.paste_step_keyframes(
            target,
            path,
            time,
            shrimply_inspector_core::InspectorCommit::Immediate(commit_name),
        )
    });
    if result.is_ok() {
        EXPRESSION_DIRTY.set(true);
        GRAPH_DIRTY.set(true);
    }
    result
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

fn can_paste_visual_modifiers(target: &InspectorTarget) -> bool {
    CONTROLLER.with_borrow(|controller| {
        PROPERTY_CLIPBOARD.with_borrow(|clipboard| {
            controller
                .as_ref()
                .expect("Qt inspector paste check requested before installation")
                .can_paste_visual_modifiers(
                    target,
                    clipboard
                        .as_ref()
                        .expect("Qt inspector paste check requested before installation"),
                )
        })
    })
}

fn add_visual_modifier(target: &InspectorTarget, kind: &str) -> Result<uuid::Uuid, String> {
    with_controller(|controller| controller.add_visual_modifier(target, kind))
}

fn paste_visual_modifiers(target: &InspectorTarget) -> Result<usize, String> {
    with_controller(|controller| {
        PROPERTY_CLIPBOARD.with_borrow(|clipboard| {
            controller.paste_visual_modifiers(
                target,
                clipboard
                    .as_ref()
                    .expect("Qt inspector paste requested before installation"),
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

fn set_visual_cache_quality(
    target: &InspectorTarget,
    id: uuid::Uuid,
    quality: &str,
) -> Result<(), String> {
    with_controller(|controller| controller.set_visual_cache_quality(target, id, quality))
}

fn toggle_visual_cache(target: &InspectorTarget, id: uuid::Uuid) -> Result<(), String> {
    with_controller(|controller| controller.toggle_visual_cache(target, id))
}

fn toggle_sam2_analysis(target: &InspectorTarget, id: uuid::Uuid) -> Result<(), String> {
    let server_url = PREFERENCES.with_borrow(|preferences| {
        shrimply_state::preferences::snapshot(
            preferences
                .as_ref()
                .expect("Qt inspector preferences requested before installation"),
        )
        .compute_server_url
    });
    with_controller(|controller| controller.toggle_sam2_analysis(target, id, server_url))
}

fn transparent_fill_analysis_control(
    target: &InspectorTarget,
    id: uuid::Uuid,
) -> Result<shrimply_inspector_core::AnalysisControlPresentation, String> {
    with_controller(|controller| controller.transparent_fill_analysis_control(target, id))
}

fn camera_analysis_control(
    target: &InspectorTarget,
) -> Result<shrimply_inspector_core::AnalysisControlPresentation, String> {
    let server_url = PREFERENCES.with_borrow(|preferences| {
        shrimply_state::preferences::snapshot(
            preferences
                .as_ref()
                .expect("Qt camera inspector used before preferences were installed"),
        )
        .compute_server_url
    });
    request_camera_models(&server_url);
    with_controller(|controller| controller.camera_analysis_control(target, &server_url))
}

fn toggle_camera_analysis(target: &InspectorTarget) -> Result<(), String> {
    let server_url = PREFERENCES.with_borrow(|preferences| {
        shrimply_state::preferences::snapshot(
            preferences
                .as_ref()
                .expect("Qt camera inspector used before preferences were installed"),
        )
        .compute_server_url
    });
    with_controller(|controller| controller.toggle_camera_analysis(target, server_url))
}

fn toggle_transparent_fill_analysis(
    target: &InspectorTarget,
    id: uuid::Uuid,
) -> Result<(), String> {
    with_controller(|controller| controller.toggle_transparent_fill_analysis(target, id))
}

fn set_sam2_point_label(
    target: &InspectorTarget,
    modifier_id: uuid::Uuid,
    point_id: uuid::Uuid,
    label: &str,
) -> Result<(), String> {
    let label = serde_json::from_value(serde_json::Value::String(label.to_string()))
        .map_err(|_| format!("unknown SAM2 point type: {label}"))?;
    with_controller(|controller| {
        controller.set_sam2_point_label(target, modifier_id, point_id, label)
    })
}

fn set_sam2_model(
    target: &InspectorTarget,
    modifier_id: uuid::Uuid,
    model: &str,
) -> Result<(), String> {
    let model = serde_json::from_value(serde_json::Value::String(model.to_string()))
        .map_err(|_| format!("unknown SAM2 model: {model}"))?;
    with_controller(|controller| controller.set_sam2_model(target, modifier_id, model))
}

fn set_sam2_point_position(
    target: &InspectorTarget,
    modifier_id: uuid::Uuid,
    point_id: uuid::Uuid,
    first: f64,
    second: f64,
) -> Result<(), String> {
    with_controller(|controller| {
        controller.set_sam2_point_position(target, modifier_id, point_id, first, second)
    })
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
        InspectorAction::ResetVisualModifier { id, effect } => {
            with_controller(|controller| controller.reset_visual_modifier(target, id, effect))
        }
        InspectorAction::SetVisualModifierEnabled { id, enabled } => {
            with_controller(|controller| {
                controller.set_visual_modifier_enabled(target, id, enabled)
            })
        }
        InspectorAction::CopyVisualModifier { id } => with_controller(|controller| {
            PROPERTY_CLIPBOARD.with_borrow(|clipboard| {
                controller
                    .copy_visual_modifier(
                        target,
                        id,
                        clipboard
                            .as_ref()
                            .expect("Qt inspector copy requested before installation"),
                    )
                    .map(|name| confirmation = Some(format!("{name} copied")))
            })
        }),
        InspectorAction::MoveVisualModifier { id, offset } => {
            with_controller(|controller| controller.move_visual_modifier(target, id, offset))
        }
        InspectorAction::RemoveVisualModifier { id } => {
            with_controller(|controller| controller.remove_visual_modifier(target, id))
        }
        InspectorAction::SetAlphaMask {
            target: mask_target,
            enabled,
        } => {
            let result = with_controller(|controller| {
                controller.set_alpha_mask_enabled(target, mask_target, enabled)
            });
            if result.is_ok() {
                focus_alpha_mask(target, mask_target, enabled);
            }
            result
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
