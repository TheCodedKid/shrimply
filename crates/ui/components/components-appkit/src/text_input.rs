use objc2::ffi::{OBJC_ASSOCIATION_RETAIN_NONATOMIC, objc_setAssociatedObject};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{DefinedClass, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSColor, NSControlTextEditingDelegate, NSFont, NSForegroundColorAttributeName, NSScrollView,
    NSTextDelegate, NSTextField, NSTextFieldDelegate, NSTextView, NSTextViewDelegate,
};
use objc2_foundation::{
    MainThreadMarker, NSNotification, NSObject, NSObjectProtocol, NSRange, NSRect, NSSize, NSString,
};
use std::cell::{Cell, RefCell};
use std::ffi::c_void;
use std::rc::Rc;

static SINGLE_TARGET_KEY: u8 = 0;
static MULTILINE_TARGET_KEY: u8 = 0;

struct SingleTargetIvars {
    commit: RefCell<shrimply_components_core::text::TextCommit>,
    max_length: Option<usize>,
    on_change: Rc<dyn Fn(String)>,
    on_commit: Rc<dyn Fn(String)>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = SingleTargetIvars]
    struct SingleTarget;

    unsafe impl NSObjectProtocol for SingleTarget {}
    unsafe impl NSTextFieldDelegate for SingleTarget {}
    unsafe impl NSControlTextEditingDelegate for SingleTarget {
        #[unsafe(method(controlTextDidChange:))]
        fn changed(&self, notification: &NSNotification) {
            let field = notification.object().expect("text notification sender")
                .downcast::<NSTextField>().expect("text notification must contain a text field");
            let entered = field.stringValue().to_string();
            let text = shrimply_components_core::text::limited_text(&entered, self.ivars().max_length);
            if text != entered { field.setStringValue(&NSString::from_str(&text)); }
            self.ivars().commit.borrow_mut().changed(text.clone());
            (self.ivars().on_change)(text);
        }

        #[unsafe(method(controlTextDidEndEditing:))]
        fn ended(&self, notification: &NSNotification) {
            let field = notification.object().expect("text notification sender")
                .downcast::<NSTextField>().expect("text notification must contain a text field");
            self.commit(&field);
        }
    }

    impl SingleTarget {
        #[unsafe(method(commit:))]
        fn commit_action(&self, sender: &NSTextField) { self.commit(sender); }
    }
);

impl SingleTarget {
    fn commit(&self, field: &NSTextField) {
        if self.ivars().commit.borrow_mut().take_commit() {
            (self.ivars().on_commit)(field.stringValue().to_string());
        }
    }
}

pub struct SingleLineTextInput {
    root: Retained<objc2_app_kit::NSView>,
    _field: Retained<NSTextField>,
}

impl SingleLineTextInput {
    pub fn new(
        value: &str,
        placeholder: Option<&str>,
        max_length: Option<usize>,
        on_change: impl Fn(String) + 'static,
        on_commit: impl Fn(String) + 'static,
        mtm: MainThreadMarker,
    ) -> Self {
        let field = NSTextField::initWithFrame(NSTextField::alloc(mtm), NSRect::ZERO);
        field.setStringValue(&NSString::from_str(
            &shrimply_components_core::text::limited_text(value, max_length),
        ));
        field.setPlaceholderString(placeholder.map(NSString::from_str).as_deref());
        let target = SingleTarget::alloc(mtm).set_ivars(SingleTargetIvars {
            commit: RefCell::new(shrimply_components_core::text::TextCommit::new(value)),
            max_length,
            on_change: Rc::new(on_change),
            on_commit: Rc::new(on_commit),
        });
        let target: Retained<SingleTarget> = unsafe { msg_send![super(target), init] };
        unsafe {
            field.setTarget(Some(&target));
            field.setAction(Some(sel!(commit:)));
            field.setDelegate(Some(ProtocolObject::from_ref(&*target)));
            retain_target(&*field, &target, &SINGLE_TARGET_KEY);
        }
        let root = objc2_app_kit::NSView::new(mtm);
        field.setTranslatesAutoresizingMaskIntoConstraints(false);
        root.addSubview(&field);
        for constraint in [
            field
                .leadingAnchor()
                .constraintEqualToAnchor(&root.leadingAnchor()),
            field
                .trailingAnchor()
                .constraintEqualToAnchor(&root.trailingAnchor()),
            field
                .centerYAnchor()
                .constraintEqualToAnchor(&root.centerYAnchor()),
            field
                .heightAnchor()
                .constraintEqualToConstant(field.intrinsicContentSize().height),
        ] {
            constraint.setActive(true);
        }
        Self {
            root,
            _field: field,
        }
    }

    pub fn view(&self) -> &objc2_app_kit::NSView {
        &self.root
    }
}

struct MultilineTargetIvars {
    commit: RefCell<shrimply_components_core::text::TextCommit>,
    syncing: Cell<bool>,
    max_length: Option<usize>,
    on_change: Rc<dyn Fn(String) -> bool>,
    on_commit: Rc<dyn Fn()>,
    syntax_highlighting: bool,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = MultilineTargetIvars]
    struct MultilineTarget;

    unsafe impl NSObjectProtocol for MultilineTarget {}
    unsafe impl NSTextDelegate for MultilineTarget {
        #[unsafe(method(textDidChange:))]
        fn changed(&self, notification: &NSNotification) {
            if self.ivars().syncing.get() {
                return;
            }
            let view = notification
                .object()
                .expect("text notification sender")
                .downcast::<NSTextView>()
                .expect("text notification must contain a text view");
            let entered = view.string().to_string();
            let text =
                shrimply_components_core::text::limited_text(&entered, self.ivars().max_length);
            if text != entered {
                self.ivars().syncing.set(true);
                view.setString(&NSString::from_str(&text));
                self.ivars().syncing.set(false);
            }
            let marks = shrimply_components_core::text::typo_marks(&text);
            view.setToolTip(
                (!marks.is_empty())
                    .then(|| {
                        NSString::from_str(
                            &marks
                                .into_iter()
                                .map(|mark| mark.message)
                                .collect::<Vec<_>>()
                                .join("\n"),
                        )
                    })
                    .as_deref(),
            );
            if (self.ivars().on_change)(text.clone()) {
                self.ivars().commit.borrow_mut().changed(text);
            }
            if self.ivars().syntax_highlighting {
                highlight_expression(&view, &entered);
            }
        }

        #[unsafe(method(textDidEndEditing:))]
        fn ended(&self, _notification: &NSNotification) {
            self.commit();
        }
    }
    unsafe impl NSTextViewDelegate for MultilineTarget {
        #[unsafe(method(textView:doCommandBySelector:))]
        unsafe fn do_command(
            &self,
            view: &NSTextView,
            command: objc2::runtime::Sel,
        ) -> objc2::runtime::Bool {
            if !self.ivars().syntax_highlighting {
                return false.into();
            }
            if command == sel!(insertTab:) {
                replace_selection(view, "    ");
                return true.into();
            }
            if command == sel!(insertNewline:) {
                let source = view.string().to_string();
                let indent = current_line_indent(&source, view.selectedRange().location);
                replace_selection(view, &format!("\n{indent}"));
                return true.into();
            }
            if command == sel!(deleteBackward:) && smart_backspace(view) {
                return true.into();
            }
            if command == sel!(moveToBeginningOfLine:) {
                move_to_smart_line_edge(view, false);
                return true.into();
            }
            if command == sel!(moveToEndOfLine:) {
                move_to_smart_line_edge(view, true);
                return true.into();
            }
            false.into()
        }

        #[unsafe(method(textViewDidChangeSelection:))]
        fn selection_changed(&self, notification: &NSNotification) {
            if !self.ivars().syntax_highlighting {
                return;
            }
            let view = notification
                .object()
                .expect("selection notification sender")
                .downcast::<NSTextView>()
                .expect("selection notification must contain a text view");
            highlight_expression(&view, &view.string().to_string());
            show_matching_bracket(&view);
        }
    }
);

impl MultilineTarget {
    fn commit(&self) {
        if self.ivars().commit.borrow_mut().take_commit() {
            (self.ivars().on_commit)();
        }
    }
}

pub struct MultilineTextInput {
    scroll: Retained<NSScrollView>,
    view: Retained<NSTextView>,
}

impl MultilineTextInput {
    pub fn new(
        value: &str,
        min_content_height: f64,
        max_length: Option<usize>,
        on_change: impl Fn(String) -> bool + 'static,
        on_commit: impl Fn() + 'static,
        mtm: MainThreadMarker,
    ) -> Self {
        Self::build(
            value,
            min_content_height,
            max_length,
            false,
            on_change,
            on_commit,
            mtm,
        )
    }

    pub fn code(
        value: &str,
        min_content_height: f64,
        on_change: impl Fn(String) -> bool + 'static,
        on_commit: impl Fn() + 'static,
        mtm: MainThreadMarker,
    ) -> Self {
        Self::build(
            value,
            min_content_height,
            None,
            true,
            on_change,
            on_commit,
            mtm,
        )
    }

    fn build(
        value: &str,
        min_content_height: f64,
        max_length: Option<usize>,
        syntax_highlighting: bool,
        on_change: impl Fn(String) -> bool + 'static,
        on_commit: impl Fn() + 'static,
        mtm: MainThreadMarker,
    ) -> Self {
        let view = NSTextView::initWithFrame(
            NSTextView::alloc(mtm),
            NSRect::new(
                objc2_foundation::NSPoint::ZERO,
                NSSize::new(400.0, min_content_height),
            ),
        );
        view.setString(&NSString::from_str(
            &shrimply_components_core::text::limited_text(value, max_length),
        ));
        view.setRichText(false);
        view.setContinuousSpellCheckingEnabled(true);
        if syntax_highlighting {
            view.setFont(Some(&NSFont::monospacedSystemFontOfSize_weight(
                NSFont::systemFontSize(),
                unsafe { objc2_app_kit::NSFontWeightRegular },
            )));
            view.setContinuousSpellCheckingEnabled(false);
            view.setAutomaticQuoteSubstitutionEnabled(false);
            view.setAutomaticDashSubstitutionEnabled(false);
            view.setAutomaticTextReplacementEnabled(false);
            view.setUsesFindPanel(true);
            view.setHorizontallyResizable(true);
            view.setVerticallyResizable(true);
            view.setMaxSize(NSSize::new(f64::MAX, f64::MAX));
            let container =
                unsafe { view.textContainer() }.expect("code editor must have a text container");
            container.setWidthTracksTextView(false);
            container.setContainerSize(NSSize::new(f64::MAX, f64::MAX));
        }
        let target = MultilineTarget::alloc(mtm).set_ivars(MultilineTargetIvars {
            commit: RefCell::new(shrimply_components_core::text::TextCommit::new(value)),
            syncing: Cell::new(false),
            max_length,
            on_change: Rc::new(on_change),
            on_commit: Rc::new(on_commit),
            syntax_highlighting,
        });
        let target: Retained<MultilineTarget> = unsafe { msg_send![super(target), init] };
        unsafe {
            view.setDelegate(Some(ProtocolObject::from_ref(&*target)));
            retain_target(&*view, &target, &MULTILINE_TARGET_KEY);
        }
        let scroll = NSScrollView::initWithFrame(
            NSScrollView::alloc(mtm),
            NSRect::new(
                objc2_foundation::NSPoint::ZERO,
                NSSize::new(400.0, min_content_height),
            ),
        );
        scroll.setHasVerticalScroller(true);
        scroll.setHasHorizontalScroller(syntax_highlighting);
        scroll.setBorderType(objc2_app_kit::NSBorderType::BezelBorder);
        scroll.setDocumentView(Some(&view));
        scroll
            .heightAnchor()
            .constraintEqualToConstant(min_content_height)
            .setActive(true);
        if syntax_highlighting {
            highlight_expression(&view, value);
        }
        Self { scroll, view }
    }

    pub fn view(&self) -> &NSScrollView {
        &self.scroll
    }

    pub fn set_text(&self, text: &str) {
        self.view.setString(&NSString::from_str(text));
    }
}

fn highlight_expression(view: &NSTextView, source: &str) {
    let storage = unsafe { view.textStorage() }.expect("text view must have text storage");
    let full_range = NSRange::new(0, source.encode_utf16().count());
    storage.beginEditing();
    unsafe {
        storage.addAttribute_value_range(
            NSForegroundColorAttributeName,
            &NSColor::textColor(),
            full_range,
        );
        for span in shrimply_components_core::syntax::expression_spans(source) {
            let color = match span.kind {
                shrimply_components_core::syntax::SyntaxKind::Keyword => NSColor::systemBlueColor(),
                shrimply_components_core::syntax::SyntaxKind::Boolean => {
                    NSColor::systemOrangeColor()
                }
                shrimply_components_core::syntax::SyntaxKind::Function => {
                    NSColor::systemTealColor()
                }
                shrimply_components_core::syntax::SyntaxKind::Variable => {
                    NSColor::systemIndigoColor()
                }
                shrimply_components_core::syntax::SyntaxKind::Number => {
                    NSColor::systemPurpleColor()
                }
                shrimply_components_core::syntax::SyntaxKind::String => NSColor::systemRedColor(),
                shrimply_components_core::syntax::SyntaxKind::Comment => {
                    NSColor::secondaryLabelColor()
                }
            };
            storage.addAttribute_value_range(
                NSForegroundColorAttributeName,
                &color,
                NSRange::new(span.start_utf16, span.length_utf16),
            );
        }
    }
    storage.endEditing();
}

fn replace_selection(view: &NSTextView, replacement: &str) {
    let selected = view.selectedRange();
    view.replaceCharactersInRange_withString(selected, &NSString::from_str(replacement));
    view.setSelectedRange(NSRange::new(
        selected.location + replacement.encode_utf16().count(),
        0,
    ));
}

fn current_line_indent(source: &str, utf16_location: usize) -> String {
    String::from_utf16_lossy(
        &source
            .encode_utf16()
            .take(utf16_location)
            .collect::<Vec<_>>(),
    )
    .rsplit_once('\n')
    .map_or_else(
        || {
            source
                .chars()
                .take_while(|character| character.is_whitespace())
                .collect()
        },
        |(_, line)| {
            line.chars()
                .take_while(|character| matches!(character, ' ' | '\t'))
                .collect()
        },
    )
}

fn smart_backspace(view: &NSTextView) -> bool {
    let selected = view.selectedRange();
    if selected.length != 0 || selected.location == 0 {
        return false;
    }
    let source = view.string().to_string().encode_utf16().collect::<Vec<_>>();
    let caret = selected.location.min(source.len());
    let start = source[..caret]
        .iter()
        .rposition(|character| *character == b'\n' as u16)
        .map_or(0, |index| index + 1);
    if source[start..caret]
        .iter()
        .any(|character| !matches!(*character as u8 as char, ' ' | '\t'))
    {
        return false;
    }
    let delete = if source[caret - 1] == b'\t' as u16 {
        1
    } else {
        ((caret - start - 1) % 4) + 1
    };
    view.replaceCharactersInRange_withString(
        NSRange::new(caret - delete, delete),
        &NSString::new(),
    );
    view.setSelectedRange(NSRange::new(caret - delete, 0));
    true
}

fn move_to_smart_line_edge(view: &NSTextView, end: bool) {
    let selected = view.selectedRange();
    let source = view.string().to_string().encode_utf16().collect::<Vec<_>>();
    let caret = selected.location.min(source.len());
    let start = source[..caret]
        .iter()
        .rposition(|character| *character == b'\n' as u16)
        .map_or(0, |index| index + 1);
    let line_end = source[caret..]
        .iter()
        .position(|character| *character == b'\n' as u16)
        .map_or(source.len(), |index| caret + index);
    let content_start = (start..line_end)
        .find(|index| !matches!(source[*index] as u8 as char, ' ' | '\t'))
        .unwrap_or(line_end);
    let content_end = (start..line_end)
        .rfind(|index| !matches!(source[*index] as u8 as char, ' ' | '\t'))
        .map_or(start, |index| index + 1);
    let target = if end {
        if caret == content_end {
            line_end
        } else {
            content_end
        }
    } else if caret == content_start {
        start
    } else {
        content_start
    };
    view.setSelectedRange(NSRange::new(target, 0));
}

fn show_matching_bracket(view: &NSTextView) {
    let source = view.string().to_string().encode_utf16().collect::<Vec<_>>();
    let caret = view.selectedRange().location.min(source.len());
    let Some(index) = caret
        .checked_sub(1)
        .filter(|index| is_bracket(source[*index]))
        .or_else(|| (caret < source.len() && is_bracket(source[caret])).then_some(caret))
    else {
        return;
    };
    let (open, close, direction) = match source[index] as u8 as char {
        '(' => (b'(' as u16, b')' as u16, 1isize),
        '[' => (b'[' as u16, b']' as u16, 1),
        '{' => (b'{' as u16, b'}' as u16, 1),
        ')' => (b'(' as u16, b')' as u16, -1),
        ']' => (b'[' as u16, b']' as u16, -1),
        '}' => (b'{' as u16, b'}' as u16, -1),
        _ => unreachable!(),
    };
    let mut depth = 0isize;
    let mut cursor = index as isize;
    loop {
        cursor += direction;
        let Ok(candidate) = usize::try_from(cursor) else {
            return;
        };
        let Some(character) = source.get(candidate).copied() else {
            return;
        };
        if character == if direction > 0 { open } else { close } {
            depth += 1;
        } else if character == if direction > 0 { close } else { open } {
            if depth == 0 {
                view.showFindIndicatorForRange(NSRange::new(candidate, 1));
                return;
            }
            depth -= 1;
        }
    }
}

fn is_bracket(character: u16) -> bool {
    matches!(character as u8 as char, '(' | ')' | '[' | ']' | '{' | '}')
}

unsafe fn retain_target<T: objc2::Message, U: objc2::Message>(
    object: &T,
    target: &Retained<U>,
    key: &'static u8,
) {
    unsafe {
        objc_setAssociatedObject(
            std::ptr::from_ref(object).cast_mut().cast(),
            std::ptr::from_ref(key).cast::<c_void>(),
            Retained::as_ptr(target).cast_mut().cast(),
            OBJC_ASSOCIATION_RETAIN_NONATOMIC,
        );
    }
}
