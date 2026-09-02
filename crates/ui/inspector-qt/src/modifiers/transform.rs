use shrimply_inspector_core::TransformModifierPresentation;

use crate::section::InspectorSection;

pub(super) fn section(value: &TransformModifierPresentation) -> InspectorSection {
    let mut section = InspectorSection::default();
    section.add(value.position.clone());
    section.add(value.anchor.clone());
    section.add(value.scale.clone());
    section.add(value.shear.clone());
    section.add(value.rotation.clone());
    section
}
