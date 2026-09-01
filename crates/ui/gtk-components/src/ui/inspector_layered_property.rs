use gtk::prelude::*;
use shrimply_component_core::layered::LayeredPropertyController;

use super::InspectorPropertyRow;

pub struct InspectorLayeredProperty {
    row: InspectorPropertyRow,
    controller: LayeredPropertyController,
}

impl InspectorLayeredProperty {
    pub fn new(
        label: &str,
        editor: &impl IsA<gtk::Widget>,
        keyframe_section: &impl IsA<gtk::Widget>,
        expression_section: &impl IsA<gtk::Widget>,
        controller: LayeredPropertyController,
    ) -> Self {
        Self::build(
            label,
            editor,
            keyframe_section,
            expression_section,
            controller,
            false,
        )
    }

    pub fn new_wide(
        label: &str,
        editor: &impl IsA<gtk::Widget>,
        keyframe_section: &impl IsA<gtk::Widget>,
        expression_section: &impl IsA<gtk::Widget>,
        controller: LayeredPropertyController,
    ) -> Self {
        Self::build(
            label,
            editor,
            keyframe_section,
            expression_section,
            controller,
            true,
        )
    }

    fn build(
        label: &str,
        editor: &impl IsA<gtk::Widget>,
        keyframe_section: &impl IsA<gtk::Widget>,
        expression_section: &impl IsA<gtk::Widget>,
        controller: LayeredPropertyController,
        wide: bool,
    ) -> Self {
        let row = if wide {
            InspectorPropertyRow::new_wide(label, editor)
        } else {
            InspectorPropertyRow::new(label, editor)
        };
        row.set_keyframe_section(keyframe_section);
        row.set_expression_section(expression_section);
        row.set_keyframes_active(controller.keyframes());
        row.set_expression_active(controller.expression());
        row.connect_keyframes_changed({
            let controller = controller.clone();
            move |enabled| controller.set_keyframes(enabled)
        });
        row.connect_expression_changed({
            let controller = controller.clone();
            move |enabled| controller.set_expression(enabled)
        });
        Self { row, controller }
    }

    pub fn widget(&self) -> &gtk::Widget {
        self.row.widget()
    }

    pub fn connect_keyframes_changed(&self, changed: impl Fn(bool) + 'static) {
        self.row.connect_keyframes_changed(changed);
    }

    pub fn connect_expression_changed(&self, changed: impl Fn(bool) + 'static) {
        self.row.connect_expression_changed(changed);
    }

    pub fn set_keyframes_active(&self, active: bool) {
        self.controller.set_keyframes(active);
        self.row.set_keyframes_active(active);
    }

    pub fn set_expression_active(&self, active: bool) {
        self.controller.set_expression(active);
        self.row.set_expression_active(active);
    }
}
