use shrimply_core::timeline_value::{TimelineBase, TimelineStep, TimelineValue};
use shrimply_project::project::Time;

use crate::{
    ControlKind, GraphPoint, InspectorControl, InspectorRuntime, LayeredState, ScalarGraph,
};

pub fn selector(
    path: impl Into<String>,
    label: impl Into<String>,
    selected: impl Into<String>,
    choices: impl IntoIterator<Item = (String, String)>,
) -> InspectorControl {
    build_selector(ControlKind::Selector, path, label, selected.into(), choices)
}

pub fn optional_selector(
    path: impl Into<String>,
    label: impl Into<String>,
    selected: Option<&str>,
    choices: impl IntoIterator<Item = (String, String)>,
) -> InspectorControl {
    build_optional_selector(
        ControlKind::OptionalSelector,
        path,
        label,
        selected.unwrap_or_default().to_string(),
        choices,
    )
}

pub fn optional_number_selector(
    path: impl Into<String>,
    label: impl Into<String>,
    selected: Option<u32>,
    choices: impl IntoIterator<Item = (String, String)>,
) -> InspectorControl {
    build_optional_selector(
        ControlKind::OptionalNumberSelector,
        path,
        label,
        selected.map(|value| value.to_string()).unwrap_or_default(),
        choices,
    )
}

pub fn button_selector(
    path: impl Into<String>,
    label: impl Into<String>,
    selected: impl Into<String>,
    choices: impl IntoIterator<Item = (String, String, String)>,
) -> InspectorControl {
    let choices = choices.into_iter().collect::<Vec<_>>();
    let icons = choices.iter().map(|(_, _, icon)| icon.clone()).collect();
    build_selector(
        ControlKind::Selector,
        path,
        label,
        selected.into(),
        choices.into_iter().map(|(value, label, _)| (value, label)),
    )
    .choice_icons(icons)
}

pub fn step_selector(
    path: impl Into<String>,
    label: impl Into<String>,
    selected: impl Into<String>,
    choices: impl IntoIterator<Item = (String, String, Option<String>)>,
) -> InspectorControl {
    build_step_selector(ControlKind::Selector, path, label, selected, choices)
}

fn build_step_selector(
    kind: ControlKind,
    path: impl Into<String>,
    label: impl Into<String>,
    selected: impl Into<String>,
    choices: impl IntoIterator<Item = (String, String, Option<String>)>,
) -> InspectorControl {
    let choices = choices.into_iter().collect::<Vec<_>>();
    let icons = complete_icons(choices.iter().map(|(_, _, icon)| icon.clone()));
    let control = build_selector(
        kind,
        path,
        label,
        selected.into(),
        choices.into_iter().map(|(value, label, _)| (value, label)),
    );
    match icons {
        Some(icons) => control.choice_icons(icons),
        None => control,
    }
}

pub fn layered_step_selector<T: TimelineStep>(
    path: impl Into<String>,
    label: impl Into<String>,
    value: &TimelineValue<T>,
    runtime: InspectorRuntime,
) -> InspectorControl {
    let path = path.into();
    let current = value.value_at(runtime.local_time.unwrap_or(Time::ZERO));
    let selected = step_variant(&current).key;
    build_step_selector(
        ControlKind::LayeredSelector,
        path.clone(),
        label,
        selected,
        T::variants().iter().map(|variant| {
            (
                variant.key.to_string(),
                variant.label.to_string(),
                variant.icon.map(str::to_string),
            )
        }),
    )
    .layered(path, LayeredState::from(value))
    .timeline(value.id, step_graph(value, runtime))
}

pub fn complete_icons<T>(icons: impl IntoIterator<Item = Option<T>>) -> Option<Vec<T>> {
    icons.into_iter().collect()
}

fn build_optional_selector(
    kind: ControlKind,
    path: impl Into<String>,
    label: impl Into<String>,
    selected: String,
    choices: impl IntoIterator<Item = (String, String)>,
) -> InspectorControl {
    let choices = std::iter::once((String::new(), "None".to_string()))
        .chain(choices)
        .collect::<Vec<_>>();
    let selected = if choices.iter().any(|(value, _)| value == &selected) {
        selected
    } else {
        String::new()
    };
    build_selector(kind, path, label, selected, choices)
}

pub(crate) fn step_graph<T: TimelineStep>(
    value: &TimelineValue<T>,
    runtime: InspectorRuntime,
) -> Option<ScalarGraph> {
    let TimelineBase::Keyframes(_) = &value.base else {
        return None;
    };
    let crate::keyframe_graph::KeyframeGraph::Step { points } =
        crate::keyframe_model::step_graph_with(value, |value| step_variant_index(value) as f64)
    else {
        unreachable!("step timeline must produce a step graph")
    };
    Some(ScalarGraph {
        points: points
            .into_iter()
            .map(|point| GraphPoint {
                time: point.time,
                value: point.value,
            })
            .collect(),
        segments: Vec::new(),
        range: runtime.keyframe_range.unwrap_or((Time::ZERO, Time::ZERO)),
        frame_step: runtime.frame_step,
        playhead: runtime.keyframe_playhead.unwrap_or(Time::ZERO),
    })
}

fn step_variant<T: TimelineStep>(
    value: &T,
) -> &'static shrimply_core::timeline_value::TimelineStepVariant<T> {
    &T::variants()[step_variant_index(value)]
}

fn step_variant_index<T: TimelineStep>(value: &T) -> usize {
    T::variants()
        .iter()
        .position(|variant| variant.value == *value)
        .expect("timeline step value must be one of its declared variants")
}

fn build_selector(
    kind: ControlKind,
    path: impl Into<String>,
    label: impl Into<String>,
    selected: String,
    choices: impl IntoIterator<Item = (String, String)>,
) -> InspectorControl {
    let (values, labels): (Vec<_>, Vec<_>) = choices.into_iter().unzip();
    assert!(
        values.iter().any(|value| value == &selected),
        "selector value must be one of its choices",
    );
    InspectorControl::new(kind, path, label)
        .value(selected)
        .choices(values, labels)
}
