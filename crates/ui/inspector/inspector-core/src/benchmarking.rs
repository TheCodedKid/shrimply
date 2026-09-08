use crate::{ControlKind, InspectorControl, InspectorSection};

pub fn section() -> InspectorSection {
    InspectorSection {
        controls: vec![
            InspectorControl::new(ControlKind::Performance, "", "Performance").read_only(),
        ],
    }
}
