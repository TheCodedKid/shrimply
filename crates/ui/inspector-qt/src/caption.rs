use serde_json::{Value, json};
use shrimply_inspector_core::{
    InspectorDetail,
    caption::{
        self, CaptionAppearance, CaptionChoice, CaptionLayout, CaptionNumberPresentation,
        CaptionText,
    },
};
use shrimply_project::project::{CaptionItem, Color};

use crate::item::{HeaderToggle, InspectorAction, InspectorItem};
use crate::list::InspectorCategory;
use crate::section::{ControlKind, InspectorControl, InspectorSection, NumberSpec};

pub(crate) fn categories(caption: &Value, details: &[InspectorDetail]) -> Vec<InspectorCategory> {
    let caption: CaptionItem =
        serde_json::from_value(caption.clone()).expect("caption inspector value must be valid");
    vec![
        InspectorCategory {
            key: "text",
            label: "Text",
            icon: "insert-text-symbolic",
            items: vec![text_item(&caption).boxed()],
        },
        InspectorCategory {
            key: "visual",
            label: "Visual",
            icon: "blend-tool-symbolic",
            items: vec![
                layout_item(&caption).boxed(),
                appearance_item(&caption).boxed(),
            ],
        },
        InspectorCategory {
            key: "info",
            label: "Info",
            icon: "info-outline-symbolic",
            items: vec![info_item(details).boxed()],
        },
    ]
}

fn text_item(caption_item: &CaptionItem) -> InspectorItem {
    let value = CaptionText::from(caption_item);
    let mut section = InspectorSection::default();
    section
        .add(InspectorControl::new(ControlKind::MultilineText, "/text", "Text").value(value.text));
    section.add(selector(
        "/writing_direction",
        "Writing",
        value.writing_direction,
        caption::WRITING_DIRECTIONS,
    ));

    let defaults = CaptionText::default();
    InspectorItem::new("caption-text", "Text", section).reset(InspectorAction::ResetFields {
        values: vec![(
            "/writing_direction".to_string(),
            serde_json::to_value(defaults.writing_direction)
                .expect("caption writing direction must serialize"),
        )],
    })
}

fn layout_item(caption_item: &CaptionItem) -> InspectorItem {
    let layout = CaptionLayout::from(caption_item);
    let mut section = InspectorSection::default();
    section.add(button_selector(
        "/h_align",
        "H align",
        layout.horizontal_align,
        caption::HORIZONTAL_ALIGNMENTS,
    ));
    section.add(button_selector(
        "/v_align",
        "V align",
        layout.vertical_align,
        caption::VERTICAL_ALIGNMENTS,
    ));
    section.add(number(
        "/position_x",
        u16::from(layout.position_x),
        caption::POSITION_X,
    ));
    section.add(number(
        "/position_y",
        u16::from(layout.position_y),
        caption::POSITION_Y,
    ));
    section.set_sensitive(layout.enabled);

    let defaults = CaptionLayout::default();
    InspectorItem::new("caption-layout", "Layout", section)
        .reset(InspectorAction::ResetFields {
            values: vec![
                ("/layout_enabled".to_string(), Value::Bool(defaults.enabled)),
                (
                    "/h_align".to_string(),
                    serde_json::to_value(defaults.horizontal_align)
                        .expect("caption horizontal alignment must serialize"),
                ),
                (
                    "/v_align".to_string(),
                    serde_json::to_value(defaults.vertical_align)
                        .expect("caption vertical alignment must serialize"),
                ),
                ("/position_x".to_string(), json!(defaults.position_x)),
                ("/position_y".to_string(), json!(defaults.position_y)),
            ],
        })
        .toggle(HeaderToggle {
            active: layout.enabled,
            tooltip: "Enable layout",
            activate: InspectorAction::SetBoolean {
                path: "/layout_enabled".to_string(),
                value: !layout.enabled,
            },
        })
}

fn appearance_item(caption_item: &CaptionItem) -> InspectorItem {
    let appearance = CaptionAppearance::from(caption_item);
    let mut section = InspectorSection::default();
    section.add(number(
        "/font_scale",
        appearance.font_scale,
        caption::FONT_SCALE,
    ));
    section.add(selector("/font", "Font", appearance.font, caption::FONTS));
    section.add(selector(
        "/edge_style",
        "Edge",
        appearance.edge_style,
        caption::EDGE_STYLES,
    ));
    section.add(color("/text_color", "Text color", appearance.text_color));
    section.add(color(
        "/background_color",
        "Background",
        appearance.background_color,
    ));
    section.add(color("/edge_color", "Edge color", appearance.edge_color));
    section.set_sensitive(appearance.enabled);

    let defaults = CaptionAppearance::default();
    InspectorItem::new("caption-appearance", "Appearance", section)
        .reset(InspectorAction::ResetFields {
            values: vec![
                (
                    "/styling_enabled".to_string(),
                    Value::Bool(defaults.enabled),
                ),
                ("/font_scale".to_string(), json!(defaults.font_scale)),
                (
                    "/font".to_string(),
                    serde_json::to_value(defaults.font).expect("caption font must serialize"),
                ),
                (
                    "/edge_style".to_string(),
                    serde_json::to_value(defaults.edge_style)
                        .expect("caption edge style must serialize"),
                ),
                (
                    "/text_color".to_string(),
                    serde_json::to_value(defaults.text_color)
                        .expect("caption text color must serialize"),
                ),
                (
                    "/background_color".to_string(),
                    serde_json::to_value(defaults.background_color)
                        .expect("caption background color must serialize"),
                ),
                (
                    "/edge_color".to_string(),
                    serde_json::to_value(defaults.edge_color)
                        .expect("caption edge color must serialize"),
                ),
            ],
        })
        .toggle(HeaderToggle {
            active: appearance.enabled,
            tooltip: "Enable styling",
            activate: InspectorAction::SetBoolean {
                path: "/styling_enabled".to_string(),
                value: !appearance.enabled,
            },
        })
}

fn info_item(details: &[InspectorDetail]) -> InspectorItem {
    let mut section = InspectorSection::default();
    for detail in details {
        section.add(
            InspectorControl::new(ControlKind::ReadOnly, "", detail.label)
                .value(detail.value.clone())
                .read_only(),
        );
    }
    InspectorItem::new("info", "Info", section)
}

fn number(path: &str, value: u16, presentation: CaptionNumberPresentation) -> InspectorControl {
    InspectorControl::new(ControlKind::Number, path, presentation.label)
        .value(value.to_string())
        .number(NumberSpec {
            minimum: f64::from(presentation.minimum),
            maximum: f64::from(presentation.maximum),
            drag_step: presentation.drag_step,
            digits: presentation.digits as i32,
            unit: presentation.unit,
        })
        .accepted_range(
            f64::from(presentation.minimum),
            f64::from(presentation.maximum),
        )
        .width_characters(6)
}

fn selector<T: Copy + Eq>(
    path: &str,
    label: &str,
    selected: T,
    choices: &[CaptionChoice<T>],
) -> InspectorControl {
    crate::selector::selector(
        path,
        label,
        caption::choice(choices, selected).key,
        choices
            .iter()
            .map(|choice| (choice.key.to_string(), choice.label.to_string())),
    )
}

fn button_selector<T: Copy + Eq>(
    path: &str,
    label: &str,
    selected: T,
    choices: &[CaptionChoice<T>],
) -> InspectorControl {
    crate::selector::button_selector(
        path,
        label,
        caption::choice(choices, selected).key,
        choices.iter().map(|choice| {
            (
                choice.key.to_string(),
                choice.label.to_string(),
                choice
                    .icon
                    .expect("caption button choices must provide icons")
                    .qt
                    .to_string(),
            )
        }),
    )
}

fn color(path: &str, label: &str, color: Color<u8>) -> InspectorControl {
    InspectorControl::new(ControlKind::Color, path, label).components(
        [color.r, color.g, color.b, color.a]
            .map(|component| component.to_string())
            .to_vec(),
    )
}
