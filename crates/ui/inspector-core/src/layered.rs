use shrimply_core::timeline_value::{TimelineBase, TimelineValue, TimelineValueType};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LayeredState {
    pub keyframes: bool,
    pub expression: bool,
    pub expression_source: String,
}

impl<T: TimelineValueType> From<&TimelineValue<T>> for LayeredState {
    fn from(value: &TimelineValue<T>) -> Self {
        Self {
            keyframes: matches!(value.base, TimelineBase::Keyframes(_)),
            expression: value
                .expression
                .as_ref()
                .is_some_and(|expression| expression.enabled),
            expression_source: value
                .expression
                .as_ref()
                .map(|expression| expression.source.clone())
                .unwrap_or_default(),
        }
    }
}
