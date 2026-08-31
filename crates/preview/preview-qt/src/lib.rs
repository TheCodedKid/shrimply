mod preview;
mod renderer;
mod system_cursor;

pub use preview::ToolkitPreview;

use shrimply_audio::AudioPlayer;
use shrimply_playback_performance as playback_performance;
use shrimply_preview_core::PreviewViewport;
use shrimply_preview_runtime::captions::{CaptionAppearance, draw_captions};
use shrimply_preview_runtime::guides;
use shrimply_preview_runtime::preferences::store as preferences_store;
use shrimply_preview_runtime::renderer::{Appearance, VideoRenderer};
use shrimply_preview_runtime::{PreviewMedia, StepDirection, rendered_frame_rate_label};
use shrimply_project::project::{Project, Time};
use shrimply_skia_adw_ui::canvas::{Rect, vec2};
use shrimply_state::player_state::{self, SharedPlayerState};
use shrimply_video::compositor::VideoEvent;
use shrimply_video::gpu::CompositedVideoFrame;
use std::cell::RefCell;
use std::rc::Rc;

use shrimply_cross_ui_core::editor::EditorSession;
use shrimply_math_color::Color;
use shrimply_timeline_ui::{ToolkitAudioMeter, ToolkitPointerButton, ToolkitTimeline};
use std::ffi::c_void;

struct Surfaces {
    timeline: ToolkitTimeline,
    preview: ToolkitPreview,
    audio_meter: ToolkitAudioMeter,
}

thread_local! {
    static SURFACES: RefCell<Option<Surfaces>> = const { RefCell::new(None) };
}

pub fn install(session: &EditorSession) -> Result<(), String> {
    let timeline = ToolkitTimeline::new(
        session.project.clone(),
        session.player_state.clone(),
        session.playback_performance.clone(),
        session.selection_state.clone(),
        session.preferences.clone(),
        session.property_clipboard.clone(),
    );
    let preview = ToolkitPreview::new(
        session.project.clone(),
        session.player_state.clone(),
        session.playback_performance.clone(),
        session.preferences.clone(),
        session.audio_player.clone(),
    )?;
    let audio_meter = ToolkitAudioMeter::new(session.audio_levels.clone());
    SURFACES.with_borrow_mut(|surfaces| {
        assert!(
            surfaces.is_none(),
            "Qt editor surfaces are already installed"
        );
        *surfaces = Some(Surfaces {
            timeline,
            preview,
            audio_meter,
        });
    });
    tracing::info!(thread = ?std::thread::current().id(), "installed Qt GPU surfaces");
    Ok(())
}

fn missing(surface: &str) -> bool {
    tracing::error!(
        surface,
        thread = ?std::thread::current().id(),
        "Qt GPU surface render requested before the shared editor lifecycle installed it"
    );
    false
}

fn render(result: Result<(), String>, surface: &str) -> bool {
    if let Err(error) = result {
        tracing::error!(%error, surface, "Qt GPU surface render failed");
        false
    } else {
        true
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn shrimply_qt_render_timeline(
    width: u32,
    height: u32,
    scale: f32,
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
    dark: bool,
) -> bool {
    shrimply_skia_adw_ui::theme::set_dark(dark);
    SURFACES.with_borrow_mut(|surfaces| {
        let Some(surfaces) = surfaces.as_mut() else {
            return missing("timeline");
        };
        render(
            surfaces
                .timeline
                .render(width, height, scale, Color::new(red, green, blue, alpha)),
            "timeline",
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shrimply_qt_render_preview(
    width: u32,
    height: u32,
    scale: f32,
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
    dark: bool,
    fullscreen: bool,
) -> bool {
    shrimply_skia_adw_ui::theme::set_dark(dark);
    SURFACES.with_borrow_mut(|surfaces| {
        let Some(surfaces) = surfaces.as_mut() else {
            return missing("preview");
        };
        render(
            surfaces.preview.render(
                width,
                height,
                scale,
                Color::new(red, green, blue, alpha),
                fullscreen,
            ),
            "preview",
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shrimply_qt_render_audio_meter(
    width: u32,
    height: u32,
    scale: f32,
    dark: bool,
) -> bool {
    shrimply_skia_adw_ui::theme::set_dark(dark);
    SURFACES.with_borrow_mut(|surfaces| {
        let Some(surfaces) = surfaces.as_mut() else {
            return missing("audio meter");
        };
        render(
            surfaces.audio_meter.render(width, height, scale),
            "audio meter",
        )
    })
}

pub fn mark_preview_step(delta: i32) {
    SURFACES.with_borrow(|surfaces| {
        if let Some(surfaces) = surfaces.as_ref() {
            surfaces.preview.mark_step(delta);
        }
    });
}

pub fn preview_frame_rate_label() -> String {
    SURFACES.with_borrow(|surfaces| {
        surfaces.as_ref().map_or_else(
            || String::from("--"),
            |surfaces| surfaces.preview.frame_rate_label().into(),
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shrimply_qt_timeline_pointer_move(x: f32, y: f32, control: bool, shift: bool) {
    SURFACES.with_borrow(|surfaces| {
        if let Some(surfaces) = surfaces.as_ref() {
            surfaces.timeline.pointer_move(x, y, control, shift);
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn shrimply_qt_timeline_pointer_cursor() -> u8 {
    SURFACES.with_borrow(|surfaces| {
        surfaces
            .as_ref()
            .map_or(0, |surfaces| surfaces.timeline.pointer_cursor() as u8)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shrimply_qt_timeline_pointer_leave() {
    SURFACES.with_borrow(|surfaces| {
        if let Some(surfaces) = surfaces.as_ref() {
            surfaces.timeline.pointer_leave();
        }
    });
}

fn pointer_button(button: u8) -> ToolkitPointerButton {
    match button {
        0 => ToolkitPointerButton::Primary,
        1 => ToolkitPointerButton::Middle,
        _ => panic!("unsupported Qt timeline pointer button {button}"),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn shrimply_qt_timeline_pointer_press(
    button: u8,
    x: f32,
    y: f32,
    control: bool,
    shift: bool,
) {
    SURFACES.with_borrow(|surfaces| {
        if let Some(surfaces) = surfaces.as_ref() {
            surfaces
                .timeline
                .pointer_press(pointer_button(button), x, y, control, shift);
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn shrimply_qt_timeline_pointer_release(
    button: u8,
    x: f32,
    y: f32,
    control: bool,
    shift: bool,
) {
    SURFACES.with_borrow(|surfaces| {
        if let Some(surfaces) = surfaces.as_ref() {
            surfaces
                .timeline
                .pointer_release(pointer_button(button), x, y, control, shift);
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn shrimply_qt_timeline_begin_pointer_lock(
    display: *mut c_void,
    surface: *mut c_void,
    seat: *mut c_void,
) -> bool {
    SURFACES.with_borrow_mut(|surfaces| {
        let Some(surfaces) = surfaces.as_mut() else {
            return false;
        };
        unsafe {
            surfaces
                .timeline
                .begin_pointer_lock(display, surface, seat, system_cursor::grabbing())
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shrimply_qt_timeline_end_pointer_lock(control: bool, shift: bool) {
    SURFACES.with_borrow_mut(|surfaces| {
        if let Some(surfaces) = surfaces.as_mut() {
            surfaces.timeline.end_pointer_lock(control, shift);
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn shrimply_qt_timeline_scroll(dx: f32, dy: f32, control: bool, shift: bool) {
    SURFACES.with_borrow(|surfaces| {
        if let Some(surfaces) = surfaces.as_ref() {
            surfaces.timeline.scroll(dx, dy, control, shift);
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn shrimply_qt_preview_pointer_move(width: f32, height: f32, x: f32, y: f32) {
    SURFACES.with_borrow_mut(|surfaces| {
        if let Some(surfaces) = surfaces.as_mut() {
            surfaces.preview.pointer_move(width, height, x, y);
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn shrimply_qt_preview_pointer_cursor() -> u8 {
    SURFACES.with_borrow(|surfaces| {
        surfaces
            .as_ref()
            .map_or(0, |surfaces| surfaces.preview.pointer_cursor())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shrimply_qt_preview_pointer_leave() {
    SURFACES.with_borrow_mut(|surfaces| {
        if let Some(surfaces) = surfaces.as_mut() {
            surfaces.preview.pointer_leave();
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn shrimply_qt_preview_pointer_press(
    width: f32,
    height: f32,
    x: f32,
    y: f32,
) -> bool {
    SURFACES.with_borrow_mut(|surfaces| {
        surfaces
            .as_mut()
            .is_some_and(|surfaces| surfaces.preview.pointer_press(width, height, x, y))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shrimply_qt_preview_pointer_release(width: f32, height: f32, x: f32, y: f32) {
    SURFACES.with_borrow_mut(|surfaces| {
        if let Some(surfaces) = surfaces.as_mut() {
            surfaces.preview.pointer_release(width, height, x, y);
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn shrimply_qt_preview_pointer_cancel() {
    SURFACES.with_borrow_mut(|surfaces| {
        if let Some(surfaces) = surfaces.as_mut() {
            surfaces.preview.pointer_cancel();
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn shrimply_qt_preview_guides_visible() -> bool {
    SURFACES.with_borrow(|surfaces| {
        surfaces
            .as_ref()
            .is_some_and(|surfaces| surfaces.preview.guides_visible())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shrimply_qt_preview_set_guides_visible(visible: bool) {
    SURFACES.with_borrow(|surfaces| {
        if let Some(surfaces) = surfaces.as_ref() {
            surfaces.preview.set_guides_visible(visible);
        }
    });
}
