use skia_safe::{
    AlphaType, Canvas, ColorType, CubicResampler, Data, Image, ImageInfo, Paint, Rect as SkiaRect,
    images,
};

use crate::Vec2;

pub struct SoftwareCursor {
    image: Image,
    hot_spot: Vec2,
    size: Vec2,
}

impl SoftwareCursor {
    pub fn from_rgba_premultiplied(
        pixels: &[u8],
        width: u32,
        height: u32,
        hot_spot: Vec2,
        size: Vec2,
    ) -> Option<Self> {
        let width = i32::try_from(width).ok()?;
        let height = i32::try_from(height).ok()?;
        let row_bytes = usize::try_from(width).ok()?.checked_mul(4)?;
        let info = ImageInfo::new(
            (width, height),
            ColorType::RGBA8888,
            AlphaType::Premul,
            None,
        );
        let image = images::raster_from_data(&info, Data::new_copy(pixels), row_bytes)?;
        Some(Self {
            image,
            hot_spot,
            size,
        })
    }

    pub fn draw(&self, canvas: &Canvas, position: Vec2) {
        let top_left = position.round() - self.hot_spot;
        canvas.draw_image_rect_with_sampling_options(
            &self.image,
            None,
            SkiaRect::from_xywh(top_left.x, top_left.y, self.size.x, self.size.y),
            CubicResampler::mitchell(),
            &Paint::default(),
        );
    }
}
