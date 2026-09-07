use super::control::Context;
use objc2::rc::Retained;
use objc2_app_kit::{NSTextField, NSView};
use objc2_foundation::{MainThreadMarker, NSString};
use shrimply_components_appkit::{
    ActionButton, SingleLineTextInput, StringChoice, ViewHost, choice_menu,
    column_append_intrinsic, column_stack, row_stack,
};
use shrimply_inspector_core::{
    InspectorControl,
    font_selector::{Browser, FamilyEdit},
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

pub(super) fn view(
    control: &InspectorControl,
    context: &Context,
    mtm: MainThreadMarker,
) -> Retained<NSView> {
    let root = column_stack(6.0, mtm);
    let families: Vec<shrimply_project::project::FontFamily> =
        serde_json::from_str(&control.value).expect("font presentation is valid");
    for (index, family) in families.iter().enumerate() {
        let row = row_stack(4.0, mtm);
        row.addArrangedSubview(&NSTextField::labelWithString(
            &NSString::from_str(family.name()),
            mtm,
        ));
        row.addArrangedSubview(&NSView::new(mtm));
        for (symbol, label, enabled, edit) in [
            (
                "chevron.up",
                "Move font up",
                index > 0,
                FamilyEdit::Move { index, offset: -1 },
            ),
            (
                "chevron.down",
                "Move font down",
                index + 1 < families.len(),
                FamilyEdit::Move { index, offset: 1 },
            ),
            ("trash", "Remove font", true, FamilyEdit::Remove(index)),
        ] {
            let context = context.clone();
            let control = control.clone();
            let button = ActionButton::symbol(
                symbol,
                label,
                move || {
                    context.refresh(context.controller.edit_control_font_family(
                        &context.target,
                        &control,
                        edit.clone(),
                    ))
                },
                mtm,
            );
            button.view().setEnabled(enabled);
            row.addArrangedSubview(button.view());
        }
        column_append_intrinsic(&root, &row);
    }
    let browser = Rc::new(RefCell::new(Browser::default()));
    browser.borrow_mut().open();
    let changed = Rc::new(Cell::new(true));
    let search = SingleLineTextInput::new(
        "",
        Some("Search installed or Google fonts"),
        None,
        {
            let browser = browser.clone();
            let changed = changed.clone();
            move |query| {
                browser.borrow_mut().search(query);
                changed.set(true);
            }
        },
        |_| {},
        mtm,
    );
    column_append_intrinsic(&root, search.view());
    let choices = Rc::new(ViewHost::new(mtm));
    column_append_intrinsic(&root, choices.view());
    let status = NSTextField::labelWithString(&NSString::from_str("Loading fonts…"), mtm);
    column_append_intrinsic(&root, &status);
    let context = context.clone();
    let control = control.clone();
    // The editor's existing poll services the shared asynchronous font browser.
    // Capture only the dirty flag/controller, not the poll registry itself.
    let controller = context.controller.clone();
    let target = context.target.clone();
    let dirty = context.dirty.clone();
    context.polls.borrow_mut().push(Box::new(move || {
        let update = browser.borrow_mut().poll();
        for activated in update.activations {
            let result = activated.and_then(|family| {
                controller.edit_control_font_family(
                    &target,
                    &control,
                    FamilyEdit::Append(shrimply_inspector_core::font_cache::project_family(
                        &family,
                    )),
                )
            });
            if let Err(error) = result {
                status.setStringValue(&NSString::from_str(&error));
            } else {
                dirty.set(true);
            }
        }
        if update.visible_changed || changed.replace(false) {
            let available = browser.borrow().visible().to_vec();
            let menu = choice_menu(
                "Add font",
                "plus",
                available
                    .iter()
                    .enumerate()
                    .map(|(index, family)| StringChoice {
                        value: index.to_string(),
                        label: family.name.clone(),
                    })
                    .collect::<Vec<_>>()
                    .into(),
                {
                    let browser = browser.clone();
                    let status = status.clone();
                    move |index| {
                        let index: usize = index.parse().expect("font menu index");
                        if let Err(error) = browser.borrow_mut().activate(available[index].clone())
                        {
                            status.setStringValue(&NSString::from_str(&error));
                        }
                    }
                },
                mtm,
            );
            choices.set_content(menu.view().into());
        }
        if update.changed {
            status.setStringValue(&NSString::from_str(browser.borrow().status()));
        }
    }));
    root.into_super()
}
