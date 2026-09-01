use gtk::prelude::*;
use shrimply_component_core::layered::LayeredPropertyController;

use super::InspectorPropertyRow;

pub struct InspectorLayeredProperty {
    row: InspectorPropertyRow,
    controller: LayeredPropertyController,
}

pub struct InspectorLayeredPropertyBuilder {
    label: String,
    editor: gtk::Widget,
    keyframe_section: Option<gtk::Widget>,
    expression_section: Option<gtk::Widget>,
    controller: LayeredPropertyController,
    wide: bool,
    on_keyframes_changed: Option<Box<dyn Fn(bool)>>,
    on_expression_changed: Option<Box<dyn Fn(bool)>>,
}

impl InspectorLayeredProperty {
    pub fn builder(
        label: impl Into<String>,
        editor: &impl IsA<gtk::Widget>,
        controller: LayeredPropertyController,
    ) -> InspectorLayeredPropertyBuilder {
        InspectorLayeredPropertyBuilder {
            label: label.into(),
            editor: editor.as_ref().clone(),
            keyframe_section: None,
            expression_section: None,
            controller,
            wide: false,
            on_keyframes_changed: None,
            on_expression_changed: None,
        }
    }

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
            Some(keyframe_section.upcast_ref()),
            Some(expression_section.upcast_ref()),
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
            Some(keyframe_section.upcast_ref()),
            Some(expression_section.upcast_ref()),
            controller,
            true,
        )
    }

    fn build(
        label: &str,
        editor: &impl IsA<gtk::Widget>,
        keyframe_section: Option<&gtk::Widget>,
        expression_section: Option<&gtk::Widget>,
        controller: LayeredPropertyController,
        wide: bool,
    ) -> Self {
        let row = if wide {
            InspectorPropertyRow::new_wide(label, editor)
        } else {
            InspectorPropertyRow::new(label, editor)
        };
        if let Some(keyframe_section) = keyframe_section {
            row.set_keyframe_section(keyframe_section);
        }
        if let Some(expression_section) = expression_section {
            row.set_expression_section(expression_section);
        }
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

impl InspectorLayeredPropertyBuilder {
    pub fn wide(mut self) -> Self {
        self.wide = true;
        self
    }

    pub fn keyframe_section(mut self, section: &impl IsA<gtk::Widget>) -> Self {
        self.keyframe_section = Some(section.as_ref().clone());
        self
    }

    pub fn expression_section(mut self, section: &impl IsA<gtk::Widget>) -> Self {
        self.expression_section = Some(section.as_ref().clone());
        self
    }

    pub fn on_keyframes_changed(mut self, changed: impl Fn(bool) + 'static) -> Self {
        self.on_keyframes_changed = Some(Box::new(changed));
        self
    }

    pub fn on_expression_changed(mut self, changed: impl Fn(bool) + 'static) -> Self {
        self.on_expression_changed = Some(Box::new(changed));
        self
    }

    pub fn build(self) -> InspectorLayeredProperty {
        let property = InspectorLayeredProperty::build(
            &self.label,
            &self.editor,
            self.keyframe_section.as_ref(),
            self.expression_section.as_ref(),
            self.controller,
            self.wide,
        );
        if let Some(changed) = self.on_keyframes_changed {
            property.connect_keyframes_changed(changed);
        }
        if let Some(changed) = self.on_expression_changed {
            property.connect_expression_changed(changed);
        }
        property
    }
}
