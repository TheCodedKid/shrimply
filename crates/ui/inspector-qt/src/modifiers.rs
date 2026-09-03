use shrimply_inspector_core::VisualModifierPresentation;
use shrimply_preview_core::PreviewTarget;
use shrimply_video_modifiers::MODIFIER_PREVIEW_FACET;

use crate::item::{
    HeaderAction, HeaderButtonToggle, HeaderToggle, InspectorAction, InspectorItem,
    InspectorListItem,
};
use crate::section::InspectorSection;

mod alpha_outline;
mod bulge_pinch;
mod channel_mixer;
mod chroma_key;
mod chromatic_aberration;
mod color_correction;
mod colorize_duotone;
mod directional_blur;
mod displacement_map;
mod drop_shadow;
mod edge_detection;
mod emboss;
mod erode_dilate;
mod film_grain;
mod fisheye;
mod gaussian_blur;
mod glow_bloom;
mod hsv;
mod invert;
mod lens_distortion;
mod luma_key;
mod mirror;
mod opacity;
mod pixelate_mosaic;
mod posterize;
mod radial_blur;
mod sharpen;
mod text_mask;
mod threshold;
mod transform;
mod twirl;
mod vignette;
mod wave_ripple;
mod zoom_blur;

pub(crate) fn items(
    modifiers: &[VisualModifierPresentation],
) -> impl Iterator<Item = InspectorListItem> + '_ {
    modifiers.iter().map(item)
}

fn item(modifier: &VisualModifierPresentation) -> InspectorListItem {
    let id = modifier.id;
    let mut section = modifier
        .body
        .as_ref()
        .map_or_else(InspectorSection::default, |body| match body {
            shrimply_inspector_core::VisualModifierBodyPresentation::AlphaOutline(value) => {
                alpha_outline::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::BulgePinch(value) => {
                bulge_pinch::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::ChannelMixer(value) => {
                channel_mixer::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::ChromaKey(value) => {
                chroma_key::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::ChromaticAberration(value) => {
                chromatic_aberration::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::ColorCorrection(value) => {
                color_correction::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::ColorizeDuotone(value) => {
                colorize_duotone::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::DirectionalBlur(value) => {
                directional_blur::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::DisplacementMap(value) => {
                displacement_map::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::DropShadow(value) => {
                drop_shadow::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::EdgeDetection(value) => {
                edge_detection::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::Emboss(value) => {
                emboss::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::ErodeDilate(value) => {
                erode_dilate::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::FilmGrain(value) => {
                film_grain::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::Fisheye(value) => {
                fisheye::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::GaussianBlur(value) => {
                gaussian_blur::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::GlowBloom(value) => {
                glow_bloom::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::Hsv(value) => {
                hsv::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::Invert(value) => {
                invert::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::LensDistortion(value) => {
                lens_distortion::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::LumaKey(value) => {
                luma_key::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::Mirror(value) => {
                mirror::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::Opacity(value) => {
                opacity::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::PixelateMosaic(value) => {
                pixelate_mosaic::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::Posterize(value) => {
                posterize::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::RadialBlur(value) => {
                radial_blur::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::Sharpen(value) => {
                sharpen::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::TextMask(value) => {
                text_mask::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::Threshold(value) => {
                threshold::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::Transform(value) => {
                transform::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::Twirl(value) => {
                twirl::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::Vignette(value) => {
                vignette::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::WaveRipple(value) => {
                wave_ripple::section(value)
            }
            shrimply_inspector_core::VisualModifierBodyPresentation::ZoomBlur(value) => {
                zoom_blur::section(value)
            }
        });
    if let Some(mask) = &modifier.alpha_mask {
        section
            .controls
            .extend(mask.section.controls.iter().cloned());
    }
    let mut item = InspectorItem::new(format!("modifier:{id}"), modifier.title, section)
        .reset(InspectorAction::ResetVisualModifier {
            id,
            effect: modifier.default_effect.clone(),
        })
        .toggle(HeaderToggle {
            active: modifier.enabled,
            tooltip: if modifier.enabled {
                "Disable modifier"
            } else {
                "Enable modifier"
            },
            activate: InspectorAction::SetVisualModifierEnabled {
                id,
                enabled: !modifier.enabled,
            },
        })
        .actions(vec![
            HeaderAction {
                icon: "edit-copy-symbolic",
                tooltip: "Copy",
                sensitive: true,
                activate: InspectorAction::CopyVisualModifier { id },
            },
            HeaderAction {
                icon: "go-up-symbolic",
                tooltip: "Move up",
                sensitive: modifier.can_move_up,
                activate: InspectorAction::MoveVisualModifier { id, offset: -1 },
            },
            HeaderAction {
                icon: "go-down-symbolic",
                tooltip: "Move down",
                sensitive: modifier.can_move_down,
                activate: InspectorAction::MoveVisualModifier { id, offset: 1 },
            },
            HeaderAction {
                icon: "user-trash-symbolic",
                tooltip: "Remove",
                sensitive: modifier.can_remove,
                activate: InspectorAction::RemoveVisualModifier { id },
            },
        ])
        .preview_target(PreviewTarget::new(id, MODIFIER_PREVIEW_FACET));
    if let Some(mask) = &modifier.alpha_mask {
        item = item.button_toggle(HeaderButtonToggle {
            icon: "select-symbolic",
            active: mask.active,
            tooltip: "Mask",
            activate: InspectorAction::SetVisualModifierAlphaMask {
                id,
                enabled: !mask.active,
            },
        });
    }
    item.boxed()
}
