use shrimply_gtk_components::tr;
use shrimply_gtk_components::ui::I18nFileFilterExt;
use std::path::PathBuf;
use std::rc::Rc;

use gtk::prelude::*;
use shrimply_core::timeline_value::{TimelineBase, TimelineBool, TimelineValue};
use shrimply_inspector_core::{
    ControlKind, InspectorControl, InspectorControlAction, InspectorTarget, NumberSpec, VideoCard,
    paint as paint_core,
};
use shrimply_project::project::{
    PaintDrawing, PaintItem, PaintPaletteEntry, PaintStrokeOptions, PaintTaper, PaintTransform,
    Project, ResolvedTransform, Time, VideoItemContent,
};

use crate::{
    InspectedItem as SelectedItem, InspectorContext,
    item::{DefaultInspectorItem, InspectorListItem},
    player_state::{self, ProjectChange},
    timeline_value::{
        boolean::{BoolTarget, bool_control},
        color::{ColorAccess, ColorTarget, color_control},
        scalar::{ScalarAccess, ScalarSpec, ScalarTarget, scalar_control},
        step::{StepTarget, step_control},
    },
    transform,
    ui::control_row,
};

pub(super) fn items(paint: &PaintItem) -> Vec<InspectorListItem> {
    let palette_card = paint_core::PALETTE_CARD;
    let stroke_card = paint_core::STROKE_CARD;
    let stroke_transform_card = paint_core::STROKE_TRANSFORM_CARD;
    vec![
        DefaultInspectorItem::new(
            palette_card.key,
            palette_card.title,
            PaintPalette(paint.palette.clone()),
            palette_controls,
            move |context, _: PaintPalette| {
                reset_shared_card(context, palette_card);
            },
        )
        .boxed(),
        DefaultInspectorItem::new(
            stroke_card.key,
            stroke_card.title,
            paint.stroke.clone(),
            {
                let drawing = paint.drawing.clone();
                move |stroke, context| stroke_controls(stroke, &drawing, context)
            },
            move |context, _: PaintStrokeOptions| {
                reset_shared_card(context, stroke_card);
            },
        )
        .boxed(),
        DefaultInspectorItem::new(
            stroke_transform_card.key,
            stroke_transform_card.title,
            StrokeTransform(paint.stroke_transform.clone()),
            move |value, context| {
                let presentation = shared_card(context, stroke_transform_card);
                transform::paint_stroke_controls(&value.0, &presentation.section.controls, context)
            },
            move |context, _: StrokeTransform| {
                reset_shared_card(context, stroke_transform_card);
            },
        )
        .default_with(|context| {
            StrokeTransform(PaintTransform::fill(context.project.borrow().canvas_size))
        })
        .boxed(),
    ]
}

fn shared_card(context: &InspectorContext, metadata: paint_core::PaintCardMetadata) -> VideoCard {
    shared_cards(context)
        .into_iter()
        .find(|card| card.key == metadata.key)
        .expect("paint card presentation must exist")
}

fn shared_cards(context: &InspectorContext) -> Vec<VideoCard> {
    let Some(key) = context.selected_item.clone() else {
        return Vec::new();
    };
    let runtime = context.inspector_core.snapshot().runtime;
    let project = context.project.borrow();
    let Some(paint) = selected_paint(&project, key) else {
        return Vec::new();
    };
    shrimply_inspector_core::paint::cards(paint, project.canvas_size, runtime)
}

fn reset_shared_card(context: &InspectorContext, metadata: paint_core::PaintCardMetadata) {
    let Some(key) = context.selected_item.clone() else {
        return;
    };
    let Some(reset) = shared_cards(context)
        .into_iter()
        .find(|card| card.key == metadata.key)
        .and_then(|card| card.reset)
    else {
        return;
    };
    if let Err(error) = context
        .inspector_core
        .reset_video(&shrimply_inspector_core::InspectorTarget::Item(key), &reset)
    {
        tracing::error!(%error, "Could not reset GTK paint card");
    }
}

#[derive(Clone)]
struct PaintPalette(Vec<PaintPaletteEntry>);

impl Default for PaintPalette {
    fn default() -> Self {
        Self(PaintItem::default().palette)
    }
}

fn palette_controls(value: &PaintPalette, context: &InspectorContext) -> Vec<gtk::Widget> {
    let section = crate::section::InspectorSection::controls();
    let presentation = shared_card(context, paint_core::PALETTE_CARD);
    for control in presentation.section.controls {
        match control.kind {
            ControlKind::LayeredColor => {
                let entry = value
                    .0
                    .iter()
                    .find(|entry| Some(entry.color.id) == control.timeline_id)
                    .expect("shared paint palette color changed");
                section.add_wide_control(&palette_color_control(entry, &control, context));
            }
            ControlKind::Action => match control.action {
                Some(InspectorControlAction::SelectPaintTexture { color_id }) => {
                    let entry = value
                        .0
                        .iter()
                        .find(|entry| entry.color.id == color_id)
                        .expect("shared paint texture changed");
                    section.add_wide_control(&texture_picker(entry, &control, context));
                }
                Some(InspectorControlAction::AddPaintPaletteColor) => {
                    section.add_wide_control(&add_palette_button(&control, context));
                }
                action => panic!("unsupported shared paint palette action: {action:?}"),
            },
            ControlKind::LayeredNumber => {
                let timeline_id = control
                    .timeline_id
                    .expect("shared paint texture number needs a timeline ID");
                let (value, expected_commit) = value
                    .0
                    .iter()
                    .filter_map(|entry| entry.texture.as_ref())
                    .flat_map(|texture| {
                        [
                            (&texture.repeat_scale, paint_core::TEXTURE_SCALE_COMMIT),
                            (
                                &texture.rotation_degrees,
                                paint_core::TEXTURE_ROTATION_COMMIT,
                            ),
                        ]
                    })
                    .find(|(value, _)| value.id == timeline_id)
                    .expect("shared paint texture number changed");
                add_palette_scalar(&section, value, context, &control, expected_commit);
            }
            kind => panic!("unsupported shared paint palette control: {kind:?}"),
        }
    }
    vec![section.into_widget()]
}

fn palette_color_control(
    entry: &PaintPaletteEntry,
    control: &InspectorControl,
    context: &InspectorContext,
) -> gtk::Widget {
    let color = &entry.color;
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let native = color_control(
        &control.label,
        color,
        context,
        ColorTarget {
            access: ColorAccess::PaintPalette { value_id: color.id },
            scope_id: Some(color.id),
            local_time: crate::video::visual_local_time,
            duration: crate::video::visual_duration,
            refresh: paint_refresh(),
            commit_name: static_control_commit(control, paint_core::PALETTE_COLOR_COMMIT),
        },
    );
    native.set_hexpand(true);
    row.append(&native);
    let Some(InspectorControlAction::RemovePaintPaletteColor { color_id }) = control.action else {
        panic!("shared paint palette color needs a remove action");
    };
    let remove = gtk::Button::builder()
        .icon_name(&control.prefix_icon)
        .tooltip_text(&control.tooltip)
        .css_classes(["flat"])
        .sensitive(control.action_sensitive)
        .build();
    let context = context.clone();
    remove.connect_clicked(move |_| {
        let Some(key) = context.selected_item.clone() else {
            return;
        };
        if let Err(error) = context
            .inspector_core
            .remove_paint_palette_color(&InspectorTarget::Item(key), color_id)
        {
            tracing::error!(%error, "Could not remove GTK paint palette color");
        }
    });
    row.append(&remove);
    row.upcast()
}

fn add_palette_button(control: &InspectorControl, context: &InspectorContext) -> gtk::Widget {
    let add = gtk::Button::builder()
        .icon_name(&control.prefix_icon)
        .label(&control.value)
        .halign(gtk::Align::End)
        .css_classes(["flat"])
        .build();
    let context = context.clone();
    add.connect_clicked(move |_| {
        let Some(key) = context.selected_item.clone() else {
            return;
        };
        if let Err(error) = context
            .inspector_core
            .add_paint_palette_color(&InspectorTarget::Item(key))
        {
            tracing::error!(%error, "Could not add GTK paint palette color");
        }
    });
    add.upcast()
}

fn stroke_controls(
    value: &PaintStrokeOptions,
    drawing: &TimelineValue<PaintDrawing>,
    context: &InspectorContext,
) -> Vec<gtk::Widget> {
    let section = crate::section::InspectorSection::controls();
    let presentation = shared_card(context, paint_core::STROKE_CARD);
    for control in presentation.section.controls {
        match control.kind {
            ControlKind::LayeredDrawing => {
                assert_eq!(control.timeline_id, Some(drawing.id));
                section.add_wide_control(&drawing_control(drawing, &control, context));
            }
            ControlKind::LayeredNumber => {
                let (timeline, get, get_mut, expected_commit) = stroke_scalar(value, &control);
                add_scalar(
                    &section,
                    timeline,
                    context,
                    &control,
                    get,
                    get_mut,
                    expected_commit,
                );
            }
            ControlKind::LayeredBoolean => {
                let (timeline, get, get_mut, expected_commit) = stroke_boolean(value, &control);
                section.add_wide_control(&bool_control(
                    &control.label,
                    timeline,
                    timeline.fallback().get(),
                    context,
                    BoolTarget::ItemValue {
                        get,
                        get_mut,
                        scope: static_control_commit(&control, expected_commit),
                        mutated: bump_revision_for_key,
                    },
                ));
            }
            ControlKind::LayeredSelector => {
                let start = control.timeline_id == Some(value.start.taper.id);
                let timeline = if start {
                    &value.start.taper
                } else {
                    assert_eq!(control.timeline_id, Some(value.end.taper.id));
                    &value.end.taper
                };
                section.add_wide_control(&taper_control(
                    &control.label,
                    timeline,
                    start,
                    context,
                    static_control_commit(
                        &control,
                        if start {
                            paint_core::STROKE_START_TAPER_COMMIT
                        } else {
                            paint_core::STROKE_END_TAPER_COMMIT
                        },
                    ),
                ));
            }
            kind => panic!("unsupported shared paint stroke control: {kind:?}"),
        }
    }
    vec![section.into_widget()]
}

fn stroke_scalar<'a>(
    value: &'a PaintStrokeOptions,
    control: &InspectorControl,
) -> (
    &'a TimelineValue<f32>,
    ScalarGet,
    ScalarGetMut,
    &'static str,
) {
    let id = control
        .timeline_id
        .expect("shared paint stroke number needs a timeline ID");
    for (timeline, get, get_mut, commit) in [
        (
            &value.width,
            stroke_width as ScalarGet,
            stroke_width_mut as ScalarGetMut,
            paint_core::STROKE_WIDTH_COMMIT,
        ),
        (
            &value.thinning,
            stroke_thinning,
            stroke_thinning_mut,
            paint_core::STROKE_THINNING_COMMIT,
        ),
        (
            &value.smoothing,
            stroke_smoothing,
            stroke_smoothing_mut,
            paint_core::STROKE_SMOOTHING_COMMIT,
        ),
        (
            &value.streamline,
            stroke_streamline,
            stroke_streamline_mut,
            paint_core::STROKE_STREAMLINE_COMMIT,
        ),
        (
            &value.simplification_tolerance,
            stroke_simplification,
            stroke_simplification_mut,
            paint_core::STROKE_SIMPLIFICATION_COMMIT,
        ),
        (
            &value.maximum_subdivision_spacing,
            stroke_subdivision,
            stroke_subdivision_mut,
            paint_core::STROKE_SUBDIVISION_COMMIT,
        ),
        (
            &value.start.taper_distance,
            stroke_start_taper_distance,
            stroke_start_taper_distance_mut,
            paint_core::STROKE_START_TAPER_DISTANCE_COMMIT,
        ),
        (
            &value.end.taper_distance,
            stroke_end_taper_distance,
            stroke_end_taper_distance_mut,
            paint_core::STROKE_END_TAPER_DISTANCE_COMMIT,
        ),
    ] {
        if timeline.id == id {
            return (timeline, get, get_mut, commit);
        }
    }
    panic!("shared paint stroke number changed")
}

type BoolGet = for<'a> fn(&'a Project, SelectedItem) -> Option<&'a TimelineValue<TimelineBool>>;
type BoolGetMut =
    for<'a> fn(&'a mut Project, SelectedItem) -> Option<&'a mut TimelineValue<TimelineBool>>;

fn stroke_boolean<'a>(
    value: &'a PaintStrokeOptions,
    control: &InspectorControl,
) -> (
    &'a TimelineValue<TimelineBool>,
    BoolGet,
    BoolGetMut,
    &'static str,
) {
    match control.timeline_id {
        Some(id) if id == value.start.cap.id => (
            &value.start.cap,
            stroke_start_cap,
            stroke_start_cap_mut,
            paint_core::STROKE_START_CAP_COMMIT,
        ),
        Some(id) if id == value.end.cap.id => (
            &value.end.cap,
            stroke_end_cap,
            stroke_end_cap_mut,
            paint_core::STROKE_END_CAP_COMMIT,
        ),
        _ => panic!("shared paint stroke boolean changed"),
    }
}

fn static_control_commit(control: &InspectorControl, expected: &'static str) -> &'static str {
    assert_eq!(control.commit_name, expected);
    assert_eq!(control.keyframe_commit_name, expected);
    assert_eq!(control.expression_commit_name, expected);
    expected
}

fn drawing_control(
    value: &TimelineValue<PaintDrawing>,
    control: &InspectorControl,
    context: &InspectorContext,
) -> gtk::Widget {
    assert_eq!(control.commit_name, paint_core::DRAWING_LIVE_COMMIT);
    assert_eq!(
        control.keyframe_commit_name,
        paint_core::DRAWING_KEYFRAME_COMMIT
    );
    assert_eq!(
        control.expression_commit_name,
        paint_core::DRAWING_EXPRESSION_COMMIT
    );
    let summary = gtk::Label::builder()
        .label(&control.value)
        .halign(gtk::Align::Start)
        .xalign(0.0)
        .css_classes(["dim-label"])
        .build();
    let mut sections = crate::timeline_value::LayeredSections::default();
    if matches!(&value.base, TimelineBase::Keyframes(_)) {
        sections.set_keyframe(drawing_keyframe_graph(value, &control.path, context));
    }
    if value
        .expression
        .as_ref()
        .is_some_and(|expression| expression.enabled)
    {
        sections.push_expression(drawing_expression_editor(value, context));
    }
    let keyframe_context = context.detached();
    let expression_context = context.detached();
    let timeline_id = value.id;
    crate::timeline_value::layered_wide_control(
        &control.label,
        value,
        summary.upcast(),
        sections,
        move |enabled| {
            let Some(key) = keyframe_context.selected_item.clone() else {
                return;
            };
            if let Err(error) = keyframe_context
                .inspector_core
                .set_paint_drawing_keyframes_enabled(
                    &shrimply_inspector_core::InspectorTarget::Item(key),
                    timeline_id,
                    enabled,
                )
            {
                tracing::error!(%error, "Could not toggle GTK paint drawing keyframes");
            }
            (keyframe_context.refresh)();
        },
        move |enabled| {
            let Some(key) = expression_context.selected_item.clone() else {
                return;
            };
            if let Err(error) = expression_context
                .inspector_core
                .set_paint_drawing_expression_enabled(
                    &shrimply_inspector_core::InspectorTarget::Item(key),
                    timeline_id,
                    enabled,
                )
            {
                tracing::error!(%error, "Could not toggle GTK paint drawing expression");
            }
            (expression_context.refresh)();
        },
    )
}

fn drawing_keyframe_graph(
    value: &TimelineValue<PaintDrawing>,
    path: &str,
    context: &InspectorContext,
) -> gtk::Widget {
    let Some(key) = context.selected_item.clone() else {
        return gtk::Box::new(gtk::Orientation::Horizontal, 0).upcast();
    };
    let graph = gtk_drawing_graph(shrimply_inspector_core::paint::drawing_graph(
        value,
        context.inspector_core.snapshot().runtime,
    ));
    let duration =
        crate::video::visual_duration(&context.project.borrow(), key.clone()).unwrap_or(Time::ZERO);
    let built = crate::keyframe_editor::build(
        context,
        graph,
        (Time::ZERO, duration),
        format!("{path}:{}", value.id),
        drawing_graph_actions(context, key.clone()),
    );
    let project = context.project.clone();
    let controller = context.inspector_core.clone();
    crate::keyframe_editor::connect_graph_refresh(
        context,
        "inspector paint drawing keyframe graph refresh",
        &built,
        move || {
            let runtime = controller.snapshot().runtime;
            selected_paint(&project.borrow(), key.clone()).map(|paint| {
                gtk_drawing_graph(shrimply_inspector_core::paint::drawing_graph(
                    &paint.drawing,
                    runtime,
                ))
            })
        },
    );
    built.widget
}

fn gtk_drawing_graph(
    graph: Option<shrimply_inspector_core::ScalarGraph>,
) -> crate::keyframe_editor::KeyframeGraph {
    let Some(graph) = graph else {
        return crate::keyframe_editor::KeyframeGraph::Speed {
            segments: Vec::new(),
            keys: Vec::new(),
            static_value: 0.0,
        };
    };
    crate::keyframe_editor::KeyframeGraph::Speed {
        segments: graph
            .segments
            .into_iter()
            .map(|segment| crate::keyframe_editor::SpeedSegment {
                owner_id: segment.owner_id,
                start: segment.start,
                end: segment.end,
                value: segment.start_value,
                interpolation: shrimply_core::timeline_value::Interpolation::KEYFRAME
                    [segment.interpolation],
            })
            .collect(),
        keys: graph.points.into_iter().map(|point| point.time).collect(),
        static_value: 0.0,
    }
}

fn drawing_graph_actions(
    context: &InspectorContext,
    key: SelectedItem,
) -> crate::keyframe_editor::KeyframeEditorActions {
    let timeline_id = selected_paint(&context.project.borrow(), key.clone())
        .expect("paint drawing graph requires a paint item")
        .drawing
        .id;
    let target = shrimply_inspector_core::InspectorTarget::Item(key);
    let controller = context.inspector_core.clone();
    let player = context.player_state.clone();
    crate::keyframe_editor::KeyframeEditorActions {
        add_at_time: {
            let controller = controller.clone();
            let target = target.clone();
            Rc::new(move |time| {
                if let Err(error) =
                    controller.add_paint_drawing_keyframe(&target, timeline_id, time)
                {
                    tracing::error!(%error, "Could not add GTK paint drawing keyframe");
                }
            })
        },
        delete_at_time: {
            let controller = controller.clone();
            let target = target.clone();
            Rc::new(move |time| {
                if let Err(error) =
                    controller.delete_paint_drawing_keyframe(&target, timeline_id, time)
                {
                    tracing::error!(%error, "Could not delete GTK paint drawing keyframe");
                }
            })
        },
        update_point: {
            let controller = controller.clone();
            let target = target.clone();
            Rc::new(move |old, time, _| {
                if let Err(error) =
                    controller.move_paint_drawing_keyframes(&target, timeline_id, &[(old, time)])
                {
                    tracing::error!(%error, "Could not move GTK paint drawing keyframe");
                }
            })
        },
        clipboard: crate::keyframe_editor::KeyframeClipboardActions::Managed {
            copy: {
                let controller = controller.clone();
                let target = target.clone();
                Rc::new(move |times| {
                    controller
                        .copy_paint_drawing_keyframes(&target, timeline_id, times)
                        .ok()
                        .filter(|count| *count > 0)
                })
            },
            paste: {
                let controller = controller.clone();
                let target = target.clone();
                Rc::new(move |time| {
                    controller
                        .paste_paint_drawing_keyframes(&target, timeline_id, time)
                        .ok()
                        .filter(|count| *count > 0)
                })
            },
        },
        set_interpolation: Some({
            let controller = controller.clone();
            let target = target.clone();
            Rc::new(move |id, interpolation| {
                let Some(index) = shrimply_core::timeline_value::Interpolation::KEYFRAME
                    .iter()
                    .position(|candidate| *candidate == interpolation)
                else {
                    return;
                };
                if let Err(error) =
                    controller.set_paint_drawing_interpolation(&target, timeline_id, id, index)
                {
                    tracing::error!(%error, "Could not set GTK paint drawing interpolation");
                }
            })
        }),
        text_interpolation: None,
        toggle_playback: Rc::new(move || player_state::toggle_playing(&player)),
    }
}

fn drawing_expression_editor(
    value: &TimelineValue<PaintDrawing>,
    context: &InspectorContext,
) -> gtk::Widget {
    let source = value.expression_source().map(str::to_string);
    let Some(key) = context.selected_item.clone() else {
        return gtk::Box::new(gtk::Orientation::Horizontal, 0).upcast();
    };
    let controller = context.inspector_core.clone();
    let timeline_id = value.id;
    crate::rhai_editor::editor(
        source,
        crate::rhai_editor::ExpressionValue::Drawing,
        move |source| {
            if let Err(error) = controller.set_paint_drawing_expression_source(
                &shrimply_inspector_core::InspectorTarget::Item(key.clone()),
                timeline_id,
                &source,
            ) {
                tracing::error!(%error, "Could not edit GTK paint drawing expression");
            }
        },
    )
}

fn taper_control(
    label: &str,
    value: &TimelineValue<PaintTaper>,
    start: bool,
    context: &InspectorContext,
    commit_name: &'static str,
) -> gtk::Widget {
    let (get, get_mut) = if start {
        (
            stroke_start_taper as TaperGet,
            stroke_start_taper_mut as TaperGetMut,
        )
    } else {
        (
            stroke_end_taper as TaperGet,
            stroke_end_taper_mut as TaperGetMut,
        )
    };
    step_control(
        label,
        value,
        context,
        StepTarget::new(get, get_mut, commit_name, paint_refresh())
            .mark_mutated(bump_revision_for_key)
            .refresh_inspector_on_value_change(),
    )
}

fn texture_picker(
    entry: &PaintPaletteEntry,
    control: &InspectorControl,
    context: &InspectorContext,
) -> gtk::Widget {
    let color_id = entry.color.id;
    let Some(InspectorControlAction::ClearPaintTexture {
        color_id: clear_color_id,
    }) = control.secondary_action
    else {
        panic!("shared paint texture needs a clear action");
    };
    assert_eq!(clear_color_id, color_id);
    let [filename, choose_label] = control.components.as_slice() else {
        panic!("shared paint texture needs filename and chooser labels");
    };
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    row.set_hexpand(true);
    let filename_label = gtk::Label::builder()
        .label(filename)
        .hexpand(true)
        .halign(gtk::Align::Start)
        .xalign(0.0)
        .ellipsize(gtk::pango::EllipsizeMode::Middle)
        .css_classes(["dim-label"])
        .build();
    filename_label
        .set_tooltip_text((!control.tooltip.is_empty()).then_some(control.tooltip.as_str()));
    row.append(&filename_label);

    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    actions.add_css_class("linked");
    let choose = gtk::Button::with_label(choose_label);
    let clear = gtk::Button::builder()
        .icon_name(&control.action_icon)
        .tooltip_text(&control.action_tooltip)
        .sensitive(!control.action_icon.is_empty())
        .build();
    actions.append(&choose);
    actions.append(&clear);
    row.append(&actions);

    let choose_context = context.detached();
    choose.connect_clicked(move |_| {
        let label = "Select paint texture";
        let filter = gtk::FileFilter::new();
        filter.set_name_i18n("Images");
        filter.add_mime_type("image/*");
        let filters = gtk::gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);
        let dialog = gtk::FileDialog::builder()
            .title(tr!(label).as_ref())
            .filters(&filters)
            .build();
        let context = choose_context.clone();
        shrimply_gtk_components::file_picker::open(
            label,
            &dialog,
            None::<&gtk::Window>,
            move |result| {
                let Ok(file) = result else { return };
                let Some(path) = file.path() else { return };
                update_texture_path(&context, color_id, path);
            },
        );
    });
    let context = context.detached();
    clear.connect_clicked(move |_| {
        let Some(key) = context.selected_item.clone() else {
            return;
        };
        if let Err(error) = context.inspector_core.clear_paint_texture(
            &shrimply_inspector_core::InspectorTarget::Item(key),
            clear_color_id,
        ) {
            tracing::error!(%error, "Could not clear GTK paint texture");
        }
    });
    control_row(&control.label, &row)
}

fn update_texture_path(context: &InspectorContext, color_id: uuid::Uuid, path: PathBuf) {
    let Some(key) = context.selected_item.clone() else {
        return;
    };
    if let Err(error) = context.inspector_core.set_paint_texture(
        &shrimply_inspector_core::InspectorTarget::Item(key),
        color_id,
        &path,
    ) {
        tracing::error!(%error, "Could not set GTK paint texture");
    }
}

type ScalarGet = for<'a> fn(&'a Project, SelectedItem) -> Option<&'a TimelineValue<f32>>;
type ScalarGetMut = for<'a> fn(&'a mut Project, SelectedItem) -> Option<&'a mut TimelineValue<f32>>;
type TaperGet = for<'a> fn(&'a Project, SelectedItem) -> Option<&'a TimelineValue<PaintTaper>>;
type TaperGetMut =
    for<'a> fn(&'a mut Project, SelectedItem) -> Option<&'a mut TimelineValue<PaintTaper>>;

fn add_scalar(
    section: &crate::section::InspectorSection,
    value: &TimelineValue<f32>,
    context: &InspectorContext,
    control: &InspectorControl,
    get: ScalarGet,
    get_mut: ScalarGetMut,
    expected_commit: &'static str,
) {
    section.add_wide_control(&scalar_control(
        &control.label,
        value,
        context,
        ScalarTarget {
            access: ScalarAccess::ItemWithMutation {
                get,
                get_mut,
                mutated: bump_revision_for_key,
            },
            scope_id: Some(value.id),
            local_time: crate::video::visual_local_time,
            duration: crate::video::visual_duration,
            refresh: paint_refresh(),
            commit_name: static_control_commit(control, expected_commit),
        },
        scalar_spec(control),
    ));
}

fn add_palette_scalar(
    section: &crate::section::InspectorSection,
    value: &TimelineValue<f32>,
    context: &InspectorContext,
    control: &InspectorControl,
    expected_commit: &'static str,
) {
    section.add_wide_control(&scalar_control(
        &control.label,
        value,
        context,
        ScalarTarget {
            access: ScalarAccess::PaintPalette { value_id: value.id },
            scope_id: Some(value.id),
            local_time: crate::video::visual_local_time,
            duration: crate::video::visual_duration,
            refresh: paint_refresh(),
            commit_name: static_control_commit(control, expected_commit),
        },
        scalar_spec(control),
    ));
}

fn scalar_spec(control: &InspectorControl) -> ScalarSpec {
    let defaults = NumberSpec::default();
    ScalarSpec {
        drag_step: control.number.drag_step,
        digits: usize::try_from(control.number.digits)
            .expect("paint scalar digits must be nonnegative"),
        integer: control.integer,
        width_chars: control.width_characters,
        minimum: (control.number.minimum != defaults.minimum).then_some(control.number.minimum),
        maximum: (control.number.maximum != defaults.maximum).then_some(control.number.maximum),
        unit_name: (!control.number.unit.is_empty()).then_some(control.number.unit),
        rotating_icon: control.prefix_icon_rotates.then(|| {
            (
                match control.prefix_icon.as_str() {
                    "arrow3-up-symbolic" => "arrow3-up-symbolic",
                    icon => panic!("unsupported shared paint rotating icon: {icon}"),
                },
                control.prefix_icon_rotation_offset_degrees,
            )
        }),
        display: f64::from,
        store: |value| value as f32,
        clamp: crate::timeline_value::scalar::ScalarClamp::Function(
            if control.number.minimum == 0.0 && control.number.maximum == 1.0 {
                |value| value.clamp(0.0, 1.0)
            } else if control.number.minimum == 0.0 {
                |value| value.max(0.0)
            } else if control.number.minimum == 0.01 {
                |value| value.max(0.01)
            } else {
                |value| value
            },
        ),
    }
}

fn paint_refresh() -> ProjectChange {
    ProjectChange {
        video: true,
        inspector: true,
        ..ProjectChange::default()
    }
}

pub(super) fn selected_paint(project: &Project, key: SelectedItem) -> Option<&PaintItem> {
    let item = project.video_item(&key)?;
    let VideoItemContent::Paint(paint) = &item.content else {
        return None;
    };
    Some(paint)
}

pub(super) fn selected_paint_mut(
    project: &mut Project,
    key: SelectedItem,
) -> Option<&mut PaintItem> {
    let item = project.video_item_mut(&key)?;
    let VideoItemContent::Paint(paint) = &mut item.content else {
        return None;
    };
    Some(paint)
}

pub(super) fn bump_revision_for_key(project: &mut Project, key: SelectedItem) {
    if let Some(paint) = selected_paint_mut(project, key) {
        bump_revision(paint);
    }
}

fn bump_revision(paint: &mut PaintItem) {
    paint.revision = paint
        .revision
        .checked_add(1)
        .expect("paint revision overflow");
}

macro_rules! timeline_accessors {
    ($get:ident, $get_mut:ident, $value:ty, $($field:ident).+) => {
        fn $get(project: &Project, key: SelectedItem) -> Option<&TimelineValue<$value>> {
            Some(&selected_paint(project, key)?$(.$field)+)
        }

        fn $get_mut(
            project: &mut Project,
            key: SelectedItem,
        ) -> Option<&mut TimelineValue<$value>> {
            Some(&mut selected_paint_mut(project, key)?$(.$field)+)
        }
    };
}

timeline_accessors!(stroke_width, stroke_width_mut, f32, stroke.width);
timeline_accessors!(stroke_thinning, stroke_thinning_mut, f32, stroke.thinning);
timeline_accessors!(
    stroke_smoothing,
    stroke_smoothing_mut,
    f32,
    stroke.smoothing
);
timeline_accessors!(
    stroke_streamline,
    stroke_streamline_mut,
    f32,
    stroke.streamline
);
timeline_accessors!(
    stroke_simplification,
    stroke_simplification_mut,
    f32,
    stroke.simplification_tolerance
);
timeline_accessors!(
    stroke_subdivision,
    stroke_subdivision_mut,
    f32,
    stroke.maximum_subdivision_spacing
);
timeline_accessors!(
    stroke_start_cap,
    stroke_start_cap_mut,
    TimelineBool,
    stroke.start.cap
);
timeline_accessors!(
    stroke_end_cap,
    stroke_end_cap_mut,
    TimelineBool,
    stroke.end.cap
);
timeline_accessors!(
    stroke_start_taper,
    stroke_start_taper_mut,
    PaintTaper,
    stroke.start.taper
);
timeline_accessors!(
    stroke_end_taper,
    stroke_end_taper_mut,
    PaintTaper,
    stroke.end.taper
);
timeline_accessors!(
    stroke_start_taper_distance,
    stroke_start_taper_distance_mut,
    f32,
    stroke.start.taper_distance
);
timeline_accessors!(
    stroke_end_taper_distance,
    stroke_end_taper_distance_mut,
    f32,
    stroke.end.taper_distance
);
struct StrokeTransform(PaintTransform);

impl Default for StrokeTransform {
    fn default() -> Self {
        Self(PaintTransform::from_resolved(ResolvedTransform::IDENTITY))
    }
}
