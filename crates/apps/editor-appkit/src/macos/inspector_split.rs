use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{ClassType, MainThreadOnly, define_class, msg_send};
use objc2_app_kit::{NSSplitView, NSSplitViewDelegate, NSSplitViewDividerStyle, NSView};
use objc2_foundation::{
    MainThreadMarker, NSNotification, NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize,
};

const INSPECTOR_MIN_WIDTH: f64 = 320.0;
const PREVIEW_MIN_WIDTH: f64 = 480.0;

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    struct DividerDelegate;

    unsafe impl NSObjectProtocol for DividerDelegate {}

    unsafe impl NSSplitViewDelegate for DividerDelegate {
        #[unsafe(method(splitViewDidResizeSubviews:))]
        fn resized(&self, notification: &NSNotification) {
            if std::env::var_os("SHRIMPLY_DEBUG_INSPECTOR_LAYOUT").is_some()
                && let Some(object) = notification.object()
                && let Some(split) = object.downcast_ref::<NSSplitView>()
            {
                eprintln!(
                    "Inspector split: bounds={:?}, panes={:?}",
                    split.bounds(),
                    split
                        .subviews()
                        .iter()
                        .map(|view| view.frame())
                        .collect::<Vec<_>>()
                );
            }
        }

        #[unsafe(method(splitView:constrainMinCoordinate:ofSubviewAt:))]
        fn minimum(&self, _split: &NSSplitView, _proposed: f64, _index: isize) -> f64 {
            INSPECTOR_MIN_WIDTH
        }

        #[unsafe(method(splitView:constrainMaxCoordinate:ofSubviewAt:))]
        fn maximum(&self, split: &NSSplitView, _proposed: f64, _index: isize) -> f64 {
            split.bounds().size.width - split.dividerThickness() - PREVIEW_MIN_WIDTH
        }
    }
);

/// AppKit owns pane frames and the divider position. Inspector content does not
/// participate in the split's Auto Layout sizing or maintain a second width.
pub struct InspectorSplit {
    split: Retained<NSSplitView>,
    inspector: Retained<NSView>,
    _delegate: Retained<DividerDelegate>,
}

impl InspectorSplit {
    pub fn new(inspector: &NSView, preview: &NSView, size: NSSize, mtm: MainThreadMarker) -> Self {
        let split =
            NSSplitView::initWithFrame(NSSplitView::alloc(mtm), NSRect::new(NSPoint::ZERO, size));
        split.setVertical(true);
        split.setDividerStyle(NSSplitViewDividerStyle::Thin);
        let delegate: Retained<DividerDelegate> =
            unsafe { msg_send![DividerDelegate::alloc(mtm), init] };
        split.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
        let inspector = pane(inspector, mtm);
        let preview = pane(preview, mtm);
        inspector.setFrame(NSRect::new(
            NSPoint::ZERO,
            NSSize::new(INSPECTOR_MIN_WIDTH, size.height),
        ));
        let preview_x = INSPECTOR_MIN_WIDTH + split.dividerThickness();
        preview.setFrame(NSRect::new(
            NSPoint::new(preview_x, 0.0),
            NSSize::new(size.width - preview_x, size.height),
        ));
        split.addSubview(&inspector);
        split.addSubview(&preview);
        Self {
            split,
            inspector,
            _delegate: delegate,
        }
    }

    pub fn view(&self) -> &NSView {
        self.split.as_super()
    }

    pub fn set_collapsed(&self, collapsed: bool) {
        if self.inspector.isHidden() != collapsed {
            self.inspector.setHidden(collapsed);
            self.split.adjustSubviews();
        }
    }
}

fn pane(content: &NSView, mtm: MainThreadMarker) -> Retained<NSView> {
    let pane = NSView::new(mtm);
    content.setTranslatesAutoresizingMaskIntoConstraints(false);
    pane.addSubview(content);
    for constraint in [
        content
            .leadingAnchor()
            .constraintEqualToAnchor(&pane.leadingAnchor()),
        content
            .trailingAnchor()
            .constraintEqualToAnchor(&pane.trailingAnchor()),
        content
            .topAnchor()
            .constraintEqualToAnchor(&pane.topAnchor()),
        content
            .bottomAnchor()
            .constraintEqualToAnchor(&pane.bottomAnchor()),
    ] {
        constraint.setActive(true);
    }
    pane
}
