use std::rc::Rc;

use gtk::prelude::*;
use shrimply_core::timeline_value::{TimelineStep, TimelineValue};
use shrimply_inspector_core::{
    AudioModifierKeyframeMove, ControlKind, InspectorControl, InspectorTarget, NumberSpec,
    ScalarGraph,
};
use shrimply_interpolation::Interpolation;
use shrimply_project::project::{Background, BackgroundGenerator, Project, Time, VideoItemContent};

use crate::player_state::{self, ProjectChange};
use crate::{
    InspectedItem, InspectorContext,
    item::{DefaultInspectorItem, InspectorListItem},
    keyframe_editor::{
        self, KeyframeClipboardActions, KeyframeEditorActions, KeyframeGraph, KeyframePoint,
        RawSegment,
    },
    section::InspectorSection,
    timeline_value::{
        ExpressionOutput,
        boolean::{BoolTarget, bool_control},
        color::{ColorAccess, ColorTarget, color_control},
        expression_section,
        scalar::{ScalarAccess, ScalarSpec, ScalarTarget, scalar_control},
        vector::vec2::{VecAccess, VecSpec, VecTarget, vec_control_with_lock},
    },
    ui::{NumberPicker, enum_dropdown},
};

pub(super) fn item(background: &Background) -> InspectorListItem {
    DefaultInspectorItem::new(
        "background",
        "Background",
        background.clone(),
        controls,
        |context, background: Background| {
            let Some(key) = context.selected_item.clone() else {
                return;
            };
            let reset = shrimply_inspector_core::background::card(
                &background,
                context.inspector_core.snapshot().runtime,
            )
            .reset
            .expect("background card must have reset behavior");
            if let Err(error) = context
                .inspector_core
                .reset_video(&InspectorTarget::Item(key), &reset)
            {
                tracing::error!(%error, "Could not reset GTK background generator");
            }
        },
    )
    .boxed()
}

fn controls(background: &Background, context: &InspectorContext) -> Vec<gtk::Widget> {
    let section = InspectorSection::controls();
    let presentation = shrimply_inspector_core::background::card(
        background,
        context.inspector_core.snapshot().runtime,
    );
    for control in presentation.section.controls {
        let standalone_label =
            (control.kind == ControlKind::Selector).then(|| control.label.clone());
        let widget = match control.kind {
            ControlKind::Selector => kind_control(background, control, context),
            ControlKind::LayeredSelector => step_control(background, &control, context),
            ControlKind::LayeredNumber if control.integer => {
                let value = background
                    .generator
                    .integer(control.timeline_id.expect("background integer needs an ID"))
                    .expect("shared background integer changed");
                integer_control(value, &control, context)
            }
            ControlKind::LayeredNumber => number_control(background, &control, context),
            ControlKind::LayeredVector2 => vector_control(background, &control, context),
            ControlKind::LayeredBoolean => boolean_control(background, &control, context),
            ControlKind::LayeredColor => color_row(background, &control, context),
            kind => panic!("unsupported shared background control: {kind:?}"),
        };
        if let Some(label) = standalone_label {
            section.add_control_row(&label, &widget);
        } else {
            section.add_wide_control(&widget);
        }
    }
    vec![section.into_widget()]
}

fn kind_control(
    background: &Background,
    control: InspectorControl,
    context: &InspectorContext,
) -> gtk::Widget {
    assert_eq!(control.path, "/content/generator/kind");
    assert_eq!(control.commit_name, "change-background-kind");
    assert!(control.commit_immediately);
    assert_eq!(
        control.value,
        shrimply_inspector_core::background::kind_key(background.generator.kind())
    );
    let target = InspectorTarget::Item(
        context
            .selected_item
            .clone()
            .expect("background inspector must have a selected item"),
    );
    let controller = context.inspector_core.clone();
    enum_dropdown(background.generator.kind(), move |kind| {
        if let Err(error) = controller.set_video_field(
            &target,
            "/content/generator/kind",
            shrimply_inspector_core::background::kind_key(kind),
            "change-background-kind",
            true,
        ) {
            tracing::error!(%error, "Could not change GTK background generator kind");
        }
    })
    .upcast()
}

fn step_control(
    background: &Background,
    control: &InspectorControl,
    context: &InspectorContext,
) -> gtk::Widget {
    let generator = &background.generator;
    match (control.path.as_str(), generator) {
        ("/content/generator/mode", BackgroundGenerator::ColorGradient(v)) => step_row(
            control,
            &v.mode,
            context,
            |g| match g {
                BackgroundGenerator::ColorGradient(v) => Some(&v.mode),
                _ => None,
            },
            |g| match g {
                BackgroundGenerator::ColorGradient(v) => Some(&mut v.mode),
                _ => None,
            },
        ),
        ("/content/generator/curve", BackgroundGenerator::ColorGradient(v)) => step_row(
            control,
            &v.curve,
            context,
            |g| match g {
                BackgroundGenerator::ColorGradient(v) => Some(&v.curve),
                _ => None,
            },
            |g| match g {
                BackgroundGenerator::ColorGradient(v) => Some(&mut v.curve),
                _ => None,
            },
        ),
        ("/content/generator/line_style", BackgroundGenerator::Grid(v)) => step_row(
            control,
            &v.line_style,
            context,
            |g| match g {
                BackgroundGenerator::Grid(v) => Some(&v.line_style),
                _ => None,
            },
            |g| match g {
                BackgroundGenerator::Grid(v) => Some(&mut v.line_style),
                _ => None,
            },
        ),
        ("/content/generator/distribution", BackgroundGenerator::WhiteNoise(v)) => step_row(
            control,
            &v.distribution,
            context,
            |g| match g {
                BackgroundGenerator::WhiteNoise(v) => Some(&v.distribution),
                _ => None,
            },
            |g| match g {
                BackgroundGenerator::WhiteNoise(v) => Some(&mut v.distribution),
                _ => None,
            },
        ),
        ("/content/generator/color_mode", BackgroundGenerator::WhiteNoise(v)) => step_row(
            control,
            &v.color_mode,
            context,
            |g| match g {
                BackgroundGenerator::WhiteNoise(v) => Some(&v.color_mode),
                _ => None,
            },
            |g| match g {
                BackgroundGenerator::WhiteNoise(v) => Some(&mut v.color_mode),
                _ => None,
            },
        ),
        ("/content/generator/mode", BackgroundGenerator::PerlinNoise(v)) => step_row(
            control,
            &v.mode,
            context,
            |g| match g {
                BackgroundGenerator::PerlinNoise(v) => Some(&v.mode),
                _ => None,
            },
            |g| match g {
                BackgroundGenerator::PerlinNoise(v) => Some(&mut v.mode),
                _ => None,
            },
        ),
        ("/content/generator/fill", BackgroundGenerator::Rainbow(v)) => step_row(
            control,
            &v.fill,
            context,
            |g| match g {
                BackgroundGenerator::Rainbow(v) => Some(&v.fill),
                _ => None,
            },
            |g| match g {
                BackgroundGenerator::Rainbow(v) => Some(&mut v.fill),
                _ => None,
            },
        ),
        ("/content/generator/bands", BackgroundGenerator::Rainbow(v)) => step_row(
            control,
            &v.bands,
            context,
            |g| match g {
                BackgroundGenerator::Rainbow(v) => Some(&v.bands),
                _ => None,
            },
            |g| match g {
                BackgroundGenerator::Rainbow(v) => Some(&mut v.bands),
                _ => None,
            },
        ),
        ("/content/generator/fill", BackgroundGenerator::Voronoi(v)) => step_row(
            control,
            &v.fill,
            context,
            |g| match g {
                BackgroundGenerator::Voronoi(v) => Some(&v.fill),
                _ => None,
            },
            |g| match g {
                BackgroundGenerator::Voronoi(v) => Some(&mut v.fill),
                _ => None,
            },
        ),
        ("/content/generator/metric", BackgroundGenerator::Voronoi(v)) => step_row(
            control,
            &v.metric,
            context,
            |g| match g {
                BackgroundGenerator::Voronoi(v) => Some(&v.metric),
                _ => None,
            },
            |g| match g {
                BackgroundGenerator::Voronoi(v) => Some(&mut v.metric),
                _ => None,
            },
        ),
        _ => panic!("shared background step no longer matches the generator"),
    }
}

fn step_row<T: TimelineStep>(
    control: &InspectorControl,
    value: &TimelineValue<T>,
    context: &InspectorContext,
    get: fn(&BackgroundGenerator) -> Option<&TimelineValue<T>>,
    get_mut: fn(&mut BackgroundGenerator) -> Option<&mut TimelineValue<T>>,
) -> gtk::Widget {
    assert_eq!(control.timeline_id, Some(value.id));
    assert_eq!(control.commit_name, "edit-background-enum");
    let timeline_id = value.id;
    crate::timeline_value::step::step_control(
        &control.label,
        value,
        context,
        crate::timeline_value::step::StepTarget::new(
            move |project, key| {
                get(generator(project, &key)?).filter(|value| value.id == timeline_id)
            },
            move |project, key| {
                get_mut(generator_mut(project, &key)?).filter(|value| value.id == timeline_id)
            },
            "edit-background-enum",
            background_refresh(),
        ),
    )
}

fn number_control(
    background: &Background,
    control: &InspectorControl,
    context: &InspectorContext,
) -> gtk::Widget {
    let value = background
        .generator
        .number(control.timeline_id.expect("background number needs an ID"))
        .expect("shared background number changed");
    assert_eq!(control.commit_name, "edit-background-scalar");
    let defaults = NumberSpec::default();
    scalar_control(
        &control.label,
        value,
        context,
        scalar_target(value.id),
        ScalarSpec {
            drag_step: control.number.drag_step,
            digits: usize::try_from(control.number.digits)
                .expect("background number digits must be non-negative"),
            integer: false,
            width_chars: control.width_characters,
            minimum: (control.number.minimum != defaults.minimum).then_some(control.number.minimum),
            maximum: (control.number.maximum != defaults.maximum).then_some(control.number.maximum),
            unit_name: (!control.number.unit.is_empty()).then_some(control.number.unit),
            rotating_icon: None,
            display: f64::from,
            store: |value| value as f32,
            clamp: crate::timeline_value::scalar::ScalarClamp::Function(|value| value),
        },
    )
}

fn vector_control(
    background: &Background,
    control: &InspectorControl,
    context: &InspectorContext,
) -> gtk::Widget {
    let value = background
        .generator
        .number2(control.timeline_id.expect("background vector needs an ID"))
        .expect("shared background vector changed");
    assert_eq!(control.commit_name, "edit-background-vector");
    vec_control_with_lock(
        &control.label,
        value,
        context,
        vec_target(value.id),
        VecSpec {
            first_prefix: "X",
            second_prefix: "Y",
            drag_step: control.number.drag_step,
            digits: usize::try_from(control.number.digits)
                .expect("background vector digits must be non-negative"),
            width_chars: control.width_characters,
            minimum: Some(control.number.minimum),
            maximum: Some(control.number.maximum),
            unit_name: control.number.unit,
        },
        control.lock,
    )
}

fn boolean_control(
    background: &Background,
    control: &InspectorControl,
    context: &InspectorContext,
) -> gtk::Widget {
    let value = background
        .generator
        .boolean(control.timeline_id.expect("background boolean needs an ID"))
        .expect("shared background boolean changed");
    bool_control(
        &control.label,
        value,
        value.fallback().get(),
        context,
        BoolTarget::Background { value_id: value.id },
    )
}

fn color_row(
    background: &Background,
    control: &InspectorControl,
    context: &InspectorContext,
) -> gtk::Widget {
    let value = background
        .generator
        .color(control.timeline_id.expect("background color needs an ID"))
        .expect("shared background color changed");
    assert_eq!(control.commit_name, "edit-background-color");
    color_control(
        &control.label,
        value,
        context,
        ColorTarget {
            access: ColorAccess::Background { value_id: value.id },
            scope_id: None,
            local_time: video_local_time_for_key,
            duration: video_duration_for_key,
            refresh: background_refresh(),
            commit_name: "edit-background-color",
        },
    )
}

pub(crate) fn integer_control(
    value: &TimelineValue<u32>,
    control: &InspectorControl,
    context: &InspectorContext,
) -> gtk::Widget {
    assert!(control.integer);
    let timeline_id = value.id;
    let path = control.path.clone();
    let target = InspectorTarget::Item(
        context
            .selected_item
            .clone()
            .expect("background inspector must have a selected item"),
    );
    let position = player_state::snapshot(&context.player_state).position;
    let current = context
        .selected_item
        .clone()
        .and_then(|key| crate::video::visual_local_time(&context.project.borrow(), key, position))
        .map_or_else(|| value.fallback(), |time| value.value_at(time));
    let edit_controller = context.inspector_core.clone();
    let edit_target = target.clone();
    let edit_path = path.clone();
    let commit_controller = context.inspector_core.clone();
    let commit_target = target.clone();
    let commit_path = path.clone();
    let parts = NumberPicker::integer_builder(current)
        .minimum(control.number.minimum)
        .maximum(control.number.maximum)
        .on_change_integer(move |value: u32| {
            if let Err(error) = edit_controller.set_background_integer_value(
                &edit_target,
                &edit_path,
                timeline_id,
                value,
            ) {
                tracing::error!(%error, "Could not update GTK background integer");
            }
        })
        .on_commit(move |_| {
            if let Err(error) = commit_controller.commit_background_integer_value(
                &commit_target,
                &commit_path,
                timeline_id,
            ) {
                tracing::error!(%error, "Could not commit GTK background integer");
            }
        })
        .build_with_handle();
    let display_controller = context.inspector_core.clone();
    let display_target = target.clone();
    let display_path = path.clone();
    let display = parts.handle.downgrade();
    let alive = Rc::downgrade(&context.listener_scope);
    player_state::connect_while_alive_named(
        &context.player_state,
        "inspector background integer display",
        move || alive.upgrade().is_some(),
        move |event| {
            if !matches!(
                event,
                player_state::PlayerEvent::State(_) | player_state::PlayerEvent::Project(_)
            ) {
                return;
            }
            let Some(display) = display.upgrade() else {
                return;
            };
            let Ok(value) = display_controller.background_integer_value(
                &display_target,
                &display_path,
                timeline_id,
            ) else {
                return;
            };
            display.set_f64(f64::from(value));
        },
    );
    let mut sections = crate::timeline_value::LayeredSections::default();
    if let Some(graph) = &control.scalar_graph {
        let graph_controller = context.inspector_core.clone();
        let graph_target = target.clone();
        let graph_path = path.clone();
        let static_value = f64::from(current);
        let built = keyframe_editor::build(
            context,
            integer_keyframe_graph(graph, static_value),
            graph.range,
            format!("background-integer:{timeline_id}"),
            integer_keyframe_actions(context, target.clone(), path.clone(), timeline_id),
        );
        keyframe_editor::connect_graph_refresh(
            context,
            "inspector background integer keyframe graph refresh",
            &built,
            move || {
                let graph = graph_controller
                    .background_integer_graph(&graph_target, &graph_path, timeline_id)
                    .ok()??;
                let current = graph_controller
                    .background_integer_value(&graph_target, &graph_path, timeline_id)
                    .ok()?;
                Some(integer_keyframe_graph(&graph, f64::from(current)))
            },
        );
        sections.set_keyframe(built.widget);
    }
    if value
        .expression
        .as_ref()
        .is_some_and(|expression| expression.enabled)
    {
        let editor_controller = context.inspector_core.clone();
        let editor_target = target.clone();
        let editor_path = path.clone();
        let output_controller = context.inspector_core.clone();
        let output_target = target.clone();
        let output_path = path.clone();
        let source = value.expression_source().map(str::to_string);
        sections.push_expression(expression_section(
            context,
            "inspector background integer expression output",
            move |refresh| {
                crate::rhai_editor::editor(
                    source,
                    crate::rhai_editor::ExpressionValue::Scalar,
                    move |source| {
                        if let Err(error) =
                            editor_controller.set_background_integer_expression_source(
                                &editor_target,
                                &editor_path,
                                timeline_id,
                                source,
                            )
                        {
                            tracing::error!(%error, "Could not update GTK background integer expression");
                        }
                        refresh();
                    },
                )
            },
            move |_, _, _, _| {
                let output = output_controller
                    .background_integer_expression_output(
                        &output_target,
                        &output_path,
                        timeline_id,
                    )
                    .ok()?;
                Some(ExpressionOutput {
                    value: output.value.to_string(),
                    error: output.error,
                })
            },
        ));
    }
    let keyframe_controller = context.inspector_core.clone();
    let keyframe_target = target.clone();
    let keyframe_path = path.clone();
    let expression_controller = context.inspector_core.clone();
    crate::timeline_value::layered_control(
        &control.label,
        value,
        parts.widget,
        sections,
        move |enabled| {
            if let Err(error) = keyframe_controller.set_background_integer_keyframes_enabled(
                &keyframe_target,
                &keyframe_path,
                timeline_id,
                enabled,
            ) {
                tracing::error!(%error, "Could not toggle GTK background integer keyframes");
            }
        },
        move |enabled| {
            if let Err(error) = expression_controller.set_background_integer_expression_enabled(
                &target,
                &path,
                timeline_id,
                enabled,
            ) {
                tracing::error!(%error, "Could not toggle GTK background integer expression");
            }
        },
    )
}

fn integer_keyframe_graph(graph: &ScalarGraph, static_value: f64) -> KeyframeGraph {
    KeyframeGraph::RawValue {
        points: graph
            .points
            .iter()
            .map(|point| KeyframePoint {
                time: point.time,
                value: point.value,
            })
            .collect(),
        segments: graph
            .segments
            .iter()
            .map(|segment| RawSegment {
                owner_id: segment.owner_id,
                start: segment.start,
                end: segment.end,
                start_value: segment.start_value,
                end_value: segment.end_value,
                interpolation: Interpolation::KEYFRAME[segment.interpolation],
            })
            .collect(),
        static_value,
    }
}

fn integer_keyframe_actions(
    context: &InspectorContext,
    target: InspectorTarget,
    path: String,
    timeline_id: uuid::Uuid,
) -> KeyframeEditorActions {
    let controller = context.inspector_core.clone();
    let add_controller = controller.clone();
    let add_target = target.clone();
    let add_path = path.clone();
    let delete_controller = controller.clone();
    let delete_target = target.clone();
    let delete_path = path.clone();
    let move_controller = controller.clone();
    let move_target = target.clone();
    let move_path = path.clone();
    let copy_controller = controller.clone();
    let copy_target = target.clone();
    let copy_path = path.clone();
    let paste_controller = controller.clone();
    let paste_target = target.clone();
    let paste_path = path.clone();
    let interpolation_controller = controller.clone();
    KeyframeEditorActions {
        add_at_time: Rc::new(move |time| {
            warned(
                "add GTK background integer keyframe",
                add_controller.add_background_integer_keyframe(
                    &add_target,
                    &add_path,
                    timeline_id,
                    time,
                ),
            );
        }),
        delete_at_time: Rc::new(move |time| {
            warned(
                "delete GTK background integer keyframe",
                delete_controller.delete_background_integer_keyframe(
                    &delete_target,
                    &delete_path,
                    timeline_id,
                    time,
                ),
            );
        }),
        update_point: Rc::new(move |old_time, time, displayed_value| {
            warned(
                "move GTK background integer keyframe",
                move_controller.move_background_integer_keyframe(
                    &move_target,
                    &move_path,
                    timeline_id,
                    AudioModifierKeyframeMove {
                        old_time,
                        time,
                        displayed_value: displayed_value.round(),
                        store_multiplier: 1.0,
                    },
                ),
            );
        }),
        clipboard: KeyframeClipboardActions::Managed {
            copy: Rc::new(move |times| {
                warned(
                    "copy GTK background integer keyframes",
                    copy_controller.copy_background_integer_keyframes(
                        &copy_target,
                        &copy_path,
                        timeline_id,
                        times,
                    ),
                )
            }),
            paste: Rc::new(move |time| {
                warned(
                    "paste GTK background integer keyframes",
                    paste_controller.paste_background_integer_keyframes(
                        &paste_target,
                        &paste_path,
                        timeline_id,
                        time,
                    ),
                )
            }),
        },
        set_interpolation: Some(Rc::new(move |owner_id, interpolation| {
            let interpolation = Interpolation::KEYFRAME
                .iter()
                .position(|candidate| *candidate == interpolation)
                .expect("background integer interpolation must be available");
            warned(
                "change GTK background integer interpolation",
                interpolation_controller.set_background_integer_interpolation(
                    &target,
                    &path,
                    timeline_id,
                    owner_id,
                    interpolation,
                ),
            );
        })),
        text_interpolation: None,
        toggle_playback: Rc::new(move || controller.toggle_keyframe_playback()),
    }
}

fn warned<T>(operation: &str, result: Result<T, String>) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(error) => {
            tracing::warn!("Could not {operation}: {error}");
            None
        }
    }
}

fn scalar_target(value_id: uuid::Uuid) -> ScalarTarget {
    ScalarTarget {
        access: ScalarAccess::Background { value_id },
        scope_id: None,
        local_time: video_local_time_for_key,
        duration: video_duration_for_key,
        refresh: background_refresh(),
        commit_name: "edit-background-scalar",
    }
}

fn vec_target(value_id: uuid::Uuid) -> VecTarget {
    VecTarget {
        access: VecAccess::Background { value_id },
        scope_id: None,
        local_time: video_local_time_for_key,
        duration: video_duration_for_key,
        refresh: background_refresh(),
        commit_name: "edit-background-vector",
    }
}

fn generator<'a>(project: &'a Project, key: &InspectedItem) -> Option<&'a BackgroundGenerator> {
    let VideoItemContent::Background(background) = &project.video_item(key)?.content else {
        return None;
    };
    Some(&background.generator)
}

fn generator_mut<'a>(
    project: &'a mut Project,
    key: &InspectedItem,
) -> Option<&'a mut BackgroundGenerator> {
    let VideoItemContent::Background(background) = &mut project.video_item_mut(key)?.content else {
        return None;
    };
    Some(&mut background.generator)
}

fn background_refresh() -> ProjectChange {
    ProjectChange {
        video: true,
        inspector: true,
        ..Default::default()
    }
}

fn video_local_time_for_key(project: &Project, key: InspectedItem, position: Time) -> Option<Time> {
    crate::video::visual_local_time(project, key, position)
}

fn video_duration_for_key(project: &Project, key: InspectedItem) -> Option<Time> {
    let item = project.video_item(&key)?;
    shrimply_project::project::generated_item_keyframe_span(item)
        .map(|(_, end)| end)
        .or_else(|| crate::video::visual_duration(project, key))
}
