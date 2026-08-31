use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QColor, QString, QStringList};
use shrimply_component_core::{color, number, project_settings, selector, text};
use shrimply_math_color::Color;
use shrimply_math_core::{
    Fraction, fraction_as_f64, fraction_denominator, fraction_from_f64, fraction_numerator,
};
use shrimply_project_core::{COMMON_FRAME_RATES, PROJECT_PRESETS};
use std::cell::RefCell;

thread_local! {
    static RECENT_COLORS: RefCell<Vec<Color<u8>>> = const { RefCell::new(Vec::new()) };
}

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("drag_input.h");
        #[namespace = "shrimply"]
        fn register_drag_input();

        include!("frame_graph.h");
        #[namespace = "shrimply"]
        fn force_component_opengl();

        include!("cxx-qt-lib/qcolor.h");
        type QColor = cxx_qt_lib::QColor;
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(f64, value)]
        #[qproperty(QString, display_text, cxx_name = "displayText")]
        #[qproperty(bool, editing)]
        #[qproperty(f64, drag_threshold, cxx_name = "dragThreshold")]
        type NumberInputBackend = super::NumberInputBackendRust;

        #[qinvokable]
        fn configure(
            self: Pin<&mut NumberInputBackend>,
            value: f64,
            minimum: f64,
            maximum: f64,
            step: f64,
            drag_pixels: f64,
            digits: i32,
        );
        #[qinvokable]
        #[cxx_name = "setExternalValue"]
        fn set_external_value(self: Pin<&mut NumberInputBackend>, value: f64);
        #[qinvokable]
        #[cxx_name = "beginEdit"]
        fn begin_edit(self: Pin<&mut NumberInputBackend>);
        #[qinvokable]
        #[cxx_name = "previewText"]
        fn preview_text(self: Pin<&mut NumberInputBackend>, value: &QString) -> bool;
        #[qinvokable]
        #[cxx_name = "commitText"]
        fn commit_text(self: Pin<&mut NumberInputBackend>, value: &QString);
        #[qinvokable]
        #[cxx_name = "beginDrag"]
        fn begin_drag(self: Pin<&mut NumberInputBackend>);
        #[qinvokable]
        fn drag(self: Pin<&mut NumberInputBackend>, offset: f64);
        #[qinvokable]
        #[cxx_name = "endDrag"]
        fn end_drag(self: Pin<&mut NumberInputBackend>);

        #[qsignal]
        fn edited(self: Pin<&mut NumberInputBackend>, value: f64);
        #[qsignal]
        fn committed(self: Pin<&mut NumberInputBackend>, value: f64);

        #[qobject]
        #[qml_element]
        #[qproperty(f64, first)]
        #[qproperty(f64, second)]
        #[qproperty(f64, third)]
        #[qproperty(bool, locked)]
        type NumberGroupBackend = super::NumberGroupBackendRust;

        #[qinvokable]
        fn configure(
            self: Pin<&mut NumberGroupBackend>,
            first: f64,
            second: f64,
            third: f64,
            dimensions: i32,
            locked: bool,
        );
        #[qinvokable]
        #[cxx_name = "setBounds"]
        fn set_group_bounds(self: Pin<&mut NumberGroupBackend>, minimum: f64, maximum: f64);
        #[qinvokable]
        fn edit(self: Pin<&mut NumberGroupBackend>, axis: i32, value: f64);
        #[qinvokable]
        #[cxx_name = "setExternalValue"]
        fn set_group_external_value(self: Pin<&mut NumberGroupBackend>, axis: i32, value: f64);
        #[qinvokable]
        #[cxx_name = "updateLocked"]
        fn update_lock(self: Pin<&mut NumberGroupBackend>, locked: bool);

        #[qsignal]
        #[cxx_name = "valueEdited"]
        fn group_value_edited(self: Pin<&mut NumberGroupBackend>, axis: i32, value: f64);

        #[qobject]
        #[qml_element]
        #[qproperty(QColor, color)]
        #[qproperty(QColor, draft)]
        #[qproperty(QString, hex)]
        #[qproperty(f32, hue)]
        #[qproperty(f32, saturation)]
        #[qproperty(f32, brightness)]
        #[qproperty(f32, alpha)]
        #[qproperty(bool, with_alpha, cxx_name = "withAlpha")]
        #[qproperty(i32, palette_count, cxx_name = "paletteCount")]
        #[qproperty(i32, recent_count, cxx_name = "recentCount")]
        type ColorPickerBackend = super::ColorPickerBackendRust;

        #[qinvokable]
        fn configure(self: Pin<&mut ColorPickerBackend>, color: &QColor, with_alpha: bool);
        #[qinvokable]
        #[cxx_name = "setHsva"]
        fn set_hsva(
            self: Pin<&mut ColorPickerBackend>,
            hue: f32,
            saturation: f32,
            brightness: f32,
            alpha: f32,
        );
        #[qinvokable]
        #[cxx_name = "applyHex"]
        fn apply_hex(self: Pin<&mut ColorPickerBackend>, value: &QString) -> bool;
        #[qinvokable]
        #[cxx_name = "paletteColor"]
        fn palette_color(self: &ColorPickerBackend, index: i32) -> QColor;
        #[qinvokable]
        #[cxx_name = "paletteLabel"]
        fn palette_label(self: &ColorPickerBackend, index: i32) -> QString;
        #[qinvokable]
        #[cxx_name = "recentColor"]
        fn recent_color(self: &ColorPickerBackend, index: i32) -> QColor;
        #[qinvokable]
        #[cxx_name = "chooseColor"]
        fn choose_color(self: Pin<&mut ColorPickerBackend>, color: &QColor);
        #[qinvokable]
        fn confirm(self: Pin<&mut ColorPickerBackend>);

        #[qsignal]
        fn selected(self: Pin<&mut ColorPickerBackend>, color: QColor);
        #[qsignal]
        fn confirmed(self: Pin<&mut ColorPickerBackend>);

        #[qobject]
        #[qml_element]
        #[qproperty(QString, text)]
        #[qproperty(bool, dirty)]
        #[qproperty(i32, typo_count, cxx_name = "typoCount")]
        #[qproperty(QString, typo_ranges, cxx_name = "typoRanges")]
        type TextInputBackend = super::TextInputBackendRust;

        #[qinvokable]
        fn configure(self: Pin<&mut TextInputBackend>, value: &QString, maximum_length: i32);
        #[qinvokable]
        fn edit(self: Pin<&mut TextInputBackend>, value: &QString) -> QString;
        #[qinvokable]
        fn commit(self: Pin<&mut TextInputBackend>);
        #[qinvokable]
        #[cxx_name = "typoMessage"]
        fn typo_message(self: &TextInputBackend, index: i32) -> QString;
        #[qinvokable]
        #[cxx_name = "typoAt"]
        fn typo_at(self: &TextInputBackend, utf16_offset: i32) -> i32;
        #[qinvokable]
        #[cxx_name = "typoStart"]
        fn typo_start(self: &TextInputBackend, index: i32) -> i32;
        #[qinvokable]
        #[cxx_name = "typoLength"]
        fn typo_length(self: &TextInputBackend, index: i32) -> i32;
        #[qinvokable]
        #[cxx_name = "typoCorrection"]
        fn typo_correction(self: &TextInputBackend, typo: i32, correction: i32) -> QString;
        #[qinvokable]
        #[cxx_name = "applyCorrection"]
        fn apply_correction(
            self: Pin<&mut TextInputBackend>,
            typo: i32,
            correction: i32,
        ) -> QString;

        #[qsignal]
        fn changed(self: Pin<&mut TextInputBackend>, value: QString);
        #[qsignal]
        fn committed(self: Pin<&mut TextInputBackend>, value: QString);

        #[qobject]
        #[qml_element]
        type SelectorBackend = super::SelectorBackendRust;

        #[qinvokable]
        #[cxx_name = "selectedIndex"]
        fn selected_index(self: &SelectorBackend, values: &QStringList, value: &QString) -> i32;
        #[qinvokable]
        #[cxx_name = "matchingIndex"]
        fn matching_index(self: &SelectorBackend, values: &QStringList, value: &QString) -> i32;
        #[qinvokable]
        #[cxx_name = "matchesQuery"]
        fn matches_query(self: &SelectorBackend, label: &QString, query: &QString) -> bool;
        #[qinvokable]
        #[cxx_name = "nextMatchingIndex"]
        fn next_matching_index(
            self: &SelectorBackend,
            labels: &QStringList,
            query: &QString,
            current: i32,
            direction: i32,
        ) -> i32;
        #[qinvokable]
        #[cxx_name = "valueAt"]
        fn value_at(self: &SelectorBackend, values: &QStringList, index: i32) -> QString;
        #[qinvokable]
        fn searchable(self: &SelectorBackend, count: i32) -> bool;

        #[qobject]
        #[qml_element]
        #[qproperty(i32, preset)]
        #[qproperty(i32, width)]
        #[qproperty(i32, height)]
        #[qproperty(i32, frame_rate, cxx_name = "frameRate")]
        #[qproperty(i32, preset_count, cxx_name = "presetCount")]
        #[qproperty(i32, frame_rate_count, cxx_name = "frameRateCount")]
        type ProjectSettingsBackend = super::ProjectSettingsBackendRust;

        #[qinvokable]
        #[cxx_name = "presetLabel"]
        fn preset_label(self: &ProjectSettingsBackend, index: i32) -> QString;
        #[qinvokable]
        #[cxx_name = "frameRateLabel"]
        fn frame_rate_label(self: &ProjectSettingsBackend, index: i32) -> QString;
        #[qinvokable]
        #[cxx_name = "selectPreset"]
        fn select_preset(self: Pin<&mut ProjectSettingsBackend>, index: i32);
        #[qinvokable]
        #[cxx_name = "setWidthValue"]
        fn set_width_value(self: Pin<&mut ProjectSettingsBackend>, width: i32);
        #[qinvokable]
        #[cxx_name = "setHeightValue"]
        fn set_height_value(self: Pin<&mut ProjectSettingsBackend>, height: i32);
        #[qinvokable]
        #[cxx_name = "setFrameRateValue"]
        fn set_frame_rate_value(self: Pin<&mut ProjectSettingsBackend>, index: i32);
        #[qinvokable]
        #[cxx_name = "fpsNumerator"]
        fn fps_numerator(self: &ProjectSettingsBackend) -> i32;
        #[qinvokable]
        #[cxx_name = "fpsDenominator"]
        fn fps_denominator(self: &ProjectSettingsBackend) -> i32;

        #[qobject]
        #[qml_element]
        #[qml_singleton]
        type ComponentTranslations = super::ComponentTranslationsRust;

        #[qinvokable]
        fn text(self: &ComponentTranslations, key: &QString) -> QString;
    }

    impl cxx_qt::Initialize for NumberInputBackend {}
    impl cxx_qt::Initialize for NumberGroupBackend {}
    impl cxx_qt::Initialize for ColorPickerBackend {}
    impl cxx_qt::Initialize for TextInputBackend {}
    impl cxx_qt::Initialize for SelectorBackend {}
    impl cxx_qt::Initialize for ProjectSettingsBackend {}
    impl cxx_qt::Initialize for ComponentTranslations {}
}

pub struct NumberInputBackendRust {
    value: f64,
    display_text: QString,
    editing: bool,
    drag_threshold: f64,
    config: number::NumberConfig,
    accepted: Fraction,
    drag_start: Fraction,
    drag_moved: bool,
}

impl Default for NumberInputBackendRust {
    fn default() -> Self {
        let accepted = fraction_from_f64(0.0);
        let config = number::NumberConfig::new(accepted);
        Self {
            value: 0.0,
            display_text: QString::from(number::format_value(&config, accepted)),
            editing: false,
            drag_threshold: number::DRAG_THRESHOLD_PIXELS,
            config,
            accepted,
            drag_start: accepted,
            drag_moved: false,
        }
    }
}

impl cxx_qt::Initialize for qobject::NumberInputBackend {
    fn initialize(self: Pin<&mut Self>) {}
}

impl qobject::NumberInputBackend {
    pub fn configure(
        mut self: Pin<&mut Self>,
        value: f64,
        minimum: f64,
        maximum: f64,
        step: f64,
        drag_pixels: f64,
        digits: i32,
    ) {
        let fallback = fraction_from_f64(value);
        let config = number::NumberConfig {
            minimum: fraction_from_f64(minimum),
            maximum: fraction_from_f64(maximum),
            drag_step: number::positive_fraction_or(
                fraction_from_f64(step),
                fraction_from_f64(1.0),
            ),
            drag_pixels: if drag_pixels > 0.0 {
                drag_pixels
            } else {
                number::DEFAULT_DRAG_PIXELS
            },
            digits: usize::try_from(digits.max(0)).unwrap_or_default(),
            fallback: number::finite_fraction_or(fallback, fraction_from_f64(0.0)),
        };
        self.as_mut().rust_mut().config = config;
        self.set_accepted(fallback, false, false);
    }

    pub fn set_external_value(mut self: Pin<&mut Self>, value: f64) {
        if !self.editing() {
            self.as_mut()
                .set_accepted(fraction_from_f64(value), false, false);
        }
    }

    pub fn begin_edit(mut self: Pin<&mut Self>) {
        self.as_mut().set_editing(true);
        let text = number::format_value(&self.rust().config, self.rust().accepted);
        self.as_mut().set_display_text(QString::from(text));
    }

    pub fn preview_text(mut self: Pin<&mut Self>, value: &QString) -> bool {
        let Some(value) = number::parse_fraction(value.to_string().trim()) else {
            return false;
        };
        let value = number::accepted_value(&self.rust().config, value);
        self.as_mut().edited(fraction_as_f64(value));
        true
    }

    pub fn commit_text(mut self: Pin<&mut Self>, value: &QString) {
        if let Some(value) = number::parse_fraction(value.to_string().trim()) {
            self.as_mut().set_accepted(value, true, true);
        } else {
            let accepted = self.rust().accepted;
            self.as_mut().set_accepted(accepted, false, false);
        }
        self.as_mut().set_editing(false);
    }

    pub fn begin_drag(mut self: Pin<&mut Self>) {
        let accepted = self.rust().accepted;
        let rust = self.as_mut().rust_mut().get_mut();
        rust.drag_start = accepted;
        rust.drag_moved = false;
    }

    pub fn drag(mut self: Pin<&mut Self>, offset: f64) {
        if offset.abs() < number::DRAG_THRESHOLD_PIXELS && !self.rust().drag_moved {
            return;
        }
        self.as_mut().rust_mut().drag_moved = true;
        let value = number::dragged_value(&self.rust().config, self.rust().drag_start, offset);
        self.as_mut().set_accepted(value, true, false);
    }

    pub fn end_drag(mut self: Pin<&mut Self>) {
        if self.rust().drag_moved {
            let value = *self.value();
            self.as_mut().committed(value);
        }
    }

    fn set_accepted(mut self: Pin<&mut Self>, value: Fraction, notify: bool, commit: bool) {
        let value = number::accepted_value(&self.rust().config, value);
        let changed = self.rust().accepted != value;
        self.as_mut().rust_mut().accepted = value;
        let numeric = fraction_as_f64(value);
        self.as_mut().set_value(numeric);
        let display = number::format_value(&self.rust().config, value);
        self.as_mut().set_display_text(QString::from(display));
        if changed && notify {
            self.as_mut().edited(numeric);
        }
        if commit && changed {
            self.as_mut().committed(numeric);
        }
    }
}

pub struct NumberGroupBackendRust {
    first: f64,
    second: f64,
    third: f64,
    locked: bool,
    dimensions: usize,
    values: [Fraction; 3],
    ratios: [Fraction; 2],
    minimum: Fraction,
    maximum: Fraction,
}

impl Default for NumberGroupBackendRust {
    fn default() -> Self {
        let values = [fraction_from_f64(0.0); 3];
        Self {
            first: 0.0,
            second: 0.0,
            third: 0.0,
            locked: false,
            dimensions: 2,
            values,
            ratios: number::triple_ratios(values),
            minimum: shrimply_math_core::fraction_from_integer(number::DEFAULT_MINIMUM),
            maximum: shrimply_math_core::fraction_from_integer(number::DEFAULT_MAXIMUM),
        }
    }
}

impl cxx_qt::Initialize for qobject::NumberGroupBackend {
    fn initialize(self: Pin<&mut Self>) {}
}

impl qobject::NumberGroupBackend {
    pub fn configure(
        mut self: Pin<&mut Self>,
        first: f64,
        second: f64,
        third: f64,
        dimensions: i32,
        locked: bool,
    ) {
        let values = [
            fraction_from_f64(first),
            fraction_from_f64(second),
            fraction_from_f64(third),
        ]
        .map(|value| number::clamped_value(value, self.rust().minimum, self.rust().maximum));
        let rust = self.as_mut().rust_mut().get_mut();
        rust.values = values;
        rust.dimensions = usize::try_from(dimensions.clamp(2, 3)).unwrap_or(2);
        rust.ratios = group_ratios(rust.dimensions, values);
        self.as_mut().update_lock(locked);
        self.as_mut().publish_values();
    }

    pub fn edit(mut self: Pin<&mut Self>, axis: i32, value: f64) {
        let Ok(axis) = usize::try_from(axis) else {
            return;
        };
        if axis >= self.rust().dimensions {
            return;
        }
        let previous = self.rust().values;
        let value = number::clamped_value(
            fraction_from_f64(value),
            self.rust().minimum,
            self.rust().maximum,
        );
        if *self.locked() {
            let values = if self.rust().dimensions == 2 {
                let pair = number::locked_pair(axis, value, self.rust().ratios[0]);
                [pair[0], pair[1], self.rust().values[2]]
            } else {
                number::locked_triple(axis, value, self.rust().ratios)
            }
            .map(|value| number::clamped_value(value, self.rust().minimum, self.rust().maximum));
            self.as_mut().rust_mut().values = values;
        } else {
            self.as_mut().rust_mut().values[axis] = value;
        }
        self.as_mut().publish_values();
        let order = match axis {
            0 => [0, 1, 2],
            1 => [1, 0, 2],
            _ => [2, 0, 1],
        };
        for changed_axis in order.into_iter().take(self.rust().dimensions) {
            let value = self.rust().values[changed_axis];
            if value != previous[changed_axis] {
                self.as_mut().group_value_edited(
                    i32::try_from(changed_axis).unwrap_or_default(),
                    fraction_as_f64(value),
                );
            }
        }
    }

    pub fn set_group_bounds(mut self: Pin<&mut Self>, minimum: f64, maximum: f64) {
        let minimum = fraction_from_f64(minimum);
        let maximum = fraction_from_f64(maximum);
        let values = self
            .rust()
            .values
            .map(|value| number::clamped_value(value, minimum, maximum));
        let rust = self.as_mut().rust_mut().get_mut();
        rust.minimum = minimum;
        rust.maximum = maximum;
        rust.values = values;
        rust.ratios = group_ratios(rust.dimensions, values);
        self.as_mut().publish_values();
    }

    pub fn set_group_external_value(mut self: Pin<&mut Self>, axis: i32, value: f64) {
        let Ok(axis) = usize::try_from(axis) else {
            return;
        };
        if axis >= self.rust().dimensions {
            return;
        }
        let value = number::clamped_value(
            fraction_from_f64(value),
            self.rust().minimum,
            self.rust().maximum,
        );
        if self.rust().values[axis] == value {
            return;
        }
        self.as_mut().rust_mut().values[axis] = value;
        let values = self.rust().values;
        let dimensions = self.rust().dimensions;
        self.as_mut().rust_mut().ratios = group_ratios(dimensions, values);
        self.as_mut().publish_values();
    }

    pub fn update_lock(mut self: Pin<&mut Self>, locked: bool) {
        if locked && !self.locked() {
            let values = self.rust().values;
            let dimensions = self.rust().dimensions;
            self.as_mut().rust_mut().ratios = group_ratios(dimensions, values);
        }
        self.as_mut().set_locked(locked);
    }

    fn publish_values(mut self: Pin<&mut Self>) {
        let values = self.rust().values.map(fraction_as_f64);
        self.as_mut().set_first(values[0]);
        self.as_mut().set_second(values[1]);
        self.as_mut().set_third(values[2]);
    }
}

pub struct ColorPickerBackendRust {
    color: QColor,
    draft: QColor,
    hex: QString,
    hue: f32,
    saturation: f32,
    brightness: f32,
    alpha: f32,
    with_alpha: bool,
    palette_count: i32,
    recent_count: i32,
    hsva: color::Hsva,
}

impl Default for ColorPickerBackendRust {
    fn default() -> Self {
        let value = Color::BLACK;
        Self {
            color: qcolor(value),
            draft: qcolor(value),
            hex: QString::from(color::color_hex(value, true)),
            hue: 0.0,
            saturation: 0.0,
            brightness: 0.0,
            alpha: 1.0,
            with_alpha: true,
            palette_count: color::PALETTE.len() as i32,
            recent_count: 0,
            hsva: color::Hsva::from_color(value),
        }
    }
}

impl cxx_qt::Initialize for qobject::ColorPickerBackend {
    fn initialize(self: Pin<&mut Self>) {}
}

impl qobject::ColorPickerBackend {
    pub fn configure(mut self: Pin<&mut Self>, value: &QColor, with_alpha: bool) {
        let mut value = color_from_qcolor(value);
        if !with_alpha {
            value.a = u8::MAX;
        }
        self.as_mut().set_with_alpha(with_alpha);
        let recent_count = RECENT_COLORS.with_borrow(|colors| colors.len() as i32);
        self.as_mut().set_recent_count(recent_count);
        self.as_mut().set_color(qcolor(value));
        self.as_mut().publish_color(value);
    }

    pub fn set_hsva(
        mut self: Pin<&mut Self>,
        hue: f32,
        saturation: f32,
        brightness: f32,
        alpha: f32,
    ) {
        let hsva = color::Hsva {
            hue: hue.rem_euclid(360.0),
            saturation: saturation.clamp(0.0, 1.0),
            value: brightness.clamp(0.0, 1.0),
            alpha: if *self.with_alpha() {
                alpha.clamp(0.0, 1.0)
            } else {
                1.0
            },
        };
        self.as_mut().publish_hsva(hsva);
    }

    pub fn apply_hex(mut self: Pin<&mut Self>, value: &QString) -> bool {
        let Some(value) = color::parse_hex(&value.to_string(), *self.with_alpha()) else {
            return false;
        };
        self.as_mut().publish_color(value);
        true
    }

    pub fn palette_color(&self, index: i32) -> QColor {
        index_of(index)
            .and_then(|index| color::PALETTE.get(index))
            .map_or_else(QColor::default, |(_, value)| qcolor(*value))
    }

    pub fn palette_label(&self, index: i32) -> QString {
        index_of(index)
            .and_then(|index| color::PALETTE.get(index))
            .map_or_else(QString::default, |(label, _)| shrimply_i18n_qt::text(label))
    }

    pub fn recent_color(&self, index: i32) -> QColor {
        RECENT_COLORS.with_borrow(|colors| {
            index_of(index)
                .and_then(|index| colors.get(index))
                .copied()
                .map_or_else(QColor::default, qcolor)
        })
    }

    pub fn choose_color(mut self: Pin<&mut Self>, value: &QColor) {
        let mut value = color_from_qcolor(value);
        if !self.with_alpha() {
            value.a = u8::MAX;
        }
        self.as_mut().publish_color(value);
    }

    pub fn confirm(mut self: Pin<&mut Self>) {
        let value = self.rust().hsva.color();
        RECENT_COLORS.with_borrow_mut(|colors| color::remember_color(colors, value));
        let count = RECENT_COLORS.with_borrow(|colors| colors.len() as i32);
        self.as_mut().set_recent_count(count);
        let selected = qcolor(value);
        let changed = color_from_qcolor(self.color()) != value;
        self.as_mut().set_color(selected.clone());
        if changed {
            self.as_mut().selected(selected);
        }
        self.as_mut().confirmed();
    }

    fn publish_color(self: Pin<&mut Self>, value: Color<u8>) {
        self.publish_hsva(color::Hsva::from_color(value));
    }

    fn publish_hsva(mut self: Pin<&mut Self>, hsva: color::Hsva) {
        let value = hsva.color();
        self.as_mut().rust_mut().hsva = hsva;
        self.as_mut().set_hue(hsva.hue);
        self.as_mut().set_saturation(hsva.saturation);
        self.as_mut().set_brightness(hsva.value);
        self.as_mut().set_alpha(hsva.alpha);
        self.as_mut().set_draft(qcolor(value));
        let with_alpha = *self.with_alpha();
        self.as_mut()
            .set_hex(QString::from(color::color_hex(value, with_alpha)));
    }
}

#[derive(Default)]
pub struct TextInputBackendRust {
    text: QString,
    dirty: bool,
    typo_count: i32,
    typo_ranges: QString,
    maximum_length: Option<usize>,
    marks: Vec<text::TypoMark>,
}

impl cxx_qt::Initialize for qobject::TextInputBackend {
    fn initialize(self: Pin<&mut Self>) {}
}

impl qobject::TextInputBackend {
    pub fn configure(mut self: Pin<&mut Self>, value: &QString, maximum_length: i32) {
        let maximum_length = usize::try_from(maximum_length)
            .ok()
            .filter(|value| *value > 0);
        self.as_mut().rust_mut().maximum_length = maximum_length;
        self.as_mut().set_text_value(
            text::limited_text(&value.to_string(), maximum_length),
            false,
        );
    }

    pub fn edit(mut self: Pin<&mut Self>, value: &QString) -> QString {
        let value = text::limited_text(&value.to_string(), self.rust().maximum_length);
        if value == self.text().to_string() {
            return QString::from(value);
        }
        self.as_mut().set_text_value(value.clone(), true);
        self.as_mut().changed(QString::from(value.clone()));
        QString::from(value)
    }

    pub fn commit(mut self: Pin<&mut Self>) {
        if *self.dirty() {
            self.as_mut().set_dirty(false);
            let value = self.text().clone();
            self.as_mut().committed(value);
        }
    }

    pub fn typo_message(&self, index: i32) -> QString {
        self.mark(index)
            .map_or_else(QString::default, |mark| QString::from(&mark.message))
    }

    pub fn typo_at(&self, utf16_offset: i32) -> i32 {
        let utf16_offset = usize::try_from(utf16_offset).unwrap_or_default();
        let text = self.text().to_string();
        let mut remaining = utf16_offset;
        let char_offset = text
            .chars()
            .take_while(|character| {
                let width = character.len_utf16();
                if remaining < width {
                    false
                } else {
                    remaining -= width;
                    true
                }
            })
            .count();
        self.rust()
            .marks
            .iter()
            .position(|mark| {
                let start = usize::try_from(mark.start).unwrap_or_default();
                let end = usize::try_from(mark.end).unwrap_or_default();
                char_offset >= start && char_offset < end
            })
            .and_then(|index| i32::try_from(index).ok())
            .unwrap_or(-1)
    }

    pub fn typo_correction(&self, typo: i32, correction: i32) -> QString {
        self.mark(typo)
            .and_then(|mark| index_of(correction).and_then(|index| mark.corrections.get(index)))
            .map_or_else(QString::default, QString::from)
    }

    pub fn typo_start(&self, index: i32) -> i32 {
        self.utf16_mark(index).map_or(-1, |(start, _)| start)
    }

    pub fn typo_length(&self, index: i32) -> i32 {
        self.utf16_mark(index).map_or(0, |(_, length)| length)
    }

    pub fn apply_correction(mut self: Pin<&mut Self>, typo: i32, correction: i32) -> QString {
        let Some(mark) = self.mark(typo).cloned() else {
            return self.text().clone();
        };
        let Some(correction) = index_of(correction)
            .and_then(|index| mark.corrections.get(index))
            .cloned()
        else {
            return self.text().clone();
        };
        let mut chars = self.text().to_string().chars().collect::<Vec<_>>();
        let start = usize::try_from(mark.start)
            .unwrap_or_default()
            .min(chars.len());
        let end = usize::try_from(mark.end)
            .unwrap_or_default()
            .min(chars.len());
        chars.splice(start..end, correction.chars());
        let value = chars.into_iter().collect::<String>();
        self.as_mut().set_text_value(value.clone(), true);
        self.as_mut().changed(QString::from(value.clone()));
        QString::from(value)
    }

    fn mark(&self, index: i32) -> Option<&text::TypoMark> {
        index_of(index).and_then(|index| self.rust().marks.get(index))
    }

    fn utf16_mark(&self, index: i32) -> Option<(i32, i32)> {
        let mark = self.mark(index)?;
        let (start, length) = utf16_range(&self.text().to_string(), mark);
        Some((
            i32::try_from(start).unwrap_or(i32::MAX),
            i32::try_from(length).unwrap_or(i32::MAX),
        ))
    }

    fn set_text_value(mut self: Pin<&mut Self>, value: String, dirty: bool) {
        let marks = text::typo_marks(&value);
        let count = i32::try_from(marks.len()).unwrap_or(i32::MAX);
        let typo_ranges = marks
            .iter()
            .map(|mark| utf16_range(&value, mark))
            .map(|(start, length)| format!("{start}:{length}"))
            .collect::<Vec<_>>()
            .join(",");
        self.as_mut().rust_mut().marks = marks;
        self.as_mut().set_text(QString::from(value));
        self.as_mut().set_dirty(dirty);
        self.as_mut().set_typo_count(count);
        self.as_mut().set_typo_ranges(QString::from(typo_ranges));
    }
}

fn utf16_range(value: &str, mark: &text::TypoMark) -> (usize, usize) {
    let start = usize::try_from(mark.start).unwrap_or_default();
    let end = usize::try_from(mark.end).unwrap_or(start);
    (
        value.chars().take(start).map(char::len_utf16).sum(),
        value
            .chars()
            .skip(start)
            .take(end.saturating_sub(start))
            .map(char::len_utf16)
            .sum(),
    )
}

#[derive(Default)]
pub struct SelectorBackendRust;

impl cxx_qt::Initialize for qobject::SelectorBackend {
    fn initialize(self: Pin<&mut Self>) {}
}

impl qobject::SelectorBackend {
    pub fn selected_index(&self, values: &QStringList, value: &QString) -> i32 {
        let choices = selector::identity_choices(values.iter().map(ToString::to_string).collect());
        i32::try_from(selector::selected_index(&value.to_string(), &choices)).unwrap_or_default()
    }

    pub fn matching_index(&self, values: &QStringList, value: &QString) -> i32 {
        let choices = selector::identity_choices(values.iter().map(ToString::to_string).collect());
        selector::matching_index(&value.to_string(), &choices)
            .and_then(|index| i32::try_from(index).ok())
            .unwrap_or(-1)
    }

    pub fn matches_query(&self, label: &QString, query: &QString) -> bool {
        selector::matches_query(&label.to_string(), &query.to_string())
    }

    pub fn next_matching_index(
        &self,
        labels: &QStringList,
        query: &QString,
        current: i32,
        direction: i32,
    ) -> i32 {
        let query = query.to_string();
        let indices = labels
            .iter()
            .enumerate()
            .filter(|(_, label)| selector::matches_query(&label.to_string(), &query))
            .filter_map(|(index, _)| i32::try_from(index).ok())
            .collect::<Vec<_>>();
        if direction < 0 {
            indices
                .iter()
                .rev()
                .copied()
                .find(|index| *index < current)
                .or_else(|| indices.last().copied())
        } else {
            indices
                .iter()
                .copied()
                .find(|index| *index > current)
                .or_else(|| indices.first().copied())
        }
        .unwrap_or(-1)
    }

    pub fn value_at(&self, values: &QStringList, index: i32) -> QString {
        isize::try_from(index)
            .ok()
            .and_then(|index| values.get(index))
            .cloned()
            .unwrap_or_default()
    }

    pub fn searchable(&self, count: i32) -> bool {
        selector::searchable(usize::try_from(count).unwrap_or_default())
    }
}

pub struct ProjectSettingsBackendRust {
    preset: i32,
    width: i32,
    height: i32,
    frame_rate: i32,
    preset_count: i32,
    frame_rate_count: i32,
    settings: project_settings::ProjectSettings,
}

impl Default for ProjectSettingsBackendRust {
    fn default() -> Self {
        let settings = project_settings::ProjectSettings::default();
        Self {
            preset: settings.preset as i32,
            width: settings.width as i32,
            height: settings.height as i32,
            frame_rate: settings.frame_rate as i32,
            preset_count: (PROJECT_PRESETS.len() + 1) as i32,
            frame_rate_count: COMMON_FRAME_RATES.len() as i32,
            settings,
        }
    }
}

impl cxx_qt::Initialize for qobject::ProjectSettingsBackend {
    fn initialize(self: Pin<&mut Self>) {}
}

impl qobject::ProjectSettingsBackend {
    pub fn preset_label(&self, index: i32) -> QString {
        match index_of(index) {
            Some(index) if index == project_settings::CUSTOM_PRESET_INDEX => {
                shrimply_i18n_qt::text("Custom")
            }
            Some(index) => PROJECT_PRESETS
                .get(index)
                .map_or_else(QString::default, |preset| {
                    shrimply_i18n_qt::text(preset.label)
                }),
            None => QString::default(),
        }
    }

    pub fn frame_rate_label(&self, index: i32) -> QString {
        index_of(index)
            .and_then(|index| COMMON_FRAME_RATES.get(index))
            .map_or_else(QString::default, |rate| QString::from(rate.label))
    }

    pub fn select_preset(mut self: Pin<&mut Self>, index: i32) {
        let Some(index) = index_of(index) else {
            return;
        };
        self.as_mut().rust_mut().settings.select_preset(index);
        self.as_mut().publish_settings();
    }

    pub fn set_width_value(mut self: Pin<&mut Self>, width: i32) {
        self.as_mut()
            .rust_mut()
            .settings
            .set_width(width.max(1) as u32);
        self.as_mut().publish_settings();
    }

    pub fn set_height_value(mut self: Pin<&mut Self>, height: i32) {
        self.as_mut()
            .rust_mut()
            .settings
            .set_height(height.max(1) as u32);
        self.as_mut().publish_settings();
    }

    pub fn set_frame_rate_value(mut self: Pin<&mut Self>, index: i32) {
        let Some(index) = index_of(index) else {
            return;
        };
        self.as_mut().rust_mut().settings.set_frame_rate(index);
        self.as_mut().publish_settings();
    }

    pub fn fps_numerator(&self) -> i32 {
        self.rate_part(true)
    }

    pub fn fps_denominator(&self) -> i32 {
        self.rate_part(false)
    }

    fn rate_part(&self, numerator: bool) -> i32 {
        let Some((_, rate)) = self.rust().settings.settings() else {
            return 0;
        };
        let value = if numerator {
            fraction_numerator(rate)
        } else {
            fraction_denominator(rate)
        };
        i32::try_from(value).unwrap_or(if value < 0 { i32::MIN } else { i32::MAX })
    }

    fn publish_settings(mut self: Pin<&mut Self>) {
        let settings = self.rust().settings;
        self.as_mut().set_preset(settings.preset as i32);
        self.as_mut().set_width(settings.width as i32);
        self.as_mut().set_height(settings.height as i32);
        self.as_mut().set_frame_rate(settings.frame_rate as i32);
    }
}

#[derive(Default)]
pub struct ComponentTranslationsRust;

impl cxx_qt::Initialize for qobject::ComponentTranslations {
    fn initialize(self: Pin<&mut Self>) {}
}

impl qobject::ComponentTranslations {
    pub fn text(&self, key: &QString) -> QString {
        shrimply_i18n_qt::text(&key.to_string())
    }
}

fn qcolor(color: Color<u8>) -> QColor {
    QColor::from_rgba_f(
        f32::from(color.r) / 255.0,
        f32::from(color.g) / 255.0,
        f32::from(color.b) / 255.0,
        f32::from(color.a) / 255.0,
    )
}

fn color_from_qcolor(color: &QColor) -> Color<u8> {
    fn channel(value: f32) -> u8 {
        (value.clamp(0.0, 1.0) * 255.0).round() as u8
    }
    Color::new(
        channel(color.red_f()),
        channel(color.green_f()),
        channel(color.blue_f()),
        channel(color.alpha_f()),
    )
}

fn index_of(index: i32) -> Option<usize> {
    usize::try_from(index).ok()
}

fn group_ratios(dimensions: usize, values: [Fraction; 3]) -> [Fraction; 2] {
    if dimensions == 2 {
        [
            number::pair_ratio(values[0], values[1]),
            fraction_from_f64(1.0),
        ]
    } else {
        number::triple_ratios(values)
    }
}
