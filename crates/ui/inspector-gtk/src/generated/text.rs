use gtk::prelude::*;

use crate::InspectedItem as SelectedItem;
use crate::player_state::ProjectChange;
use crate::timeline_value::*;
use shrimply_inspector_core::generated::text::{
    COLOR_EDIT_COMMIT, DIRECTION_PATH, FONT_EDIT_COMMIT, FONT_FAMILIES_PATH, FONT_STYLE_PATH,
    FONT_VARIATION_EDIT_COMMIT, HORIZONTAL_ALIGN_PATH, SCALAR_EDIT_COMMIT, TEXT_EDIT_COMMIT,
    TEXT_EXPRESSION_COMMIT, TEXT_KEYFRAME_COMMITS, TEXT_PATH, VECTOR_EDIT_COMMIT,
    VERTICAL_ALIGN_PATH,
};
use shrimply_inspector_core::{ControlKind, InspectorControl};
use shrimply_project::project::{
    FontFamily, Project, TextItem, Time, VideoItem, VideoItemContent, generated_item_keyframe_span,
};

use crate::{
    InspectorContext,
    font_selector::font_selector_list,
    item::{DefaultInspectorItem, InspectorListItem},
    section::InspectorSection,
    timeline_value::color::{ColorTarget, color_control},
    timeline_value::scalar::{ScalarSpec, ScalarTarget},
    timeline_value::vector::vec2::{VecSpec, VecTarget, vec_control},
};

pub(crate) fn text_items(text: &TextItem, context: &InspectorContext) -> Vec<InspectorListItem> {
    let cards = shrimply_inspector_core::generated::text::cards(
        text,
        context.project.borrow().canvas_size,
        context.inspector_core.snapshot().runtime,
        Some(&shrimply_state::preferences::snapshot(&context.preferences).default_text_font_family),
    );
    vec![
        DefaultInspectorItem::new_with_default(
            cards[0].key,
            cards[0].title,
            text.clone(),
            text_content_controls,
            default_text,
            |context, value| reset_text_card(context, &value, 0),
        )
        .boxed(),
        DefaultInspectorItem::new_with_default(
            cards[1].key,
            cards[1].title,
            text.clone(),
            text_appearance_controls,
            default_text,
            |context, value| reset_text_card(context, &value, 1),
        )
        .preview_facet(
            cards[1]
                .preview_facet
                .expect("shared text appearance card must have a preview facet"),
        )
        .boxed(),
    ]
}

fn text_content_controls(value: &TextItem, context: &InspectorContext) -> Vec<gtk::Widget> {
    shared_controls(value, context, 0)
}

fn text_appearance_controls(value: &TextItem, context: &InspectorContext) -> Vec<gtk::Widget> {
    shared_controls(value, context, 1)
}

fn shared_controls(
    value: &TextItem,
    context: &InspectorContext,
    card_index: usize,
) -> Vec<gtk::Widget> {
    let card = shrimply_inspector_core::generated::text::cards(
        value,
        context.project.borrow().canvas_size,
        context.inspector_core.snapshot().runtime,
        Some(&shrimply_state::preferences::snapshot(&context.preferences).default_text_font_family),
    )[card_index]
        .clone();
    let section = InspectorSection::controls();
    for control in card.section.controls {
        section.add_wide_control(&text_control(value, &control, context));
    }
    vec![section.into_widget()]
}

fn text_control(
    text: &TextItem,
    control: &InspectorControl,
    context: &InspectorContext,
) -> gtk::Widget {
    match control.path.as_str() {
        TEXT_PATH => {
            assert_eq!(control.kind, ControlKind::LayeredText);
            assert_eq!(control.timeline_id, Some(text.text.id));
            assert_eq!(control.commit_name, TEXT_EDIT_COMMIT);
            assert_eq!(control.keyframe_commit_name, TEXT_KEYFRAME_COMMITS.toggle);
            assert_eq!(control.text_keyframe_commits, Some(TEXT_KEYFRAME_COMMITS));
            assert_eq!(control.expression_commit_name, TEXT_EXPRESSION_COMMIT);
            assert!(!control.commit_immediately);
            crate::timeline_value::text::text_control(&control.label, &text.text, context)
        }
        FONT_FAMILIES_PATH => font_families_row(control, &text.font_families, context),
        HORIZONTAL_ALIGN_PATH => text_step_button(
            control,
            &text.h_align,
            context,
            |text| &text.h_align,
            |text| &mut text.h_align,
        ),
        VERTICAL_ALIGN_PATH => text_step_button(
            control,
            &text.v_align,
            context,
            |text| &text.v_align,
            |text| &mut text.v_align,
        ),
        DIRECTION_PATH => text_step_button(
            control,
            &text.direction,
            context,
            |text| &text.direction,
            |text| &mut text.direction,
        ),
        FONT_STYLE_PATH => text_step_button(
            control,
            &text.font_style,
            context,
            |text| &text.font_style,
            |text| &mut text.font_style,
        ),
        path if path.starts_with("/content/font_variations/") => {
            font_variation_control(control, context)
        }
        "/content/font_size" => {
            scalar_control(control, &text.font_size, context, TextField::FontSize)
        }
        "/content/font_weight" => {
            scalar_control(control, &text.font_weight, context, TextField::FontWeight)
        }
        "/content/tracking" => {
            scalar_control(control, &text.tracking, context, TextField::Tracking)
        }
        "/content/line_height" => {
            scalar_control(control, &text.line_height, context, TextField::LineHeight)
        }
        "/content/background_roundness" => scalar_control(
            control,
            &text.background_roundness,
            context,
            TextField::BackgroundRoundness,
        ),
        "/content/outline_width" => scalar_control(
            control,
            &text.outline_width,
            context,
            TextField::OutlineWidth,
        ),
        "/content/shadow_distance" => scalar_control(
            control,
            &text.shadow_distance,
            context,
            TextField::ShadowDistance,
        ),
        "/content/shadow_direction_degrees" => scalar_control(
            control,
            &text.shadow_direction_degrees,
            context,
            TextField::ShadowDirectionDegrees,
        ),
        "/content/shadow_width" => {
            scalar_control(control, &text.shadow_width, context, TextField::ShadowWidth)
        }
        "/content/shadow_blur" => {
            scalar_control(control, &text.shadow_blur, context, TextField::ShadowBlur)
        }
        "/content/background_padding" => {
            background_padding_control(control, &text.background_padding, context)
        }
        "/content/color" => text_color_control(control, &text.color, context, text_color),
        "/content/background_color" => text_color_control(
            control,
            &text.background_color,
            context,
            text_background_color,
        ),
        "/content/outline_color" => {
            text_color_control(control, &text.outline_color, context, text_outline_color)
        }
        "/content/shadow_color" => {
            text_color_control(control, &text.shadow_color, context, text_shadow_color)
        }
        path => panic!("unsupported shared text control: {path}"),
    }
}

fn default_text(context: &InspectorContext) -> TextItem {
    let mut text = default_text_for_canvas(context.project.borrow().canvas_size);
    text.font_families =
        vec![shrimply_state::preferences::snapshot(&context.preferences).default_text_font_family];
    text
}

fn default_text_for_canvas(canvas_size: shrimply_project::project::CanvasSize) -> TextItem {
    let VideoItemContent::Text(text) =
        VideoItem::text_item(canvas_size, Time::ZERO, Time::ZERO).content
    else {
        unreachable!()
    };
    *text
}

fn reset_text_card(context: &InspectorContext, defaults: &TextItem, index: usize) {
    let Some(key) = context.selected_item.clone() else {
        return;
    };
    let default_font = defaults
        .font_families
        .first()
        .expect("default text must have a font family");
    let reset = shrimply_inspector_core::generated::text::cards(
        defaults,
        context.project.borrow().canvas_size,
        context.inspector_core.snapshot().runtime,
        Some(default_font),
    )[index]
        .reset
        .clone()
        .expect("shared text card must have reset behavior");
    if let Err(error) = context
        .inspector_core
        .reset_video(&shrimply_inspector_core::InspectorTarget::Item(key), &reset)
    {
        tracing::error!(%error, "Could not reset GTK text inspector card");
    }
}

const PERCENT: f64 = 100.0;
const MIN_LINE_HEIGHT: f32 = 0.01;

#[derive(Clone, Copy)]
enum TextField {
    FontSize,
    FontWeight,
    Tracking,
    LineHeight,
    BackgroundRoundness,
    OutlineWidth,
    ShadowDistance,
    ShadowDirectionDegrees,
    ShadowWidth,
    ShadowBlur,
}

fn font_families_row(
    control: &InspectorControl,
    value: &[FontFamily],
    context: &InspectorContext,
) -> gtk::Widget {
    assert_eq!(control.kind, ControlKind::FontFamilies);
    assert_eq!(control.commit_name, FONT_EDIT_COMMIT);
    let target = context
        .selected_item
        .clone()
        .map(shrimply_inspector_core::InspectorTarget::Item);
    let controller = context.inspector_core.clone();
    let path = control.path.clone();
    let commit = control.commit_name.clone();
    let fonts = font_selector_list(value, move |next| {
        let Some(target) = &target else {
            return;
        };
        let value = serde_json::to_string(&next).expect("text font families must serialize");
        if let Err(error) = controller.set_video_field(target, &path, &value, &commit, true) {
            tracing::error!(%error, "Could not update GTK text fonts");
        }
    });
    let row = crate::ui::control_row(&control.label, &fonts);
    row.first_child()
        .expect("font control row has a label")
        .set_valign(gtk::Align::Start);
    row
}

fn font_variation_control(control: &InspectorControl, context: &InspectorContext) -> gtk::Widget {
    assert_eq!(control.kind, ControlKind::Number);
    assert_eq!(control.commit_name, FONT_VARIATION_EDIT_COMMIT);
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let label = gtk::Label::builder()
        .label(&control.label)
        .halign(gtk::Align::Start)
        .hexpand(true)
        .tooltip_text(shrimply_gtk_components::i18n::text(&control.tooltip))
        .build();
    let spin = gtk::SpinButton::with_range(
        control.number.minimum,
        control.number.maximum,
        control.number.drag_step,
    );
    spin.set_digits(
        u32::try_from(control.number.digits).expect("font variation digits must be non-negative"),
    );
    spin.set_width_chars(control.width_characters);
    spin.set_value(
        control
            .value
            .parse()
            .expect("shared font variation value must be numeric"),
    );
    let target = context
        .selected_item
        .clone()
        .map(shrimply_inspector_core::InspectorTarget::Item);
    let controller = context.inspector_core.clone();
    let path = control.path.clone();
    let commit = control.commit_name.clone();
    spin.connect_value_changed(move |spin| {
        let Some(target) = &target else {
            return;
        };
        if let Err(error) =
            controller.set_video_field(target, &path, &spin.value().to_string(), &commit, true)
        {
            tracing::error!(%error, "Could not update GTK text font variation");
        }
    });
    row.append(&label);
    row.append(&spin);
    row.upcast()
}

fn text_step_button<T: TimelineStep>(
    control: &InspectorControl,
    value: &TimelineValue<T>,
    context: &InspectorContext,
    get: fn(&TextItem) -> &TimelineValue<T>,
    get_mut: fn(&mut TextItem) -> &mut TimelineValue<T>,
) -> gtk::Widget {
    assert_timeline_control(
        control,
        ControlKind::LayeredSelector,
        value.id,
        &control.commit_name,
    );
    let timeline_id = value.id;
    let commit_name = match control.path.as_str() {
        HORIZONTAL_ALIGN_PATH => "edit-text-horizontal-align",
        VERTICAL_ALIGN_PATH => "edit-text-vertical-align",
        DIRECTION_PATH => "edit-text-direction",
        FONT_STYLE_PATH => "edit-text-font-style",
        path => panic!("unsupported text step control: {path}"),
    };
    crate::timeline_value::step::step_button_control(
        &control.label,
        value,
        context,
        crate::timeline_value::step::StepTarget::new(
            move |project, key| {
                let value = selected_text(project, key).map(get)?;
                (value.id == timeline_id).then_some(value)
            },
            move |project, key| {
                let value = selected_text_mut(project, key).map(get_mut)?;
                (value.id == timeline_id).then_some(value)
            },
            commit_name,
            ProjectChange {
                video: true,
                inspector: true,
                ..Default::default()
            },
        ),
    )
}

fn scalar_control(
    control: &InspectorControl,
    value: &TimelineValue<f32>,
    context: &InspectorContext,
    field: TextField,
) -> gtk::Widget {
    assert_timeline_control(
        control,
        ControlKind::LayeredNumber,
        value.id,
        SCALAR_EDIT_COMMIT,
    );
    crate::timeline_value::scalar::scalar_control(
        &control.label,
        value,
        context,
        text_scalar_target(field, value.id),
        text_scalar_spec(control, field),
    )
}

fn background_padding_control(
    control: &InspectorControl,
    value: &TimelineValue<glam::Vec2>,
    context: &InspectorContext,
) -> gtk::Widget {
    assert_timeline_control(
        control,
        ControlKind::LayeredVector2,
        value.id,
        VECTOR_EDIT_COMMIT,
    );
    assert_eq!(control.prefixes, ["X", "Y"]);
    vec_control(
        &control.label,
        value,
        context,
        VecTarget {
            access: crate::timeline_value::vector::vec2::VecAccess::ItemScoped {
                get: text_background_padding,
                get_mut: text_background_padding_mut,
                value_id: value.id,
            },
            scope_id: Some(value.id),
            local_time: video_local_time_for_key,
            duration: video_duration_for_key,
            refresh: ProjectChange {
                video: true,
                inspector: true,
                ..ProjectChange::default()
            },
            commit_name: VECTOR_EDIT_COMMIT,
        },
        VecSpec {
            first_prefix: "X",
            second_prefix: "Y",
            drag_step: control.number.drag_step,
            digits: usize::try_from(control.number.digits)
                .expect("text vector digits must be non-negative"),
            width_chars: control.width_characters,
            minimum: Some(control.number.minimum),
            maximum: None,
            unit_name: control.number.unit,
        },
    )
}

fn text_scalar_target(field: TextField, value_id: uuid::Uuid) -> ScalarTarget {
    ScalarTarget {
        access: crate::timeline_value::scalar::ScalarAccess::ItemScoped {
            get: match field {
                TextField::FontSize => text_font_size,
                TextField::FontWeight => text_font_weight,
                TextField::Tracking => text_tracking,
                TextField::LineHeight => text_line_height,
                TextField::BackgroundRoundness => text_background_roundness,
                TextField::OutlineWidth => text_outline_width,
                TextField::ShadowDistance => text_shadow_distance,
                TextField::ShadowDirectionDegrees => text_shadow_direction_degrees,
                TextField::ShadowWidth => text_shadow_width,
                TextField::ShadowBlur => text_shadow_blur,
            },
            get_mut: match field {
                TextField::FontSize => text_font_size_mut,
                TextField::FontWeight => text_font_weight_mut,
                TextField::Tracking => text_tracking_mut,
                TextField::LineHeight => text_line_height_mut,
                TextField::BackgroundRoundness => text_background_roundness_mut,
                TextField::OutlineWidth => text_outline_width_mut,
                TextField::ShadowDistance => text_shadow_distance_mut,
                TextField::ShadowDirectionDegrees => text_shadow_direction_degrees_mut,
                TextField::ShadowWidth => text_shadow_width_mut,
                TextField::ShadowBlur => text_shadow_blur_mut,
            },
            value_id,
        },
        scope_id: Some(value_id),
        local_time: video_local_time_for_key,
        duration: video_duration_for_key,
        refresh: ProjectChange {
            video: true,
            inspector: true,
            ..ProjectChange::default()
        },
        commit_name: SCALAR_EDIT_COMMIT,
    }
}

fn text_scalar_spec(control: &InspectorControl, field: TextField) -> ScalarSpec {
    ScalarSpec {
        drag_step: control.number.drag_step,
        digits: usize::try_from(control.number.digits)
            .expect("text scalar digits must be non-negative"),
        integer: false,
        width_chars: control.width_characters,
        minimum: match field {
            TextField::ShadowDirectionDegrees | TextField::Tracking => None,
            TextField::FontSize | TextField::FontWeight | TextField::LineHeight => Some(1.0),
            TextField::OutlineWidth
            | TextField::BackgroundRoundness
            | TextField::ShadowDistance
            | TextField::ShadowWidth
            | TextField::ShadowBlur => Some(0.0),
        },
        maximum: matches!(field, TextField::FontWeight).then_some(control.number.maximum),
        unit_name: match field {
            TextField::FontWeight => None,
            TextField::LineHeight => Some("%"),
            TextField::ShadowDirectionDegrees => Some("deg"),
            _ => Some("px"),
        },
        rotating_icon: match field {
            TextField::ShadowDirectionDegrees => Some(("arrow3-up-symbolic", 90.0)),
            _ => None,
        },
        display: match field {
            TextField::LineHeight => |value| value as f64 * PERCENT,
            _ => |value| value as f64,
        },
        store: match field {
            TextField::LineHeight => |value| (value / PERCENT) as f32,
            _ => |value| value as f32,
        },
        clamp: crate::timeline_value::scalar::ScalarClamp::Function(match field {
            TextField::FontSize => |value| value.max(1.0),
            TextField::FontWeight => |value| value.round().clamp(1.0, 1000.0),
            TextField::LineHeight => |value| value.max(MIN_LINE_HEIGHT),
            TextField::OutlineWidth
            | TextField::BackgroundRoundness
            | TextField::ShadowDistance
            | TextField::ShadowWidth
            | TextField::ShadowBlur => |value| value.max(0.0),
            TextField::ShadowDirectionDegrees | TextField::Tracking => |value| value,
        }),
    }
}

fn text_color_control(
    control: &InspectorControl,
    value: &TimelineValue<shrimply_core::Color<u8>>,
    context: &InspectorContext,
    get_mut: fn(&mut Project, SelectedItem) -> Option<&mut TimelineValue<shrimply_core::Color<u8>>>,
) -> gtk::Widget {
    assert_timeline_control(
        control,
        ControlKind::LayeredColor,
        value.id,
        COLOR_EDIT_COMMIT,
    );
    color_control(
        &control.label,
        value,
        context,
        ColorTarget {
            access: crate::timeline_value::color::ColorAccess::ItemScoped {
                get_mut,
                value_id: value.id,
            },
            scope_id: Some(value.id),
            local_time: video_local_time_for_key,
            duration: video_duration_for_key,
            refresh: ProjectChange {
                video: true,
                inspector: true,
                ..ProjectChange::default()
            },
            commit_name: COLOR_EDIT_COMMIT,
        },
    )
}

fn assert_timeline_control(
    control: &InspectorControl,
    kind: ControlKind,
    timeline_id: uuid::Uuid,
    commit: &str,
) {
    assert_eq!(control.kind, kind);
    assert_eq!(control.timeline_id, Some(timeline_id));
    assert_eq!(control.commit_name, commit);
    assert_eq!(control.keyframe_commit_name, commit);
    assert_eq!(control.expression_commit_name, commit);
    assert!(!control.commit_immediately);
}

fn selected_text(project: &Project, key: SelectedItem) -> Option<&TextItem> {
    let item = project.video_item(&key)?;
    let VideoItemContent::Text(text) = &item.content else {
        return None;
    };
    Some(text)
}

fn selected_text_mut(project: &mut Project, key: SelectedItem) -> Option<&mut TextItem> {
    let item = project.video_item_mut(&key)?;
    let VideoItemContent::Text(text) = &mut item.content else {
        return None;
    };
    Some(text)
}

fn text_font_size(project: &Project, key: SelectedItem) -> Option<&TimelineValue<f32>> {
    selected_text(project, key.clone()).map(|text| &text.font_size)
}

fn text_font_size_mut(project: &mut Project, key: SelectedItem) -> Option<&mut TimelineValue<f32>> {
    selected_text_mut(project, key.clone()).map(|text| &mut text.font_size)
}

fn text_font_weight(project: &Project, key: SelectedItem) -> Option<&TimelineValue<f32>> {
    selected_text(project, key.clone()).map(|text| &text.font_weight)
}

fn text_font_weight_mut(
    project: &mut Project,
    key: SelectedItem,
) -> Option<&mut TimelineValue<f32>> {
    selected_text_mut(project, key.clone()).map(|text| &mut text.font_weight)
}

fn text_tracking(project: &Project, key: SelectedItem) -> Option<&TimelineValue<f32>> {
    selected_text(project, key.clone()).map(|text| &text.tracking)
}

fn text_tracking_mut(project: &mut Project, key: SelectedItem) -> Option<&mut TimelineValue<f32>> {
    selected_text_mut(project, key.clone()).map(|text| &mut text.tracking)
}

fn text_line_height(project: &Project, key: SelectedItem) -> Option<&TimelineValue<f32>> {
    selected_text(project, key.clone()).map(|text| &text.line_height)
}

fn text_line_height_mut(
    project: &mut Project,
    key: SelectedItem,
) -> Option<&mut TimelineValue<f32>> {
    selected_text_mut(project, key.clone()).map(|text| &mut text.line_height)
}

fn text_outline_width(project: &Project, key: SelectedItem) -> Option<&TimelineValue<f32>> {
    selected_text(project, key.clone()).map(|text| &text.outline_width)
}

fn text_background_roundness(project: &Project, key: SelectedItem) -> Option<&TimelineValue<f32>> {
    selected_text(project, key.clone()).map(|text| &text.background_roundness)
}

fn text_background_roundness_mut(
    project: &mut Project,
    key: SelectedItem,
) -> Option<&mut TimelineValue<f32>> {
    selected_text_mut(project, key.clone()).map(|text| &mut text.background_roundness)
}

fn text_background_padding(
    project: &Project,
    key: SelectedItem,
) -> Option<&TimelineValue<glam::Vec2>> {
    selected_text(project, key.clone()).map(|text| &text.background_padding)
}

fn text_background_padding_mut(
    project: &mut Project,
    key: SelectedItem,
) -> Option<&mut TimelineValue<glam::Vec2>> {
    selected_text_mut(project, key.clone()).map(|text| &mut text.background_padding)
}

fn text_outline_width_mut(
    project: &mut Project,
    key: SelectedItem,
) -> Option<&mut TimelineValue<f32>> {
    selected_text_mut(project, key.clone()).map(|text| &mut text.outline_width)
}

fn text_shadow_distance(project: &Project, key: SelectedItem) -> Option<&TimelineValue<f32>> {
    selected_text(project, key.clone()).map(|text| &text.shadow_distance)
}

fn text_shadow_distance_mut(
    project: &mut Project,
    key: SelectedItem,
) -> Option<&mut TimelineValue<f32>> {
    selected_text_mut(project, key.clone()).map(|text| &mut text.shadow_distance)
}

fn text_shadow_direction_degrees(
    project: &Project,
    key: SelectedItem,
) -> Option<&TimelineValue<f32>> {
    selected_text(project, key.clone()).map(|text| &text.shadow_direction_degrees)
}

fn text_shadow_direction_degrees_mut(
    project: &mut Project,
    key: SelectedItem,
) -> Option<&mut TimelineValue<f32>> {
    selected_text_mut(project, key.clone()).map(|text| &mut text.shadow_direction_degrees)
}

fn text_shadow_width(project: &Project, key: SelectedItem) -> Option<&TimelineValue<f32>> {
    selected_text(project, key.clone()).map(|text| &text.shadow_width)
}

fn text_shadow_width_mut(
    project: &mut Project,
    key: SelectedItem,
) -> Option<&mut TimelineValue<f32>> {
    selected_text_mut(project, key.clone()).map(|text| &mut text.shadow_width)
}

fn text_shadow_blur(project: &Project, key: SelectedItem) -> Option<&TimelineValue<f32>> {
    selected_text(project, key.clone()).map(|text| &text.shadow_blur)
}

fn text_shadow_blur_mut(
    project: &mut Project,
    key: SelectedItem,
) -> Option<&mut TimelineValue<f32>> {
    selected_text_mut(project, key.clone()).map(|text| &mut text.shadow_blur)
}

fn text_color(
    project: &mut Project,
    key: SelectedItem,
) -> Option<&mut TimelineValue<shrimply_core::Color<u8>>> {
    selected_text_mut(project, key.clone()).map(|text| &mut text.color)
}

fn text_outline_color(
    project: &mut Project,
    key: SelectedItem,
) -> Option<&mut TimelineValue<shrimply_core::Color<u8>>> {
    selected_text_mut(project, key.clone()).map(|text| &mut text.outline_color)
}

fn text_background_color(
    project: &mut Project,
    key: SelectedItem,
) -> Option<&mut TimelineValue<shrimply_core::Color<u8>>> {
    selected_text_mut(project, key.clone()).map(|text| &mut text.background_color)
}

fn text_shadow_color(
    project: &mut Project,
    key: SelectedItem,
) -> Option<&mut TimelineValue<shrimply_core::Color<u8>>> {
    selected_text_mut(project, key.clone()).map(|text| &mut text.shadow_color)
}

fn video_local_time_for_key(project: &Project, key: SelectedItem, position: Time) -> Option<Time> {
    crate::video::visual_local_time(project, key, position)
}

fn video_duration_for_key(project: &Project, key: SelectedItem) -> Option<Time> {
    let item = project.video_item(&key)?;
    generated_item_keyframe_span(item)
        .map(|(_, end)| end)
        .or_else(|| crate::video::visual_duration(project, key))
}
