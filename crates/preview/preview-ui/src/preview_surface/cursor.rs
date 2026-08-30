use shrimply_preview_core::Cursor;

pub(super) const fn name(cursor: Cursor) -> &'static str {
    match cursor {
        Cursor::Default => "default",
        Cursor::Pointer => "pointer",
        Cursor::Crosshair => "crosshair",
        Cursor::Move => "move",
        Cursor::Grab => "grab",
        Cursor::Grabbing => "grabbing",
        Cursor::Text => "text",
        Cursor::ResizeHorizontal => "ew-resize",
        Cursor::ResizeVertical => "ns-resize",
        Cursor::ResizeDiagonalDown => "nwse-resize",
        Cursor::ResizeDiagonalUp => "nesw-resize",
        Cursor::Hidden => "none",
    }
}
