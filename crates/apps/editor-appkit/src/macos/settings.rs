use super::Editor;
use objc2::{DefinedClass, MainThreadOnly, sel};
use objc2_app_kit::{
    NSAutoresizingMaskOptions, NSBackingStoreType, NSButton, NSColor, NSColorWell, NSFont,
    NSFontManager, NSModalResponseOK, NSOpenPanel, NSPopUpButton, NSScrollView, NSTextAlignment,
    NSTextField, NSView, NSWindow, NSWindowStyleMask,
};
use objc2_foundation::{MainThreadMarker, NSPoint, NSRect, NSSize, NSString, ns_string};
use shrimply_state::preferences::{self, PreferenceId, PreferenceValue, SharedPreferences};

const WINDOW_WIDTH: f64 = 640.0;
const WINDOW_HEIGHT: f64 = 650.0;
const CONTENT_HEIGHT: f64 = 850.0;
const CONTENT_MARGIN: f64 = 28.0;
const CONTROL_X: f64 = 330.0;
const CONTROL_WIDTH: f64 = WINDOW_WIDTH - CONTROL_X - CONTENT_MARGIN;
const ROW_HEIGHT: f64 = 38.0;
const SECTION_HEIGHT: f64 = 34.0;
const LABEL_HEIGHT: f64 = 24.0;
const CONTROL_HEIGHT: f64 = 28.0;

const TAG_CAPTION_FONT_SIZE: isize = 1;
const TAG_DEFAULT_VISUAL_DURATION: isize = 2;
const TAG_TIMELINE_SNAP_RADIUS: isize = 3;
const TAG_PREVIEW_PADDING: isize = 4;
const TAG_PREVIEW_SHADOW: isize = 5;
const TAG_DECODER_POOL: isize = 6;
const TAG_GPU_MEMORY: isize = 7;
const TAG_UPSAMPLE: isize = 101;
const TAG_DOWNSAMPLE: isize = 102;

pub(super) fn numeric_preference(tag: isize) -> Option<(PreferenceId, i64)> {
    Some(match tag {
        TAG_CAPTION_FONT_SIZE => (PreferenceId::CaptionFontSize, 1),
        TAG_DEFAULT_VISUAL_DURATION => (
            PreferenceId::DefaultVisualDuration,
            preferences::integer_range(PreferenceId::DefaultVisualDuration)?.scale,
        ),
        TAG_TIMELINE_SNAP_RADIUS => (PreferenceId::TimelineSnapRadius, 1),
        TAG_PREVIEW_PADDING => (PreferenceId::PreviewPadding, 1),
        TAG_PREVIEW_SHADOW => (PreferenceId::PreviewShadowSize, 1),
        TAG_DECODER_POOL => (PreferenceId::TemporalDecoderPoolSize, 1),
        TAG_GPU_MEMORY => (
            PreferenceId::GpuHostMemory,
            preferences::integer_range(PreferenceId::GpuHostMemory)?.scale,
        ),
        _ => return None,
    })
}

pub(super) fn choice_preference(tag: isize) -> Option<PreferenceId> {
    match tag {
        TAG_UPSAMPLE => Some(PreferenceId::PreviewUpsampleMethod),
        TAG_DOWNSAMPLE => Some(PreferenceId::PreviewDownsampleMethod),
        _ => None,
    }
}

pub(super) fn show(editor: &Editor) -> objc2::rc::Retained<NSWindow> {
    let mtm = editor.mtm();
    let window = unsafe {
        NSWindow::initWithContentRect_styleMask_backing_defer(
            NSWindow::alloc(mtm),
            NSRect::new(NSPoint::ZERO, NSSize::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
            NSWindowStyleMask::Titled | NSWindowStyleMask::Closable,
            NSBackingStoreType::Buffered,
            false,
        )
    };
    unsafe { window.setReleasedWhenClosed(false) };
    window.setTitle(ns_string!("Settings"));
    let scroll = NSScrollView::initWithFrame(
        NSScrollView::alloc(mtm),
        NSRect::new(NSPoint::ZERO, NSSize::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
    );
    scroll.setAutoresizingMask(
        NSAutoresizingMaskOptions::ViewWidthSizable | NSAutoresizingMaskOptions::ViewHeightSizable,
    );
    scroll.setHasVerticalScroller(true);
    scroll.setDrawsBackground(false);
    let content = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(NSPoint::ZERO, NSSize::new(WINDOW_WIDTH, CONTENT_HEIGHT)),
    );
    content.setAutoresizingMask(NSAutoresizingMaskOptions::ViewWidthSizable);
    let store = &editor
        .ivars()
        .session
        .get()
        .expect("project loaded")
        .preferences;
    let snapshot = preferences::snapshot(store);
    let mut y = CONTENT_HEIGHT - CONTENT_MARGIN - LABEL_HEIGHT;

    section(&content, "Appearance", &mut y, mtm);
    numeric_row(
        &content,
        editor,
        "Caption font size",
        TAG_CAPTION_FONT_SIZE,
        f64::from(snapshot.caption_font_size),
        &mut y,
        mtm,
    );
    color_row(
        &content,
        editor,
        snapshot.caption_background_color,
        &mut y,
        mtm,
    );
    font_row(
        &content,
        editor,
        snapshot.default_text_font_family.name(),
        &mut y,
        mtm,
    );

    section(&content, "Timeline", &mut y, mtm);
    numeric_row(
        &content,
        editor,
        "Default visual duration (seconds)",
        TAG_DEFAULT_VISUAL_DURATION,
        snapshot.default_visual_duration.as_secs_f64(),
        &mut y,
        mtm,
    );
    numeric_row(
        &content,
        editor,
        "Snap attraction radius (pixels)",
        TAG_TIMELINE_SNAP_RADIUS,
        f64::from(snapshot.timeline_snap_radius_px),
        &mut y,
        mtm,
    );

    section(&content, "Preview", &mut y, mtm);
    numeric_row(
        &content,
        editor,
        "Padding (pixels)",
        TAG_PREVIEW_PADDING,
        f64::from(snapshot.preview_padding_px),
        &mut y,
        mtm,
    );
    numeric_row(
        &content,
        editor,
        "Shadow size (pixels)",
        TAG_PREVIEW_SHADOW,
        f64::from(snapshot.preview_shadow_size_px),
        &mut y,
        mtm,
    );
    choice_row(
        &content,
        editor,
        "Upsample method",
        TAG_UPSAMPLE,
        &["Nearest", "Bilinear"],
        snapshot.preview_upsample_method as isize,
        &mut y,
        mtm,
    );
    choice_row(
        &content,
        editor,
        "Downsample method",
        TAG_DOWNSAMPLE,
        &["Nearest", "Bilinear", "Trilinear"],
        snapshot.preview_downsample_method as isize,
        &mut y,
        mtm,
    );

    section(&content, "Performance", &mut y, mtm);
    numeric_row(
        &content,
        editor,
        "Temporal decoder pool size",
        TAG_DECODER_POOL,
        f64::from(snapshot.temporal_decoder_pool_size),
        &mut y,
        mtm,
    );
    numeric_row(
        &content,
        editor,
        "GPU host memory budget (GiB)",
        TAG_GPU_MEMORY,
        shrimply_math_core::fraction_as_f64(snapshot.gpu_host_memory_gib),
        &mut y,
        mtm,
    );

    section(&content, "Integrations", &mut y, mtm);
    text_row(
        &content,
        editor,
        "Compute server",
        &snapshot.compute_server_url,
        sel!(changeComputeServer:),
        &mut y,
        mtm,
    );
    blender_row(
        &content,
        editor,
        snapshot.blender_binary.as_deref(),
        &mut y,
        mtm,
    );

    scroll.setDocumentView(Some(&content));
    window.setContentView(Some(&scroll));
    window.center();
    window.makeKeyAndOrderFront(None);
    window
}

fn section(content: &NSView, title: &str, y: &mut f64, mtm: MainThreadMarker) {
    let label = NSTextField::labelWithString(&NSString::from_str(title), mtm);
    label.setFont(Some(&NSFont::boldSystemFontOfSize(15.0)));
    label.setFrame(NSRect::new(
        NSPoint::new(CONTENT_MARGIN, *y),
        NSSize::new(CONTROL_X - CONTENT_MARGIN, LABEL_HEIGHT),
    ));
    content.addSubview(&label);
    *y -= SECTION_HEIGHT;
}

fn label(content: &NSView, title: &str, y: f64, mtm: MainThreadMarker) {
    let label = NSTextField::labelWithString(&NSString::from_str(title), mtm);
    label.setFrame(NSRect::new(
        NSPoint::new(CONTENT_MARGIN, y),
        NSSize::new(CONTROL_X - CONTENT_MARGIN * 2.0, LABEL_HEIGHT),
    ));
    content.addSubview(&label);
}

fn numeric_row(
    content: &NSView,
    editor: &Editor,
    title: &str,
    tag: isize,
    value: f64,
    y: &mut f64,
    mtm: MainThreadMarker,
) {
    label(content, title, *y, mtm);
    let field = NSTextField::initWithFrame(
        NSTextField::alloc(mtm),
        NSRect::new(
            NSPoint::new(CONTROL_X, *y - 3.0),
            NSSize::new(CONTROL_WIDTH, CONTROL_HEIGHT),
        ),
    );
    field.setAlignment(NSTextAlignment::Right);
    field.setDoubleValue(value);
    field.setTag(tag);
    unsafe {
        field.setTarget(Some(editor));
        field.setAction(Some(sel!(changeNumericPreference:)));
    }
    content.addSubview(&field);
    *y -= ROW_HEIGHT;
}

fn choice_row(
    content: &NSView,
    editor: &Editor,
    title: &str,
    tag: isize,
    choices: &[&str],
    selected: isize,
    y: &mut f64,
    mtm: MainThreadMarker,
) {
    label(content, title, *y, mtm);
    let popup = NSPopUpButton::initWithFrame_pullsDown(
        NSPopUpButton::alloc(mtm),
        NSRect::new(
            NSPoint::new(CONTROL_X, *y - 4.0),
            NSSize::new(CONTROL_WIDTH, CONTROL_HEIGHT),
        ),
        false,
    );
    for choice in choices {
        popup.addItemWithTitle(&NSString::from_str(choice));
    }
    popup.selectItemAtIndex(selected);
    popup.setTag(tag);
    unsafe {
        popup.setTarget(Some(editor));
        popup.setAction(Some(sel!(changeChoicePreference:)));
    }
    content.addSubview(&popup);
    *y -= ROW_HEIGHT;
}

fn color_row(
    content: &NSView,
    editor: &Editor,
    color: shrimply_project::Color<u8>,
    y: &mut f64,
    mtm: MainThreadMarker,
) {
    label(content, "Caption background color", *y, mtm);
    let well = NSColorWell::initWithFrame(
        NSColorWell::alloc(mtm),
        NSRect::new(
            NSPoint::new(CONTROL_X, *y - 3.0),
            NSSize::new(CONTROL_WIDTH, CONTROL_HEIGHT),
        ),
    );
    well.setSupportsAlpha(true);
    well.setColor(&NSColor::colorWithSRGBRed_green_blue_alpha(
        f64::from(color.r) / f64::from(u8::MAX),
        f64::from(color.g) / f64::from(u8::MAX),
        f64::from(color.b) / f64::from(u8::MAX),
        f64::from(color.a) / f64::from(u8::MAX),
    ));
    unsafe {
        well.setTarget(Some(editor));
        well.setAction(Some(sel!(changeCaptionColor:)));
    }
    content.addSubview(&well);
    *y -= ROW_HEIGHT;
}

fn font_row(content: &NSView, editor: &Editor, current: &str, y: &mut f64, mtm: MainThreadMarker) {
    label(content, "Default text font", *y, mtm);
    let popup = NSPopUpButton::initWithFrame_pullsDown(
        NSPopUpButton::alloc(mtm),
        NSRect::new(
            NSPoint::new(CONTROL_X, *y - 4.0),
            NSSize::new(CONTROL_WIDTH, CONTROL_HEIGHT),
        ),
        false,
    );
    for family in NSFontManager::sharedFontManager(mtm)
        .availableFontFamilies()
        .iter()
    {
        popup.addItemWithTitle(&family);
    }
    popup.selectItemWithTitle(&NSString::from_str(current));
    unsafe {
        popup.setTarget(Some(editor));
        popup.setAction(Some(sel!(changeDefaultFont:)));
    }
    content.addSubview(&popup);
    *y -= ROW_HEIGHT;
}

fn text_row(
    content: &NSView,
    editor: &Editor,
    title: &str,
    value: &str,
    action: objc2::runtime::Sel,
    y: &mut f64,
    mtm: MainThreadMarker,
) {
    label(content, title, *y, mtm);
    let field = NSTextField::initWithFrame(
        NSTextField::alloc(mtm),
        NSRect::new(
            NSPoint::new(CONTROL_X, *y - 3.0),
            NSSize::new(CONTROL_WIDTH, CONTROL_HEIGHT),
        ),
    );
    field.setStringValue(&NSString::from_str(value));
    unsafe {
        field.setTarget(Some(editor));
        field.setAction(Some(action));
    }
    content.addSubview(&field);
    *y -= ROW_HEIGHT;
}

fn blender_row(
    content: &NSView,
    editor: &Editor,
    path: Option<&std::path::Path>,
    y: &mut f64,
    mtm: MainThreadMarker,
) {
    label(content, "Blender binary", *y, mtm);
    let clear_width = 70.0;
    let gap = 8.0;
    let choose = unsafe {
        NSButton::buttonWithTitle_target_action(
            &NSString::from_str(
                path.and_then(std::path::Path::file_name)
                    .and_then(std::ffi::OsStr::to_str)
                    .unwrap_or("Choose…"),
            ),
            Some(editor),
            Some(sel!(chooseBlender:)),
            mtm,
        )
    };
    choose.setFrame(NSRect::new(
        NSPoint::new(CONTROL_X, *y - 3.0),
        NSSize::new(CONTROL_WIDTH - clear_width - gap, CONTROL_HEIGHT),
    ));
    content.addSubview(&choose);
    let clear = unsafe {
        NSButton::buttonWithTitle_target_action(
            ns_string!("Clear"),
            Some(editor),
            Some(sel!(clearBlender:)),
            mtm,
        )
    };
    clear.setFrame(NSRect::new(
        NSPoint::new(WINDOW_WIDTH - CONTENT_MARGIN - clear_width, *y - 3.0),
        NSSize::new(clear_width, CONTROL_HEIGHT),
    ));
    clear.setEnabled(path.is_some());
    content.addSubview(&clear);
    *y -= ROW_HEIGHT;
}

pub(super) fn set_caption_color(store: &SharedPreferences, well: &NSColorWell) {
    let mut red = 0.0;
    let mut green = 0.0;
    let mut blue = 0.0;
    let mut alpha = 0.0;
    unsafe {
        well.color()
            .getRed_green_blue_alpha(&mut red, &mut green, &mut blue, &mut alpha)
    };
    let component = |value: f64| (value.clamp(0.0, 1.0) * f64::from(u8::MAX)).round() as u8;
    preferences::set_value(
        store,
        PreferenceId::CaptionBackgroundColor,
        PreferenceValue::Color(shrimply_project::Color::new(
            component(red),
            component(green),
            component(blue),
            component(alpha),
        )),
    )
    .expect("caption background color has a fixed color type");
}

pub(super) fn choose_blender(
    store: &SharedPreferences,
    mtm: MainThreadMarker,
) -> Result<(), String> {
    let panel = NSOpenPanel::openPanel(mtm);
    panel.setCanChooseDirectories(false);
    panel.setAllowsMultipleSelection(false);
    panel.setTitle(Some(ns_string!("Choose Blender Binary")));
    if panel.runModal() != NSModalResponseOK {
        return Ok(());
    }
    let path = panel
        .URL()
        .ok_or("Blender selection has no URL")?
        .to_file_path()
        .ok_or("Blender must be a local file")?;
    let path = preferences::validate_blender_binary(&path)?;
    preferences::apply_blender_binary(store, Some(path));
    Ok(())
}
