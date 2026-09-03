use shrimply_core::modifier_model::ModifierModel;
use shrimply_project::project::{
    AlphaMaskShape, ItemAddress, Time, VideoItem, VisualAlphaMask, VisualAlphaMaskTarget,
    VisualModifier,
};
use shrimply_state::player_state::{self, ProjectChange};
use shrimply_video_modifiers::{
    ModifierEffect, RasterModifierEffect, VectorModifierEffect, VisualKind,
    scene_3d::Scene3dModifierEffect,
};

use crate::{
    ControlKind, InspectorControl, InspectorController, InspectorRuntime, InspectorSection,
    InspectorTarget, LayeredState, NumberSpec,
};

mod alpha_outline;
mod bulge_pinch;
mod cache;
mod channel_mixer;
mod chroma_key;
mod chromatic_aberration;
mod color_correction;
mod colorize_duotone;
mod corner_pin;
mod crop;
mod directional_blur;
mod displacement_map;
mod dithering;
mod drop_shadow;
mod edge_detection;
mod emboss;
mod erode_dilate;
mod film_grain;
mod fisheye;
mod gaussian_blur;
mod glow_bloom;
mod ground;
mod halftone;
mod hsv;
mod invert;
mod kaleidoscope;
mod kuwahara;
mod lens_distortion;
mod luma_key;
mod mask;
mod mirror;
mod object_3d;
mod opacity;
mod path_offset;
mod pixelate_mosaic;
mod point_light;
mod posterize;
mod radial_blur;
mod rasterize;
mod repeat;
mod sam2;
mod sampling;
mod scanlines_crt;
mod shaky_path;
mod shape_3d;
mod sharpen;
mod sun_light;
mod text_3d;
mod text_mask;
mod texture_bounds;
mod threshold;
mod transform;
mod transparent_fill;
mod twirl;
mod vectorize;
mod vignette;
mod wave_ripple;
mod zoom_blur;

pub use cache::visual_cache_status;
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
    AlphaOutline(InspectorSection),
    BulgePinch(InspectorSection),
    Cache(InspectorSection),
    ChannelMixer(InspectorSection),
    ChromaKey(InspectorSection),
    ChromaticAberration(InspectorSection),
    ColorCorrection(InspectorSection),
    ColorizeDuotone(InspectorSection),
    CornerPin(InspectorSection),
    Crop(InspectorSection),
    DirectionalBlur(InspectorSection),
    DisplacementMap(InspectorSection),
    Dithering(InspectorSection),
    DropShadow(InspectorSection),
    EdgeDetection(InspectorSection),
    Emboss(InspectorSection),
    ErodeDilate(InspectorSection),
    FilmGrain(InspectorSection),
    Fisheye(InspectorSection),
    GaussianBlur(InspectorSection),
    GlowBloom(InspectorSection),
    Ground(InspectorSection),
    Halftone(InspectorSection),
    Hsv(InspectorSection),
    Invert(InspectorSection),
    LensDistortion(InspectorSection),
    LumaKey(InspectorSection),
    Mirror(InspectorSection),
    Opacity(Box<OpacityModifierPresentation>),
    PixelateMosaic(InspectorSection),
    Posterize(InspectorSection),
    RadialBlur(InspectorSection),
    Sharpen(InspectorSection),
    TextMask(InspectorSection),
    Threshold(InspectorSection),
    Transform(Box<TransformModifierPresentation>),
    Twirl(InspectorSection),
    Vectorize(InspectorSection),
    Vignette(InspectorSection),
    WaveRipple(InspectorSection),
    ZoomBlur(InspectorSection),
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
                    VectorModifierEffect::Hsv(value) => Some(
                        VisualModifierBodyPresentation::Hsv(hsv::presentation(
                            value, index, runtime,
                        )),
                    ),
                    VectorModifierEffect::TextMask(value) => {
                        Some(VisualModifierBodyPresentation::TextMask(
                            text_mask::presentation(value, index, modifier.id, runtime),
                        ))
                    }
                    _ => None,
                },
                ModifierEffect::Vectorize(value) => Some(
                    VisualModifierBodyPresentation::Vectorize(vectorize::presentation(
                        value,
                        index,
                        modifier.id,
                        runtime,
                    )),
                ),
                ModifierEffect::Raster(effect) => match &**effect {
                    RasterModifierEffect::AlphaOutline(value) => {
                        Some(VisualModifierBodyPresentation::AlphaOutline(
                            alpha_outline::presentation(value, index, runtime),
                        ))
                    }
                    RasterModifierEffect::BulgePinch(value) => {
                        Some(VisualModifierBodyPresentation::BulgePinch(
                            bulge_pinch::presentation(value, index, runtime),
                        ))
                    }
                    RasterModifierEffect::Cache(value) => {
                        Some(VisualModifierBodyPresentation::Cache(cache::presentation(
                            value,
                            index,
                            modifier.id,
                            runtime,
                        )))
                    }
                    RasterModifierEffect::ChannelMixer(value) => {
                        Some(VisualModifierBodyPresentation::ChannelMixer(
                            channel_mixer::presentation(value, index, runtime),
                        ))
                    }
                    RasterModifierEffect::ChromaKey(value) => {
                        Some(VisualModifierBodyPresentation::ChromaKey(
                            chroma_key::presentation(value, index, runtime),
                        ))
                    }
                    RasterModifierEffect::ChromaticAberration(value) => {
                        Some(VisualModifierBodyPresentation::ChromaticAberration(
                            chromatic_aberration::presentation(value, index, runtime),
                        ))
                    }
                    RasterModifierEffect::ColorCorrection(value) => {
                        Some(VisualModifierBodyPresentation::ColorCorrection(
                            color_correction::presentation(value, index, runtime),
                        ))
                    }
                    RasterModifierEffect::ColorizeDuotone(value) => {
                        Some(VisualModifierBodyPresentation::ColorizeDuotone(
                            colorize_duotone::presentation(value, index, runtime),
                        ))
                    }
                    RasterModifierEffect::CornerPin(value) => {
                        Some(VisualModifierBodyPresentation::CornerPin(
                            corner_pin::presentation(value, index, modifier.id, runtime),
                        ))
                    }
                    RasterModifierEffect::Crop(value) => {
                        Some(VisualModifierBodyPresentation::Crop(crop::presentation(
                            value,
                            index,
                            modifier.id,
                            runtime,
                        )))
                    }
                    RasterModifierEffect::DirectionalBlur(value) => {
                        Some(VisualModifierBodyPresentation::DirectionalBlur(
                            directional_blur::presentation(value, index, runtime),
                        ))
                    }
                    RasterModifierEffect::DisplacementMap(value) => {
                        Some(VisualModifierBodyPresentation::DisplacementMap(
                            displacement_map::presentation(value, index, runtime),
                        ))
                    }
                    RasterModifierEffect::Dithering(value) => {
                        Some(VisualModifierBodyPresentation::Dithering(
                            dithering::presentation(value, index, modifier.id, runtime),
                        ))
                    }
                    RasterModifierEffect::DropShadow(value) => {
                        Some(VisualModifierBodyPresentation::DropShadow(
                            drop_shadow::presentation(value, index, runtime),
                        ))
                    }
                    RasterModifierEffect::EdgeDetection(value) => {
                        Some(VisualModifierBodyPresentation::EdgeDetection(
                            edge_detection::presentation(value, index, runtime),
                        ))
                    }
                    RasterModifierEffect::Emboss(value) => {
                        Some(VisualModifierBodyPresentation::Emboss(
                            emboss::presentation(value, index, runtime),
                        ))
                    }
                    RasterModifierEffect::ErodeDilate(value) => {
                        Some(VisualModifierBodyPresentation::ErodeDilate(
                            erode_dilate::presentation(value, index, runtime),
                        ))
                    }
                    RasterModifierEffect::FilmGrain(value) => {
                        Some(VisualModifierBodyPresentation::FilmGrain(
                            film_grain::presentation(value, index, runtime),
                        ))
                    }
                    RasterModifierEffect::Fisheye(value) => {
                        Some(VisualModifierBodyPresentation::Fisheye(
                            fisheye::presentation(value, index, runtime),
                        ))
                    }
                    RasterModifierEffect::GaussianBlur(value) => {
                        Some(VisualModifierBodyPresentation::GaussianBlur(
                            gaussian_blur::presentation(value, index, runtime),
                        ))
                    }
                    RasterModifierEffect::GlowBloom(value) => {
                        Some(VisualModifierBodyPresentation::GlowBloom(
                            glow_bloom::presentation(value, index, runtime),
                        ))
                    }
                    RasterModifierEffect::Halftone(value) => {
                        Some(VisualModifierBodyPresentation::Halftone(
                            halftone::presentation(value, index, runtime),
                        ))
                    }
                    RasterModifierEffect::Invert(value) => {
                        Some(VisualModifierBodyPresentation::Invert(
                            invert::presentation(value, index, runtime),
                        ))
                    }
                    RasterModifierEffect::LensDistortion(value) => {
                        Some(VisualModifierBodyPresentation::LensDistortion(
                            lens_distortion::presentation(value, index, runtime),
                        ))
                    }
                    RasterModifierEffect::LumaKey(value) => {
                        Some(VisualModifierBodyPresentation::LumaKey(
                            luma_key::presentation(value, index, modifier.id, runtime),
                        ))
                    }
                    RasterModifierEffect::Mirror(value) => {
                        Some(VisualModifierBodyPresentation::Mirror(
                            mirror::presentation(value, index, modifier.id, runtime),
                        ))
                    }
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
                    RasterModifierEffect::PixelateMosaic(value) => {
                        Some(VisualModifierBodyPresentation::PixelateMosaic(
                            pixelate_mosaic::presentation(value, index, runtime),
                        ))
                    }
                    RasterModifierEffect::Posterize(value) => {
                        Some(VisualModifierBodyPresentation::Posterize(
                            posterize::presentation(value, index, runtime),
                        ))
                    }
                    RasterModifierEffect::RadialBlur(value) => {
                        Some(VisualModifierBodyPresentation::RadialBlur(
                            radial_blur::presentation(value, index, runtime),
                        ))
                    }
                    RasterModifierEffect::Sharpen(value) => {
                        Some(VisualModifierBodyPresentation::Sharpen(
                            sharpen::presentation(value, index, runtime),
                        ))
                    }
                    RasterModifierEffect::Threshold(value) => {
                        Some(VisualModifierBodyPresentation::Threshold(
                            threshold::presentation(value, index, runtime),
                        ))
                    }
                    RasterModifierEffect::Twirl(value) => {
                        Some(VisualModifierBodyPresentation::Twirl(twirl::presentation(
                            value, index, runtime,
                        )))
                    }
                    RasterModifierEffect::Vignette(value) => {
                        Some(VisualModifierBodyPresentation::Vignette(
                            vignette::presentation(value, index, runtime),
                        ))
                    }
                    RasterModifierEffect::WaveRipple(value) => {
                        Some(VisualModifierBodyPresentation::WaveRipple(
                            wave_ripple::presentation(value, index, runtime),
                        ))
                    }
                    RasterModifierEffect::ZoomBlur(value) => {
                        Some(VisualModifierBodyPresentation::ZoomBlur(
                            zoom_blur::presentation(value, index, runtime),
                        ))
                    }
                    _ => None,
                },
                ModifierEffect::Scene3d(effect) => match &**effect {
                    Scene3dModifierEffect::Ground(value) => {
                        Some(VisualModifierBodyPresentation::Ground(ground::presentation(
                            value,
                            index,
                            modifier.id,
                            runtime,
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

pub(crate) fn visual_modifier_color<'a>(
    item: &'a VideoItem,
    path: &str,
    timeline_id: uuid::Uuid,
) -> Option<&'a shrimply_core::timeline_value::TimelineValue<shrimply_core::Color<u8>>> {
    let (modifier, field) = visual_modifier_at_path(item, path)?;
    let ModifierEffect::Raster(effect) = &modifier.effect else {
        return None;
    };
    let timeline = match (&**effect, field) {
        (RasterModifierEffect::AlphaOutline(value), "effect/effect/config/color") => &value.color,
        (RasterModifierEffect::ChromaKey(value), "effect/effect/config/key_color") => {
            &value.key_color
        }
        (RasterModifierEffect::EdgeDetection(value), "effect/effect/config/edge_color") => {
            &value.edge_color
        }
        (RasterModifierEffect::EdgeDetection(value), "effect/effect/config/background_color") => {
            &value.background_color
        }
        (RasterModifierEffect::ColorizeDuotone(value), "effect/effect/config/shadow_color") => {
            &value.shadow_color
        }
        (RasterModifierEffect::ColorizeDuotone(value), "effect/effect/config/highlight_color") => {
            &value.highlight_color
        }
        (RasterModifierEffect::DropShadow(value), "effect/effect/config/color") => &value.color,
        (RasterModifierEffect::Dithering(value), field) => {
            return dithering::palette_color(value, field, timeline_id);
        }
        (RasterModifierEffect::Threshold(value), "effect/effect/config/low_color") => {
            &value.low_color
        }
        (RasterModifierEffect::Threshold(value), "effect/effect/config/high_color") => {
            &value.high_color
        }
        _ => return None,
    };
    (timeline.id == timeline_id).then_some(timeline)
}

pub(crate) fn visual_modifier_number<'a>(
    item: &'a VideoItem,
    path: &str,
    id: uuid::Uuid,
) -> Option<&'a shrimply_core::timeline_value::TimelineValue<f32>> {
    visual_modifier_at_path(item, path)?.0.number(id)
}

pub(crate) fn visual_modifier_matches(item: &VideoItem, path: &str, id: uuid::Uuid) -> bool {
    visual_modifier_at_path(item, path).is_some_and(|(modifier, _)| modifier.id == id)
}

pub(crate) fn visual_modifier_vector2<'a>(
    item: &'a VideoItem,
    path: &str,
    id: uuid::Uuid,
) -> Option<&'a shrimply_core::timeline_value::TimelineValue<glam::Vec2>> {
    visual_modifier_at_path(item, path)?.0.number2(id)
}

pub(crate) fn visual_modifier_vector3<'a>(
    item: &'a VideoItem,
    path: &str,
    id: uuid::Uuid,
) -> Option<&'a shrimply_core::timeline_value::TimelineValue<glam::Vec3>> {
    visual_modifier_at_path(item, path)?.0.effect.number3(id)
}

pub(crate) fn set_visual_modifier_field(
    item: &mut VideoItem,
    path: &str,
    text: &str,
) -> Option<Result<bool, String>> {
    vectorize::set_field(item, path, text)
}

pub(crate) fn erode_dilate_operation<'a>(
    item: &'a VideoItem,
    path: &str,
    id: uuid::Uuid,
) -> Option<
    &'a shrimply_core::timeline_value::TimelineValue<
        shrimply_video_modifiers::erode_dilate::ErodeDilateOperation,
    >,
> {
    let (modifier, field) = visual_modifier_at_path(item, path)?;
    if field != "effect/effect/config/operation" {
        return None;
    }
    let ModifierEffect::Raster(effect) = &modifier.effect else {
        return None;
    };
    let RasterModifierEffect::ErodeDilate(value) = &**effect else {
        return None;
    };
    (value.operation.id == id).then_some(&value.operation)
}

pub(crate) fn halftone_mode<'a>(
    item: &'a VideoItem,
    path: &str,
    id: uuid::Uuid,
) -> Option<
    &'a shrimply_core::timeline_value::TimelineValue<
        shrimply_video_modifiers::halftone::HalftoneMode,
    >,
> {
    halftone::mode(item, path, id)
}

pub(crate) fn dithering_pattern<'a>(
    item: &'a VideoItem,
    path: &str,
    id: uuid::Uuid,
) -> Option<
    &'a shrimply_core::timeline_value::TimelineValue<
        shrimply_video_modifiers::dithering::DitheringPattern,
    >,
> {
    dithering::pattern(item, path, id)
}

pub(crate) fn dithering_color_mode<'a>(
    item: &'a VideoItem,
    path: &str,
    id: uuid::Uuid,
) -> Option<
    &'a shrimply_core::timeline_value::TimelineValue<
        shrimply_video_modifiers::dithering::DitheringColorMode,
    >,
> {
    dithering::color_mode(item, path, id)
}

pub(crate) fn visual_modifier_at_path<'a, 'b>(
    item: &'a VideoItem,
    path: &'b str,
) -> Option<(&'a VisualModifier, &'b str)> {
    let (index, field) = path.strip_prefix("/modifiers/")?.split_once('/')?;
    Some((item.modifiers.get(index.parse::<usize>().ok()?)?, field))
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
    label: impl Into<String>,
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
        .timeline(
            timeline.id,
            crate::transform::scalar_graph(timeline, value, runtime),
        )
        .live_commit("visual-modifier-value");
    if rotating {
        control.rotating_icon("rotation.svg", 0.0)
    } else {
        control
    }
}

pub(super) fn modifier_vector2_control(
    path: String,
    label: impl Into<String>,
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
        .timeline(
            timeline.id,
            crate::transform::vector_speed_graph(timeline, runtime),
        )
        .live_commit("visual-modifier-vector");
    if lock { control.lock() } else { control }
}

pub(super) fn modifier_vector3_control(
    path: String,
    label: impl Into<String>,
    timeline: &shrimply_core::timeline_value::TimelineValue<glam::Vec3>,
    runtime: InspectorRuntime,
    number: NumberSpec,
    lock: bool,
    rotating: bool,
) -> InspectorControl {
    let value = timeline.value_at(runtime.local_time.unwrap_or(Time::ZERO));
    let control = InspectorControl::new(ControlKind::LayeredVector3, path.clone(), label)
        .components(vec![
            value.x.to_string(),
            value.y.to_string(),
            value.z.to_string(),
        ])
        .number(number)
        .width_characters(5)
        .prefixes(["X", "Y", "Z"])
        .layered(path, LayeredState::from(timeline))
        .timeline(timeline.id, vector3_speed_graph(timeline, runtime))
        .live_commit("visual-modifier-vector");
    let control = if lock { control.lock() } else { control };
    if rotating {
        control.rotating_icon("rotation.svg", 0.0)
    } else {
        control
    }
}

pub(super) fn modifier_color_control(
    path: String,
    label: impl Into<String>,
    timeline: &shrimply_core::timeline_value::TimelineValue<shrimply_core::Color<u8>>,
    runtime: InspectorRuntime,
) -> InspectorControl {
    let value = timeline.value_at(runtime.local_time.unwrap_or(Time::ZERO));
    InspectorControl::new(ControlKind::LayeredColor, path.clone(), label)
        .components(vec![
            value.r.to_string(),
            value.g.to_string(),
            value.b.to_string(),
            value.a.to_string(),
        ])
        .layered(path, LayeredState::from(timeline))
        .timeline(timeline.id, color_speed_graph(timeline, runtime))
        .live_commit("visual-modifier-color")
}

pub(super) fn modifier_boolean_control(
    path: String,
    label: impl Into<String>,
    value: bool,
    commit: &'static str,
) -> InspectorControl {
    InspectorControl::new(ControlKind::Boolean, path, label)
        .value(value.to_string())
        .immediate_commit(commit)
}

pub(super) fn modifier_text_control(
    path: String,
    label: impl Into<String>,
    timeline: &shrimply_core::timeline_value::TimelineValue<String>,
    runtime: InspectorRuntime,
) -> InspectorControl {
    let value = timeline.value_at(runtime.local_time.unwrap_or(Time::ZERO));
    InspectorControl::new(ControlKind::LayeredText, path.clone(), label)
        .value(value)
        .layered(path, LayeredState::from(timeline))
        .timeline(timeline.id, text_speed_graph(timeline, runtime))
        .live_commit("edit-3d-text")
}

fn text_speed_graph(
    timeline: &shrimply_core::timeline_value::TimelineValue<String>,
    runtime: InspectorRuntime,
) -> Option<crate::ScalarGraph> {
    let shrimply_core::timeline_value::TimelineBase::Keyframes(keyframes) = &timeline.base else {
        return None;
    };
    let points = keyframes
        .iter()
        .map(|keyframe| crate::GraphPoint {
            time: keyframe.time,
            value: 0.0,
        })
        .collect();
    let segments = keyframes
        .windows(2)
        .filter_map(|pair| {
            let seconds = pair[1].time.signed_sub(pair[0].time).as_secs_f64();
            (seconds > f64::EPSILON).then(|| {
                let speed = shrimply_core::timeline_value::text_edit_count(
                    &pair[0].value,
                    &pair[1].value,
                    pair[0].text_interpolation_to_next,
                ) as f64
                    / seconds;
                crate::GraphSegment {
                    owner_id: pair[0].id,
                    start: pair[0].time,
                    end: pair[1].time,
                    start_value: speed,
                    end_value: speed,
                    interpolation: shrimply_core::timeline_value::Interpolation::KEYFRAME
                        .iter()
                        .position(|candidate| *candidate == pair[0].interpolation_to_next)
                        .expect("text interpolation must be available"),
                }
            })
        })
        .collect();
    Some(crate::ScalarGraph {
        points,
        segments,
        range: runtime.keyframe_range.unwrap_or((Time::ZERO, Time::ZERO)),
        frame_step: runtime.frame_step,
        playhead: runtime.keyframe_playhead.unwrap_or(Time::ZERO),
    })
}

pub(crate) fn vector3_speed_graph(
    timeline: &shrimply_core::timeline_value::TimelineValue<glam::Vec3>,
    runtime: InspectorRuntime,
) -> Option<crate::ScalarGraph> {
    speed_graph(timeline, runtime, |left, right| (*right - *left).length())
}

pub(crate) fn color_speed_graph(
    timeline: &shrimply_core::timeline_value::TimelineValue<shrimply_core::Color<u8>>,
    runtime: InspectorRuntime,
) -> Option<crate::ScalarGraph> {
    speed_graph(timeline, runtime, |left, right| {
        left.oklaba_distance(*right)
    })
}

fn speed_graph<T>(
    timeline: &shrimply_core::timeline_value::TimelineValue<T>,
    runtime: InspectorRuntime,
    distance: impl Fn(&T, &T) -> f32,
) -> Option<crate::ScalarGraph>
where
    T: shrimply_core::timeline_value::TimelineVector
        + serde::Serialize
        + serde::de::DeserializeOwned,
{
    let shrimply_core::timeline_value::TimelineBase::Keyframes(keyframes) = &timeline.base else {
        return None;
    };
    let points = keyframes
        .iter()
        .map(|keyframe| crate::GraphPoint {
            time: shrimply_core::timeline_value::TimelineKeyframe::time(keyframe),
            value: 0.0,
        })
        .collect();
    let segments = keyframes
        .windows(2)
        .filter_map(|pair| {
            let start = shrimply_core::timeline_value::TimelineKeyframe::time(&pair[0]);
            let end = shrimply_core::timeline_value::TimelineKeyframe::time(&pair[1]);
            let seconds = end.signed_sub(start).as_secs_f64();
            (seconds > f64::EPSILON).then(|| {
                let speed = f64::from(distance(
                    shrimply_core::timeline_value::TimelineKeyframe::value(&pair[0]),
                    shrimply_core::timeline_value::TimelineKeyframe::value(&pair[1]),
                )) / seconds;
                crate::GraphSegment {
                    owner_id: shrimply_core::timeline_value::TimelineKeyframe::id(&pair[0]),
                    start,
                    end,
                    start_value: speed,
                    end_value: speed,
                    interpolation: shrimply_core::timeline_value::Interpolation::KEYFRAME
                        .iter()
                        .position(|candidate| *candidate == pair[0].interpolation_to_next)
                        .expect("modifier interpolation must be available"),
                }
            })
        })
        .collect();
    Some(crate::ScalarGraph {
        points,
        segments,
        range: runtime.keyframe_range.unwrap_or((Time::ZERO, Time::ZERO)),
        frame_step: runtime.frame_step,
        playhead: runtime.keyframe_playhead.unwrap_or(Time::ZERO),
    })
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
