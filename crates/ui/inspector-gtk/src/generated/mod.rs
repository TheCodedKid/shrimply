mod shape;
mod text;

pub(super) fn items(
    item: &shrimply_project::project::VideoItem,
    context: &crate::InspectorContext,
) -> Option<Vec<crate::item::InspectorListItem>> {
    match shrimply_inspector_core::generated::item(item)? {
        shrimply_inspector_core::generated::GeneratedItem::Shape(shape) => {
            Some(shape::shape_items(shape, context))
        }
        shrimply_inspector_core::generated::GeneratedItem::Text(text) => {
            Some(text::text_items(text, context))
        }
    }
}
