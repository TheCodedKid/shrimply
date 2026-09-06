use super::*;
#[allow(clippy::too_many_arguments)]
pub(super) fn timeline_gtk(
    runtime: &mut TimelineRuntime,
    painter: &TimelinePainter,
    width: f64,
    height: f64,
    accent_color: Color,
) {
    runtime.scene.draw_frame(
        painter.canvas(),
        vec2(width as f32, height as f32),
        shrimply_timeline_core::scene::Frame {
            before_seek: None,
            accent_color,
            active_audio_recording_key: None,
            active_video_recording_key: None,
            live_recording: None,
            live_video_recording: None,
        },
    );
}
