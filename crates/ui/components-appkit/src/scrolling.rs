use objc2::rc::Retained;
use objc2::{MainThreadOnly, define_class, msg_send};
use objc2_app_kit::{NSScrollView, NSView};
use objc2_foundation::{MainThreadMarker, NSObjectProtocol, NSRect};

struct ScrollingDocumentIvars;

define_class!(
    #[unsafe(super(NSView))]
    #[thread_kind = MainThreadOnly]
    #[ivars = ScrollingDocumentIvars]
    struct ScrollingDocument;

    unsafe impl NSObjectProtocol for ScrollingDocument {}

    impl ScrollingDocument {
        #[unsafe(method(isFlipped))]
        fn is_flipped(&self) -> bool { true }
    }
);

#[derive(Clone)]
pub struct ScrollingColumn {
    scroll: Retained<NSScrollView>,
}

impl ScrollingColumn {
    pub fn new(content: &NSView, mtm: MainThreadMarker) -> Self {
        let scroll = NSScrollView::initWithFrame(NSScrollView::alloc(mtm), NSRect::ZERO);
        scroll.setHasVerticalScroller(true);
        scroll.setHasHorizontalScroller(false);
        scroll.setAutohidesScrollers(true);
        let document = ScrollingDocument::alloc(mtm).set_ivars(ScrollingDocumentIvars);
        let document: Retained<ScrollingDocument> =
            unsafe { msg_send![super(document), initWithFrame: NSRect::ZERO] };
        content.setTranslatesAutoresizingMaskIntoConstraints(false);
        document.setTranslatesAutoresizingMaskIntoConstraints(false);
        document.addSubview(content);
        scroll.setDocumentView(Some(&document));
        for constraint in [
            document
                .leadingAnchor()
                .constraintEqualToAnchor(&scroll.contentView().leadingAnchor()),
            document
                .trailingAnchor()
                .constraintEqualToAnchor(&scroll.contentView().trailingAnchor()),
            document
                .topAnchor()
                .constraintEqualToAnchor(&scroll.contentView().topAnchor()),
            content
                .leadingAnchor()
                .constraintEqualToAnchor(&document.leadingAnchor()),
            content
                .trailingAnchor()
                .constraintEqualToAnchor(&document.trailingAnchor()),
            content
                .topAnchor()
                .constraintEqualToAnchor(&document.topAnchor()),
            content
                .bottomAnchor()
                .constraintEqualToAnchor(&document.bottomAnchor()),
        ] {
            constraint.setActive(true);
        }
        // Anchor the document's origin and width, as in Apple's scrollable-stack
        // sample. Its bottom is unconstrained: content alone determines height.
        Self { scroll }
    }

    pub fn view(&self) -> &NSScrollView {
        &self.scroll
    }

    pub fn position(&self) -> f64 {
        self.scroll.contentView().bounds().origin.y
    }

    pub fn set_position(&self, position: f64) {
        self.scroll.layoutSubtreeIfNeeded();
        let clip = self.scroll.contentView();
        let mut bounds = clip.bounds();
        bounds.origin.y = position;
        clip.scrollToPoint(clip.constrainBoundsRect(bounds).origin);
        self.scroll.reflectScrolledClipView(&clip);
    }
}
