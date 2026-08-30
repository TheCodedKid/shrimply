use crate::canvas::TimelinePainter;
use crate::canvas::{Color, Rect, Stroke, vec2};
use gtk::gdk;
use gtk::gdk::prelude::*;
use shrimply_skia_adw_ui::cursor::SoftwareCursor;

pub const DEFAULT_CURSOR_THEME_SIZE: i32 = 24;

pub fn software_cursor_from_name(name: &str, display: &gdk::Display) -> SoftwareCursor {
    let (name, hot_spot) = match name {
        "crosshair" => ("crosshair", vec2(15.0, 15.0)),
        "e-resize" => ("e-resize", vec2(25.0, 17.0)),
        "w-resize" => ("w-resize", vec2(8.0, 17.0)),
        "ew-resize" => ("ew-resize", vec2(16.0, 15.0)),
        "grabbing" => ("grabbing", vec2(15.0, 14.0)),
        _ => ("default", vec2(5.0, 5.0)),
    };
    let texture = gdk::Texture::from_resource(&format!("/org/gtk/libgdk/cursor/{name}"));
    let width = texture.width();
    let height = texture.height();
    let stride = usize::try_from(width).expect("positive cursor width") * 4;
    let mut bgra = vec![0; stride * usize::try_from(height).expect("positive cursor height")];
    texture.download(&mut bgra, stride);
    let mut rgba = Vec::with_capacity(bgra.len());
    for pixel in bgra.chunks_exact(4) {
        rgba.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
    }
    let theme_size = gtk::Settings::for_display(display).gtk_cursor_theme_size();
    let theme_size = if theme_size > 0 {
        theme_size
    } else {
        DEFAULT_CURSOR_THEME_SIZE
    };
    let scale = theme_size as f32 / width as f32;
    SoftwareCursor::from_rgba_premultiplied(
        &rgba,
        width as u32,
        height as u32,
        (hot_spot * scale).round(),
        vec2(
            (width as f32 * scale).round(),
            (height as f32 * scale).round(),
        ),
    )
    .expect("GTK system cursor must have valid pixels")
}

pub struct PlayheadStyle {
    pub ruler_height: f64,
    pub frame_y: Option<f64>,
    pub handle_width: f64,
    pub handle_height: f64,
    pub handle_top: f64,
    pub triangle_height: f64,
}

pub fn draw_playhead(
    painter: &TimelinePainter,
    playhead_x: f64,
    frame_width: f64,
    height: f64,
    color: Color,
    style: PlayheadStyle,
) {
    if height <= 0.0 {
        return;
    }

    let frame_width = frame_width.max(1.0);
    let frame_y = style.frame_y.unwrap_or(style.ruler_height - 5.0);
    let handle_left = playhead_x - style.handle_width / 2.0;
    let handle_bottom = style.handle_top + style.handle_height;
    let triangle_tip_y = handle_bottom + style.triangle_height;

    painter.rect_filled(
        rect(
            handle_left,
            style.handle_top,
            style.handle_width,
            style.handle_height,
        ),
        1,
        color,
    );
    painter.convex_polygon(
        &[
            vec2(handle_left as f32, handle_bottom as f32),
            vec2(
                (handle_left + style.handle_width) as f32,
                handle_bottom as f32,
            ),
            vec2(playhead_x as f32, triangle_tip_y as f32),
        ],
        color,
        Stroke::none(),
    );
    painter.rect_filled(rect(playhead_x, frame_y, frame_width, 4.0), 0, color);
    painter.rect_filled(
        rect(
            playhead_x - 1.0,
            triangle_tip_y,
            2.0,
            height - triangle_tip_y,
        ),
        0,
        color,
    );
}

fn rect(x: f64, y: f64, width: f64, height: f64) -> Rect {
    Rect::from_min_size(
        vec2(x as f32, y as f32),
        vec2(width.max(0.0) as f32, height.max(0.0) as f32),
    )
}
