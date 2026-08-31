use adw::prelude::*;
use shrimply_gtk_components::playback_shortcuts::attach_space_play_toggle;
use shrimply_gtk_components::ui::{
    ColorPicker, FrameGraph, MultilineTextInput, Number2Picker, Number3Picker, NumberPicker,
    ProgressButton, ProgressButtonState, ReadOnlyField, SingleLineTextInput, StringChoice, code_editor,
    control_row, labeled_string_selector, read_only_field, split_button, switch_row, tabs,
};
use shrimply_math_color::Color;

fn main() {
    shrimply_gtk_components::i18n::init_system_locale();
    let app = adw::Application::new(
        Some("dev.shrimply.ComponentsShowcase"),
        gtk::gio::ApplicationFlags::NON_UNIQUE,
    );
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &adw::Application) {
    shrimply_cross_ui_theme::set_dark(adw::StyleManager::default().is_dark());
    shrimply_gtk_components::icons::register_bundled();
    let general = gtk::Box::new(gtk::Orientation::Vertical, 10);
    general.set_margin_top(16);
    general.set_margin_bottom(16);
    general.set_margin_start(16);
    general.set_margin_end(16);

    let events = gtk::TextView::builder()
        .editable(false)
        .monospace(true)
        .height_request(140)
        .build();
    let log = {
        let buffer = events.buffer();
        move |message: String| {
            let (start, end) = buffer.bounds();
            let previous = buffer.text(&start, &end, true);
            buffer.set_text(&format!("{message}\n{previous}"));
        }
    };

    let number = NumberPicker::builder(12.5)
        .accepted_range(-100.0, 100.0)
        .drag_step(0.25)
        .digits(2)
        .unit_name("px")
        .on_change({
            let log = log.clone();
            move |value| log(format!("number changed {value}"))
        })
        .on_commit({
            let log = log.clone();
            move |value| log(format!("number committed {value}"))
        })
        .build();
    general.append(&control_row("Number", &number));

    let pair = Number2Picker::builder(1920.0, 1080.0)
        .minimum(1.0)
        .maximum(16_384.0)
        .digits(0)
        .first_prefix("W")
        .second_prefix("H")
        .unit_name("px")
        .enable_lock()
        .on_first_change({
            let log = log.clone();
            move |value| log(format!("pair first {value}"))
        })
        .on_second_change({
            let log = log.clone();
            move |value| log(format!("pair second {value}"))
        })
        .build_with_handles();
    general.append(&control_row("Pair", &pair.widget));

    let vector = Number3Picker::builder(1.0, 2.0, 3.0)
        .prefixes(["X", "Y", "Z"])
        .enable_lock()
        .on_change(0, {
            let log = log.clone();
            move |value| log(format!("vector 0 {value}"))
        })
        .on_change(1, {
            let log = log.clone();
            move |value| log(format!("vector 1 {value}"))
        })
        .on_change(2, {
            let log = log.clone();
            move |value| log(format!("vector 2 {value}"))
        })
        .build_with_handles();
    general.append(&control_row("Vector", &vector.widget));

    let single = SingleLineTextInput::builder("Editable text")
        .placeholder("Type here")
        .max_length(40)
        .on_commit({
            let log = log.clone();
            move |value| log(format!("text committed {value}"))
        })
        .build();
    general.append(&control_row("Single line", &single));

    let multiline = MultilineTextInput::builder("Try a typo such as teh.")
        .min_content_height(110)
        .max_length(240)
        .on_change(|_| true)
        .on_commit({
            let log = log.clone();
            move || log("multiline committed".to_string())
        })
        .build();
    general.append(&control_row("Multiline", multiline.widget()));

    let selector = labeled_string_selector(
        "Searchable dropdown",
        "two",
        [
            "one", "two", "three", "four", "five", "six", "seven", "eight",
        ]
        .into_iter()
        .map(|value| StringChoice {
            value: value.to_string(),
            label: value.to_uppercase(),
        })
        .collect(),
        {
            let log = log.clone();
            move |value| log(format!("selected {value}"))
        },
    );
    general.append(selector.widget());

    let graph = FrameGraph::with_actions(shrimply_keyframe_graph_ui::FrameGraphState::sample(), {
        let log = log.clone();
        move |_| log("keyframe action".to_string())
    });
    let number_modes = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let graph_editor = graph.clone();
    let number_mode_value = NumberPicker::builder(graph.state().borrow().status().value)
        .digits(2)
        .on_change(move |value| graph_editor.edit_value(value))
        .build_with_handle();
    graph.connect_status({
        let value = number_mode_value.handle.clone();
        move |status| value.set_f64(status.value)
    });
    number_modes.append(&number_mode_value.widget);
    let keyframes = gtk::ToggleButton::builder()
        .icon_name("stopwatch-symbolic")
        .tooltip_text("Keyframes")
        .css_classes(["flat"])
        .active(true)
        .build();
    let expression = gtk::ToggleButton::builder()
        .icon_name("code-symbolic")
        .tooltip_text("Expression")
        .css_classes(["flat"])
        .active(true)
        .build();
    number_modes.append(&keyframes);
    number_modes.append(&expression);
    general.append(&control_row("Number modes", &number_modes));

    let mode_content = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let graph_revealer = gtk::Revealer::builder()
        .child(graph.widget())
        .reveal_child(true)
        .build();
    let expression_content = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let expression_output = read_only_field("Output · 84.0");
    expression_content.append(&code_editor("value * 2.0", Some("rhai"), {
        let expression_output = expression_output.clone();
        let log = log.clone();
        move |value| {
            expression_output.set_label(&format!(
                "Output · expression updated ({} chars)",
                value.len()
            ));
            log(format!("expression edited ({} chars)", value.len()));
        }
    }));
    expression_content.append(&expression_output);
    let expression_revealer = gtk::Revealer::builder()
        .child(&expression_content)
        .reveal_child(true)
        .build();
    mode_content.append(&graph_revealer);
    mode_content.append(&expression_revealer);
    keyframes.connect_toggled({
        let graph_revealer = graph_revealer.clone();
        let log = log.clone();
        move |button| {
            graph_revealer.set_reveal_child(button.is_active());
            log(format!("keyframes {}", button.is_active()));
        }
    });
    expression.connect_toggled({
        let expression_revealer = expression_revealer.clone();
        let log = log.clone();
        move |button| {
            expression_revealer.set_reveal_child(button.is_active());
            log(format!("expression {}", button.is_active()));
        }
    });
    general.append(&mode_content);
    general.append(&switch_row("Enabled", Some("Toggle this option"), true, {
        let log = log.clone();
        move |value| log(format!("switch {value}"))
    }));

    let color = ColorPicker::builder(Color::new(0x35, 0x84, 0xe4, 0xcc))
        .on_change({
            let log = log.clone();
            move |value| {
                log(format!(
                    "color #{:02X}{:02X}{:02X}{:02X}",
                    value.r, value.g, value.b, value.a
                ))
            }
        })
        .build();
    general.append(&control_row("Color", &color));
    general.append(&control_row(
        "Split",
        &split_button(
            "Primary",
            "Secondary",
            {
                let log = log.clone();
                move |_| log("primary".to_string())
            },
            {
                let log = log.clone();
                move || log("secondary".to_string())
            },
        ),
    ));

    let progress_row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let idle = ProgressButton::new("Idle");
    let indeterminate = ProgressButton::new("Working");
    indeterminate.set_state(ProgressButtonState::Indeterminate);
    let half = ProgressButton::new("Half");
    half.set_state(ProgressButtonState::Progress(0.5));
    progress_row.append(idle.widget());
    progress_row.append(indeterminate.widget());
    progress_row.append(half.widget());
    general.append(&control_row("Progress", &progress_row));

    let playback = gtk::Frame::builder()
        .child(&gtk::Label::new(Some("Click, then press Space or L")))
        .height_request(44)
        .build();
    attach_space_play_toggle(
        &playback,
        {
            let log = log.clone();
            move || log("toggle playback".to_string())
        },
        {
            let log = log.clone();
            move || log("step playback speed".to_string())
        },
    );
    general.append(&control_row("Playback keys", &playback));

    let general_scroller = gtk::ScrolledWindow::builder()
        .child(&general)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .build();

    let info = gtk::Box::new(gtk::Orientation::Vertical, 10);
    info.set_margin_top(16);
    info.set_margin_bottom(16);
    info.set_margin_start(16);
    info.set_margin_end(16);
    info.append(&info_row(
        "Selected item",
        &info_value("Example clip · 00:00:02:00"),
    ));
    info.append(&info_row(
        "Component package",
        &info_value("shrimply-gtk-components"),
    ));
    info.append(&info_row(
        "Frame graph",
        &info_value("Shared Rust renderer"),
    ));
    let home = gtk::glib::home_dir();
    let folder = ReadOnlyField::builder(home.display().to_string())
        .right_aligned()
        .action("folder-open-symbolic", "Show in Folder", {
        let home = home.clone();
        let log = log.clone();
        move |button| {
            if let Err(error) = shrimply_gtk_components::desktop_open::show_path_in_folder(
                button.upcast_ref(),
                home.clone(),
            ) {
                log(format!("show in folder failed: {error}"));
            }
        }
    })
        .build();
    info.append(&info_row("Home folder", &folder));
    let info_scroller = gtk::ScrolledWindow::builder()
        .child(&info)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .build();

    let log_page = gtk::Box::new(gtk::Orientation::Vertical, 10);
    log_page.set_margin_top(16);
    log_page.set_margin_bottom(16);
    log_page.set_margin_start(16);
    log_page.set_margin_end(16);
    let log_scroller = gtk::ScrolledWindow::builder()
        .child(&events)
        .hscrollbar_policy(gtk::PolicyType::Automatic)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .vexpand(true)
        .build();
    log_page.set_vexpand(true);
    log_page.append(&log_scroller);

    let pages = tabs([
        (
            "General".to_string(),
            "preferences-system-symbolic".to_string(),
            general_scroller.upcast(),
        ),
        (
            "Info".to_string(),
            "dialog-information-symbolic".to_string(),
            info_scroller.upcast(),
        ),
        (
            "Log".to_string(),
            "utilities-terminal-symbolic".to_string(),
            log_page.upcast(),
        ),
    ]);
    let toolbar = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();
    header.set_show_end_title_buttons(true);
    header.set_title_widget(Some(
        &gtk::Label::builder()
            .label("Shrimply GTK Components")
            .css_classes(["title"])
            .build(),
    ));
    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(&pages));
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Shrimply GTK Components")
        .default_width(920)
        .default_height(900)
        .content(&toolbar)
        .build();
    window.present();
}

fn info_value(value: &str) -> gtk::Label {
    let value = read_only_field(value);
    value.set_xalign(1.0);
    value
}

fn info_row(label: &str, child: &impl IsA<gtk::Widget>) -> gtk::Widget {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    row.set_hexpand(true);
    row.append(
        &gtk::Label::builder()
            .label(label)
            .halign(gtk::Align::Start)
            .valign(gtk::Align::Center)
            .width_chars(18)
            .xalign(0.0)
            .css_classes(["dim-label"])
            .build(),
    );
    child.set_hexpand(true);
    row.append(child);
    row.upcast()
}
