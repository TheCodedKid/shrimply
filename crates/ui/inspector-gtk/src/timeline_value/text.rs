use std::cell::RefCell;
use std::rc::Rc;

use shrimply_gtk_components::ui::MultilineTextInput;
use shrimply_inspector_core::generated::text::{
    TEXT_EDIT_COMMIT, TEXT_EXPRESSION_COMMIT, TEXT_KEYFRAME_COMMITS,
};
use shrimply_project::project::{Project, TextItem, Time, VideoItemContent};
use uuid::Uuid;

use crate::InspectedItem as SelectedItem;
use crate::keyframe_editor::{self, KeyframeEditorActions, TextInterpolationActions};
use crate::player_state::{self, ProjectChange, SharedPlayerState};
use crate::{InspectorContext, keyframe_model};

use super::{
    ExpressionOutput, Interpolation, LayeredSections, TextInterpolation, TimelineBase,
    TimelineValue, evaluate_expression, expression_section, layered_wide_control,
};

pub(crate) fn text_control(
    label: &str,
    value: &TimelineValue<String>,
    context: &InspectorContext,
) -> gtk::Widget {
    let timeline_id = value.id;
    let Some(key) = context.selected_item.clone() else {
        let input = MultilineTextInput::builder(value.fallback())
            .min_content_height(86)
            .build();
        return layered_wide_control(
            label,
            value,
            input.widget().clone(),
            LayeredSections::default(),
            |_| {},
            |_| {},
        );
    };
    let position = player_state::snapshot(&context.player_state).position;
    let time = local_time(&context.project.borrow(), key.clone(), position).unwrap_or(Time::ZERO);
    let project = context.project.clone();
    let player = context.player_state.clone();
    let commit_project = context.project.clone();
    let input_key = key.clone();
    let input = MultilineTextInput::builder(shrimply_inspector_core::timeline_text::value_at(
        value, time,
    ))
    .min_content_height(86)
    .on_change(move |text| update_value(&project, &player, input_key.clone(), timeline_id, text))
    .on_commit(move || {
        shrimply_project::project::commit_edit(&commit_project.borrow(), TEXT_EDIT_COMMIT);
    })
    .build();

    let mut sections = LayeredSections::default();
    if matches!(value.base, TimelineBase::Keyframes(_)) {
        let built = keyframe_editor::build(
            context,
            shrimply_inspector_core::timeline_text::keyframe_graph(value),
            visible_area(&context.project.borrow(), key.clone())
                .unwrap_or((Time::ZERO, Time::ZERO)),
            format!("text:{}", value.id),
            actions(context, key.clone(), timeline_id),
        );
        let graph_project = context.project.clone();
        let graph_key = key.clone();
        keyframe_editor::connect_graph_refresh(
            context,
            "inspector text keyframe graph refresh",
            &built,
            move || {
                text_value(&graph_project.borrow(), graph_key.clone(), timeline_id)
                    .map(shrimply_inspector_core::timeline_text::keyframe_graph)
            },
        );
        sections.set_keyframe(built.widget);
    }
    if value
        .expression
        .as_ref()
        .is_some_and(|expression| expression.enabled)
    {
        sections.push_expression(expression_editor(context, key.clone(), timeline_id));
    }

    connect_text_refresh(context, key.clone(), timeline_id, input.set_text_handler());

    let project = context.project.clone();
    let player = context.player_state.clone();
    let refresh = context.refresh.clone();
    let expression_project = context.project.clone();
    let expression_player = context.player_state.clone();
    let expression_refresh = context.refresh.clone();
    let keyframe_key = key.clone();
    let expression_key = key;
    layered_wide_control(
        label,
        value,
        input.widget().clone(),
        sections,
        move |enabled| {
            if toggle_keyframes(
                &project,
                &player,
                keyframe_key.clone(),
                timeline_id,
                enabled,
            ) {
                refresh();
            }
        },
        move |enabled| {
            if toggle_expression(
                &expression_project,
                &expression_player,
                expression_key.clone(),
                timeline_id,
                enabled,
            ) {
                expression_refresh();
            }
        },
    )
}

fn connect_text_refresh(
    context: &InspectorContext,
    key: SelectedItem,
    timeline_id: Uuid,
    set_text: Rc<dyn Fn(&str)>,
) {
    let project = context.project.clone();
    let player = context.player_state.clone();
    let alive = Rc::downgrade(&context.listener_scope);
    let alive_for_prune = alive.clone();
    player_state::connect_while_alive_named(
        &context.player_state,
        "inspector text display refresh",
        move || alive_for_prune.upgrade().is_some(),
        move |event| {
            if !matches!(
                event,
                player_state::PlayerEvent::State(_) | player_state::PlayerEvent::Project(_)
            ) || alive.upgrade().is_none()
            {
                return;
            }
            let position = player_state::snapshot(&player).position;
            let project = project.borrow();
            let Some(time) = local_time(&project, key.clone(), position) else {
                return;
            };
            if let Some(value) = text_value(&project, key.clone(), timeline_id) {
                set_text(&shrimply_inspector_core::timeline_text::value_at(
                    value, time,
                ));
            }
        },
    );
}

fn expression_editor(
    context: &InspectorContext,
    key: SelectedItem,
    timeline_id: Uuid,
) -> gtk::Widget {
    let source = text_value(&context.project.borrow(), key.clone(), timeline_id)
        .and_then(|value| value.expression_source().map(str::to_string));
    let project = context.project.clone();
    let player = context.player_state.clone();
    let editor_key = key.clone();
    let output_key = key;
    expression_section(
        context,
        "inspector text expression feedback",
        move |refresh| {
            crate::rhai_editor::editor(
                source,
                crate::rhai_editor::ExpressionValue::Text,
                move |source| {
                    update_expression(&project, &player, editor_key.clone(), timeline_id, source);
                    refresh();
                },
            )
        },
        move |project, position, audio, cache| {
            let value = text_value(project, output_key.clone(), timeline_id)?;
            let outcome = evaluate_expression(project, &output_key, position, audio, cache, value)?;
            Some(ExpressionOutput {
                value: outcome.value,
                error: outcome.error,
            })
        },
    )
}

fn actions(
    context: &InspectorContext,
    key: SelectedItem,
    timeline_id: Uuid,
) -> KeyframeEditorActions {
    let project = context.project.clone();
    let player = context.player_state.clone();
    KeyframeEditorActions {
        add_at_time: {
            let key = key.clone();
            let project = project.clone();
            let player = player.clone();
            Rc::new(move |time| add_key(&project, &player, key.clone(), timeline_id, time))
        },
        delete_at_time: {
            let key = key.clone();
            let project = project.clone();
            let player = player.clone();
            Rc::new(move |time| delete_key(&project, &player, key.clone(), timeline_id, time))
        },
        update_point: {
            let key = key.clone();
            let project = project.clone();
            let player = player.clone();
            Rc::new(move |old, time, _| {
                move_key(&project, &player, key.clone(), timeline_id, old, time)
            })
        },
        clipboard: crate::keyframe_editor::KeyframeClipboardActions::Local {
            copy: {
                let key = key.clone();
                let project = project.clone();
                Rc::new(move |times| {
                    text_value(&project.borrow(), key.clone(), timeline_id).and_then(|value| {
                        shrimply_inspector_core::timeline_text::copy_keyframes(value, times)
                    })
                })
            },
            paste: {
                let key = key.clone();
                let project = project.clone();
                let player = player.clone();
                Rc::new(move |clipboard, time| {
                    let mut project = project.borrow_mut();
                    let value = text_value_mut(&mut project, key.clone(), timeline_id)?;
                    let times = shrimply_inspector_core::timeline_text::paste_keyframes(
                        value, clipboard, time,
                    )?;
                    shrimply_project::project::commit_edit(&project, TEXT_KEYFRAME_COMMITS.paste);
                    drop(project);
                    refresh(&player, true);
                    Some(times)
                })
            },
        },
        set_interpolation: Some({
            let key = key.clone();
            let project = project.clone();
            let player = player.clone();
            Rc::new(move |id, interpolation| {
                set_interpolation(
                    &project,
                    &player,
                    key.clone(),
                    timeline_id,
                    id,
                    interpolation,
                )
            })
        }),
        text_interpolation: Some(TextInterpolationActions {
            get: {
                let key = key.clone();
                let project = project.clone();
                Rc::new(move |id| {
                    let project = project.borrow();
                    shrimply_inspector_core::timeline_text::text_interpolation(
                        text_value(&project, key.clone(), timeline_id)?,
                        id,
                    )
                })
            },
            set: {
                let key = key.clone();
                let project = project.clone();
                let player = player.clone();
                Rc::new(move |id, mode| {
                    set_text_interpolation(&project, &player, key.clone(), timeline_id, id, mode)
                })
            },
        }),
        toggle_playback: Rc::new(move || player_state::toggle_playing(&player)),
    }
}

fn update_value(
    project: &Rc<RefCell<Project>>,
    player: &SharedPlayerState,
    key: SelectedItem,
    timeline_id: Uuid,
    next: String,
) -> bool {
    let position = player_state::snapshot(player).position;
    let mut project = project.borrow_mut();
    let Some(time) = project.keyframe_time(&key, position) else {
        return false;
    };
    let step = keyframe_editor::project_frame_step(&project, Some(&key));
    let Some(value) = text_value_mut(&mut project, key.clone(), timeline_id) else {
        return false;
    };
    let changed = shrimply_inspector_core::timeline_text::set_value(value, time, next, step);
    drop(project);
    if changed {
        refresh(player, false);
    }
    changed
}

fn toggle_keyframes(
    project: &Rc<RefCell<Project>>,
    player: &SharedPlayerState,
    key: SelectedItem,
    timeline_id: Uuid,
    enabled: bool,
) -> bool {
    let position = player_state::snapshot(player).position;
    let mut project = project.borrow_mut();
    let Some(evaluation_time) = local_time(&project, key.clone(), position) else {
        return false;
    };
    let Some(keyframe_time) = project.keyframe_time(&key, position) else {
        return false;
    };
    let Some(value) = text_value_mut(&mut project, key.clone(), timeline_id) else {
        return false;
    };
    if !shrimply_inspector_core::timeline_text::set_keyframes_enabled(
        value,
        evaluation_time,
        keyframe_time,
        enabled,
    ) {
        return false;
    }
    shrimply_project::project::commit_edit(&project, TEXT_KEYFRAME_COMMITS.toggle);
    drop(project);
    refresh(player, true);
    true
}

fn toggle_expression(
    project: &Rc<RefCell<Project>>,
    player: &SharedPlayerState,
    key: SelectedItem,
    timeline_id: Uuid,
    enabled: bool,
) -> bool {
    let mut project = project.borrow_mut();
    let Some(value) = text_value_mut(&mut project, key.clone(), timeline_id) else {
        return false;
    };
    let changed = shrimply_inspector_core::timeline_text::set_expression_enabled(value, enabled);
    if !changed {
        return false;
    }
    shrimply_project::project::commit_edit(&project, TEXT_EXPRESSION_COMMIT);
    drop(project);
    refresh(player, true);
    true
}

fn update_expression(
    project: &Rc<RefCell<Project>>,
    player: &SharedPlayerState,
    key: SelectedItem,
    timeline_id: Uuid,
    source: String,
) {
    let mut project = project.borrow_mut();
    let Some(value) = text_value_mut(&mut project, key.clone(), timeline_id) else {
        return;
    };
    if !shrimply_inspector_core::timeline_text::set_expression_source(value, source) {
        return;
    }
    shrimply_project::project::commit_coalesced_edit(&project, TEXT_EXPRESSION_COMMIT);
    drop(project);
    refresh(player, false);
}

fn add_key(
    project: &Rc<RefCell<Project>>,
    player: &SharedPlayerState,
    key: SelectedItem,
    timeline_id: Uuid,
    time: Time,
) {
    let mut project = project.borrow_mut();
    let step = keyframe_editor::project_frame_step(&project, Some(&key));
    let Some(value) = text_value_mut(&mut project, key.clone(), timeline_id) else {
        return;
    };
    shrimply_inspector_core::timeline_text::add_keyframe(value, time, step);
    shrimply_project::project::commit_edit(&project, TEXT_KEYFRAME_COMMITS.add);
    drop(project);
    refresh(player, true);
}

fn delete_key(
    project: &Rc<RefCell<Project>>,
    player: &SharedPlayerState,
    key: SelectedItem,
    timeline_id: Uuid,
    time: Time,
) {
    let mut project = project.borrow_mut();
    let step = keyframe_editor::project_frame_step(&project, Some(&key));
    let Some(value) = text_value_mut(&mut project, key.clone(), timeline_id) else {
        return;
    };
    shrimply_inspector_core::timeline_text::delete_keyframe(value, time, step);
    shrimply_project::project::commit_edit(&project, TEXT_KEYFRAME_COMMITS.delete);
    drop(project);
    refresh(player, true);
}

fn move_key(
    project: &Rc<RefCell<Project>>,
    player: &SharedPlayerState,
    key: SelectedItem,
    timeline_id: Uuid,
    old: Time,
    time: Time,
) {
    let mut project = project.borrow_mut();
    let changed = text_value_mut(&mut project, key.clone(), timeline_id).is_some_and(|value| {
        shrimply_inspector_core::timeline_text::move_keyframes(value, &[(old, time)])
    });
    if changed {
        shrimply_project::project::commit_coalesced_edit(
            &project,
            TEXT_KEYFRAME_COMMITS.move_keyframe,
        );
    }
    drop(project);
    player_state::refresh_project(
        player,
        keyframe_model::live_refresh(ProjectChange {
            video: true,
            ..Default::default()
        }),
    );
}

fn set_interpolation(
    project: &Rc<RefCell<Project>>,
    player: &SharedPlayerState,
    key: SelectedItem,
    timeline_id: Uuid,
    id: Uuid,
    interpolation: Interpolation,
) {
    let mut project = project.borrow_mut();
    let Some(value) = text_value_mut(&mut project, key.clone(), timeline_id) else {
        return;
    };
    if shrimply_inspector_core::timeline_text::set_interpolation(value, id, interpolation).is_err()
    {
        return;
    }
    shrimply_project::project::commit_edit(&project, TEXT_KEYFRAME_COMMITS.interpolation);
    drop(project);
    refresh(player, false);
}

fn set_text_interpolation(
    project: &Rc<RefCell<Project>>,
    player: &SharedPlayerState,
    key: SelectedItem,
    timeline_id: Uuid,
    id: Uuid,
    interpolation: TextInterpolation,
) {
    let mut project = project.borrow_mut();
    let Some(value) = text_value_mut(&mut project, key.clone(), timeline_id) else {
        return;
    };
    if shrimply_inspector_core::timeline_text::set_text_interpolation(value, id, interpolation)
        .is_err()
    {
        return;
    }
    shrimply_project::project::commit_edit(&project, TEXT_KEYFRAME_COMMITS.text_interpolation);
    drop(project);
    refresh(player, false);
}

fn refresh(player: &SharedPlayerState, inspector: bool) {
    player_state::refresh_project(
        player,
        ProjectChange {
            video: true,
            inspector,
            ..Default::default()
        },
    );
}

fn text_value(
    project: &Project,
    key: SelectedItem,
    timeline_id: Uuid,
) -> Option<&TimelineValue<String>> {
    let value = &selected_text(project, key)?.text;
    (value.id == timeline_id).then_some(value)
}

fn text_value_mut(
    project: &mut Project,
    key: SelectedItem,
    timeline_id: Uuid,
) -> Option<&mut TimelineValue<String>> {
    let value = &mut selected_text_mut(project, key)?.text;
    (value.id == timeline_id).then_some(value)
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

fn local_time(project: &Project, key: SelectedItem, position: Time) -> Option<Time> {
    crate::video::visual_local_time(project, key, position)
}

fn visible_area(project: &Project, key: SelectedItem) -> Option<(Time, Time)> {
    crate::video::visual_visible_area(project, key)
}
