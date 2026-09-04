use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, mpsc};

use serde_json::Value;
use shrimply_project::project::{
    Asset, AssetSnapshot, BlenderItem, BlenderPreviewDownsample, BlenderRenderMethod, Time,
};

use crate::{InspectorControl, InspectorSection};

use super::{ReloadKind, VideoCard, VideoCardAction};

pub(super) const SCENE_PATH: &str = "/content/scene";
const VIEW_LAYER_PATH: &str = "/content/view_layer";
const CAMERA_PATH: &str = "/content/camera";

#[derive(Clone)]
pub enum MetadataState {
    Loading,
    Ready(Arc<shrimply_blender::Metadata>),
    Failed(String),
}

#[derive(Clone, Eq, Hash, PartialEq)]
struct CacheKey {
    path: PathBuf,
    revision: u64,
    binary: Option<PathBuf>,
}

struct LoadedMetadata {
    key: CacheKey,
    snapshot: AssetSnapshot,
    result: Result<Arc<shrimply_blender::Metadata>, String>,
}

struct MetadataCache {
    results: HashMap<CacheKey, Result<Arc<shrimply_blender::Metadata>, String>>,
    pending: HashSet<CacheKey>,
    sender: mpsc::Sender<LoadedMetadata>,
    receiver: mpsc::Receiver<LoadedMetadata>,
}

impl Default for MetadataCache {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            results: HashMap::new(),
            pending: HashSet::new(),
            sender,
            receiver,
        }
    }
}

struct ResolvedMetadata<'a> {
    pub scene: &'a shrimply_blender::SceneMetadata,
    pub view_layer: String,
    pub camera: String,
}

fn resolve_metadata<'a>(
    blender: &BlenderItem,
    metadata: &'a shrimply_blender::Metadata,
) -> Option<ResolvedMetadata<'a>> {
    let scene = metadata
        .scenes
        .iter()
        .find(|scene| scene.name == blender.scene)
        .or_else(|| metadata.scenes.first())?;
    Some(ResolvedMetadata {
        view_layer: scene
            .view_layers
            .iter()
            .find(|name| **name == blender.view_layer)
            .cloned()
            .unwrap_or_else(|| scene.active_view_layer.clone()),
        camera: scene
            .cameras
            .iter()
            .find(|name| **name == blender.camera)
            .cloned()
            .unwrap_or_else(|| scene.active_camera.clone()),
        scene,
    })
}

fn source_duration(metadata: &ResolvedMetadata<'_>) -> Time {
    Time {
        seconds: metadata.scene.duration(),
    }
}

impl crate::InspectorController {
    pub fn sync_blender_metadata(
        &self,
        target: &crate::InspectorTarget,
        metadata: &shrimply_blender::Metadata,
    ) -> Result<bool, String> {
        let crate::InspectorTarget::Item(address) = target else {
            return Err("Blender metadata target is not an item".to_string());
        };
        let mut project = self.project.borrow_mut();
        let item = project
            .video_item_mut(address)
            .ok_or_else(|| "Blender item is no longer available".to_string())?;
        let shrimply_project::project::VideoItemContent::Blender(blender) = &mut item.content
        else {
            return Err("video item is not a Blender source".to_string());
        };
        let Some(resolved) = resolve_metadata(blender, metadata) else {
            return Ok(false);
        };
        let duration = source_duration(&resolved);
        let changed = blender.scene != resolved.scene.name
            || blender.view_layer != resolved.view_layer
            || blender.camera != resolved.camera
            || item.source_duration != duration;
        if !changed {
            return Ok(false);
        }
        blender.scene.clone_from(&resolved.scene.name);
        blender.view_layer = resolved.view_layer;
        blender.camera = resolved.camera;
        item.source_duration = duration;
        shrimply_project::project::commit_edit(&project, "blender-metadata");
        drop(project);
        shrimply_state::player_state::refresh_project(
            &self.player_state,
            shrimply_state::player_state::ProjectChange {
                video: true,
                inspector: true,
                ..Default::default()
            },
        );
        Ok(true)
    }
}

pub fn metadata(source: &Asset, binary: Option<&Path>) -> MetadataState {
    let snapshot = match source.snapshot() {
        Ok(snapshot) => snapshot,
        Err(error) => return MetadataState::Failed(error),
    };
    let key = CacheKey {
        path: snapshot.path().to_path_buf(),
        revision: snapshot.revision(),
        binary: binary.map(Path::to_path_buf),
    };
    let cache = cache();
    let mut cache = cache.lock().expect("Blender metadata cache mutex poisoned");
    receive_metadata(&mut cache);
    if let Some(result) = cache.results.get(&key) {
        return match result {
            Ok(metadata) => MetadataState::Ready(Arc::clone(metadata)),
            Err(error) => MetadataState::Failed(error.clone()),
        };
    }
    if cache.pending.insert(key.clone()) {
        let sender = cache.sender.clone();
        std::thread::spawn(move || {
            let result = key
                .binary
                .as_deref()
                .ok_or_else(|| "Choose a compatible Blender binary in Preferences".to_string())
                .and_then(|binary| shrimply_blender::discover(binary, snapshot.path()))
                .map(Arc::new);
            let _ = sender.send(LoadedMetadata {
                key,
                snapshot,
                result,
            });
        });
    }
    MetadataState::Loading
}

pub fn poll_metadata() -> bool {
    let cache = cache();
    receive_metadata(&mut cache.lock().expect("Blender metadata cache mutex poisoned"))
}

pub fn metadata_controls(blender: &BlenderItem, metadata: &MetadataState) -> Vec<InspectorControl> {
    let mut section = InspectorSection::default();
    match metadata {
        MetadataState::Loading => loading_controls(&mut section, blender),
        MetadataState::Ready(metadata) => ready_controls(&mut section, blender, metadata),
        MetadataState::Failed(error) => failed_controls(&mut section, blender, error),
    }
    section.controls
}

pub fn settings_controls(blender: &BlenderItem) -> Vec<InspectorControl> {
    let mut section = InspectorSection::default();
    section.add(
        crate::selector::selector(
            "/content/render_method",
            "Accurate Render Method",
            enum_text(blender.render_method),
            render_methods(),
        )
        .immediate_commit("blender-render-method"),
    );
    section.add(
        crate::selector::selector(
            "/content/preview_render_method",
            "Preview Render Method",
            enum_text(blender.preview_render_method),
            render_methods(),
        )
        .immediate_commit("blender-preview-render-method"),
    );
    section.add(
        crate::selector::selector(
            "/content/preview_downsample",
            "Preview Downsampling",
            enum_text(blender.preview_downsample),
            [
                (
                    enum_text(BlenderPreviewDownsample::Full),
                    "Off (Full Resolution)".to_string(),
                ),
                (enum_text(BlenderPreviewDownsample::X2), "2×".to_string()),
                (enum_text(BlenderPreviewDownsample::X4), "4×".to_string()),
                (enum_text(BlenderPreviewDownsample::X8), "8×".to_string()),
                (enum_text(BlenderPreviewDownsample::X16), "16×".to_string()),
                (enum_text(BlenderPreviewDownsample::X32), "32×".to_string()),
            ],
        )
        .immediate_commit("blender-preview-downsampling"),
    );
    section.controls
}

pub fn card(blender: &BlenderItem, asset: &str, metadata: &MetadataState) -> VideoCard {
    let mut section = InspectorSection {
        controls: metadata_controls(blender, metadata),
    };
    section.controls.extend(settings_controls(blender));

    let default = BlenderItem::default();
    VideoCard::new("blender", "Blender", section)
        .reset_fields(
            [
                (SCENE_PATH, Value::String(default.scene)),
                (VIEW_LAYER_PATH, Value::String(default.view_layer)),
                (CAMERA_PATH, Value::String(default.camera)),
                (
                    "/content/render_method",
                    serde_json::to_value(default.render_method)
                        .expect("Blender render method must serialize"),
                ),
                (
                    "/content/preview_render_method",
                    serde_json::to_value(default.preview_render_method)
                        .expect("Blender preview render method must serialize"),
                ),
                (
                    "/content/preview_downsample",
                    serde_json::to_value(default.preview_downsample)
                        .expect("Blender preview downsampling must serialize"),
                ),
            ],
            "reset-blender",
        )
        .actions([crate::item::HeaderAction {
            icon: "view-refresh-symbolic",
            tooltip: "Reload Blender file and scene metadata",
            sensitive: true,
            activate: VideoCardAction::ReloadAsset {
                asset: asset.to_string(),
                kind: ReloadKind::Blender,
            },
        }])
}

pub(super) fn set_field(
    item: &mut shrimply_project::project::VideoItem,
    path: &str,
    value: &str,
) -> Option<bool> {
    let shrimply_project::project::VideoItemContent::Blender(blender) = &mut item.content else {
        return None;
    };
    let current = match path {
        SCENE_PATH => &mut blender.scene,
        VIEW_LAYER_PATH => &mut blender.view_layer,
        CAMERA_PATH => &mut blender.camera,
        _ => return None,
    };
    if current == value {
        return Some(false);
    }
    current.clear();
    current.push_str(value);
    if path == SCENE_PATH {
        blender.view_layer.clear();
        blender.camera.clear();
    }
    Some(true)
}

fn loading_controls(section: &mut InspectorSection, blender: &BlenderItem) {
    for (path, label, value, placeholder) in [
        (SCENE_PATH, "Scene", &blender.scene, "Loading scenes…"),
        (
            VIEW_LAYER_PATH,
            "View Layer",
            &blender.view_layer,
            "Loading view layers…",
        ),
        (CAMERA_PATH, "Camera", &blender.camera, "Loading cameras…"),
    ] {
        let value = if value.is_empty() { placeholder } else { value };
        let commit_name = match path {
            SCENE_PATH => "blender-scene",
            VIEW_LAYER_PATH => "blender-view-layer",
            CAMERA_PATH => "blender-camera",
            _ => unreachable!("Blender loading control path is declared above"),
        };
        section.add(
            crate::selector::selector(path, label, value, [(value.to_string(), value.to_string())])
                .sensitive(false)
                .immediate_commit(commit_name),
        );
    }
}

fn ready_controls(
    section: &mut InspectorSection,
    blender: &BlenderItem,
    metadata: &shrimply_blender::Metadata,
) {
    let Some(resolved) = resolve_metadata(blender, metadata) else {
        loading_controls(section, blender);
        section.controls[0].tooltip = "The Blender file contains no scenes".to_string();
        return;
    };
    section.add(
        crate::selector::selector(
            SCENE_PATH,
            "Scene",
            resolved.scene.name.clone(),
            metadata
                .scenes
                .iter()
                .map(|scene| (scene.name.clone(), scene.name.clone())),
        )
        .immediate_commit("blender-scene"),
    );
    section.add(
        crate::selector::selector(
            VIEW_LAYER_PATH,
            "View Layer",
            resolved.view_layer,
            resolved
                .scene
                .view_layers
                .iter()
                .cloned()
                .map(|name| (name.clone(), name)),
        )
        .sensitive(!resolved.scene.view_layers.is_empty())
        .immediate_commit("blender-view-layer"),
    );
    section.add(
        crate::selector::selector(
            CAMERA_PATH,
            "Camera",
            resolved.camera,
            resolved
                .scene
                .cameras
                .iter()
                .cloned()
                .map(|name| (name.clone(), name)),
        )
        .sensitive(!resolved.scene.cameras.is_empty())
        .immediate_commit("blender-camera"),
    );
}

fn failed_controls(section: &mut InspectorSection, blender: &BlenderItem, error: &str) {
    loading_controls(section, blender);
    let scene = &mut section.controls[0];
    scene.value = "Could not load Blender".to_string();
    scene.values = vec![scene.value.clone()];
    scene.labels = vec![scene.value.clone()];
    scene.tooltip = error.to_string();
}

fn render_methods() -> [(String, String); 3] {
    [
        (enum_text(BlenderRenderMethod::Solid), "Solid".to_string()),
        (
            enum_text(BlenderRenderMethod::MaterialPreview),
            "Material Preview".to_string(),
        ),
        (
            enum_text(BlenderRenderMethod::SceneRenderer),
            "Scene Renderer".to_string(),
        ),
    ]
}

fn enum_text(value: impl serde::Serialize) -> String {
    serde_json::to_value(value)
        .expect("Blender option must serialize")
        .as_str()
        .expect("Blender option must serialize as text")
        .to_string()
}

fn receive_metadata(cache: &mut MetadataCache) -> bool {
    let mut changed = false;
    while let Ok(loaded) = cache.receiver.try_recv() {
        cache.pending.remove(&loaded.key);
        changed = true;
        if !loaded.snapshot.is_current() {
            continue;
        }
        cache
            .results
            .retain(|key, _| key.path != loaded.key.path || key == &loaded.key);
        cache.results.insert(loaded.key, loaded.result);
    }
    changed
}

fn cache() -> &'static Mutex<MetadataCache> {
    static CACHE: OnceLock<Mutex<MetadataCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(MetadataCache::default()))
}
