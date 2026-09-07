mod audio;
mod audio_generator;
mod audio_modifiers;
mod caption;
pub mod graph;
mod layered;
mod metadata;
mod modifiers;
mod project;
mod track;
mod video;

use serde_json::Value;
use shrimply_project::project::ItemAddress;

use crate::{
    ControlKind, InspectorCommit, InspectorControl, InspectorDetail, InspectorSection,
    InspectorSnapshot, InspectorTarget,
};

#[derive(Clone, Debug, PartialEq)]
pub struct InspectorDocument {
    pub target: InspectorTarget,
    pub title: String,
    pub categories: Vec<InspectorCategory>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InspectorCategory {
    pub key: &'static str,
    pub label: &'static str,
    pub icon: CategoryIcon,
    pub items: Vec<InspectorListItem>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CategoryIcon {
    Project,
    Track,
    Text,
    Visual,
    Audio,
    Playback,
    Info,
    Performance,
    Transition,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BasicInspectorAction {
    Reset {
        path: String,
        value: Value,
    },
    ResetFields {
        values: Vec<(String, Value)>,
    },
    ResetAudioOutput,
    ResetAudioGenerator,
    ResetVideo(crate::VideoReset),
    SetBoolean {
        path: String,
        value: bool,
    },
    SetModifierEnabled {
        id: uuid::Uuid,
        enabled: bool,
        audio: bool,
    },
    ResetModifier {
        id: uuid::Uuid,
        audio: bool,
    },
    MoveModifier {
        id: uuid::Uuid,
        offset: isize,
        audio: bool,
    },
    RemoveModifier {
        id: uuid::Uuid,
        audio: bool,
    },
    SetAlphaMask {
        target: shrimply_project::project::VisualAlphaMaskTarget,
        enabled: bool,
    },
    Video(crate::video::VideoCardAction),
    CopyModifier {
        id: uuid::Uuid,
        audio: bool,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct InspectorItem {
    pub presentation: crate::item::InspectorItemPresentation,
    pub section: InspectorSection,
    pub reset: Option<BasicInspectorAction>,
    pub actions: Vec<crate::item::HeaderAction<BasicInspectorAction>>,
    pub toggle: Option<crate::item::HeaderToggle<BasicInspectorAction>>,
}

impl InspectorItem {
    pub fn new(
        key: impl Into<String>,
        title: impl Into<String>,
        section: InspectorSection,
    ) -> Self {
        Self {
            presentation: crate::item::InspectorItemPresentation::new(key, title),
            section,
            reset: None,
            actions: Vec::new(),
            toggle: None,
        }
    }

    pub fn reset(mut self, reset: BasicInspectorAction) -> Self {
        self.reset = Some(reset);
        self
    }

    pub fn toggle(mut self, toggle: crate::item::HeaderToggle<BasicInspectorAction>) -> Self {
        self.toggle = Some(toggle);
        self
    }

    pub fn preview_facet(mut self, facet: shrimply_preview_core::PreviewFacetKey) -> Self {
        self.presentation = self.presentation.preview_facet(facet);
        self
    }

    pub fn boxed(self) -> InspectorListItem {
        InspectorListItem::Item(Box::new(self))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum InspectorListItem {
    Item(Box<InspectorItem>),
    Flat(InspectorSection),
}

pub fn basic(snapshot: InspectorSnapshot) -> InspectorDocument {
    let categories = match &snapshot.target {
        InspectorTarget::Project => project::categories(
            snapshot
                .project
                .as_ref()
                .expect("project snapshot must include project presentation"),
        ),
        InspectorTarget::Track(_) => track::categories(
            snapshot
                .track
                .as_ref()
                .expect("track snapshot must include track presentation"),
        ),
        InspectorTarget::Transition { .. } => {
            let transition = snapshot
                .transition
                .as_ref()
                .expect("transition snapshot must include transition presentation");
            vec![InspectorCategory {
                key: "transition",
                label: transition.title,
                icon: CategoryIcon::Transition,
                items: vec![
                    InspectorItem::new("transition", transition.title, transition.section())
                        .boxed(),
                ],
            }]
        }
        InspectorTarget::Item(ItemAddress::Caption { .. }) => {
            caption::categories(&snapshot.value, &snapshot.details)
        }
        InspectorTarget::Item(ItemAddress::Audio { .. }) => {
            audio::categories(&snapshot.value, &snapshot.details, snapshot.runtime)
        }
        InspectorTarget::Item(ItemAddress::Video { .. }) => video::categories(
            snapshot
                .video
                .as_ref()
                .expect("video snapshot must include video presentation"),
            &snapshot.details,
        ),
    };
    InspectorDocument {
        target: snapshot.target,
        title: snapshot.title,
        categories,
    }
}

impl crate::InspectorController {
    pub fn poll_document(&self) -> bool {
        let media = self.media_metadata.borrow_mut().poll();
        let voices = self.voice_models.borrow_mut().poll();
        let cameras = self.camera_models.borrow_mut().poll();
        let analysis = self.poll_camera_analysis();
        let sam2 = self.poll_sam2_analysis();
        let tts = self.poll_tts();
        let fill = self.poll_transparent_fill_analysis();
        let caches = self.poll_visual_caches();
        media || voices || cameras || analysis || sam2 || tts || fill || caches
    }

    pub fn document(
        &self,
        format_date: fn(i64) -> Option<String>,
        server_url: &str,
        remembered_tts_model: &str,
    ) -> InspectorDocument {
        let camera_models = crate::camera_source::cached_tracking_models(server_url);
        let snapshot = self.snapshot_with_camera_models(camera_models.as_ref());
        if snapshot.video.as_ref().is_some_and(|video| {
            video.visual.iter().any(|card| {
                card.section
                    .controls
                    .iter()
                    .any(|control| control.path == crate::camera_source::MODEL_PATH)
            })
        }) {
            self.camera_models.borrow_mut().request(
                server_url,
                camera_models,
                crate::camera_source::tracking_models,
            );
        }
        let media = self
            .media_metadata
            .borrow_mut()
            .request(snapshot.media.as_ref(), format_date);
        let source = snapshot
            .media
            .as_ref()
            .map_or(crate::info::SourceMetadata::None, |media| media.selected);
        let alpha_mask = snapshot
            .value
            .get("alpha_mask_video")
            .and_then(Value::as_u64)
            .and_then(|value| u32::try_from(value).ok());
        let mut document = basic(snapshot);
        if let Some(media) = media
            && let Some(category) = document
                .categories
                .iter_mut()
                .find(|category| category.key == "info")
        {
            category
                .items
                .push(InspectorListItem::Flat(metadata::section(
                    &media, source, alpha_mask,
                )));
        }
        for category in &mut document.categories {
            for item in &mut category.items {
                let section = match item {
                    InspectorListItem::Flat(section) => section,
                    InspectorListItem::Item(item) => &mut item.section,
                };
                for control in &mut section.controls {
                    if control.kind == ControlKind::TtsEditor {
                        match self.tts_editor(&document.target, server_url, remembered_tts_model) {
                            Ok(presentation) => control.tts = Some(Box::new(presentation)),
                            Err(error) => {
                                control.subtitle = error;
                                control.sensitive = false;
                            }
                        }
                    }
                    if control.kind == ControlKind::VoiceModel {
                        self.voice_models.borrow_mut().populate(server_url, control);
                    }
                    let can_paste = match control.kind {
                        ControlKind::AudioModifierMenu => self
                            .can_paste_audio_modifiers(&document.target, &self.property_clipboard),
                        ControlKind::VisualModifierMenu => self
                            .can_paste_visual_modifiers(&document.target, &self.property_clipboard),
                        _ => continue,
                    };
                    if can_paste {
                        control.values.push("__paste__".into());
                        control.labels.push("Paste modifiers".into());
                        control.search_terms.push("Paste copied modifiers".into());
                    }
                }
            }
        }
        document
    }

    pub fn add_control_modifier(
        &self,
        target: &InspectorTarget,
        kind: ControlKind,
        value: &str,
    ) -> Result<(), String> {
        match (kind, value) {
            (ControlKind::AudioModifierMenu, "__paste__") => self
                .paste_audio_modifiers(target, &self.property_clipboard)
                .map(|_| ()),
            (ControlKind::VisualModifierMenu, "__paste__") => self
                .paste_visual_modifiers(target, &self.property_clipboard)
                .map(|_| ()),
            (ControlKind::AudioModifierMenu, value) => self.add_audio_modifier(target, value),
            (ControlKind::VisualModifierMenu, value) => {
                self.add_visual_modifier(target, value).map(|_| ())
            }
            _ => Err("control is not a modifier menu".into()),
        }
    }

    pub fn apply_basic_action(
        &self,
        target: &InspectorTarget,
        action: &BasicInspectorAction,
    ) -> Result<(), String> {
        match action {
            BasicInspectorAction::CopyModifier { id, audio } => {
                if *audio {
                    self.copy_audio_modifier(target, *id, &self.property_clipboard)
                        .map(|_| ())
                } else {
                    self.copy_visual_modifier(target, *id, &self.property_clipboard)
                        .map(|_| ())
                }
            }
            BasicInspectorAction::SetModifierEnabled { id, enabled, audio } => {
                if *audio {
                    self.set_audio_modifier_enabled(target, *id, *enabled)
                } else {
                    self.set_visual_modifier_enabled(target, *id, *enabled)
                }
            }
            BasicInspectorAction::ResetModifier { id, audio } => {
                let InspectorTarget::Item(address) = target else {
                    return Err("modifier reset requires an item".into());
                };
                let project = self.project.borrow();
                if *audio {
                    let effect = project
                        .audio_item(address)
                        .and_then(|item| item.modifiers.iter().find(|modifier| modifier.id == *id))
                        .map(|modifier| crate::default_audio_modifier_effect(&modifier.effect))
                        .ok_or("audio modifier is no longer available")?;
                    drop(project);
                    self.reset_audio_modifier_effect(target, *id, effect)
                } else {
                    let effect = project
                        .video_item(address)
                        .and_then(|item| item.modifiers.iter().find(|modifier| modifier.id == *id))
                        .map(|modifier| crate::default_visual_modifier_effect(&modifier.effect))
                        .ok_or("visual modifier is no longer available")?;
                    drop(project);
                    self.reset_visual_modifier_effect(target, *id, effect)
                }
            }
            BasicInspectorAction::MoveModifier { id, offset, audio } => {
                if *audio {
                    self.move_audio_modifier(target, *id, *offset)
                } else {
                    self.move_visual_modifier(target, *id, *offset)
                }
            }
            BasicInspectorAction::RemoveModifier { id, audio } => {
                if *audio {
                    self.remove_audio_modifier(target, *id)
                } else {
                    self.remove_visual_modifier(target, *id)
                }
            }
            BasicInspectorAction::SetAlphaMask {
                target: mask,
                enabled,
            } => self.set_alpha_mask_enabled(target, *mask, *enabled),
            BasicInspectorAction::Video(crate::video::VideoCardAction::ReloadAsset {
                asset,
                kind,
            }) => crate::video::reload_asset(asset, *kind),
            BasicInspectorAction::Reset { path, value } => {
                self.set_value(target, path, value.clone())
            }
            BasicInspectorAction::ResetFields { values } => self.set_values(target, values),
            BasicInspectorAction::ResetAudioOutput => self.set_values(
                target,
                &[
                    ("/enabled".into(), Value::Bool(true)),
                    (
                        "/gain".into(),
                        serde_json::to_value(shrimply_audio_modifiers::GainModifier::default())
                            .expect("default audio gain must serialize"),
                    ),
                ],
            ),
            BasicInspectorAction::ResetAudioGenerator => {
                let defaults =
                    serde_json::to_value(shrimply_project::project::AudioGenerator::default())
                        .expect("default audio generator must serialize");
                self.set_values(
                    target,
                    &defaults
                        .as_object()
                        .expect("generator must be an object")
                        .iter()
                        .map(|(field, value)| (format!("/source/{field}"), value.clone()))
                        .collect::<Vec<_>>(),
                )
            }
            BasicInspectorAction::ResetVideo(reset) => self.reset_video(target, reset),
            BasicInspectorAction::SetBoolean { path, value } => {
                self.set_value(target, path, Value::Bool(*value))
            }
        }
    }

    pub fn set_basic_control_value(
        &self,
        target: &InspectorTarget,
        control: &InspectorControl,
        value: &str,
    ) -> Result<(), String> {
        match control.action {
            Some(crate::InspectorControlAction::SetSam2Model { modifier_id }) => {
                let model = serde_json::from_value(Value::String(value.to_string()))
                    .map_err(|error| format!("invalid SAM2 model: {error}"))?;
                return self.set_sam2_model(target, modifier_id, model);
            }
            Some(crate::InspectorControlAction::SetSam2PointLabel {
                modifier_id,
                point_id,
            }) => {
                let label = serde_json::from_value(Value::String(value.to_string()))
                    .map_err(|error| format!("invalid SAM2 point type: {error}"))?;
                return self.set_sam2_point_label(target, modifier_id, point_id, label);
            }
            _ => {}
        }
        if let Some(id) = control.target_id {
            match control.kind {
                ControlKind::AudioCachePreset => {
                    return self.set_audio_cache_preset(target, id, value);
                }
                ControlKind::VisualCacheQuality => {
                    return self.set_visual_cache_quality(target, id, value);
                }
                _ => {}
            }
        }
        if control.audio_modifier {
            let id = control
                .target_id
                .ok_or("audio modifier control has no owner")?;
            return if control.kind == ControlKind::LayeredNumber {
                let timeline = control
                    .timeline_id
                    .ok_or("audio modifier control has no timeline")?;
                let number = value
                    .parse::<f64>()
                    .map_err(|_| "invalid modifier number")?;
                self.set_audio_modifier_timeline_base(
                    target,
                    id,
                    timeline,
                    control.store_number(number) as f32,
                )
            } else if control.kind == ControlKind::Number {
                self.set_audio_modifier_live_field(target, id, &control.path, value)
            } else {
                self.set_audio_modifier_field(target, id, &control.path, value)
            };
        }
        match control.kind {
            ControlKind::OptionalSelector => {
                self.set_optional_field(target, &control.path, (!value.is_empty()).then_some(value))
            }
            ControlKind::OptionalNumberSelector => self.set_optional_number_field(
                target,
                &control.path,
                (!value.is_empty()).then_some(value),
            ),
            ControlKind::LayeredNumber => value
                .parse::<f64>()
                .map_err(|_| format!("invalid numeric inspector value: {value}"))
                .and_then(|value| {
                    let value = control.store_number(value);
                    if control.scalar_storage == crate::section::ScalarStorage::UnsignedInteger {
                        if !value.is_finite()
                            || value.fract() != 0.0
                            || !(0.0..=f64::from(u32::MAX)).contains(&value)
                        {
                            return Err("invalid unsigned integer timeline value".to_string());
                        }
                        Ok(Value::from(value as u32))
                    } else {
                        finite_number(value)
                    }
                })
                .and_then(|value| {
                    self.set_timeline_base_with_commit(
                        target,
                        &control.path,
                        value,
                        control_commit(control),
                    )
                }),
            ControlKind::LayeredText => self.set_text_value(
                target,
                &control.path,
                control.timeline_id.ok_or("text timeline is unavailable")?,
                value.to_string(),
                control_commit(control),
            ),
            ControlKind::LayeredBoolean => value
                .parse::<bool>()
                .map_err(|_| format!("invalid boolean inspector value: {value}"))
                .and_then(|value| self.set_bool_value(target, &control.path, value)),
            ControlKind::LayeredSelector => self.set_timeline_base_with_commit(
                target,
                &control.path,
                Value::String(value.to_string()),
                control_commit(control),
            ),
            ControlKind::Number => value
                .parse::<f64>()
                .map(|value| control.store_number(value).to_string())
                .map_err(|_| format!("invalid numeric inspector value: {value}"))
                .and_then(|value| self.set_basic_regular_value(target, control, &value)),
            _ => self.set_basic_regular_value(target, control, value),
        }
    }

    pub fn set_basic_control_components(
        &self,
        target: &InspectorTarget,
        control: &InspectorControl,
        values: &[f64],
    ) -> Result<(), String> {
        let expected = match control.kind {
            ControlKind::Vector2 | ControlKind::LayeredVector2 => 2,
            ControlKind::Vector3 | ControlKind::LayeredVector3 => 3,
            ControlKind::Color | ControlKind::LayeredColor => 4,
            _ => return Err("inspector control does not have numeric components".into()),
        };
        if values.len() != expected || values.iter().any(|value| !value.is_finite()) {
            return Err(format!(
                "inspector control requires {expected} finite components"
            ));
        }
        if let Some(crate::InspectorControlAction::SetSam2PointPosition {
            modifier_id,
            point_id,
        }) = control.action
        {
            return self.set_sam2_point_position(
                target,
                modifier_id,
                point_id,
                values[0] * control.store_multiplier,
                values[1] * control.store_multiplier,
            );
        }
        if control.kind == ControlKind::LayeredColor {
            if values.iter().any(|value| !(0.0..=255.0).contains(value)) {
                return Err("color channels must be between 0 and 255".into());
            }
            return self.set_color_value(
                target,
                &control.path,
                control.timeline_id.ok_or("color timeline is unavailable")?,
                shrimply_core::Color::new(
                    values[0] as u8,
                    values[1] as u8,
                    values[2] as u8,
                    values[3] as u8,
                ),
                control_commit(control),
            );
        }
        if control.kind == ControlKind::LayeredVector2 {
            return self.set_vector2_value(
                target,
                &control.path,
                values[0] * control.store_multiplier,
                values[1] * control.store_multiplier,
                control_commit(control),
            );
        }
        if control.kind == ControlKind::LayeredVector3 {
            return self.set_vector3_value(
                target,
                &control.path,
                values[0] * control.store_multiplier,
                values[1] * control.store_multiplier,
                values[2] * control.store_multiplier,
                control_commit(control),
            );
        }
        let values = values
            .iter()
            .enumerate()
            .map(|(index, value)| (index, (value * control.store_multiplier).to_string()))
            .collect::<Vec<_>>();
        if matches!(target, InspectorTarget::Transition { .. }) && !control.commit_name.is_empty() {
            self.set_transition_components(
                target,
                &control.path,
                &values,
                &control.commit_name,
                control.commit_immediately,
            )
        } else {
            self.set_components(target, &control.path, &values)
        }
    }

    pub fn set_basic_control_fraction(
        &self,
        target: &InspectorTarget,
        control: &InspectorControl,
        value: shrimply_math_core::Fraction,
    ) -> Result<(), String> {
        if matches!(target, InspectorTarget::Item(ItemAddress::Video { .. }))
            && !control.commit_name.is_empty()
        {
            self.set_video_fraction(target, &control.path, value, &control.commit_name)
        } else {
            self.set_fraction(target, &control.path, value)
        }
    }

    pub fn commit_basic_control(
        &self,
        target: &InspectorTarget,
        control: &InspectorControl,
    ) -> Result<(), String> {
        if matches!(
            control.kind,
            ControlKind::LayeredNumber
                | ControlKind::LayeredBoolean
                | ControlKind::LayeredSelector
                | ControlKind::LayeredVector2
                | ControlKind::LayeredVector3
        ) {
            self.finish_live_inspector_edit(target)
        } else if matches!(target, InspectorTarget::Transition { .. })
            && !control.commit_name.is_empty()
            && !control.commit_immediately
        {
            self.commit_transition_field(target, &control.commit_name)
        } else if matches!(target, InspectorTarget::Item(ItemAddress::Video { .. }))
            && !control.commit_name.is_empty()
            && !control.commit_immediately
        {
            self.commit_video_field(target, &control.commit_name)
        } else {
            self.finish_live_edit();
            Ok(())
        }
    }

    fn set_basic_regular_value(
        &self,
        target: &InspectorTarget,
        control: &InspectorControl,
        value: &str,
    ) -> Result<(), String> {
        if matches!(target, InspectorTarget::Transition { .. }) && !control.commit_name.is_empty() {
            self.set_transition_field(
                target,
                &control.path,
                value,
                &control.commit_name,
                control.commit_immediately,
            )
        } else if matches!(target, InspectorTarget::Item(ItemAddress::Video { .. }))
            && !control.commit_name.is_empty()
        {
            self.set_video_field(
                target,
                &control.path,
                value,
                &control.commit_name,
                control.commit_immediately,
            )
        } else {
            self.set_field(target, &control.path, value)
        }
    }
}

fn control_commit(control: &InspectorControl) -> InspectorCommit<'_> {
    if control.commit_immediately {
        InspectorCommit::Immediate(&control.commit_name)
    } else {
        InspectorCommit::Coalesced(&control.commit_name)
    }
}

fn finite_number(value: f64) -> Result<Value, String> {
    serde_json::Number::from_f64(value)
        .map(Value::Number)
        .ok_or_else(|| "timeline value must be finite".to_string())
}

fn detail_item(details: &[InspectorDetail]) -> InspectorListItem {
    let mut section = InspectorSection::default();
    for detail in details {
        section.add(
            crate::InspectorControl::new(
                if matches!(detail.label, "File Location" | "Project File") {
                    crate::ControlKind::FileLocation
                } else {
                    crate::ControlKind::ReadOnly
                },
                "",
                detail.label,
            )
            .value(&detail.value)
            .read_only(),
        );
    }
    InspectorListItem::Flat(section)
}

fn text<'a>(value: &'a Value, path: &str) -> &'a str {
    value
        .pointer(path)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("inspector text is unavailable: {path}"))
}
