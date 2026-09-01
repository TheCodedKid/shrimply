use std::{cell::RefCell, rc::Rc};

use gtk::prelude::*;
use gtk::{gdk, gio};
use shrimply_core::timeline_value::TextInterpolation;
use shrimply_gtk_components::tr;
use shrimply_gtk_components::ui::{FrameGraph, SearchMenuItem, matches_query, searchable_popover};
use shrimply_interpolation::Interpolation;
use shrimply_keyframe_graph_ui::{FrameGraphAction, FrameGraphComponentAction, FrameGraphState};
use shrimply_project::project::{ItemAddress, Project, Time};
use shrimply_state::preferences;
use uuid::Uuid;

use super::{InspectorContext, keyframe_model};
use crate::player_state;

pub(crate) use super::keyframe_graph::{KeyframeGraph, KeyframePoint, RawSegment, SpeedSegment};

const KEYFRAME_CLIPBOARD_MARKER: &str = "shrimply keyframes";

thread_local! {
    static KEYFRAME_CLIPBOARD: RefCell<Option<keyframe_model::KeyframeClipboard>> = const { RefCell::new(None) };
}

pub(crate) struct BuiltKeyframeEditor {
    pub(crate) widget: gtk::Widget,
    pub(crate) frame_graph: FrameGraph,
    pub(crate) update_graph: Rc<dyn Fn(KeyframeGraph)>,
    update_playhead: Rc<dyn Fn()>,
}

pub(crate) type CopyKeyframes = Rc<dyn Fn(&[Time]) -> Option<keyframe_model::KeyframeClipboard>>;
pub(crate) type PasteKeyframes =
    Rc<dyn Fn(&keyframe_model::KeyframeClipboard, &[Time]) -> Option<Vec<Time>>>;

pub(crate) struct KeyframeEditorActions {
    pub(crate) add_at_time: Rc<dyn Fn(Time)>,
    pub(crate) delete_at_time: Rc<dyn Fn(Time)>,
    pub(crate) update_point: Rc<dyn Fn(Time, Time, f64)>,
    pub(crate) copy_keyframes: CopyKeyframes,
    pub(crate) paste_keyframes: PasteKeyframes,
    pub(crate) set_interpolation: Option<Rc<dyn Fn(Uuid, Interpolation)>>,
    pub(crate) text_interpolation: Option<TextInterpolationActions>,
    pub(crate) toggle_playback: Rc<dyn Fn()>,
}

pub(crate) struct TextInterpolationActions {
    pub(crate) get: Rc<dyn Fn(Uuid) -> Option<TextInterpolation>>,
    pub(crate) set: Rc<dyn Fn(Uuid, TextInterpolation)>,
}

struct GraphActionContext {
    actions: Rc<KeyframeEditorActions>,
    project: Rc<RefCell<Project>>,
    selected_item: Option<ItemAddress>,
    select_time: Rc<dyn Fn(Time)>,
    graph_area: Rc<RefCell<Option<gtk::GLArea>>>,
}

pub(crate) fn project_frame_step(project: &Project, item: Option<&ItemAddress>) -> Time {
    item.and_then(|item| project.keyframe_step(item))
        .filter(|step| *step > Time::ZERO)
        .unwrap_or_else(|| project.frame_step())
}

pub(crate) fn project_frame_keyframe_time(
    project: &Project,
    item: Option<&ItemAddress>,
    time: Time,
) -> Option<Time> {
    let Some(item) = item else {
        return Some(time.snapped(project.frame_step()));
    };
    project
        .keyframe_timeline_time(item, time)
        .and_then(|timeline_time| project.keyframe_time(item, timeline_time))
}

pub(crate) fn build(
    context: &InspectorContext,
    graph: KeyframeGraph,
    visible_area: (Time, Time),
    view_state_scope: impl Into<String>,
    actions: KeyframeEditorActions,
) -> BuiltKeyframeEditor {
    let project = context.project.clone();
    let selected_item = context.selected_item.clone();
    let frame_step = project_frame_step(&project.borrow(), selected_item.as_ref());
    let item_range =
        clip_bounded_visible_area(&project.borrow(), selected_item.as_ref(), visible_area);
    let playhead = local_playhead(context);
    let mut initial = FrameGraphState::new(graph.clone(), item_range, frame_step, playhead());
    configure_state(
        &mut initial,
        &context.preferences,
        actions.text_interpolation.is_some(),
    );
    let state = context.keyframe_graph_state(view_state_scope, initial);
    {
        let mut state = state.borrow_mut();
        state.replace_graph(graph);
        state.set_item_range(item_range);
        state.set_frame_step(frame_step);
        state.set_playhead(playhead());
        configure_state(
            &mut state,
            &context.preferences,
            actions.text_interpolation.is_some(),
        );
    }

    let action_context = Rc::new(GraphActionContext {
        actions: Rc::new(actions),
        project: project.clone(),
        selected_item: selected_item.clone(),
        select_time: select_time(context),
        graph_area: Rc::new(RefCell::new(None)),
    });
    let frame_graph = FrameGraph::with_shared_components(state.clone(), {
        let context = action_context.clone();
        move |action| dispatch_action(&context, action)
    });
    action_context
        .graph_area
        .replace(Some(frame_graph.graph_area().clone()));

    let update_graph = {
        let frame_graph = frame_graph.clone();
        let preferences = context.preferences.clone();
        let text_interpolation = action_context.actions.text_interpolation.is_some();
        let playhead = playhead.clone();
        Rc::new(move |updated| {
            let project = project.borrow();
            let item_range =
                clip_bounded_visible_area(&project, selected_item.as_ref(), visible_area);
            let frame_step = project_frame_step(&project, selected_item.as_ref());
            drop(project);
            let mut state = state.borrow_mut();
            state.replace_graph(updated);
            state.set_item_range(item_range);
            state.set_frame_step(frame_step);
            state.set_playhead(playhead());
            configure_state(&mut state, &preferences, text_interpolation);
            drop(state);
            frame_graph.refresh();
        }) as Rc<dyn Fn(KeyframeGraph)>
    };
    let update_playhead = {
        let frame_graph = frame_graph.clone();
        let playhead = playhead.clone();
        Rc::new(move || {
            if frame_graph.graph_area().is_mapped() {
                frame_graph.set_playhead(playhead());
            }
        }) as Rc<dyn Fn()>
    };
    frame_graph.graph_area().connect_map({
        let frame_graph = frame_graph.clone();
        let playhead = playhead.clone();
        move |_| frame_graph.set_playhead(playhead())
    });

    BuiltKeyframeEditor {
        widget: frame_graph.widget().clone().upcast(),
        frame_graph,
        update_graph,
        update_playhead,
    }
}

pub(crate) fn connect_graph_refresh_impl(
    context: &InspectorContext,
    label: &'static str,
    editor: &BuiltKeyframeEditor,
    graph: impl Fn() -> Option<KeyframeGraph> + 'static,
) {
    let update_graph = editor.update_graph.clone();
    let update_playhead = editor.update_playhead.clone();
    let graph = Rc::new(graph);
    editor.frame_graph.graph_area().connect_map({
        let graph = graph.clone();
        let update_graph = update_graph.clone();
        move |_| {
            if let Some(graph) = graph() {
                update_graph(graph);
            }
        }
    });
    let graph_area = editor.frame_graph.graph_area().clone();
    let alive = Rc::downgrade(&context.listener_scope);
    player_state::connect_while_alive_named(
        &context.player_state,
        label,
        move || alive.upgrade().is_some(),
        move |event| match event {
            player_state::PlayerEvent::State(_) => update_playhead(),
            player_state::PlayerEvent::Project(_) => {
                if graph_area.is_mapped()
                    && let Some(graph) = graph()
                {
                    update_graph(graph);
                }
            }
        },
    );
}

pub(crate) use connect_graph_refresh_impl as connect_graph_refresh;

fn configure_state(
    state: &mut FrameGraphState,
    preferences: &preferences::SharedPreferences,
    text_interpolation: bool,
) {
    let preferences = preferences::snapshot(preferences);
    state.set_snapping(
        preferences.timeline_magnet == "true",
        f64::from(preferences.timeline_snap_radius_px),
    );
    state.set_external_clipboard(true);
    state.set_text_interpolation(text_interpolation);
}

fn local_playhead(context: &InspectorContext) -> Rc<dyn Fn() -> Time> {
    let project = context.project.clone();
    let player_state = context.player_state.clone();
    let selected_item = context.selected_item.clone();
    Rc::new(move || {
        let position = player_state::snapshot(&player_state).position;
        selected_item
            .as_ref()
            .and_then(|key| project.borrow().keyframe_time(key, position))
            .unwrap_or(position)
    })
}

fn select_time(context: &InspectorContext) -> Rc<dyn Fn(Time)> {
    let project = context.project.clone();
    let player_state = context.player_state.clone();
    let selected_item = context.selected_item.clone();
    Rc::new(move |time| {
        let position = selected_item
            .as_ref()
            .and_then(|key| project.borrow().keyframe_timeline_time(key, time))
            .unwrap_or(time);
        player_state::seek_time(&player_state, position);
    })
}

fn dispatch_action(context: &GraphActionContext, action: FrameGraphComponentAction) {
    assert_eq!(action.component, 0, "production keyframe graph is scalar");
    match action.action {
        FrameGraphAction::PlayheadChanged(time) => (context.select_time)(time),
        FrameGraphAction::KeysMoved(moves) => {
            for key_move in moves {
                let time = {
                    let project = context.project.borrow();
                    project_frame_keyframe_time(
                        &project,
                        context.selected_item.as_ref(),
                        key_move.time,
                    )
                };
                let Some(time) = time else {
                    continue;
                };
                (context.actions.update_point)(key_move.old_time, time, key_move.value);
            }
        }
        FrameGraphAction::KeysDeleted(times) => {
            for time in times {
                (context.actions.delete_at_time)(time);
            }
        }
        FrameGraphAction::KeyAdded(point) => {
            let time = {
                let project = context.project.borrow();
                project_frame_keyframe_time(&project, context.selected_item.as_ref(), point.time)
            };
            if let Some(time) = time {
                (context.actions.add_at_time)(time);
            }
        }
        FrameGraphAction::CopyRequested(times) => copy_keyframes(context, &times),
        FrameGraphAction::PasteRequested(time) => paste_keyframes(context, time),
        FrameGraphAction::TogglePlayback => (context.actions.toggle_playback)(),
        FrameGraphAction::InterpolationRequested {
            owner_id,
            interpolation,
            ..
        } => {
            if let Some(set) = &context.actions.set_interpolation {
                set(owner_id, interpolation);
            }
        }
        FrameGraphAction::TextInterpolationRequested { owner_id, x, y } => {
            show_text_interpolation(context, owner_id, x, y);
        }
        FrameGraphAction::KeysChanged(_) | FrameGraphAction::KeysPasted(_) => {
            panic!("authoritative keyframe graph received a local-only mutation")
        }
    }
}

fn copy_keyframes(context: &GraphActionContext, selected: &[Time]) {
    let Some(mut clipboard) = (context.actions.copy_keyframes)(selected) else {
        return;
    };
    let project = context.project.borrow();
    let timeline_times = clipboard
        .times
        .iter()
        .map(|time| {
            context
                .selected_item
                .as_ref()
                .and_then(|item| project.keyframe_timeline_time(item, *time))
                .unwrap_or(*time)
                .snapped(project.frame_step())
        })
        .collect::<Vec<_>>();
    let Some(origin) = timeline_times.first().copied() else {
        return;
    };
    clipboard.times = timeline_times
        .into_iter()
        .map(|time| Time {
            seconds: time.seconds - origin.seconds,
        })
        .collect();
    drop(project);
    let count = clipboard.len();
    let Some(area) = context.graph_area.borrow().clone() else {
        return;
    };
    area.display()
        .clipboard()
        .set_text(KEYFRAME_CLIPBOARD_MARKER);
    KEYFRAME_CLIPBOARD.with(|stored| stored.replace(Some(clipboard)));
    show_count_toast(&area, count, false);
}

fn paste_keyframes(context: &GraphActionContext, time: Time) {
    let clipboard = KEYFRAME_CLIPBOARD.with(|stored| stored.borrow().clone());
    let Some(clipboard) = clipboard else {
        return;
    };
    let times = {
        let project = context.project.borrow();
        let anchor = context
            .selected_item
            .as_ref()
            .and_then(|item| project.keyframe_timeline_time(item, time))
            .unwrap_or(time)
            .snapped(project.frame_step());
        clipboard
            .times
            .iter()
            .filter_map(|offset| {
                let timeline_time = Time {
                    seconds: anchor.seconds + offset.seconds,
                };
                context
                    .selected_item
                    .as_ref()
                    .map(|item| project.keyframe_time(item, timeline_time))
                    .unwrap_or(Some(timeline_time))
            })
            .collect::<Vec<_>>()
    };
    if times.len() != clipboard.len() {
        return;
    }
    let Some(area) = context.graph_area.borrow().clone() else {
        return;
    };
    let paste = context.actions.paste_keyframes.clone();
    area.display()
        .clipboard()
        .read_text_async(None::<&gio::Cancellable>, move |result| {
            if result.ok().flatten().as_deref() != Some(KEYFRAME_CLIPBOARD_MARKER) {
                return;
            }
            let Some(times) = paste(&clipboard, &times) else {
                return;
            };
            show_count_toast(&area, times.len(), true);
        });
}

fn show_count_toast(area: &gtk::GLArea, count: usize, pasted: bool) {
    let message = match (count, pasted) {
        (1, false) => tr!("1 keyframe copied").into_owned(),
        (1, true) => tr!("1 keyframe pasted").into_owned(),
        (_, false) => shrimply_gtk_components::i18n::text_args(
            "%{count} keyframes copied",
            &[("count", count.to_string())],
        ),
        (_, true) => shrimply_gtk_components::i18n::text_args(
            "%{count} keyframes pasted",
            &[("count", count.to_string())],
        ),
    };
    shrimply_gtk_components::toast::show_confirmation_text_for_widget(area, &message);
}

fn show_text_interpolation(context: &GraphActionContext, owner_id: Uuid, x: f64, y: f64) {
    let Some(actions) = &context.actions.text_interpolation else {
        return;
    };
    let Some(selected) = (actions.get)(owner_id) else {
        return;
    };
    let Some(area) = context.graph_area.borrow().clone() else {
        return;
    };
    let set = actions.set.clone();
    let popover = searchable_popover(
        tr!("Search interpolations").as_ref(),
        280,
        180,
        240,
        move |query| {
            TextInterpolation::ALL
                .into_iter()
                .filter(|mode| matches_query(mode.label(), query))
                .map(|mode| {
                    let set = set.clone();
                    SearchMenuItem::new(tr!(mode.label()).as_ref(), move || set(owner_id, mode))
                        .selected(mode == selected)
                        .tooltip(text_interpolation_tooltip(mode))
                })
                .collect()
        },
    );
    popover.set_parent(&area);
    popover.set_pointing_to(Some(&gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
    popover.connect_closed(|popover| popover.unparent());
    popover.popup();
}

fn text_interpolation_tooltip(mode: TextInterpolation) -> &'static str {
    match mode {
        TextInterpolation::Jump => "Change all at once",
        TextInterpolation::Type => "Clear and rewrite the whole text",
        TextInterpolation::Append => "Edit after the shared beginning",
        TextInterpolation::Insert => "Edit between the shared ends",
        TextInterpolation::Diff => "Edit only the changed characters",
        TextInterpolation::Decode => "Scramble, resize, then reveal the new text",
    }
}

fn clip_bounded_visible_area(
    project: &Project,
    selected_item: Option<&ItemAddress>,
    (start, end): (Time, Time),
) -> (Time, Time) {
    let clip_duration = selected_item
        .and_then(|key| project.item(key))
        .map(|item| {
            let (start, end) = item.times();
            end.saturating_sub(start).max(start.saturating_sub(end))
        })
        .unwrap_or(Time::ZERO);
    (start, end.max(start.saturating_add(clip_duration)))
}
