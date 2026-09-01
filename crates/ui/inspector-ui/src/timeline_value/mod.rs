use std::{cell::RefCell, rc::Rc};

use gtk::prelude::*;
use shrimply_component_core::layered::LayeredPropertyController;
use shrimply_evaluation::{
    ExpressionOutcome, FrameAudioAnalysis, TransformExpressionCache, VisualEvaluation,
};
use shrimply_gtk_components::ui::InspectorLayeredProperty;
use shrimply_project::project::{Project, Time};

use crate::{InspectedItem, InspectorContext, player_state};

pub(crate) mod boolean;
pub(crate) mod color;
pub(crate) mod scalar;
pub(crate) mod step;
pub(crate) mod text;
pub(crate) mod vector;

pub(crate) use shrimply_core::timeline_value::*;

#[derive(Default)]
pub(crate) struct LayeredSections {
    keyframe: Option<gtk::Widget>,
    expression: Vec<gtk::Widget>,
}

impl LayeredSections {
    pub(crate) fn set_keyframe(&mut self, section: impl IsA<gtk::Widget>) {
        assert!(
            self.keyframe.replace(section.upcast()).is_none(),
            "layered property has more than one keyframe section",
        );
    }

    pub(crate) fn push_expression(&mut self, section: impl IsA<gtk::Widget>) {
        self.expression.push(section.upcast());
    }
}

pub(crate) fn layered_control<T: TimelineValueType>(
    label: &str,
    value: &TimelineValue<T>,
    editor: gtk::Widget,
    sections: LayeredSections,
    on_keyframes_changed: impl Fn(bool) + 'static,
    on_expression_changed: impl Fn(bool) + 'static,
) -> gtk::Widget {
    layered_property(
        label,
        value,
        editor,
        sections,
        on_keyframes_changed,
        on_expression_changed,
        false,
    )
}

pub(crate) fn layered_wide_control<T: TimelineValueType>(
    label: &str,
    value: &TimelineValue<T>,
    editor: gtk::Widget,
    sections: LayeredSections,
    on_keyframes_changed: impl Fn(bool) + 'static,
    on_expression_changed: impl Fn(bool) + 'static,
) -> gtk::Widget {
    layered_property(
        label,
        value,
        editor,
        sections,
        on_keyframes_changed,
        on_expression_changed,
        true,
    )
}

fn layered_property<T: TimelineValueType>(
    label: &str,
    value: &TimelineValue<T>,
    editor: gtk::Widget,
    sections: LayeredSections,
    on_keyframes_changed: impl Fn(bool) + 'static,
    on_expression_changed: impl Fn(bool) + 'static,
    wide: bool,
) -> gtk::Widget {
    let controller = LayeredPropertyController::default();
    controller.set_keyframes(matches!(value.base, TimelineBase::Keyframes(_)));
    controller.set_expression(
        value
            .expression
            .as_ref()
            .is_some_and(|expression| expression.enabled),
    );
    let mut property = InspectorLayeredProperty::builder(label, &editor, controller);
    if wide {
        property = property.wide();
    }
    if let Some(section) = sections.keyframe {
        property = property.keyframe_section(&section);
    }
    let expression_section = gtk::Box::new(gtk::Orientation::Vertical, 6);
    for section in sections.expression {
        expression_section.append(&section);
    }
    if expression_section.first_child().is_some() {
        property = property.expression_section(&expression_section);
    }
    property
        .on_keyframes_changed(on_keyframes_changed)
        .on_expression_changed(on_expression_changed)
        .build()
        .widget()
        .clone()
}

pub(crate) struct ExpressionOutput {
    pub(crate) value: String,
    pub(crate) error: Option<String>,
}

pub(crate) fn expression_section(
    context: &InspectorContext,
    listener_name: &'static str,
    editor: impl FnOnce(Rc<dyn Fn()>) -> gtk::Widget,
    evaluate: impl Fn(
        &Project,
        Time,
        &FrameAudioAnalysis,
        &mut TransformExpressionCache,
    ) -> Option<ExpressionOutput>
    + 'static,
) -> gtk::Widget {
    let output = gtk::Label::builder()
        .hexpand(true)
        .xalign(1.0)
        .selectable(true)
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .build();
    let output_row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let output_title = gtk::Label::new(Some(shrimply_gtk_components::tr!("Output").as_ref()));
    output_title.add_css_class("dim-label");
    output_row.append(&output_title);
    output_row.append(&output);

    let project = context.project.clone();
    let player = context.player_state.clone();
    let volume = context.volume.clone();
    let cache = Rc::new(RefCell::new(TransformExpressionCache::default()));
    let evaluate = Rc::new(evaluate);
    let refresh: Rc<dyn Fn()> = Rc::new({
        let output = output.clone();
        move || {
            if !output.is_mapped() {
                return;
            }
            let snapshot = player_state::snapshot(&player);
            let project = project.borrow();
            let audio = volume
                .borrow_mut()
                .sample(&project, snapshot.position, snapshot.revision);
            let Some(result) =
                evaluate(&project, snapshot.position, &audio, &mut cache.borrow_mut())
            else {
                return;
            };
            output.set_label(&result.value);
            output.set_tooltip_text(result.error.as_deref());
            if result.error.is_some() {
                output.add_css_class("error");
            } else {
                output.remove_css_class("error");
            }
        }
    });

    let section = gtk::Box::new(gtk::Orientation::Vertical, 6);
    section.append(&editor(refresh.clone()));
    section.append(&output_row);
    section.connect_map({
        let refresh = refresh.clone();
        move |_| refresh()
    });

    let alive = Rc::downgrade(&context.listener_scope);
    let output = output.downgrade();
    player_state::connect_while_alive_named(
        &context.player_state,
        listener_name,
        move || alive.upgrade().is_some(),
        move |_| {
            if output.upgrade().is_some_and(|output| output.is_mapped()) {
                refresh();
            }
        },
    );
    section.upcast()
}

pub(crate) fn evaluate_expression<T: TimelineExpressionValue>(
    project: &Project,
    key: &InspectedItem,
    position: Time,
    audio: &FrameAudioAnalysis,
    cache: &mut TransformExpressionCache,
    value: &TimelineValue<T>,
) -> Option<ExpressionOutcome<T>> {
    let position = crate::video::visual_sequence_time(project, key, position)?;
    let item = project.video_item(key)?;
    let evaluation = VisualEvaluation::for_item_with_audio(project, item, position, audio);
    Some(shrimply_evaluation::resolve_with_error(
        value,
        &evaluation,
        cache,
    ))
}
