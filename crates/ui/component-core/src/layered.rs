use std::{cell::Cell, rc::Rc};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LayeredEdit<T> {
    Base(T),
    Keyframe(T),
}

#[derive(Clone, Default)]
pub struct LayeredPropertyController {
    keyframes: Rc<Cell<bool>>,
    expression: Rc<Cell<bool>>,
}

impl LayeredPropertyController {
    pub fn set_keyframes(&self, enabled: bool) {
        self.keyframes.set(enabled);
    }

    pub fn set_expression(&self, enabled: bool) {
        self.expression.set(enabled);
    }

    pub fn keyframes(&self) -> bool {
        self.keyframes.get()
    }

    pub fn expression(&self) -> bool {
        self.expression.get()
    }

    pub fn edit<T>(&self, value: T) -> LayeredEdit<T> {
        if self.keyframes.get() {
            LayeredEdit::Keyframe(value)
        } else {
            LayeredEdit::Base(value)
        }
    }

    pub fn edit_component<const N: usize>(
        &self,
        value: [f64; N],
        component: usize,
    ) -> LayeredEdit<([f64; N], f64)> {
        self.edit((
            value,
            *value.get(component).expect("layered value component"),
        ))
    }
}
