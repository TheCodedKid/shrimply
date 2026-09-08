use super::*;

pub(super) fn draw_onion_skins(
    canvas: &shrimply_preview_provider_skia::PreviewCanvas,
    frames: &[Option<PaintOnionFrame>; 2],
    cache: &mut shrimply_paint_skia::PaintCache,
    state: &PaintPreviewState,
) {
    for (frame, color) in [
        (
            state.onion_previous.then_some(&frames[0]),
            Color::<u8>::new(240, 71, 71, 148),
        ),
        (
            state.onion_next.then_some(&frames[1]),
            Color::<u8>::new(71, 143, 255, 148),
        ),
    ] {
        let Some(frame) = frame.and_then(Option::as_ref) else {
            continue;
        };
        let palette: Vec<_> = frame
            .palette
            .iter()
            .map(|entry| shrimply_paint_skia::ResolvedPaintPaletteEntry {
                color,
                texture: entry.texture.clone(),
            })
            .collect();
        let prepared = shrimply_paint_skia::prepare_frame(
            cache,
            (&frame.drawing, frame.revision),
            &frame.stroke_options,
            frame.fill_options,
            &frame.path_offsets,
            frame.mapping.stroke,
            frame.mapping.source_size,
        );
        draw_paint_layer(
            canvas,
            PreviewPaintRenderer {
                mapping: frame.mapping,
                stroke_options: &frame.stroke_options,
                fill_options: frame.fill_options,
                path_offsets: &frame.path_offsets,
                path_effect: frame.path_effect.as_ref(),
                canvas_operations: &frame.canvas_operations,
                opacity: frame.opacity,
                blend_mode: frame.blend_mode,
                palette: &palette,
            },
            |canvas| {
                shrimply_paint_skia::draw(
                    cache,
                    canvas,
                    &prepared,
                    shrimply_paint_skia::ResolvedPaintAppearance {
                        palette: &palette,
                        reveal: None,
                    },
                    frame.path_effect.as_ref(),
                )
                .expect("paint onion skin could not be rendered");
            },
        );
    }
}
