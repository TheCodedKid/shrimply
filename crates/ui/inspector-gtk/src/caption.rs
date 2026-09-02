use shrimply_gtk_components::tr;
use std::cell::RefCell;
use std::rc::Rc;

use crate::InspectedItem as SelectedItem;
use crate::player_state::{self, ProjectChange, SharedPlayerState};
use crate::ui::NumberPicker;
use adw::prelude::*;
use shrimply_gtk_components::ui::{ColorPicker, MultilineTextInput, switch_row};
use shrimply_inspector_core::caption::{
    self, CaptionAppearance, CaptionChoice, CaptionLayout, CaptionNumberPresentation, CaptionText,
};
use shrimply_project::project::{
    CaptionEdgeStyle, CaptionFont, CaptionItem, CaptionWritingDirection, Color, HorizontalAlign,
    Project, VerticalAlign,
};

use super::{
    Inspectable, InspectorContext,
    item::{DefaultInspectorItem, HeaderToggle},
    list,
    section::InspectorSection,
    selector::selector,
};

impl Inspectable for CaptionItem {
    fn title(&self) -> &'static str {
        "Caption"
    }

    fn add_rows(&self, section: &InspectorSection, context: &InspectorContext) {
        let Some(key) = context.selected_item.clone() else {
            return;
        };
        add_text_editor(section, context, key.clone(), &self.text);
        add_enabled_editor(
            section,
            context,
            key.clone(),
            "Enable layout",
            self.layout_enabled,
            |item, enabled| item.layout_enabled = enabled,
        );
        add_enabled_editor(
            section,
            context,
            key.clone(),
            "Enable styling",
            self.styling_enabled,
            |item, enabled| item.styling_enabled = enabled,
        );
        add_h_align_editor(section, context, key.clone(), self.h_align);
        add_v_align_editor(section, context, key.clone(), self.v_align);
        add_number_editor(
            section,
            context,
            key.clone(),
            caption::POSITION_X,
            self.position_x.into(),
            |item, value| item.position_x = value as u8,
        );
        add_number_editor(
            section,
            context,
            key.clone(),
            caption::POSITION_Y,
            self.position_y.into(),
            |item, value| item.position_y = value as u8,
        );
        add_number_editor(
            section,
            context,
            key.clone(),
            caption::FONT_SCALE,
            self.font_scale,
            |item, value| item.font_scale = value,
        );
        add_font_editor(section, context, key.clone(), self.font);
        add_direction_editor(section, context, key.clone(), self.writing_direction);
        add_edge_editor(section, context, key.clone(), self.edge_style);
        add_color_editor(
            section,
            context,
            key.clone(),
            "Text color",
            self.text_color,
            |item, value| item.text_color = value,
        );
        add_color_editor(
            section,
            context,
            key.clone(),
            "Background",
            self.background_color,
            |item, value| item.background_color = value,
        );
        add_color_editor(
            section,
            context,
            key.clone(),
            "Edge color",
            self.edge_color,
            |item, value| item.edge_color = value,
        );
    }

    fn inspect(&self, context: &InspectorContext) -> Vec<gtk::Widget> {
        let info_items = vec![super::info::item(
            context,
            super::info::ItemInfo {
                leading: Vec::new(),
                kind: "Caption",
                natural_duration: None,
                start: self.start,
                end: self.end,
                source_offset: None,
                dimensions: None,
                file: None,
                source_metadata: super::info::SourceMetadata::None,
            },
        )];
        let text_items = vec![
            DefaultInspectorItem::new(
                "caption-text",
                "Text",
                CaptionText::from(self),
                |value, context| {
                    let section = InspectorSection::controls();
                    if let Some(key) = context.selected_item.clone() {
                        add_text_editor(&section, context, key.clone(), &value.text);
                        add_direction_editor(
                            &section,
                            context,
                            key.clone(),
                            value.writing_direction,
                        );
                    }
                    vec![section.into_widget()]
                },
                |context, value: CaptionText| {
                    apply_caption_reset(context, "reset-caption-text", move |item| {
                        item.writing_direction = value.writing_direction
                    });
                },
            )
            .boxed(),
        ];
        let visual_items = vec![
            DefaultInspectorItem::new(
                "caption-layout",
                "Layout",
                CaptionLayout::from(self),
                caption_layout_controls,
                |context, value: CaptionLayout| {
                    apply_caption_reset(context, "reset-caption-layout", move |item| {
                        value.apply(item)
                    });
                },
            )
            .toggle(enabled_header_toggle(
                context,
                self.layout_enabled,
                "Enable layout",
                |item, enabled| item.layout_enabled = enabled,
            ))
            .boxed(),
            DefaultInspectorItem::new(
                "caption-appearance",
                "Appearance",
                CaptionAppearance::from(self),
                caption_appearance_controls,
                |context, value: CaptionAppearance| {
                    apply_caption_reset(context, "reset-caption-appearance", move |item| {
                        value.apply(item)
                    });
                },
            )
            .toggle(enabled_header_toggle(
                context,
                self.styling_enabled,
                "Enable styling",
                |item, enabled| item.styling_enabled = enabled,
            ))
            .boxed(),
        ];
        list::render_categories(
            vec![
                list::InspectorCategory {
                    key: "text",
                    label: "Text",
                    icon: "insert-text-symbolic",
                    items: text_items,
                },
                list::InspectorCategory {
                    key: "visual",
                    label: "Visual",
                    icon: "blend-tool-symbolic",
                    items: visual_items,
                },
                list::InspectorCategory {
                    key: "info",
                    label: "Info",
                    icon: "info-outline-symbolic",
                    items: info_items,
                },
            ],
            context,
        )
    }
}

fn enabled_header_toggle(
    context: &InspectorContext,
    active: bool,
    tooltip: &'static str,
    set: fn(&mut CaptionItem, bool),
) -> HeaderToggle {
    let project = context.project.clone();
    let player_state = context.player_state.clone();
    let refresh = context.refresh.clone();
    let key = context.selected_item.clone().unwrap();
    HeaderToggle {
        active,
        tooltip,
        activate: Rc::new(move |enabled| {
            update_caption(&project, &player_state, key.clone(), |item| {
                set(item, enabled)
            });
            refresh();
        }),
    }
}

fn caption_layout_controls(value: &CaptionLayout, context: &InspectorContext) -> Vec<gtk::Widget> {
    let section = InspectorSection::controls();
    let Some(key) = context.selected_item.clone() else {
        return vec![section.into_widget()];
    };
    add_h_align_editor(&section, context, key.clone(), value.horizontal_align);
    add_v_align_editor(&section, context, key.clone(), value.vertical_align);
    add_number_editor(
        &section,
        context,
        key.clone(),
        caption::POSITION_X,
        value.position_x.into(),
        |item, value| item.position_x = value as u8,
    );
    add_number_editor(
        &section,
        context,
        key.clone(),
        caption::POSITION_Y,
        value.position_y.into(),
        |item, value| item.position_y = value as u8,
    );
    let controls = section.into_widget();
    controls.set_sensitive(value.enabled);
    vec![controls]
}

fn caption_appearance_controls(
    value: &CaptionAppearance,
    context: &InspectorContext,
) -> Vec<gtk::Widget> {
    let section = InspectorSection::controls();
    let Some(key) = context.selected_item.clone() else {
        return vec![section.into_widget()];
    };
    add_number_editor(
        &section,
        context,
        key.clone(),
        caption::FONT_SCALE,
        value.font_scale,
        |item, value| item.font_scale = value,
    );
    add_font_editor(&section, context, key.clone(), value.font);
    add_edge_editor(&section, context, key.clone(), value.edge_style);
    add_color_editor(
        &section,
        context,
        key.clone(),
        "Text color",
        value.text_color,
        |item, value| item.text_color = value,
    );
    add_color_editor(
        &section,
        context,
        key.clone(),
        "Background",
        value.background_color,
        |item, value| item.background_color = value,
    );
    add_color_editor(
        &section,
        context,
        key.clone(),
        "Edge color",
        value.edge_color,
        |item, value| item.edge_color = value,
    );
    let controls = section.into_widget();
    controls.set_sensitive(value.enabled);
    vec![controls]
}

fn apply_caption_reset(
    context: &InspectorContext,
    commit_name: &'static str,
    update: impl FnOnce(&mut CaptionItem),
) {
    let Some(key) = context.selected_item.clone() else {
        return;
    };
    let mut project = context.project.borrow_mut();
    let Some(item) = project.caption_item_mut(&key) else {
        return;
    };
    update(item);
    shrimply_project::project::commit_edit(&project, commit_name);
    drop(project);
    player_state::refresh_project(
        &context.player_state,
        ProjectChange {
            captions: true,
            inspector: true,
            ..ProjectChange::default()
        },
    );
}

fn add_text_editor(
    section: &InspectorSection,
    context: &InspectorContext,
    key: SelectedItem,
    text: &str,
) {
    let project = context.project.clone();
    let player_state = context.player_state.clone();
    let commit_project = context.project.clone();
    let editor = MultilineTextInput::builder(text)
        .on_change(move |text| update_caption_text_live(&project, &player_state, key.clone(), text))
        .on_commit(move || {
            shrimply_project::project::commit_edit(&commit_project.borrow(), "caption");
        })
        .build();
    section.add_control_row("Text", editor.widget());
}

fn add_enabled_editor(
    section: &InspectorSection,
    context: &InspectorContext,
    key: SelectedItem,
    label: &str,
    enabled: bool,
    set: fn(&mut CaptionItem, bool),
) {
    let project = context.project.clone();
    let player_state = context.player_state.clone();
    let refresh = context.refresh.clone();
    let row = switch_row(label, None, enabled, move |enabled| {
        update_caption(&project, &player_state, key.clone(), |item| {
            set(item, enabled);
        });
        refresh();
    });
    section.add_wide_control(&row);
}

fn add_number_editor(
    section: &InspectorSection,
    context: &InspectorContext,
    key: SelectedItem,
    presentation: CaptionNumberPresentation,
    value: u16,
    set: fn(&mut CaptionItem, u16),
) {
    let project = context.project.clone();
    let player_state = context.player_state.clone();
    let picker = NumberPicker::builder(f64::from(value))
        .drag_step(presentation.drag_step)
        .digits(presentation.digits)
        .width_chars(6)
        .minimum(f64::from(presentation.minimum))
        .accepted_range(
            f64::from(presentation.minimum),
            f64::from(presentation.maximum),
        )
        .unit_name(presentation.unit)
        .on_change(move |value| {
            update_caption(&project, &player_state, key.clone(), |item| {
                set(item, value as u16)
            });
        })
        .build();
    section.add_control_row(presentation.label, &picker);
}

fn add_font_editor(
    section: &InspectorSection,
    context: &InspectorContext,
    key: SelectedItem,
    value: CaptionFont,
) {
    let project = context.project.clone();
    let player_state = context.player_state.clone();
    let dropdown = selector(
        "Font",
        value,
        caption::FONTS
            .iter()
            .map(|choice| (choice.value, choice.label)),
        move |value| {
            update_caption(&project, &player_state, key.clone(), |item| {
                item.font = value
            })
        },
    );
    section.add_wide_control(&dropdown);
}

fn add_direction_editor(
    section: &InspectorSection,
    context: &InspectorContext,
    key: SelectedItem,
    value: CaptionWritingDirection,
) {
    let project = context.project.clone();
    let player_state = context.player_state.clone();
    let dropdown = selector(
        "Writing",
        value,
        caption::WRITING_DIRECTIONS
            .iter()
            .map(|choice| (choice.value, choice.label)),
        move |value| {
            update_caption(&project, &player_state, key.clone(), |item| {
                item.writing_direction = value
            });
        },
    );
    section.add_wide_control(&dropdown);
}

fn add_edge_editor(
    section: &InspectorSection,
    context: &InspectorContext,
    key: SelectedItem,
    value: CaptionEdgeStyle,
) {
    let project = context.project.clone();
    let player_state = context.player_state.clone();
    let dropdown = selector(
        "Edge",
        value,
        caption::EDGE_STYLES
            .iter()
            .map(|choice| (choice.value, choice.label)),
        move |value| {
            update_caption(&project, &player_state, key.clone(), |item| {
                item.edge_style = value
            });
        },
    );
    section.add_wide_control(&dropdown);
}

fn add_h_align_editor(
    section: &InspectorSection,
    context: &InspectorContext,
    key: SelectedItem,
    align: HorizontalAlign,
) {
    add_alignment_editor(
        section,
        context,
        key,
        "H align",
        align,
        caption::HORIZONTAL_ALIGNMENTS,
        |item, align| item.h_align = align,
    );
}

fn add_v_align_editor(
    section: &InspectorSection,
    context: &InspectorContext,
    key: SelectedItem,
    align: VerticalAlign,
) {
    add_alignment_editor(
        section,
        context,
        key,
        "V align",
        align,
        caption::VERTICAL_ALIGNMENTS,
        |item, align| item.v_align = align,
    );
}

fn add_alignment_editor<T: Copy + Eq + 'static>(
    section: &InspectorSection,
    context: &InspectorContext,
    key: SelectedItem,
    label: &str,
    value: T,
    choices: &'static [CaptionChoice<T>],
    set: fn(&mut CaptionItem, T),
) {
    let group = adw::ToggleGroup::builder()
        .active(
            choices
                .iter()
                .position(|choice| choice.value == value)
                .expect("caption alignment must have a declared choice") as u32,
        )
        .halign(gtk::Align::End)
        .homogeneous(true)
        .build();
    for choice in choices {
        group.add(
            adw::Toggle::builder()
                .name(choice.key)
                .icon_name(
                    choice
                        .icon
                        .expect("caption alignment choices must provide icons")
                        .gtk,
                )
                .tooltip(choice.label)
                .build(),
        );
    }
    section.add_control_row(label, &group);

    let project = context.project.clone();
    let player_state = context.player_state.clone();
    group.connect_active_notify(move |group| {
        let Some(choice) = choices.get(group.active() as usize) else {
            return;
        };
        update_caption(&project, &player_state, key.clone(), |item| {
            set(item, choice.value);
        });
    });
}

fn add_color_editor(
    section: &InspectorSection,
    context: &InspectorContext,
    key: SelectedItem,
    label: &str,
    color: Color<u8>,
    set: fn(&mut CaptionItem, Color<u8>),
) {
    let project = context.project.clone();
    let player_state = context.player_state.clone();
    let button = ColorPicker::builder(color)
        .title(tr!("Caption text color").as_ref())
        .hexpand(true)
        .on_change(move |color| {
            update_caption(&project, &player_state, key.clone(), |item| {
                set(item, color);
            });
        })
        .build();
    section.add_control_row(label, &button);
}

fn update_caption(
    project: &Rc<RefCell<Project>>,
    player_state: &SharedPlayerState,
    key: SelectedItem,
    update: impl FnOnce(&mut CaptionItem),
) {
    let mut project = project.borrow_mut();
    let Some(item) = project.caption_item_mut(&key) else {
        return;
    };
    update(item);
    shrimply_project::project::commit_coalesced_edit(&project, "caption");
    drop(project);

    player_state::refresh_project(
        player_state,
        ProjectChange {
            captions: true,
            ..ProjectChange::default()
        },
    );
}

fn update_caption_text_live(
    project: &Rc<RefCell<Project>>,
    player_state: &SharedPlayerState,
    key: SelectedItem,
    text: String,
) -> bool {
    let mut project = project.borrow_mut();
    let Some(item) = project.caption_item_mut(&key) else {
        return false;
    };
    if item.text == text {
        return false;
    }
    item.text = text;
    drop(project);

    player_state::refresh_project(
        player_state,
        ProjectChange {
            captions: true,
            ..ProjectChange::default()
        },
    );
    true
}
