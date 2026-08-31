use shrimply_core::timeline_value::{TimelineBase, TimelineValue, TimelineValueType};
use shrimply_gtk_components::ui::InspectorPropertyRow;

pub(crate) fn control<T: TimelineValueType>(
    label: &str,
    value: &TimelineValue<T>,
    editor: gtk::Widget,
    body: Vec<gtk::Widget>,
    on_keyframes_changed: impl Fn(bool) + 'static,
    on_expression_changed: impl Fn(bool) + 'static,
) -> gtk::Widget {
    build(
        label,
        value,
        editor,
        body,
        on_keyframes_changed,
        on_expression_changed,
        false,
    )
}

pub(crate) fn wide_control<T: TimelineValueType>(
    label: &str,
    value: &TimelineValue<T>,
    editor: gtk::Widget,
    body: Vec<gtk::Widget>,
    on_keyframes_changed: impl Fn(bool) + 'static,
    on_expression_changed: impl Fn(bool) + 'static,
) -> gtk::Widget {
    build(
        label,
        value,
        editor,
        body,
        on_keyframes_changed,
        on_expression_changed,
        true,
    )
}

fn build<T: TimelineValueType>(
    label: &str,
    value: &TimelineValue<T>,
    editor: gtk::Widget,
    body: Vec<gtk::Widget>,
    on_keyframes_changed: impl Fn(bool) + 'static,
    on_expression_changed: impl Fn(bool) + 'static,
    wide: bool,
) -> gtk::Widget {
    let row = if wide {
        InspectorPropertyRow::new_wide(label, &editor)
    } else {
        InspectorPropertyRow::new(label, &editor)
    };
    row.set_keyframes_active(matches!(value.base, TimelineBase::Keyframes(_)));
    row.set_expression_active(
        value
            .expression
            .as_ref()
            .is_some_and(|expression| expression.enabled),
    );
    for widget in body {
        row.append_body(&widget);
    }
    row.connect_keyframes_changed(on_keyframes_changed);
    row.connect_expression_changed(on_expression_changed);
    row.widget().clone()
}
