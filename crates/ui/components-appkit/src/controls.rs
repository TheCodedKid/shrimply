use crate::action;
use objc2::MainThreadOnly;
use objc2::rc::Retained;
use objc2_app_kit::{
    NSBezelStyle, NSButton, NSColor, NSColorWell, NSComboBox, NSControlStateValueOff,
    NSControlStateValueOn, NSFont, NSImage, NSLayoutAttribute, NSProgressIndicator,
    NSProgressIndicatorStyle, NSScrollView, NSSegmentedControl, NSStackView,
    NSStackViewDistribution, NSTextAlignment, NSTextField, NSUserInterfaceLayoutOrientation,
    NSView,
};
use objc2_foundation::{MainThreadMarker, NSPoint, NSRect, NSSize, NSString};
use shrimply_math_color::Color;
use std::rc::Rc;

pub use shrimply_component_core::selector::StringChoice;

const ROW_LABEL_WIDTH: f64 = 150.0;
const CONTROL_HEIGHT: f64 = 28.0;
const ROW_GAP: f64 = 6.0;

pub fn stack(vertical: bool, spacing: f64, mtm: MainThreadMarker) -> Retained<NSStackView> {
    let view = NSStackView::initWithFrame(NSStackView::alloc(mtm), NSRect::ZERO);
    view.setOrientation(if vertical {
        NSUserInterfaceLayoutOrientation::Vertical
    } else {
        NSUserInterfaceLayoutOrientation::Horizontal
    });
    view.setAlignment(if vertical {
        NSLayoutAttribute::Width
    } else {
        NSLayoutAttribute::CenterY
    });
    view.setSpacing(spacing);
    view
}

pub fn control_row(label: &str, child: &NSView, mtm: MainThreadMarker) -> Retained<NSStackView> {
    let row = stack(false, ROW_GAP, mtm);
    let label = NSTextField::labelWithString(&NSString::from_str(label), mtm);
    label.setTextColor(Some(&NSColor::secondaryLabelColor()));
    label.setAlignment(NSTextAlignment::Left);
    label
        .widthAnchor()
        .constraintEqualToConstant(ROW_LABEL_WIDTH)
        .setActive(true);
    row.addArrangedSubview(&label);
    child
        .heightAnchor()
        .constraintGreaterThanOrEqualToConstant(CONTROL_HEIGHT)
        .setActive(true);
    row.addArrangedSubview(child);
    row
}

pub struct StringSelector {
    view: Retained<NSView>,
}

impl StringSelector {
    pub fn new(
        value: &str,
        choices: Vec<StringChoice>,
        on_change: impl Fn(String) + 'static,
        mtm: MainThreadMarker,
    ) -> Self {
        if shrimply_component_core::selector::searchable(choices.len()) {
            let combo = NSComboBox::initWithFrame(NSComboBox::alloc(mtm), NSRect::ZERO);
            combo.setCompletes(true);
            for choice in &choices {
                unsafe {
                    combo.addItemWithObjectValue(&NSString::from_str(&choice.label));
                }
            }
            let selected = shrimply_component_core::selector::selected_index(value, &choices);
            combo.selectItemAtIndex(selected as isize);
            combo.setStringValue(&NSString::from_str(&choices[selected].label));
            let choices = Rc::new(choices);
            action::attach(
                &combo,
                move |control| {
                    let index = control
                        .downcast_ref::<NSComboBox>()
                        .expect("selector sender")
                        .indexOfSelectedItem();
                    if let Ok(index) = usize::try_from(index)
                        && let Some(choice) = choices.get(index)
                    {
                        on_change(choice.value.clone());
                    }
                },
                mtm,
            );
            Self {
                view: combo.into_super().into_super().into_super(),
            }
        } else {
            let popup = objc2_app_kit::NSPopUpButton::initWithFrame_pullsDown(
                objc2_app_kit::NSPopUpButton::alloc(mtm),
                NSRect::ZERO,
                false,
            );
            for choice in &choices {
                popup.addItemWithTitle(&NSString::from_str(&choice.label));
            }
            popup.selectItemAtIndex(shrimply_component_core::selector::selected_index(
                value, &choices,
            ) as isize);
            let choices = Rc::new(choices);
            action::attach(
                &popup,
                move |control| {
                    let index = control
                        .downcast_ref::<objc2_app_kit::NSPopUpButton>()
                        .expect("selector sender")
                        .indexOfSelectedItem();
                    if let Ok(index) = usize::try_from(index)
                        && let Some(choice) = choices.get(index)
                    {
                        on_change(choice.value.clone());
                    }
                },
                mtm,
            );
            Self {
                view: popup.into_super().into_super().into_super(),
            }
        }
    }

    pub fn view(&self) -> &NSView {
        &self.view
    }
}

pub fn switch_row(
    label: &str,
    tooltip: Option<&str>,
    active: bool,
    on_change: impl Fn(bool) + 'static,
    mtm: MainThreadMarker,
) -> Retained<NSStackView> {
    let button = unsafe {
        NSButton::checkboxWithTitle_target_action(&NSString::from_str(label), None, None, mtm)
    };
    button.setState(if active {
        NSControlStateValueOn
    } else {
        NSControlStateValueOff
    });
    button.setToolTip(tooltip.map(NSString::from_str).as_deref());
    action::attach(
        &button,
        move |control| {
            on_change(
                control
                    .downcast_ref::<NSButton>()
                    .expect("switch sender")
                    .state()
                    == NSControlStateValueOn,
            )
        },
        mtm,
    );
    let row = stack(false, 0.0, mtm);
    row.addArrangedSubview(&button);
    row
}

pub struct ColorPicker {
    view: Retained<NSColorWell>,
}

impl ColorPicker {
    pub fn new(
        color: Color<u8>,
        on_change: impl Fn(Color<u8>) + 'static,
        mtm: MainThreadMarker,
    ) -> Self {
        let well = NSColorWell::initWithFrame(NSColorWell::alloc(mtm), NSRect::ZERO);
        well.setColor(&native_color(color));
        action::attach(
            &well,
            move |control| {
                let color = control
                    .downcast_ref::<NSColorWell>()
                    .expect("color sender")
                    .color()
                    .colorUsingColorSpace(&objc2_app_kit::NSColorSpace::sRGBColorSpace())
                    .expect("convert color to sRGB");
                on_change(Color::new(
                    channel(color.redComponent()),
                    channel(color.greenComponent()),
                    channel(color.blueComponent()),
                    channel(color.alphaComponent()),
                ));
            },
            mtm,
        );
        Self { view: well }
    }

    pub fn view(&self) -> &NSColorWell {
        &self.view
    }
}

fn native_color(color: Color<u8>) -> Retained<NSColor> {
    NSColor::colorWithSRGBRed_green_blue_alpha(
        f64::from(color.r) / 255.0,
        f64::from(color.g) / 255.0,
        f64::from(color.b) / 255.0,
        f64::from(color.a) / 255.0,
    )
}

fn channel(value: f64) -> u8 {
    (value * 255.0).round().clamp(0.0, 255.0) as u8
}

pub fn split_button(
    primary: &str,
    secondary: &str,
    on_primary: impl Fn() + 'static,
    on_secondary: impl Fn() + 'static,
    mtm: MainThreadMarker,
) -> Retained<NSStackView> {
    let row = stack(false, 1.0, mtm);
    let primary = unsafe {
        NSButton::buttonWithTitle_target_action(&NSString::from_str(primary), None, None, mtm)
    };
    primary.setBezelStyle(NSBezelStyle::Push);
    action::attach(&primary, move |_| on_primary(), mtm);
    let secondary = unsafe {
        NSButton::buttonWithTitle_target_action(&NSString::from_str(secondary), None, None, mtm)
    };
    secondary.setBezelStyle(NSBezelStyle::Push);
    action::attach(&secondary, move |_| on_secondary(), mtm);
    row.addArrangedSubview(&primary);
    row.addArrangedSubview(&secondary);
    row
}

#[derive(Clone, Copy)]
pub enum ProgressButtonState {
    Idle,
    Indeterminate,
    Progress(f64),
}

pub struct ProgressButton {
    root: Retained<NSStackView>,
    indicator: Retained<NSProgressIndicator>,
}

impl ProgressButton {
    pub fn new(label: &str, mtm: MainThreadMarker) -> Self {
        let root = stack(false, 6.0, mtm);
        let indicator =
            NSProgressIndicator::initWithFrame(NSProgressIndicator::alloc(mtm), NSRect::ZERO);
        indicator.setStyle(NSProgressIndicatorStyle::Spinning);
        indicator.setDisplayedWhenStopped(false);
        indicator.setHidden(true);
        indicator
            .widthAnchor()
            .constraintEqualToConstant(16.0)
            .setActive(true);
        indicator
            .heightAnchor()
            .constraintEqualToConstant(16.0)
            .setActive(true);
        let button = unsafe {
            NSButton::buttonWithTitle_target_action(&NSString::from_str(label), None, None, mtm)
        };
        button.setEnabled(false);
        root.addArrangedSubview(&indicator);
        root.addArrangedSubview(&button);
        Self { root, indicator }
    }

    pub fn view(&self) -> &NSStackView {
        &self.root
    }

    pub fn set_state(&self, state: ProgressButtonState) {
        match state {
            ProgressButtonState::Idle => {
                unsafe {
                    self.indicator.stopAnimation(None);
                }
                self.indicator.setHidden(true);
            }
            ProgressButtonState::Indeterminate => {
                self.indicator.setIndeterminate(true);
                self.indicator.setHidden(false);
                unsafe {
                    self.indicator.startAnimation(None);
                }
            }
            ProgressButtonState::Progress(value) => {
                unsafe {
                    self.indicator.stopAnimation(None);
                }
                self.indicator.setStyle(NSProgressIndicatorStyle::Bar);
                self.indicator.setIndeterminate(false);
                self.indicator.setMinValue(0.0);
                self.indicator.setMaxValue(1.0);
                self.indicator.setDoubleValue(value.clamp(0.0, 1.0));
                self.indicator.setHidden(false);
                self.indicator
                    .widthAnchor()
                    .constraintEqualToConstant(48.0)
                    .setActive(true);
            }
        }
    }
}

pub struct ReadOnlyField {
    root: Retained<NSStackView>,
}

impl ReadOnlyField {
    pub fn new(value: &str, right_aligned: bool, mtm: MainThreadMarker) -> Self {
        let root = stack(false, 4.0, mtm);
        let field = NSTextField::labelWithString(&NSString::from_str(value), mtm);
        field.setSelectable(true);
        field.setAlignment(if right_aligned {
            NSTextAlignment::Right
        } else {
            NSTextAlignment::Left
        });
        root.addArrangedSubview(&field);
        Self { root }
    }

    pub fn with_action(
        value: &str,
        action_label: &str,
        on_action: impl Fn() + 'static,
        mtm: MainThreadMarker,
    ) -> Self {
        let this = Self::new(value, true, mtm);
        let button = unsafe {
            NSButton::buttonWithImage_target_action(
                &symbol("folder", action_label),
                None,
                None,
                mtm,
            )
        };
        button.setToolTip(Some(&NSString::from_str(action_label)));
        action::attach(&button, move |_| on_action(), mtm);
        this.root.addArrangedSubview(&button);
        this
    }

    pub fn view(&self) -> &NSStackView {
        &self.root
    }
}

pub struct Tabs {
    root: Retained<NSStackView>,
}

impl Tabs {
    pub fn new(pages: Vec<(&str, Retained<NSView>)>, mtm: MainThreadMarker) -> Self {
        assert!(!pages.is_empty(), "tabs need at least one page");
        let root = stack(true, 8.0, mtm);
        let selector = unsafe {
            NSSegmentedControl::segmentedControlWithLabels_trackingMode_target_action(
                &objc2_foundation::NSArray::from_retained_slice(
                    &pages
                        .iter()
                        .map(|(label, _)| NSString::from_str(label))
                        .collect::<Vec<_>>(),
                ),
                objc2_app_kit::NSSegmentSwitchTracking::SelectOne,
                None,
                None,
                mtm,
            )
        };
        selector.setSelectedSegment(0);
        let content = NSView::initWithFrame(NSView::alloc(mtm), NSRect::ZERO);
        let page_views = Rc::new(pages.into_iter().map(|(_, view)| view).collect::<Vec<_>>());
        for (index, page) in page_views.iter().enumerate() {
            page.setTranslatesAutoresizingMaskIntoConstraints(false);
            page.setHidden(index != 0);
            content.addSubview(page);
            for constraint in [
                page.leadingAnchor()
                    .constraintEqualToAnchor(&content.leadingAnchor()),
                page.trailingAnchor()
                    .constraintEqualToAnchor(&content.trailingAnchor()),
                page.topAnchor()
                    .constraintEqualToAnchor(&content.topAnchor()),
                page.bottomAnchor()
                    .constraintEqualToAnchor(&content.bottomAnchor()),
            ] {
                constraint.setActive(true);
            }
        }
        let callback_pages = page_views.clone();
        action::attach(
            &selector,
            move |control| {
                let selected = control
                    .downcast_ref::<NSSegmentedControl>()
                    .expect("tabs sender")
                    .selectedSegment();
                for (index, page) in callback_pages.iter().enumerate() {
                    page.setHidden(index as isize != selected);
                }
            },
            mtm,
        );
        root.addArrangedSubview(&selector);
        root.addArrangedSubview(&content);
        root.setDistribution(NSStackViewDistribution::Fill);
        Self { root }
    }

    pub fn view(&self) -> &NSStackView {
        &self.root
    }
}

pub fn playback_shortcuts(
    on_toggle: impl Fn() + 'static,
    on_speed: impl Fn() + 'static,
    mtm: MainThreadMarker,
) -> Retained<NSStackView> {
    let row = stack(false, 6.0, mtm);
    let play = unsafe {
        NSButton::buttonWithTitle_target_action(
            &NSString::from_str("Play / Pause (Space)"),
            None,
            None,
            mtm,
        )
    };
    play.setKeyEquivalent(&NSString::from_str(" "));
    action::attach(&play, move |_| on_toggle(), mtm);
    let speed = unsafe {
        NSButton::buttonWithTitle_target_action(&NSString::from_str("Speed (L)"), None, None, mtm)
    };
    speed.setKeyEquivalent(&NSString::from_str("l"));
    action::attach(&speed, move |_| on_speed(), mtm);
    row.addArrangedSubview(&play);
    row.addArrangedSubview(&speed);
    row
}

pub fn modifier_menu(
    choices: Vec<StringChoice>,
    on_change: impl Fn(String) + 'static,
    mtm: MainThreadMarker,
) -> StringSelector {
    StringSelector::new("", choices, on_change, mtm)
}

pub fn live_performance(mtm: MainThreadMarker) -> Retained<NSScrollView> {
    let scroll = NSScrollView::initWithFrame(
        NSScrollView::alloc(mtm),
        NSRect::new(NSPoint::ZERO, NSSize::new(400.0, 100.0)),
    );
    scroll.setHasVerticalScroller(true);
    let rows = shrimply_component_core::performance::rows();
    let content = stack(true, 2.0, mtm);
    if rows.is_empty() {
        content.addArrangedSubview(&NSTextField::labelWithString(
            &NSString::from_str("No performance samples"),
            mtm,
        ));
    } else {
        for row in rows {
            let label = NSTextField::labelWithString(
                &NSString::from_str(&format!("{} · {}", row.title, row.subtitle)),
                mtm,
            );
            label.setFont(Some(&NSFont::monospacedSystemFontOfSize_weight(
                NSFont::smallSystemFontSize(),
                unsafe { objc2_app_kit::NSFontWeightRegular },
            )));
            content.addArrangedSubview(&label);
        }
    }
    scroll.setDocumentView(Some(&content));
    scroll
}

fn symbol(name: &str, label: &str) -> Retained<NSImage> {
    NSImage::imageWithSystemSymbolName_accessibilityDescription(
        &NSString::from_str(name),
        Some(&NSString::from_str(label)),
    )
    .unwrap_or_else(|| panic!("macOS must provide the {name} system symbol"))
}
