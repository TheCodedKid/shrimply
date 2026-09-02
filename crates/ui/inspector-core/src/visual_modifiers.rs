use shrimply_core::modifier_model::ModifierModel;
use shrimply_project::project::{
    AlphaMaskShape, ItemAddress, Time, VideoItem, VisualAlphaMask, VisualAlphaMaskTarget,
    VisualModifier,
};
use shrimply_state::player_state::{self, ProjectChange};
use shrimply_video_modifiers::{
    ModifierEffect, RasterModifierEffect, VectorModifierEffect, VisualKind,
};

use crate::{
    ControlKind, InspectorControl, InspectorController, InspectorRuntime, InspectorSection,
    InspectorTarget, LayeredState, NumberSpec,
};

mod opacity;
mod transform;

pub use opacity::OpacityModifierPresentation;
pub use transform::TransformModifierPresentation;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VisualModifierChoice {
    pub key: String,
    pub label: &'static str,
    pub search_text: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VisualModifierPresentation {
    pub id: uuid::Uuid,
    pub index: usize,
    pub title: &'static str,
    pub enabled: bool,
    pub default_effect: serde_json::Value,
    pub can_move_up: bool,
    pub can_move_down: bool,
    pub can_remove: bool,
    pub body: Option<VisualModifierBodyPresentation>,
    pub alpha_mask: Option<VisualModifierAlphaMaskPresentation>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum VisualModifierBodyPresentation {
    Opacity(Box<OpacityModifierPresentation>),
    Transform(Box<TransformModifierPresentation>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct VisualModifierAlphaMaskPresentation {
    pub active: bool,
    pub section: InspectorSection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VisualModifierChainAction {
    MoveUp,
    MoveDown,
    Remove,
}

pub fn visual_modifier_presentations(
    item: &VideoItem,
    runtime: InspectorRuntime,
) -> Vec<VisualModifierPresentation> {
    item.modifiers
        .iter()
        .enumerate()
        .map(|(index, modifier)| VisualModifierPresentation {
            id: modifier.id,
            index,
            title: modifier.effect.display_name(),
            enabled: modifier.enabled,
            default_effect: serde_json::to_value(default_visual_modifier_effect(&modifier.effect))
                .expect("visual modifier effect must serialize"),
            can_move_up: visual_modifier_action_valid(
                item,
                modifier.id,
                VisualModifierChainAction::MoveUp,
            ),
            can_move_down: visual_modifier_action_valid(
                item,
                modifier.id,
                VisualModifierChainAction::MoveDown,
            ),
            can_remove: visual_modifier_action_valid(
                item,
                modifier.id,
                VisualModifierChainAction::Remove,
            ),
            body: match &modifier.effect {
                ModifierEffect::Vector(effect) => match &**effect {
                    VectorModifierEffect::Transform(value) => {
                        Some(VisualModifierBodyPresentation::Transform(Box::new(
                            transform::presentation(value, index, runtime),
                        )))
                    }
                    VectorModifierEffect::Opacity(value) => {
                        Some(VisualModifierBodyPresentation::Opacity(Box::new(
                            opacity::presentation(value, index, runtime),
                        )))
                    }
                    _ => None,
                },
                ModifierEffect::Raster(effect) => match &**effect {
                    RasterModifierEffect::Transform(value) => {
                        Some(VisualModifierBodyPresentation::Transform(Box::new(
                            transform::presentation(value, index, runtime),
                        )))
                    }
                    RasterModifierEffect::Opacity(value) => {
                        Some(VisualModifierBodyPresentation::Opacity(Box::new(
                            opacity::presentation(value, index, runtime),
                        )))
                    }
                    _ => None,
                },
                _ => None,
            },
            alpha_mask: matches!(
                modifier.effect,
                ModifierEffect::Raster(ref effect)
                    if !matches!(&**effect, RasterModifierEffect::Cache(_))
            )
            .then(|| modifier_alpha_mask_presentation(index, modifier, runtime)),
        })
        .collect()
}

pub fn default_visual_modifier_effect(effect: &ModifierEffect) -> ModifierEffect {
    if let ModifierEffect::Raster(effect) = effect {
        match &**effect {
            RasterModifierEffect::Transform(_) => {
                return ModifierEffect::raster(RasterModifierEffect::Transform(Default::default()));
            }
            RasterModifierEffect::Opacity(_) => {
                return ModifierEffect::raster(RasterModifierEffect::Opacity(Default::default()));
            }
            _ => {}
        }
    }
    let key = visual_modifier_key(effect);
    ModifierEffect::catalog()
        .find(|candidate| visual_modifier_key(candidate) == key)
        .expect("every visual modifier effect must have a catalog default")
}

fn modifier_alpha_mask_presentation(
    index: usize,
    modifier: &VisualModifier,
    runtime: InspectorRuntime,
) -> VisualModifierAlphaMaskPresentation {
    let active = modifier
        .alpha_mask
        .as_ref()
        .is_some_and(|mask| mask.enabled);
    let mut section = InspectorSection::default();
    if let Some(mask) = modifier.alpha_mask.as_ref().filter(|mask| mask.enabled) {
        let base = format!("/modifiers/{index}/alpha_mask");
        section.add(
            InspectorControl::new(ControlKind::Selector, format!("{base}/shape"), "Shape")
                .value(enum_text(mask.shape))
                .choices(
                    vec!["rectangle".into(), "ellipse".into(), "polygon".into()],
                    vec!["Rectangle".into(), "Ellipse".into(), "Polygon".into()],
                )
                .immediate_commit("alpha-mask-shape"),
        );
        section.add(
            InspectorControl::new(ControlKind::Boolean, format!("{base}/invert"), "Invert")
                .value(mask.invert.to_string())
                .immediate_commit("invert-alpha-mask"),
        );
        section.add(alpha_mask_vector(
            format!("{base}/center"),
            "Center",
            &mask.center,
            runtime,
            NumberSpec {
                drag_step: 0.01,
                digits: 2,
                unit: "x",
                ..NumberSpec::default()
            },
        ));
        section.add(alpha_mask_vector(
            format!("{base}/size"),
            "Size",
            &mask.size,
            runtime,
            NumberSpec {
                minimum: 0.0,
                drag_step: 0.01,
                digits: 2,
                unit: "x",
                ..NumberSpec::default()
            },
        ));
        section.add(alpha_mask_scalar(
            format!("{base}/rotation_degrees"),
            "Rotation",
            &mask.rotation_degrees,
            runtime,
            NumberSpec {
                drag_step: 1.0,
                digits: 1,
                unit: "°",
                ..NumberSpec::default()
            },
            1.0,
            true,
        ));
        if mask.shape == AlphaMaskShape::Rectangle {
            section.add(alpha_mask_scalar(
                format!("{base}/rounding"),
                "Roundness",
                &mask.rounding,
                runtime,
                percent_spec(),
                100.0,
                false,
            ));
        }
        section.add(alpha_mask_scalar(
            format!("{base}/feather"),
            "Feather",
            &mask.feather,
            runtime,
            percent_spec(),
            100.0,
            false,
        ));
    }
    VisualModifierAlphaMaskPresentation { active, section }
}

fn alpha_mask_vector(
    path: String,
    label: &'static str,
    timeline: &shrimply_core::timeline_value::TimelineValue<glam::Vec2>,
    runtime: InspectorRuntime,
    number: NumberSpec,
) -> InspectorControl {
    let value = timeline.value_at(runtime.local_time.unwrap_or(Time::ZERO));
    InspectorControl::new(ControlKind::LayeredVector2, path.clone(), label)
        .components(vec![value.x.to_string(), value.y.to_string()])
        .number(number)
        .width_characters(7)
        .prefixes(["X", "Y"])
        .layered(path, LayeredState::from(timeline))
        .graph(crate::transform::vector_speed_graph(timeline, runtime))
        .live_commit("visual-alpha-mask-vector")
}

fn alpha_mask_scalar(
    path: String,
    label: &'static str,
    timeline: &shrimply_core::timeline_value::TimelineValue<f32>,
    runtime: InspectorRuntime,
    number: NumberSpec,
    display_multiplier: f64,
    rotating: bool,
) -> InspectorControl {
    let value = f64::from(timeline.value_at(runtime.local_time.unwrap_or(Time::ZERO)));
    let mut graph = crate::transform::scalar_graph(timeline, value as f32, runtime);
    if display_multiplier != 1.0
        && let Some(graph) = &mut graph
    {
        graph
            .points
            .iter_mut()
            .for_each(|point| point.value *= display_multiplier);
        graph.segments.iter_mut().for_each(|segment| {
            segment.start_value *= display_multiplier;
            segment.end_value *= display_multiplier;
        });
    }
    let control = InspectorControl::new(ControlKind::LayeredNumber, path.clone(), label)
        .value((value * display_multiplier).to_string())
        .number(number)
        .store_multiplier(display_multiplier.recip())
        .width_characters(8)
        .layered(path, LayeredState::from(timeline))
        .graph(graph)
        .live_commit("visual-alpha-mask-scalar");
    if rotating {
        control.rotating_icon("rotation.svg", 0.0)
    } else {
        control
    }
}

pub(super) fn modifier_scalar_control(
    path: String,
    label: &'static str,
    timeline: &shrimply_core::timeline_value::TimelineValue<f32>,
    runtime: InspectorRuntime,
    number: NumberSpec,
    rotating: bool,
) -> InspectorControl {
    let value = timeline.value_at(runtime.local_time.unwrap_or(Time::ZERO));
    let control = InspectorControl::new(ControlKind::LayeredNumber, path.clone(), label)
        .value(value.to_string())
        .number(number)
        .width_characters(8)
        .layered(path, LayeredState::from(timeline))
        .graph(crate::transform::scalar_graph(timeline, value, runtime))
        .live_commit("visual-modifier-value");
    if rotating {
        control.rotating_icon("rotation.svg", 0.0)
    } else {
        control
    }
}

pub(super) fn modifier_vector2_control(
    path: String,
    label: &'static str,
    timeline: &shrimply_core::timeline_value::TimelineValue<glam::Vec2>,
    runtime: InspectorRuntime,
    number: NumberSpec,
    lock: bool,
) -> InspectorControl {
    let value = timeline.value_at(runtime.local_time.unwrap_or(Time::ZERO));
    let control = InspectorControl::new(ControlKind::LayeredVector2, path.clone(), label)
        .components(vec![value.x.to_string(), value.y.to_string()])
        .number(number)
        .width_characters(7)
        .prefixes(["X", "Y"])
        .layered(path, LayeredState::from(timeline))
        .graph(crate::transform::vector_speed_graph(timeline, runtime))
        .live_commit("visual-modifier-vector");
    if lock { control.lock() } else { control }
}

fn percent_spec() -> NumberSpec {
    NumberSpec {
        minimum: 0.0,
        maximum: 100.0,
        drag_step: 1.0,
        digits: 1,
        unit: "%",
    }
}

fn enum_text(value: impl serde::Serialize) -> String {
    serde_json::to_value(value)
        .expect("visual modifier enum must serialize")
        .as_str()
        .expect("visual modifier enum must serialize as text")
        .to_string()
}

pub fn visual_modifier_action_valid(
    item: &VideoItem,
    id: uuid::Uuid,
    action: VisualModifierChainAction,
) -> bool {
    edited_visual_modifier_chain(item, id, action).is_some()
}

pub fn visual_modifier_catalog(item: &VideoItem) -> Vec<VisualModifierChoice> {
    let Ok(state) = item.modifier_output_state() else {
        return Vec::new();
    };
    ModifierEffect::catalog()
        .filter_map(|effect| {
            let key = visual_modifier_key(&effect);
            let effect = effect.adapted_for(state)?;
            Some(VisualModifierChoice {
                key,
                label: effect.display_name(),
                search_text: std::iter::once(effect.display_name())
                    .chain(effect.keywords().iter().copied())
                    .collect::<Vec<_>>()
                    .join(" "),
            })
        })
        .collect()
}

impl InspectorController {
    pub fn set_visual_modifier_alpha_mask(
        &self,
        target: &InspectorTarget,
        id: uuid::Uuid,
        enabled: bool,
    ) -> Result<(), String> {
        let mut project = self.project.borrow_mut();
        let item = project
            .video_item_mut(video_address(target)?)
            .ok_or_else(|| "visual modifier item is no longer available".to_string())?;
        let target = VisualAlphaMaskTarget::Modifier(id);
        if enabled {
            if let Some(mask) = item.alpha_mask_mut(target) {
                if mask.enabled {
                    return Ok(());
                }
                mask.enabled = true;
            } else if !item.set_alpha_mask(target, Some(VisualAlphaMask::default())) {
                return Err("visual modifier is no longer available".to_string());
            }
        } else if item.alpha_mask(target).is_none() {
            return Ok(());
        } else if !item.set_alpha_mask(target, None) {
            return Err("visual modifier is no longer available".to_string());
        }
        shrimply_project::project::commit_edit(
            &project,
            if enabled {
                "add-alpha-mask"
            } else {
                "remove-alpha-mask"
            },
        );
        drop(project);
        refresh(&self.player_state);
        Ok(())
    }

    pub fn set_visual_modifier_enabled(
        &self,
        target: &InspectorTarget,
        id: uuid::Uuid,
        enabled: bool,
    ) -> Result<(), String> {
        let mut project = self.project.borrow_mut();
        let item = project
            .video_item_mut(video_address(target)?)
            .ok_or_else(|| "visual modifier item is no longer available".to_string())?;
        let index = item
            .modifiers
            .iter()
            .position(|modifier| modifier.id == id)
            .ok_or_else(|| "visual modifier is no longer available".to_string())?;
        if item.modifiers[index].enabled == enabled {
            return Ok(());
        }
        item.modifiers = visual_modifier_enabled_chain(item, id, enabled)
            .ok_or_else(|| "visual modifier cannot be toggled in this chain".to_string())?;
        shrimply_project::project::commit_edit(&project, "toggle-visual-modifier");
        drop(project);
        refresh(&self.player_state);
        Ok(())
    }

    pub fn reset_visual_modifier(
        &self,
        target: &InspectorTarget,
        id: uuid::Uuid,
        effect: serde_json::Value,
    ) -> Result<(), String> {
        let effect = serde_json::from_value(effect)
            .map_err(|error| format!("invalid visual modifier: {error}"))?;
        let mut project = self.project.borrow_mut();
        let item = project
            .video_item_mut(video_address(target)?)
            .ok_or_else(|| "visual modifier item is no longer available".to_string())?;
        let index = item
            .modifiers
            .iter()
            .position(|modifier| modifier.id == id)
            .ok_or_else(|| "visual modifier is no longer available".to_string())?;
        let mut modifiers = item.modifiers.clone();
        modifiers[index].effect = effect;
        modifiers[index].alpha_mask = None;
        if !modifier_chain_is_valid(item, &modifiers) {
            return Err("visual modifier reset would invalidate the chain".to_string());
        }
        item.modifiers = modifiers;
        shrimply_project::project::commit_edit(&project, "reset-visual-modifier");
        drop(project);
        refresh(&self.player_state);
        Ok(())
    }

    pub fn copy_visual_modifier(
        &self,
        target: &InspectorTarget,
        id: uuid::Uuid,
        clipboard: &shrimply_property_transfer::SharedClipboard,
    ) -> Result<String, String> {
        let project = self.project.borrow();
        let modifier = project
            .video_item(video_address(target)?)
            .and_then(|item| item.modifiers.iter().find(|modifier| modifier.id == id))
            .ok_or_else(|| "visual modifier is no longer available".to_string())?;
        let title = modifier.effect.display_name().to_string();
        clipboard.borrow_mut().copy_visual_modifier(modifier);
        drop(project);
        player_state::refresh_project(
            &self.player_state,
            ProjectChange {
                inspector: true,
                ..Default::default()
            },
        );
        Ok(title)
    }

    pub fn move_visual_modifier(
        &self,
        target: &InspectorTarget,
        id: uuid::Uuid,
        offset: isize,
    ) -> Result<(), String> {
        let action = match offset {
            -1 => VisualModifierChainAction::MoveUp,
            1 => VisualModifierChainAction::MoveDown,
            _ => return Err("visual modifier move must be one position".to_string()),
        };
        self.edit_visual_modifier_chain(target, id, action)
    }

    pub fn remove_visual_modifier(
        &self,
        target: &InspectorTarget,
        id: uuid::Uuid,
    ) -> Result<(), String> {
        let project = self.project.borrow();
        let item = project
            .video_item(video_address(target)?)
            .ok_or_else(|| "visual modifier item is no longer available".to_string())?;
        if !visual_modifier_action_valid(item, id, VisualModifierChainAction::Remove) {
            return Err("visual modifier removal would invalidate the chain".to_string());
        }
        let cached = item
            .modifiers
            .iter()
            .find(|modifier| modifier.id == id)
            .is_some_and(|modifier| {
                matches!(
                    modifier.effect,
                    ModifierEffect::Raster(ref effect)
                        if matches!(&**effect, RasterModifierEffect::Cache(_))
                )
            });
        drop(project);
        if cached {
            shrimply_video::modifier_cache::invalidate(id)?;
        }
        self.edit_visual_modifier_chain(target, id, VisualModifierChainAction::Remove)
    }

    fn edit_visual_modifier_chain(
        &self,
        target: &InspectorTarget,
        id: uuid::Uuid,
        action: VisualModifierChainAction,
    ) -> Result<(), String> {
        let mut project = self.project.borrow_mut();
        let item = project
            .video_item_mut(video_address(target)?)
            .ok_or_else(|| "visual modifier item is no longer available".to_string())?;
        item.modifiers = edited_visual_modifier_chain(item, id, action)
            .ok_or_else(|| "visual modifier action would invalidate the chain".to_string())?;
        shrimply_project::project::commit_edit(
            &project,
            if action == VisualModifierChainAction::Remove {
                "remove-visual-modifier"
            } else {
                "move-visual-modifier"
            },
        );
        drop(project);
        refresh(&self.player_state);
        Ok(())
    }

    pub fn add_visual_modifier(
        &self,
        target: &InspectorTarget,
        key: &str,
    ) -> Result<uuid::Uuid, String> {
        let address = video_address(target)?;
        let position = player_state::snapshot(&self.player_state).position;
        let revision = player_state::snapshot(&self.player_state).revision;
        let mut project = self.project.borrow_mut();
        let audio = self
            .audio_sampler
            .borrow_mut()
            .sample(&project, position, revision);
        let item = project
            .video_item(address)
            .ok_or_else(|| "video item is no longer available".to_string())?;
        let state = item.modifier_output_state()?;
        let effect = ModifierEffect::catalog()
            .find(|effect| visual_modifier_key(effect) == key)
            .and_then(|effect| effect.adapted_for(state))
            .ok_or_else(|| format!("visual modifier is not available: {key}"))?;
        let effect = configured_effect(&project, item, position, &audio, effect);
        let modifier = VisualModifier::new(effect);
        let id = modifier.id;
        let item = project
            .video_item_mut(address)
            .expect("validated video item must remain available");
        let mut modifiers = item.modifiers.clone();
        modifiers.push(modifier);
        if !modifier_chain_is_valid(item, &modifiers) {
            return Err("visual modifier is not valid in this chain".to_string());
        }
        item.modifiers = modifiers;
        shrimply_project::project::commit_edit(&project, "add-visual-modifier");
        drop(project);
        refresh(&self.player_state);
        Ok(id)
    }

    pub fn can_paste_visual_modifiers(
        &self,
        target: &InspectorTarget,
        clipboard: &shrimply_property_transfer::SharedClipboard,
    ) -> bool {
        let Ok(address) = video_address(target) else {
            return false;
        };
        clipboard
            .borrow()
            .can_append_modifiers(&self.project.borrow(), std::slice::from_ref(address))
    }

    pub fn paste_visual_modifiers(
        &self,
        target: &InspectorTarget,
        clipboard: &shrimply_property_transfer::SharedClipboard,
    ) -> Result<usize, String> {
        let address = video_address(target)?.clone();
        let mut project = self.project.borrow_mut();
        let result = clipboard
            .borrow()
            .append_modifiers(&mut project, std::slice::from_ref(&address));
        if !result.changed {
            return Ok(0);
        }
        shrimply_project::project::commit_edit(&project, "paste-item-modifiers");
        drop(project);
        player_state::refresh_project(
            &self.player_state,
            ProjectChange {
                video: result.video,
                inspector: true,
                ..Default::default()
            },
        );
        Ok(result.modifiers_added)
    }
}

fn configured_effect(
    project: &shrimply_project::project::Project,
    item: &VideoItem,
    position: Time,
    audio: &shrimply_evaluation::FrameAudioAnalysis,
    mut effect: ModifierEffect,
) -> ModifierEffect {
    let canvas = project.canvas_size;
    let canvas_size = glam::Vec2::new(canvas.width.max(1) as f32, canvas.height.max(1) as f32);
    let fallback = canvas_size * 0.5;
    let center = shrimply_evaluation::resolve_item_transform_with_audio(
        project,
        item,
        position,
        audio,
        &mut Default::default(),
    )
    .position;
    let center = if center.is_finite() { center } else { fallback };
    match &mut effect {
        ModifierEffect::Vector(effect) => {
            if let VectorModifierEffect::Transform(transform) = &mut **effect {
                **transform =
                    shrimply_video_modifiers::transform::TransformModifier::centered_at(center);
            }
        }
        ModifierEffect::Rasterize(rasterize) => {
            *rasterize = shrimply_video_modifiers::rasterize::RasterizeModifier::new(canvas_size);
        }
        ModifierEffect::Raster(effect) => {
            if let RasterModifierEffect::Transform(transform) = &mut **effect {
                **transform =
                    shrimply_video_modifiers::transform::TransformModifier::centered_at(center);
            }
        }
        ModifierEffect::Scene3d(_) | ModifierEffect::Vectorize(_) => {}
    }
    effect
}

fn visual_modifier_key(effect: &ModifierEffect) -> String {
    let value = serde_json::to_value(effect).expect("visual modifier catalog must serialize");
    let stage = value
        .get("stage")
        .and_then(serde_json::Value::as_str)
        .expect("visual modifier stage must serialize as text");
    value
        .get("effect")
        .and_then(|effect| effect.get("kind"))
        .and_then(serde_json::Value::as_str)
        .map_or_else(|| stage.to_string(), |kind| format!("{stage}:{kind}"))
}

fn video_address(target: &InspectorTarget) -> Result<&ItemAddress, String> {
    match target {
        InspectorTarget::Item(address @ ItemAddress::Video { .. }) => Ok(address),
        _ => Err("inspector target is not a video item".to_string()),
    }
}

pub fn edited_visual_modifier_chain(
    item: &VideoItem,
    id: uuid::Uuid,
    action: VisualModifierChainAction,
) -> Option<Vec<VisualModifier>> {
    let mut modifiers = item.modifiers.clone();
    let index = modifiers.iter().position(|modifier| modifier.id == id)?;
    match action {
        VisualModifierChainAction::MoveUp if index > 0 => modifiers.swap(index, index - 1),
        VisualModifierChainAction::MoveDown if index + 1 < modifiers.len() => {
            modifiers.swap(index, index + 1);
        }
        VisualModifierChainAction::Remove => {
            modifiers.remove(index);
        }
        _ => return None,
    }
    modifier_chain_is_valid(item, &modifiers).then_some(modifiers)
}

pub fn visual_modifier_enabled_chain(
    item: &VideoItem,
    id: uuid::Uuid,
    enabled: bool,
) -> Option<Vec<VisualModifier>> {
    let mut modifiers = item.modifiers.clone();
    modifiers
        .iter_mut()
        .find(|modifier| modifier.id == id)?
        .enabled = enabled;
    modifier_chain_is_valid(item, &modifiers).then_some(modifiers)
}

pub fn modifier_chain_is_valid(item: &VideoItem, modifiers: &[VisualModifier]) -> bool {
    let Ok(state) = item.modifier_output_state_for(modifiers) else {
        return false;
    };
    !item
        .compositing
        .alpha_mask
        .as_ref()
        .is_some_and(|mask| mask.enabled)
        || state.kind == VisualKind::Raster
}

fn refresh(state: &shrimply_state::player_state::SharedPlayerState) {
    player_state::refresh_project(
        state,
        ProjectChange {
            video: true,
            inspector: true,
            ..Default::default()
        },
    );
}
