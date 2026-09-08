use shrimply_project_document::project::CanvasSize;
use skia_safe::Image;

/// Read a completed compositor image as tightly packed, straight-alpha RGBA.
pub(super) fn rgba(image: &Image, size: CanvasSize) -> Result<Vec<u8>, String> {
    if image.width() != size.width as i32 || image.height() != size.height as i32 {
        return Err("Captured image does not match the requested canvas size".into());
    }
    let info = skia_safe::ImageInfo::new(
        image.dimensions(),
        skia_safe::ColorType::RGBA8888,
        skia_safe::AlphaType::Unpremul,
        None,
    );
    let stride = info.min_row_bytes();
    let mut pixels = vec![0_u8; info.compute_byte_size(stride)];
    if !image.read_pixels(
        &info,
        &mut pixels,
        stride,
        (0, 0),
        skia_safe::image::CachingHint::Disallow,
    ) {
        return Err("Could not read the captured frame".into());
    }
    Ok(pixels)
}
