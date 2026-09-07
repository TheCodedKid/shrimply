use serde_json::{Value, json};
use shrimply_project::project::{CaptionItem, Color};

use crate::{ControlKind, InspectorControl, InspectorDetail, InspectorSection, NumberSpec};

use super::{BasicInspectorAction, CategoryIcon, InspectorCategory, InspectorItem, detail_item};

pub(super) fn categories(value: &Value, details: &[InspectorDetail]) -> Vec<InspectorCategory> {
    let caption: CaptionItem =
        serde_json::from_value(value.clone()).expect("caption inspector value must be valid");
    vec![
        InspectorCategory {
            key: "text",
            label: "Text",
            icon: CategoryIcon::Text,
            items: vec![text_item(&caption).boxed()],
        },
        InspectorCategory {
            key: "visual",
            label: "Visual",
            icon: CategoryIcon::Visual,
            items: vec![
                layout_item(&caption).boxed(),
                appearance_item(&caption).boxed(),
            ],
        },
        InspectorCategory {
            key: "info",
            label: "Info",
            icon: CategoryIcon::Info,
            items: vec![detail_item(details)],
        },
    ]
}

fn text_item(caption: &CaptionItem) -> InspectorItem {
    let value = crate::caption::CaptionText::from(caption);
    let mut section = InspectorSection::default();
    section
        .add(InspectorControl::new(ControlKind::MultilineText, "/text", "Text").value(value.text));
    section.add(selector(
        "/writing_direction",
        "Writing",
        value.writing_direction,
        crate::caption::WRITING_DIRECTIONS,
    ));
    InspectorItem::new("caption-text", "Text", section).reset(BasicInspectorAction::ResetFields {
        values: vec![(
            "/writing_direction".to_string(),
            json!(crate::caption::CaptionText::default().writing_direction),
        )],
    })
}

fn layout_item(caption: &CaptionItem) -> InspectorItem {
    let layout = crate::caption::CaptionLayout::from(caption);
    let mut section = InspectorSection::default();
    section.add(selector(
        "/h_align",
        "H align",
        layout.horizontal_align,
        crate::caption::HORIZONTAL_ALIGNMENTS,
    ));
    section.add(selector(
        "/v_align",
        "V align",
        layout.vertical_align,
        crate::caption::VERTICAL_ALIGNMENTS,
    ));
    section.add(number(
        "/position_x",
        u16::from(layout.position_x),
        crate::caption::POSITION_X,
    ));
    section.add(number(
        "/position_y",
        u16::from(layout.position_y),
        crate::caption::POSITION_Y,
    ));
    section.set_sensitive(layout.enabled);
    let defaults = crate::caption::CaptionLayout::default();
    InspectorItem::new("caption-layout", "Layout", section)
        .reset(BasicInspectorAction::ResetFields {
            values: vec![
                ("/layout_enabled".to_string(), Value::Bool(defaults.enabled)),
                ("/h_align".to_string(), json!(defaults.horizontal_align)),
                ("/v_align".to_string(), json!(defaults.vertical_align)),
                ("/position_x".to_string(), json!(defaults.position_x)),
                ("/position_y".to_string(), json!(defaults.position_y)),
            ],
        })
        .toggle(crate::item::HeaderToggle {
            active: layout.enabled,
            tooltip: "Enable layout",
            activate: BasicInspectorAction::SetBoolean {
                path: "/layout_enabled".to_string(),
                value: !layout.enabled,
            },
        })
}

fn appearance_item(caption: &CaptionItem) -> InspectorItem {
    let appearance = crate::caption::CaptionAppearance::from(caption);
    let mut section = InspectorSection::default();
    section.add(number(
        "/font_scale",
        appearance.font_scale,
        crate::caption::FONT_SCALE,
    ));
    section.add(selector(
        "/font",
        "Font",
        appearance.font,
        crate::caption::FONTS,
    ));
    section.add(selector(
        "/edge_style",
        "Edge",
        appearance.edge_style,
        crate::caption::EDGE_STYLES,
    ));
    section.add(color("/text_color", "Text color", appearance.text_color));
    section.add(color(
        "/background_color",
        "Background",
        appearance.background_color,
    ));
    section.add(color("/edge_color", "Edge color", appearance.edge_color));
    section.set_sensitive(appearance.enabled);
    let defaults = crate::caption::CaptionAppearance::default();
    InspectorItem::new("caption-appearance", "Appearance", section)
        .reset(BasicInspectorAction::ResetFields {
            values: vec![
                (
                    "/styling_enabled".to_string(),
                    Value::Bool(defaults.enabled),
                ),
                ("/font_scale".to_string(), json!(defaults.font_scale)),
                ("/font".to_string(), json!(defaults.font)),
                ("/edge_style".to_string(), json!(defaults.edge_style)),
                ("/text_color".to_string(), json!(defaults.text_color)),
                (
                    "/background_color".to_string(),
                    json!(defaults.background_color),
                ),
                ("/edge_color".to_string(), json!(defaults.edge_color)),
            ],
        })
        .toggle(crate::item::HeaderToggle {
            active: appearance.enabled,
            tooltip: "Enable styling",
            activate: BasicInspectorAction::SetBoolean {
                path: "/styling_enabled".to_string(),
                value: !appearance.enabled,
            },
        })
}

fn number(
    path: &str,
    value: u16,
    presentation: crate::caption::CaptionNumberPresentation,
) -> InspectorControl {
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
    choices: &[crate::caption::CaptionChoice<T>],
) -> InspectorControl {
    crate::selector::selector(
        path,
        label,
        crate::caption::choice(choices, selected).key,
        choices
            .iter()
            .map(|choice| (choice.key.to_string(), choice.label.to_string())),
    )
}

fn color(path: &str, label: &str, color: Color<u8>) -> InspectorControl {
    InspectorControl::new(ControlKind::Color, path, label).components(
        [color.r, color.g, color.b, color.a]
            .map(|component| component.to_string())
            .to_vec(),
    )
}
