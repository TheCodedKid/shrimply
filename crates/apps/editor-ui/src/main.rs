use shrimply_export_ui as export;
use shrimply_support::crash;
use shrimply_ui_foundation::tr;
use shrimply_ui_foundation::ui::I18nAlertDialogExt;
use shrimply_ui_foundation::ui::I18nFileFilterExt;
mod header_menu;
mod mcp;
use shrimply_inspector_ui as inspector;
use shrimply_state::player_state;
mod preferences;
use shrimply_preview_gtk as video_player;
use shrimply_timeline_ui as timeline;
use shrimply_ui_foundation::project_settings::ProjectSettingsSelector;

pub use shrimply_audio as audio;
pub use shrimply_project::project;

use adw::prelude::*;
use ffmpeg_next as ffmpeg;
use gdk_pixbuf::prelude::PixbufAnimationExtManual;
use gtk::gdk::prelude::GdkCairoContextExt;
use gtk::{gio, glib};
use shrimply_cross_ui_core::editor::{EditorSession, LoadEvent, ProjectLoader, SessionEvent};
use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::SystemTime;

const DEFAULT_WINDOW_WIDTH: i32 = 1800;
const DEFAULT_WINDOW_HEIGHT: i32 = 1100;
const DEFAULT_INSPECTOR_WIDTH: i32 = inspector::INSPECTOR_MIN_WIDTH;
const DEFAULT_TOP_PANEL_HEIGHT: i32 = 660;
const PANEL_ANIMATION_DURATION_MS: u32 = 250;
const LOADING_WINDOW_WIDTH: i32 = 800;
const LOADING_WINDOW_HEIGHT: i32 = 600;
const LOADING_SHRIMP_WIDTH: i32 = 160;
const LOADING_SHRIMP_HEIGHT: i32 = 180;
const KDENLIVE_IMPORT_DOCS: &str = "https://shrimply.pages.dev/guides/kdenlive-import";

#[derive(Clone, Copy)]
enum PanelSide {
    Start,
    End,
}

fn main() -> glib::ExitCode {
    crash::install();
    shrimply_support::diagnostics::init();
    shrimply_ui_foundation::i18n::init_system_locale();
    crash::install_glib_hooks();
    let mut args = std::env::args_os().skip(1);
    let Some(project_path) = args.next().map(PathBuf::from) else {
        eprintln!(
            "usage: shrimply-editor PROJECT.shrimp|PROJECT.json|TIMELINE.otio|PROJECT.kdenlive"
        );
        return glib::ExitCode::FAILURE;
    };
    if args.next().is_some() {
        eprintln!(
            "usage: shrimply-editor PROJECT.shrimp|PROJECT.json|TIMELINE.otio|PROJECT.kdenlive"
        );
        return glib::ExitCode::FAILURE;
    }
    tracing::info!(version = env!("CARGO_PKG_VERSION"), "Shrimply started");

    let app = adw::Application::new(
        Some("dev.shrimply.Shrimply.Editor"),
        gio::ApplicationFlags::NON_UNIQUE,
    );

    app.connect_activate(move |app| {
        let style = adw::StyleManager::default();
        shrimply_skia_adw_ui::theme::set_dark(style.is_dark());
        style.connect_dark_notify(|style| {
            shrimply_skia_adw_ui::theme::set_dark(style.is_dark());
        });
        begin_project_load(app, project_path.clone());
    });
    let status = app.run_with_args(&[env!("CARGO_BIN_NAME")]);
    let save_result = project::shutdown_history();
    project::clear_project_file_locks();
    if let Err(error) = save_result {
        tracing::error!("Could not save project during shutdown: {error}");
        return glib::ExitCode::FAILURE;
    }
    status
}

fn build_ui(window: &adw::ApplicationWindow, project: project::Project) {
    // Finish GDK's lazy Vulkan initialization before the video worker can initialize Vulkan too.
    drop(gtk::prelude::WidgetExt::display(window).dmabuf_formats());
    let session = Rc::new(
        EditorSession::new(project)
            .unwrap_or_else(|error| panic!("could not initialize editor session: {error}")),
    );
    let project = session.project.clone();
    let player_state = session.player_state.clone();
    let playback_performance = session.playback_performance.clone();
    let selection_state = session.selection_state.clone();
    let preview_focus = session.preview_focus.clone();
    let property_clipboard = session.property_clipboard.clone();
    let preferences = session.preferences.clone();
    let audio_levels = session.audio_levels.clone();
    let audio_player = session.audio_player.clone();
    let polling_session = session.clone();
    let polling_window = window.clone();
    window.add_tick_callback(move |_, _| {
        if let Some(SessionEvent::AudioPlaybackStopped(error)) = polling_session.poll() {
            let dialog = adw::AlertDialog::new(Some("Audio playback stopped"), Some(&error));
            dialog.present(Some(&polling_window));
        }
        glib::ControlFlow::Continue
    });
    let mcp_server = RefCell::new(Some(
        mcp::start(
            project.clone(),
            player_state.clone(),
            selection_state.clone(),
            preferences.clone(),
        )
        .unwrap_or_else(|error| panic!("could not start live MCP bridge: {error}")),
    ));
    window.connect_destroy(move |_| {
        mcp_server.borrow_mut().take();
    });
    let video_player = video_player::new(
        project.clone(),
        player_state.clone(),
        playback_performance.clone(),
        selection_state.clone(),
        preview_focus.clone(),
        preferences.clone(),
        audio_player,
    );
    let inspector = inspector::new(
        project.clone(),
        player_state.clone(),
        selection_state.clone(),
        preview_focus,
        preferences.clone(),
        video_player.clone(),
        property_clipboard.clone(),
    );
    let top = gtk::Paned::new(gtk::Orientation::Horizontal);
    top.set_wide_handle(false);
    top.set_start_child(Some(&inspector));
    top.set_end_child(Some(&video_player));
    top.set_position(DEFAULT_INSPECTOR_WIDTH);
    top.set_resize_start_child(true);
    top.set_resize_end_child(true);
    top.set_shrink_start_child(false);
    top.set_shrink_end_child(false);

    let timeline = timeline::new(
        project.clone(),
        player_state.clone(),
        playback_performance,
        selection_state.clone(),
        preferences.clone(),
        audio_levels,
        property_clipboard,
    );
    let layout = gtk::Paned::new(gtk::Orientation::Vertical);
    layout.set_wide_handle(false);
    layout.set_start_child(Some(&top));
    layout.set_end_child(Some(&timeline));
    layout.set_position(DEFAULT_TOP_PANEL_HEIGHT);
    layout.set_resize_start_child(true);
    layout.set_resize_end_child(true);
    layout.set_shrink_start_child(false);
    layout.set_shrink_end_child(false);

    let (header_bar, title) = header_bar();
    let inspector_toggle = gtk::ToggleButton::builder()
        .active(true)
        .icon_name("dock-left-symbolic")
        .tooltip_text(tr!("Toggle Inspector").as_ref())
        .build();
    connect_panel_toggle(
        &inspector_toggle,
        &top,
        &inspector,
        PanelSide::Start,
        DEFAULT_INSPECTOR_WIDTH,
    );
    let timeline_toggle = gtk::ToggleButton::builder()
        .active(true)
        .icon_name("dock-bottom-symbolic")
        .tooltip_text(tr!("Toggle Timeline").as_ref())
        .build();
    connect_panel_toggle(
        &timeline_toggle,
        &layout,
        &timeline,
        PanelSide::End,
        DEFAULT_WINDOW_HEIGHT - DEFAULT_TOP_PANEL_HEIGHT,
    );
    let panel_toggles = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    panel_toggles.append(&inspector_toggle);
    panel_toggles.append(&timeline_toggle);
    header_bar.pack_start(&panel_toggles);
    let project_name = Rc::new(RefCell::new(project.borrow().name.clone()));
    let commit_status = Rc::new(RefCell::new(project::CommitStatus::Idle));
    let status_project_name = project_name.clone();
    let current_commit_status = commit_status.clone();
    let status_title = title.clone();
    let status_window = window.clone();
    project::connect_commit_status(move |status| {
        update_project_title(
            &status_window,
            &status_title,
            &status_project_name.borrow(),
            &status,
        );
        *current_commit_status.borrow_mut() = status;
    });

    let name_project = project.clone();
    let name_window = window.clone();
    let name_commit_status = commit_status.clone();
    player_state::connect_named(&player_state, "editor project name", move |event| {
        if !matches!(event, player_state::PlayerEvent::Project(_)) {
            return;
        }
        let name = name_project.borrow().name.clone();
        if *project_name.borrow() == name {
            return;
        }
        *project_name.borrow_mut() = name.clone();
        update_project_title(&name_window, &title, &name, &name_commit_status.borrow());
        if let Err(error) =
            shrimply_support::recent_projects::touch(&project::active_project_path(), &name)
        {
            tracing::warn!("Could not update recent projects: {error}");
        }
    });

    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&header_bar);
    toolbar.set_content(Some(&layout));

    let toasts = adw::ToastOverlay::new();
    toasts.set_child(Some(&toolbar));

    window.set_visible(false);
    window.set_default_size(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT);
    window.set_content(Some(&toasts));

    header_menu::add(
        &header_bar,
        window,
        &toasts,
        project.clone(),
        player_state.clone(),
        preferences.clone(),
    );
    window.present();
    video_player.grab_focus();
}

fn connect_panel_toggle(
    button: &gtk::ToggleButton,
    paned: &gtk::Paned,
    panel: &impl IsA<gtk::Widget>,
    side: PanelSide,
    default_size: i32,
) {
    let panel = panel.as_ref().clone();
    let expanded_size = Rc::new(Cell::new(default_size));
    let weak_paned = paned.downgrade();
    let target = adw::CallbackAnimationTarget::new(move |size| {
        let Some(paned) = weak_paned.upgrade() else {
            return;
        };
        let size = (size.round() as i32).max(0);
        paned.set_position(match side {
            PanelSide::Start => size,
            PanelSide::End => paned.height().saturating_sub(size),
        });
    });
    let animation = adw::TimedAnimation::new(
        paned,
        f64::from(default_size),
        f64::from(default_size),
        PANEL_ANIMATION_DURATION_MS,
        target,
    );

    let weak_button = button.downgrade();
    let weak_paned = paned.downgrade();
    let weak_panel = panel.downgrade();
    animation.connect_done(move |_| {
        let (Some(button), Some(paned), Some(panel)) = (
            weak_button.upgrade(),
            weak_paned.upgrade(),
            weak_panel.upgrade(),
        ) else {
            return;
        };
        if !button.is_active() {
            panel.set_visible(false);
        }
        match side {
            PanelSide::Start => paned.set_shrink_start_child(false),
            PanelSide::End => paned.set_shrink_end_child(false),
        }
    });

    let paned = paned.clone();
    button.connect_toggled(move |button| {
        let current_size = match side {
            PanelSide::Start => paned.position(),
            PanelSide::End => paned.height().saturating_sub(paned.position()),
        };
        if !button.is_active() && animation.state() != adw::AnimationState::Playing {
            expanded_size.set(current_size);
        }
        if button.is_active() {
            panel.set_visible(true);
        }
        match side {
            PanelSide::Start => paned.set_shrink_start_child(true),
            PanelSide::End => paned.set_shrink_end_child(true),
        }
        animation.set_value_from(f64::from(current_size));
        animation.set_value_to(f64::from(if button.is_active() {
            expanded_size.get()
        } else {
            0
        }));
        animation.play();
    });
}

fn update_project_title(
    window: &adw::ApplicationWindow,
    title: &gtk::Label,
    project_name: &str,
    status: &project::CommitStatus,
) {
    let label = match status {
        project::CommitStatus::InProgress(action) => shrimply_ui_foundation::i18n::text_args(
            "%{project} — %{action}",
            &[
                ("project", project_name.to_owned()),
                ("action", tr!(action).into_owned()),
            ],
        ),
        project::CommitStatus::SavePending => shrimply_ui_foundation::i18n::text_args(
            "%{project} — Unsaved",
            &[("project", project_name.to_owned())],
        ),
        project::CommitStatus::Saving => shrimply_ui_foundation::i18n::text_args(
            "%{project} — Saving",
            &[("project", project_name.to_owned())],
        ),
        project::CommitStatus::SaveFailed(_) => shrimply_ui_foundation::i18n::text_args(
            "%{project} — Unsaved — Save failed",
            &[("project", project_name.to_owned())],
        ),
        project::CommitStatus::Idle => project_name.to_string(),
    };
    title.set_label(&label);
    title.set_tooltip_text(match status {
        project::CommitStatus::SaveFailed(error) => Some(error),
        _ => None,
    });
    window.set_title(Some(&label));
}

fn begin_project_load(app: &adw::Application, path: PathBuf) {
    shrimply_ui_foundation::icons::register_bundled();
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title(tr!("Shrimply").as_ref())
        .default_width(LOADING_WINDOW_WIDTH)
        .default_height(LOADING_WINDOW_HEIGHT)
        .build();
    window.set_content(Some(&project_loading_view(&path)));
    window.present();

    let loader = Rc::new(RefCell::new(ProjectLoader::new(path)));
    let event = loader.borrow_mut().begin();
    handle_load_event(app, &window, loader, event);
}

fn handle_load_event(
    app: &adw::Application,
    window: &adw::ApplicationWindow,
    loader: Rc<RefCell<ProjectLoader>>,
    event: LoadEvent,
) {
    match event {
        LoadEvent::ConfirmKdenlive => confirm_kdenlive_conversion(app, window, loader),
        LoadEvent::ChooseOtioSettings => choose_otio_settings(app, window, loader),
        LoadEvent::Progress(subtitle) => {
            window.set_content(Some(&project_loading_view_with_subtitle(subtitle)));
            let poll_app = app.clone();
            let poll_window = window.clone();
            window.add_tick_callback(move |_, _| {
                let Some(event) = loader.borrow_mut().poll() else {
                    return glib::ControlFlow::Continue;
                };
                handle_load_event(&poll_app, &poll_window, loader.clone(), event);
                glib::ControlFlow::Break
            });
        }
        LoadEvent::ConfirmFrameGridRepair => confirm_frame_grid_repair(app, window, loader),
        LoadEvent::ChooseDestination {
            title,
            suggested_name,
        } => choose_project_destination(app, window, loader, title, suggested_name),
        LoadEvent::ImportWarnings(warnings) => show_import_warnings(app, window, loader, warnings),
        LoadEvent::LockedByOtherInstance(pid) => show_project_lock_dialog(app, window, loader, pid),
        LoadEvent::Ready { path, project } => {
            if let Err(error) = shrimply_support::recent_projects::touch(&path, &project.name) {
                tracing::warn!(%error, "could not update recent projects");
            }
            ffmpeg::init().expect("FFmpeg should initialize");
            ffmpeg::util::log::set_level(ffmpeg::util::log::Level::Error);
            build_ui(window, *project);
        }
        LoadEvent::Error { heading, body } => show_project_load_error(app, window, heading, &body),
        LoadEvent::Canceled => app.quit(),
    }
}

fn confirm_kdenlive_conversion(
    app: &adw::Application,
    window: &adw::ApplicationWindow,
    loader: Rc<RefCell<ProjectLoader>>,
) {
    let message = gtk::Label::new(Some(
        tr!("Shrimply supports only some Kdenlive features. Unsupported content may be changed or omitted.")
            .as_ref(),
    ));
    message.set_justify(gtk::Justification::Center);
    message.set_wrap(true);
    let details = gtk::LinkButton::builder()
        .label(tr!("Learn more").as_ref())
        .uri(KDENLIVE_IMPORT_DOCS)
        .halign(gtk::Align::Center)
        .build();
    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.append(&message);
    content.append(&details);
    let dialog = adw::AlertDialog::builder()
        .heading(tr!("Convert Kdenlive Project?").as_ref())
        .extra_child(&content)
        .build();
    dialog.add_responses_i18n(&[("abort", "Abort"), ("convert", "Convert")]);
    dialog.set_default_response(Some("convert"));
    dialog.set_close_response("abort");
    let callback_app = app.clone();
    let callback_window = window.clone();
    dialog.choose(
        Some(window.upcast_ref::<gtk::Widget>()),
        None::<&gio::Cancellable>,
        move |answer| {
            let event = loader.borrow_mut().confirm_kdenlive(answer == "convert");
            handle_load_event(&callback_app, &callback_window, loader, event);
        },
    );
}

fn choose_otio_settings(
    app: &adw::Application,
    window: &adw::ApplicationWindow,
    loader: Rc<RefCell<ProjectLoader>>,
) {
    let selector = ProjectSettingsSelector::new();
    let content = adw::PreferencesGroup::builder()
        .title(tr!("Project Settings").as_ref())
        .build();
    content.add(&selector.preset);
    content.add(&selector.width);
    content.add(&selector.height);
    content.add(&selector.fps);
    let dialog = adw::AlertDialog::builder()
        .heading(tr!("OTIO Project Settings").as_ref())
        .body(tr!("OTIO does not include the Kdenlive project profile.").as_ref())
        .extra_child(&content)
        .build();
    dialog.add_responses_i18n(&[("cancel", "Cancel"), ("import", "Import")]);
    dialog.set_default_response(Some("import"));
    dialog.set_close_response("cancel");
    let callback_app = app.clone();
    let callback_window = window.clone();
    dialog.choose(
        Some(window.upcast_ref::<gtk::Widget>()),
        None::<&gio::Cancellable>,
        move |answer| {
            if answer != "import" {
                let event = loader.borrow_mut().choose_otio_settings(None);
                handle_load_event(&callback_app, &callback_window, loader, event);
                return;
            }
            let Some(settings) = selector.settings() else {
                show_project_load_error(
                    &callback_app,
                    &callback_window,
                    "Could not import OTIO",
                    "The selected project settings are invalid.",
                );
                return;
            };
            let event = loader.borrow_mut().choose_otio_settings(Some(settings));
            handle_load_event(&callback_app, &callback_window, loader, event);
        },
    );
}

fn confirm_frame_grid_repair(
    app: &adw::Application,
    window: &adw::ApplicationWindow,
    loader: Rc<RefCell<ProjectLoader>>,
) {
    let dialog = adw::AlertDialog::builder()
        .heading(tr!("Project Timing Needs Repair").as_ref())
        .body(
            tr!("Some clips are not aligned to the project frame grid. Fixing them will snap and minimally shift clip boundaries, then save a new project without changing the original.")
                .as_ref(),
        )
        .build();
    dialog.add_responses_i18n(&[("abort", "Abort"), ("fix", "Fix")]);
    dialog.set_default_response(Some("fix"));
    dialog.set_close_response("abort");
    let callback_app = app.clone();
    let callback_window = window.clone();
    dialog.choose(
        Some(window.upcast_ref::<gtk::Widget>()),
        None::<&gio::Cancellable>,
        move |answer| {
            let event = loader
                .borrow_mut()
                .confirm_frame_grid_repair(answer == "fix");
            handle_load_event(&callback_app, &callback_window, loader, event);
        },
    );
}

fn choose_project_destination(
    app: &adw::Application,
    window: &adw::ApplicationWindow,
    loader: Rc<RefCell<ProjectLoader>>,
    title: &'static str,
    suggested_name: String,
) {
    let filter = gtk::FileFilter::new();
    filter.set_name_i18n("Shrimply projects");
    filter.add_pattern("*.shrimp");
    let filters = gio::ListStore::new::<gtk::FileFilter>();
    filters.append(&filter);
    let dialog = gtk::FileDialog::builder()
        .title(tr!(title).as_ref())
        .initial_name(suggested_name)
        .filters(&filters)
        .default_filter(&filter)
        .build();
    let callback_app = app.clone();
    let callback_window = window.clone();
    shrimply_ui_foundation::file_picker::save(
        title,
        &dialog,
        Some(window.upcast_ref::<gtk::Window>()),
        move |result| {
            let destination = result.ok().and_then(|file| file.path());
            let event = loader.borrow_mut().choose_destination(destination);
            handle_load_event(&callback_app, &callback_window, loader, event);
        },
    );
}

fn show_import_warnings(
    app: &adw::Application,
    window: &adw::ApplicationWindow,
    loader: Rc<RefCell<ProjectLoader>>,
    warnings: Vec<String>,
) {
    let dialog = adw::AlertDialog::new(
        Some("OTIO imported with limitations"),
        Some(&warnings.join("\n")),
    );
    dialog.add_response("open", "Open Project");
    dialog.set_default_response(Some("open"));
    dialog.set_close_response("open");
    let callback_app = app.clone();
    let callback_window = window.clone();
    dialog.choose(
        Some(window.upcast_ref::<gtk::Widget>()),
        None::<&gio::Cancellable>,
        move |_| {
            let event = loader.borrow_mut().acknowledge_warnings();
            handle_load_event(&callback_app, &callback_window, loader, event);
        },
    );
}

fn show_project_lock_dialog(
    app: &adw::Application,
    window: &adw::ApplicationWindow,
    loader: Rc<RefCell<ProjectLoader>>,
    pid: u32,
) {
    let dialog = adw::AlertDialog::builder()
        .heading(tr!("Project is in use").as_ref())
        .body(format!(
            "The project lock is held by another editor process (PID {pid})."
        ))
        .prefer_wide_layout(false)
        .build();
    dialog.add_responses_i18n(&[
        ("close", "Close"),
        ("stop", "Stop Other Editor"),
        ("retry", "Retry"),
    ]);
    dialog.set_default_response(Some("retry"));
    dialog.set_close_response("close");
    dialog.set_response_appearance("stop", adw::ResponseAppearance::Destructive);
    let callback_app = app.clone();
    let callback_window = window.clone();
    dialog.choose(
        Some(window.upcast_ref::<gtk::Widget>()),
        None::<&gio::Cancellable>,
        move |answer| {
            let event = match answer.as_str() {
                "retry" => loader.borrow_mut().retry_locked_project(false, pid),
                "stop" => loader.borrow_mut().retry_locked_project(true, pid),
                _ => loader.borrow_mut().cancel(),
            };
            handle_load_event(&callback_app, &callback_window, loader, event);
        },
    );
}

fn show_project_load_error(
    app: &adw::Application,
    window: &adw::ApplicationWindow,
    heading: &str,
    message: &str,
) {
    let dialog = adw::AlertDialog::new(Some(heading), Some(message));
    dialog.add_response("close", "Close");
    dialog.set_close_response("close");
    dialog.set_default_response(Some("close"));
    let app = app.clone();
    dialog.choose(
        Some(window.upcast_ref::<gtk::Widget>()),
        None::<&gio::Cancellable>,
        move |_| app.quit(),
    );
}
fn project_loading_view(path: &Path) -> adw::ToolbarView {
    let filename = path
        .file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_else(|| path.to_string_lossy());
    project_loading_view_with_subtitle(filename.as_ref())
}

fn project_loading_view_with_subtitle(subtitle: &str) -> adw::ToolbarView {
    let bytes = glib::Bytes::from_static(include_bytes!("../assets/loading-shrimp.gif"));
    let stream = gio::MemoryInputStream::from_bytes(&bytes);
    let animation = gdk_pixbuf::PixbufAnimation::from_stream(&stream, None::<&gio::Cancellable>)
        .expect("bundled loading animation should decode")
        .iter(Some(SystemTime::now()));
    let shrimp = gtk::DrawingArea::new();
    shrimp.set_content_width(LOADING_SHRIMP_WIDTH);
    shrimp.set_content_height(LOADING_SHRIMP_HEIGHT);
    shrimp.set_halign(gtk::Align::Center);
    shrimp.set_valign(gtk::Align::Center);
    let drawing_animation = animation.clone();
    shrimp.set_draw_func(move |_, context, width, height| {
        let frame = drawing_animation.pixbuf();
        context.scale(
            f64::from(width) / f64::from(frame.width()),
            f64::from(height) / f64::from(frame.height()),
        );
        context.set_source_pixbuf(&frame, 0.0, 0.0);
        context.source().set_filter(gtk::cairo::Filter::Nearest);
        context.paint().expect("loading animation should draw");
    });
    shrimp.add_tick_callback(move |area, _| {
        if animation.advance(SystemTime::now()) {
            area.queue_draw();
        }
        glib::ControlFlow::Continue
    });
    let status = gtk::Label::builder()
        .label(tr!("Loading project…").as_ref())
        .css_classes(["title-3"])
        .build();
    let subtitle = gtk::Label::builder()
        .label(subtitle)
        .css_classes(["dim-label"])
        .ellipsize(pango::EllipsizeMode::Middle)
        .max_width_chars(48)
        .build();
    let body = gtk::Box::new(gtk::Orientation::Vertical, 16);
    body.set_halign(gtk::Align::Center);
    body.set_valign(gtk::Align::Center);
    body.set_hexpand(true);
    body.set_vexpand(true);
    body.append(&shrimp);
    body.append(&status);
    body.append(&subtitle);
    let toolbar = adw::ToolbarView::new();
    toolbar.add_css_class("background");
    toolbar.add_top_bar(
        &adw::HeaderBar::builder()
            .title_widget(&gtk::Box::new(gtk::Orientation::Horizontal, 0))
            .build(),
    );
    toolbar.set_content(Some(&body));
    toolbar
}

fn header_bar() -> (adw::HeaderBar, gtk::Label) {
    let title = gtk::Label::builder()
        .label(tr!("Shrimply").as_ref())
        .css_classes(["title"])
        .build();

    (
        adw::HeaderBar::builder().title_widget(&title).build(),
        title,
    )
}
