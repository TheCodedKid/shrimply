//! Opt-in diagnostics only; never edits the project or interacts with other apps.
use objc2_app_kit::{NSLayoutConstraintOrientation, NSView};

pub(super) fn expanded_cards(mtm: objc2_foundation::MainThreadMarker) {
    use objc2::MainThreadOnly;
    use objc2_app_kit::{
        NSBackingStoreType, NSLayoutPriorityRequired, NSTextField, NSWindow, NSWindowStyleMask,
    };
    use objc2_foundation::{NSRect, NSSize, NSString};
    use shrimply_components_appkit::{InspectorCard, ScrollingColumn, column_append, column_stack};

    let weak_children = objc2::rc::autoreleasepool(|_| {
        let pair = shrimply_components_appkit::Number2Picker::builder(1.0, 2.0)
            .enable_lock()
            .build_with_handles(mtm);
        let triple = shrimply_components_appkit::Number3Picker::builder([1.0, 2.0, 3.0])
            .enable_lock()
            .build_with_handles(mtm);
        [pair.widget.subviews(), triple.widget.subviews()]
            .into_iter()
            .flat_map(|views| {
                views
                    .iter()
                    .map(|view| objc2::rc::Weak::new(&*view))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    });
    eprintln!(
        "Vector-ownership diagnostic retained-children={}",
        weak_children
            .iter()
            .filter(|view| view.load().is_some())
            .count()
    );

    let column = column_stack(10.0, mtm);
    column.setHuggingPriority_forOrientation(
        NSLayoutPriorityRequired,
        NSLayoutConstraintOrientation::Vertical,
    );
    let cards = [
        InspectorCard::without_reset("First expanded card", true, mtm),
        InspectorCard::without_reset("Second expanded card", true, mtm),
    ];
    for card in &cards {
        for _ in 0..5 {
            let row =
                NSTextField::labelWithString(&NSString::from_str("Natural-height content"), mtm);
            card.append(&row);
        }
        column_append(&column, card.view());
    }
    let scroll = ScrollingColumn::new(&column, mtm);
    // A separate, never-shown window gives AppKit a real layout root without
    // resizing the user's editor or changing the selected inspector document.
    let window = unsafe {
        NSWindow::initWithContentRect_styleMask_backing_defer(
            NSWindow::alloc(mtm),
            NSRect::ZERO,
            NSWindowStyleMask::Borderless,
            NSBackingStoreType::Buffered,
            false,
        )
    };
    unsafe {
        window.setReleasedWhenClosed(false);
    }
    window.setContentView(Some(scroll.view()));
    for height in [120.0, 1000.0, 120.0] {
        window.setContentSize(NSSize::new(400.0, height));
        scroll.view().layoutSubtreeIfNeeded();
        let document = scroll.view().documentView().expect("scroll document");
        eprintln!(
            "Expanded-card diagnostic viewport={:?} document={:?} fitting={:?}",
            scroll.view().contentView().bounds().size,
            document.frame().size,
            document.fittingSize()
        );
        for card in &cards {
            eprintln!(
                "Expanded-card diagnostic card={:?} fitting={:?} ambiguous={}",
                card.view().frame().size,
                card.view().fittingSize(),
                card.view().hasAmbiguousLayout()
            );
        }
        scroll.set_position(document.frame().size.height);
        eprintln!(
            "Expanded-card diagnostic scroll-bottom={}",
            scroll.position()
        );
    }
    // Compare actual editor columns, not just the enclosing card's dimensions.
    use shrimply_components_appkit::{
        ActionButton, Number2Picker, NumberPicker, StringChoice, StringSelector,
        control_row_with_suffix, row_stack,
    };
    let selectors = StringSelector::new(
        "Bicubic",
        vec![StringChoice {
            value: "Bicubic".into(),
            label: "Bicubic".into(),
        }],
        |_| {},
        mtm,
    );
    let editors = [
        (
            "Position",
            Number2Picker::builder(1067.0, 540.0)
                .build_with_handles(mtm)
                .widget,
        ),
        (
            "Anchor",
            Number2Picker::builder(688.0, 352.0)
                .build_with_handles(mtm)
                .widget,
        ),
        ("Opacity", NumberPicker::builder(100.0).build(mtm)),
        ("Upsampling", selectors.view().into()),
    ];
    let mut rows = Vec::new();
    for (label, editor) in editors {
        let suffix = row_stack(6.0, mtm);
        let keyframe = ActionButton::symbol("stopwatch", "Keyframes", || {}, mtm);
        let expression = ActionButton::symbol(
            "chevron.left.forwardslash.chevron.right",
            "Expression",
            || {},
            mtm,
        );
        suffix.addArrangedSubview(keyframe.view());
        suffix.addArrangedSubview(expression.view());
        let row = control_row_with_suffix(label, &editor, Some(&suffix), mtm);
        cards[0].append(&row);
        rows.push((label, row, editor, suffix));
    }
    for width in [296.0, 800.0, 296.0] {
        window.setContentSize(NSSize::new(width, 120.0));
        scroll.view().layoutSubtreeIfNeeded();
        for (label, row, editor, suffix) in &rows {
            let slot = unsafe { editor.superview() }.expect("editor slot");
            eprintln!(
                "Row-alignment diagnostic viewport={width} label={label} row={:?} editor-leading={} editor-width={} suffix={:?} ambiguous={}",
                row.frame(),
                slot.frame().origin.x + editor.frame().origin.x,
                editor.frame().size.width,
                suffix.frame(),
                row.hasAmbiguousLayout(),
            );
            dump(row, 0);
        }
    }
}

pub(super) fn dump(view: &NSView, depth: usize) {
    if view.isHiddenOrHasHiddenAncestor() {
        return;
    }
    let ambiguous = view.hasAmbiguousLayout();
    eprintln!(
        "Inspector layout depth={depth} class={:?} frame={:?} fitting={:?} ambiguous={ambiguous}",
        view.class().name(),
        view.frame(),
        view.fittingSize()
    );
    if ambiguous {
        for orientation in [
            NSLayoutConstraintOrientation::Horizontal,
            NSLayoutConstraintOrientation::Vertical,
        ] {
            eprintln!(
                "Inspector constraints depth={depth} orientation={orientation:?}: {:?}",
                view.constraintsAffectingLayoutForOrientation(orientation)
            );
        }
    }
    for child in view.subviews().iter() {
        dump(&child, depth + 1);
    }
}
