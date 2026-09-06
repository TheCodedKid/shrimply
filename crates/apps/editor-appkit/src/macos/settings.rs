use super::Editor;
use objc2::{DefinedClass, MainThreadOnly, sel};
use objc2_app_kit::{
    NSAutoresizingMaskOptions, NSBackingStoreType, NSButton, NSColor, NSColorWell, NSFont,
    NSFontManager, NSImage, NSModalResponseOK, NSOpenPanel, NSPopUpButton, NSTabViewController,
    NSTabViewControllerTabStyle, NSTabViewItem, NSTextAlignment, NSTextField, NSView,
    NSViewController, NSWindow, NSWindowButton, NSWindowStyleMask, NSWindowTabbingMode,
    NSWindowToolbarStyle,
};
use objc2_foundation::{MainThreadMarker, NSPoint, NSRect, NSSize, NSString, ns_string};
use shrimply_state::preferences::{self, PreferenceId, PreferenceValue, SharedPreferences};

const PANE_WIDTH: f64 = 560.0;
const PANE_HEIGHT: f64 = 320.0;
const CONTENT_MARGIN: f64 = 24.0;
const CONTROL_X: f64 = 285.0;
const CONTROL_WIDTH: f64 = PANE_WIDTH - CONTROL_X - CONTENT_MARGIN;
const ROW_HEIGHT: f64 = 38.0;
const SECTION_HEIGHT: f64 = 30.0;
const LABEL_HEIGHT: f64 = 22.0;
const CONTROL_HEIGHT: f64 = 26.0;
const CONTROL_Y_OFFSET: f64 = 2.0;
const BLENDER_CLEAR_WIDTH: f64 = 64.0;
const BLENDER_BUTTON_GAP: f64 = 8.0;

const TAG_CAPTION_FONT_SIZE: isize = 1;
const TAG_DEFAULT_VISUAL_DURATION: isize = 2;
const TAG_TIMELINE_SNAP_RADIUS: isize = 3;
const TAG_PREVIEW_PADDING: isize = 4;
const TAG_PREVIEW_SHADOW: isize = 5;
const TAG_BLENDER_CHOOSE: isize = 201;
const TAG_BLENDER_CLEAR: isize = 202;

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
        _ => return None,
    })
}

pub(super) fn numeric_value(store: &SharedPreferences, id: PreferenceId, scale: i64) -> f64 {
    match preferences::value(store, id) {
        PreferenceValue::Integer(value) => value as f64 / scale as f64,
        _ => panic!("numeric preference must store an integer"),
    }
}

pub(super) fn show(editor: &Editor) -> objc2::rc::Retained<NSWindow> {
    let mtm = editor.mtm();
    let window = unsafe {
        NSWindow::initWithContentRect_styleMask_backing_defer(
            NSWindow::alloc(mtm),
            NSRect::new(NSPoint::ZERO, NSSize::new(PANE_WIDTH, PANE_HEIGHT)),
            NSWindowStyleMask::Titled | NSWindowStyleMask::Closable,
            NSBackingStoreType::Buffered,
            false,
        )
    };
    unsafe { window.setReleasedWhenClosed(false) };
    window.setTitle(ns_string!("General"));
    window.setTabbingMode(NSWindowTabbingMode::Disallowed);
    window.setToolbarStyle(NSWindowToolbarStyle::Preference);
    for button in [
        NSWindowButton::MiniaturizeButton,
        NSWindowButton::ZoomButton,
    ] {
        if let Some(button) = window.standardWindowButton(button) {
            button.setEnabled(false);
        }
    }

    let store = &editor
        .ivars()
        .session
        .get()
        .expect("project loaded")
        .preferences;
    let snapshot = preferences::snapshot(store);

    let general = pane(mtm);
    let mut y = initial_y();
    section(&general, "Appearance", &mut y, mtm);
    numeric_row(
        &general,
        editor,
        "Caption font size",
        TAG_CAPTION_FONT_SIZE,
        f64::from(snapshot.caption_font_size),
        &mut y,
        mtm,
    );
    color_row(
        &general,
        editor,
        snapshot.caption_background_color,
        &mut y,
        mtm,
    );
    font_row(
        &general,
        editor,
        snapshot.default_text_font_family.name(),
        &mut y,
        mtm,
    );

    section(&general, "Timeline", &mut y, mtm);
    numeric_row(
        &general,
        editor,
        "Default visual duration (seconds)",
        TAG_DEFAULT_VISUAL_DURATION,
        snapshot.default_visual_duration.as_secs_f64(),
        &mut y,
        mtm,
    );
    numeric_row(
        &general,
        editor,
        "Snap attraction radius (pixels)",
        TAG_TIMELINE_SNAP_RADIUS,
        f64::from(snapshot.timeline_snap_radius_px),
        &mut y,
        mtm,
    );

    let preview = pane(mtm);
    let mut y = initial_y();
    section(&preview, "Preview", &mut y, mtm);
    numeric_row(
        &preview,
        editor,
        "Padding (pixels)",
        TAG_PREVIEW_PADDING,
        f64::from(snapshot.preview_padding_px),
        &mut y,
        mtm,
    );
    numeric_row(
        &preview,
        editor,
        "Shadow size (pixels)",
        TAG_PREVIEW_SHADOW,
        f64::from(snapshot.preview_shadow_size_px),
        &mut y,
        mtm,
    );
    let integrations = pane(mtm);
    let mut y = initial_y();
    section(&integrations, "Integrations", &mut y, mtm);
    text_row(
        &integrations,
        editor,
        "Compute server",
        &snapshot.compute_server_url,
        sel!(changeComputeServer:),
        &mut y,
        mtm,
    );
    blender_row(
        &integrations,
        editor,
        snapshot.blender_binary.as_deref(),
        &mut y,
        mtm,
    );

    let tabs = NSTabViewController::new(mtm);
    tabs.setTabStyle(NSTabViewControllerTabStyle::Toolbar);
    tabs.setCanPropagateSelectedChildViewControllerTitle(true);
    for item in [
        tab("General", "gearshape", general, mtm),
        tab("Preview", "play.rectangle", preview, mtm),
        tab("Integrations", "puzzlepiece.extension", integrations, mtm),
    ] {
        tabs.addTabViewItem(&item);
    }
    tabs.setSelectedTabViewItemIndex(0);
    window.setContentViewController(Some(&tabs));
    window.center();
    window.makeKeyAndOrderFront(None);
    window
}

fn pane(mtm: MainThreadMarker) -> objc2::rc::Retained<NSView> {
    let view = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(NSPoint::ZERO, NSSize::new(PANE_WIDTH, PANE_HEIGHT)),
    );
    view.setAutoresizingMask(
        NSAutoresizingMaskOptions::ViewWidthSizable | NSAutoresizingMaskOptions::ViewHeightSizable,
    );
    view
}

fn tab(
    title: &str,
    symbol: &str,
    view: objc2::rc::Retained<NSView>,
    mtm: MainThreadMarker,
) -> objc2::rc::Retained<NSTabViewItem> {
    let controller = NSViewController::new(mtm);
    let title = NSString::from_str(title);
    controller.setTitle(Some(&title));
    controller.setPreferredContentSize(NSSize::new(PANE_WIDTH, PANE_HEIGHT));
    controller.setView(&view);
    let item = NSTabViewItem::tabViewItemWithViewController(&controller);
    item.setLabel(&title);
    if let Some(image) = NSImage::imageWithSystemSymbolName_accessibilityDescription(
        &NSString::from_str(symbol),
        Some(&title),
    ) {
        item.setImage(Some(&image));
    }
    item
}

fn initial_y() -> f64 {
    PANE_HEIGHT - CONTENT_MARGIN - LABEL_HEIGHT
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
            NSPoint::new(CONTROL_X, *y - CONTROL_Y_OFFSET),
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
            NSPoint::new(CONTROL_X, *y - CONTROL_Y_OFFSET),
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
            NSPoint::new(CONTROL_X, *y - CONTROL_Y_OFFSET),
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
            NSPoint::new(CONTROL_X, *y - CONTROL_Y_OFFSET),
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
        NSPoint::new(CONTROL_X, *y - CONTROL_Y_OFFSET),
        NSSize::new(
            CONTROL_WIDTH - BLENDER_CLEAR_WIDTH - BLENDER_BUTTON_GAP,
            CONTROL_HEIGHT,
        ),
    ));
    choose.setTag(TAG_BLENDER_CHOOSE);
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
        NSPoint::new(
            PANE_WIDTH - CONTENT_MARGIN - BLENDER_CLEAR_WIDTH,
            *y - CONTROL_Y_OFFSET,
        ),
        NSSize::new(BLENDER_CLEAR_WIDTH, CONTROL_HEIGHT),
    ));
    clear.setTag(TAG_BLENDER_CLEAR);
    clear.setEnabled(path.is_some());
    content.addSubview(&clear);
    *y -= ROW_HEIGHT;
}

fn sync_blender_row(content: &NSView, path: Option<&std::path::Path>, checking: bool) {
    let choose = content
        .viewWithTag(TAG_BLENDER_CHOOSE)
        .and_then(|view| view.downcast::<NSButton>().ok())
        .expect("Blender choose button installed");
    choose.setTitle(&NSString::from_str(if checking {
        "Checking…"
    } else {
        path.and_then(std::path::Path::file_name)
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("Choose…")
    }));
    choose.setEnabled(!checking);
    let clear = content
        .viewWithTag(TAG_BLENDER_CLEAR)
        .and_then(|view| view.downcast::<NSButton>().ok())
        .expect("Blender clear button installed");
    clear.setEnabled(path.is_some() && !checking);
}

pub(super) fn sync_blender_window(
    window: &NSWindow,
    path: Option<&std::path::Path>,
    checking: bool,
    mtm: MainThreadMarker,
) {
    let tabs = window
        .contentViewController()
        .and_then(|controller| controller.downcast::<NSTabViewController>().ok())
        .expect("Settings tab controller installed");
    let content = tabs
        .tabViewItems()
        .iter()
        .find(|item| item.label().to_string() == "Integrations")
        .and_then(|item| item.view(mtm))
        .expect("Integrations settings pane installed");
    sync_blender_row(&content, path, checking);
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

pub(super) fn choose_blender_path(
    mtm: MainThreadMarker,
) -> Result<Option<std::path::PathBuf>, String> {
    let panel = NSOpenPanel::openPanel(mtm);
    panel.setCanChooseDirectories(false);
    panel.setAllowsMultipleSelection(false);
    panel.setTitle(Some(ns_string!("Choose Blender Binary")));
    if panel.runModal() != NSModalResponseOK {
        return Ok(None);
    }
    let path = panel
        .URL()
        .ok_or("Blender selection has no URL")?
        .to_file_path()
        .ok_or("Blender must be a local file")?;
    Ok(Some(path))
}
