use std::rc::Rc;

use gtk::prelude::*;
pub(super) use shrimply_inspector_core::item::PreviewFocusTarget;
use shrimply_inspector_core::item::{
    HeaderAction as SharedHeaderAction, HeaderButtonToggle as SharedHeaderButtonToggle,
    HeaderToggle as SharedHeaderToggle, InspectorItemPresentation,
};

use super::InspectorContext;
use crate::preview_focus::{PreviewFacetKey, PreviewTarget};

pub(super) type HeaderAction = SharedHeaderAction<Rc<dyn Fn()>>;
pub(super) type HeaderToggle = SharedHeaderToggle<Rc<dyn Fn(bool)>>;
pub(super) type HeaderButtonToggle = SharedHeaderButtonToggle<Rc<dyn Fn(bool)>>;

pub(super) trait InspectorItem {
    fn key(&self) -> &str;
    fn title(&self) -> &str;
    fn controls(&self, context: &InspectorContext) -> Vec<gtk::Widget>;
    fn reset(&self, context: &InspectorContext) -> Rc<dyn Fn()>;
    fn actions(&self) -> &[HeaderAction];
    fn toggle(&self) -> Option<&HeaderToggle>;
    fn button_toggle(&self) -> Option<&HeaderButtonToggle>;
    fn preview_target(&self) -> PreviewFocusTarget;
}

pub(super) enum InspectorListItem {
    Item(Box<dyn InspectorItem>),
    Flat(gtk::Widget),
}

type Controls<T> = dyn Fn(&T, &InspectorContext) -> Vec<gtk::Widget>;
type DefaultValue<T> = dyn Fn(&InspectorContext) -> T;
type Apply<T> = dyn Fn(&InspectorContext, T);

pub(super) struct DefaultInspectorItem<T: 'static> {
    presentation: InspectorItemPresentation,
    value: T,
    controls: Rc<Controls<T>>,
    default_value: Rc<DefaultValue<T>>,
    apply: Rc<Apply<T>>,
    actions: Vec<HeaderAction>,
    toggle: Option<HeaderToggle>,
    button_toggle: Option<HeaderButtonToggle>,
}

impl<T: Default + 'static> DefaultInspectorItem<T> {
    pub(super) fn new(
        key: impl Into<String>,
        title: impl Into<String>,
        value: T,
        controls: impl Fn(&T, &InspectorContext) -> Vec<gtk::Widget> + 'static,
        apply: impl Fn(&InspectorContext, T) + 'static,
    ) -> Self {
        Self::new_with_default(key, title, value, controls, |_| T::default(), apply)
    }
}

impl<T: 'static> DefaultInspectorItem<T> {
    pub(super) fn new_with_default(
        key: impl Into<String>,
        title: impl Into<String>,
        value: T,
        controls: impl Fn(&T, &InspectorContext) -> Vec<gtk::Widget> + 'static,
        default_value: impl Fn(&InspectorContext) -> T + 'static,
        apply: impl Fn(&InspectorContext, T) + 'static,
    ) -> Self {
        Self {
            presentation: InspectorItemPresentation::new(key, title),
            value,
            controls: Rc::new(controls),
            default_value: Rc::new(default_value),
            apply: Rc::new(apply),
            actions: Vec::new(),
            toggle: None,
            button_toggle: None,
        }
    }

    pub(super) fn default_with(
        mut self,
        default_value: impl Fn(&InspectorContext) -> T + 'static,
    ) -> Self {
        self.default_value = Rc::new(default_value);
        self
    }

    pub(super) fn actions(mut self, actions: Vec<HeaderAction>) -> Self {
        self.actions = actions;
        self
    }

    pub(super) fn toggle(mut self, toggle: HeaderToggle) -> Self {
        self.toggle = Some(toggle);
        self
    }

    pub(super) fn button_toggle(mut self, toggle: HeaderButtonToggle) -> Self {
        self.button_toggle = Some(toggle);
        self
    }

    pub(super) fn preview_facet(mut self, facet: PreviewFacetKey) -> Self {
        self.presentation = self.presentation.preview_facet(facet);
        self
    }

    pub(super) fn preview_target(mut self, target: PreviewTarget) -> Self {
        self.presentation = self.presentation.preview_target(target);
        self
    }

    pub(super) fn boxed(self) -> InspectorListItem {
        InspectorListItem::Item(Box::new(self))
    }
}

impl<T: 'static> InspectorItem for DefaultInspectorItem<T> {
    fn key(&self) -> &str {
        &self.presentation.key
    }

    fn title(&self) -> &str {
        &self.presentation.title
    }

    fn controls(&self, context: &InspectorContext) -> Vec<gtk::Widget> {
        (self.controls)(&self.value, context)
    }

    fn reset(&self, context: &InspectorContext) -> Rc<dyn Fn()> {
        let context = context.detached();
        let default_value = self.default_value.clone();
        let apply = self.apply.clone();
        Rc::new(move || apply(&context, default_value(&context)))
    }

    fn actions(&self) -> &[HeaderAction] {
        &self.actions
    }

    fn toggle(&self) -> Option<&HeaderToggle> {
        self.toggle.as_ref()
    }

    fn button_toggle(&self) -> Option<&HeaderButtonToggle> {
        self.button_toggle.as_ref()
    }

    fn preview_target(&self) -> PreviewFocusTarget {
        self.presentation.preview_target
    }
}

pub(super) fn flat(widget: impl IsA<gtk::Widget>) -> InspectorListItem {
    InspectorListItem::Flat(widget.upcast())
}
