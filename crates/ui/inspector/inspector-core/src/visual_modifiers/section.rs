use super::{InspectorSection, VisualModifierBodyPresentation};

impl VisualModifierBodyPresentation {
    pub fn section(&self) -> InspectorSection {
        match self {
            Self::AlphaOutline(section)
            | Self::BulgePinch(section)
            | Self::Cache(section)
            | Self::ChannelMixer(section)
            | Self::ChromaKey(section)
            | Self::ChromaticAberration(section)
            | Self::ColorCorrection(section)
            | Self::ColorizeDuotone(section)
            | Self::CornerPin(section)
            | Self::Crop(section)
            | Self::DirectionalBlur(section)
            | Self::DisplacementMap(section)
            | Self::Dithering(section)
            | Self::DropShadow(section)
            | Self::EdgeDetection(section)
            | Self::Emboss(section)
            | Self::ErodeDilate(section)
            | Self::FilmGrain(section)
            | Self::Fisheye(section)
            | Self::GaussianBlur(section)
            | Self::GlowBloom(section)
            | Self::Ground(section)
            | Self::Halftone(section)
            | Self::Hsv(section)
            | Self::Invert(section)
            | Self::Kaleidoscope(section)
            | Self::Kuwahara(section)
            | Self::LensDistortion(section)
            | Self::LumaKey(section)
            | Self::Mask(section)
            | Self::Mirror(section)
            | Self::Object3d(section)
            | Self::PointLight(section)
            | Self::PathOffset(section)
            | Self::PixelateMosaic(section)
            | Self::Posterize(section)
            | Self::RadialBlur(section)
            | Self::Rasterize(section)
            | Self::Sam2(section)
            | Self::Repeat(section)
            | Self::Sampling(section)
            | Self::ScanlinesCrt(section)
            | Self::Shape3d(section)
            | Self::ShakyPath(section)
            | Self::Sharpen(section)
            | Self::SunLight(section)
            | Self::TextMask(section)
            | Self::Text3d(section)
            | Self::TextureBounds(section)
            | Self::Threshold(section)
            | Self::TransparentFill(section)
            | Self::Twirl(section)
            | Self::Vectorize(section)
            | Self::Vignette(section)
            | Self::WaveRipple(section)
            | Self::ZoomBlur(section) => section.clone(),
            Self::Opacity(value) => InspectorSection {
                controls: vec![value.opacity.clone()],
            },
            Self::Transform(value) => InspectorSection {
                controls: vec![
                    value.position.clone(),
                    value.anchor.clone(),
                    value.scale.clone(),
                    value.shear.clone(),
                    value.rotation.clone(),
                ],
            },
        }
    }
}
