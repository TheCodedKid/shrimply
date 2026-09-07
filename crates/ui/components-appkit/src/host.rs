use objc2::rc::Retained;
use objc2_app_kit::NSView;
use objc2_foundation::MainThreadMarker;
use std::cell::RefCell;

pub struct ViewHost {
    root: Retained<NSView>,
    content: RefCell<Option<Retained<NSView>>>,
}

impl ViewHost {
    pub fn new(mtm: MainThreadMarker) -> Self {
        Self {
            root: NSView::new(mtm),
            content: RefCell::new(None),
        }
    }

    pub fn view(&self) -> &NSView {
        &self.root
    }

    pub fn set_content(&self, content: Retained<NSView>) {
        if let Some(previous) = self.content.replace(Some(content.clone())) {
            previous.removeFromSuperview();
        }
        content.setTranslatesAutoresizingMaskIntoConstraints(false);
        self.root.addSubview(&content);
        for constraint in [
            content
                .leadingAnchor()
                .constraintEqualToAnchor(&self.root.leadingAnchor()),
            content
                .trailingAnchor()
                .constraintEqualToAnchor(&self.root.trailingAnchor()),
            content
                .topAnchor()
                .constraintEqualToAnchor(&self.root.topAnchor()),
            content
                .bottomAnchor()
                .constraintEqualToAnchor(&self.root.bottomAnchor()),
        ] {
            constraint.setActive(true);
        }
    }
}
