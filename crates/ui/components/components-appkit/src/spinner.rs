use objc2::rc::Retained;
use objc2_app_kit::{NSControlSize, NSProgressIndicator, NSProgressIndicatorStyle};
use objc2_foundation::MainThreadMarker;

pub struct Spinner {
    view: Retained<NSProgressIndicator>,
}

impl Spinner {
    pub fn new(mtm: MainThreadMarker) -> Self {
        let view = NSProgressIndicator::new(mtm);
        view.setStyle(NSProgressIndicatorStyle::Spinning);
        view.setControlSize(NSControlSize::Small);
        view.setIndeterminate(true);
        view.setDisplayedWhenStopped(false);
        view.sizeToFit();
        view.widthAnchor()
            .constraintEqualToConstant(view.frame().size.width)
            .setActive(true);
        view.heightAnchor()
            .constraintEqualToConstant(view.frame().size.height)
            .setActive(true);
        Self { view }
    }

    pub fn view(&self) -> &NSProgressIndicator {
        &self.view
    }

    pub fn set_active(&self, active: bool) {
        unsafe {
            if active {
                self.view.startAnimation(None);
            } else {
                self.view.stopAnimation(None);
            }
        }
    }
}
