use shrimply_inspector_core::OpacityModifierPresentation;

use crate::section::InspectorSection;

pub(super) fn section(value: &OpacityModifierPresentation) -> InspectorSection {
    let mut section = InspectorSection::default();
    section.add(value.opacity.clone());
    section
}
