use core::pin::Pin;
use std::collections::HashMap;

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QColor, QString, QStringList, QUrl};
use shrimply_inspector_core::InspectorTarget;

use crate::item::{InspectorAction, InspectorListItem};
use crate::list::{InspectorDocument, InspectorListState};
use crate::section::{ControlKind, ControlRowRole, InspectorControl};
use crate::value_backend::{
    boolean_action, control_value, fraction_value, optional_action, timeline_value,
};

const MISSING_CONTROL_INDEX: i32 = -1;

type AnalysisControlKey = (usize, usize, usize);

#[derive(Clone, Debug, PartialEq)]
struct CachedAnalysisControl {
    target: InspectorTarget,
    action: shrimply_inspector_core::InspectorControlAction,
    presentation: shrimply_inspector_core::AnalysisControlPresentation,
}

fn analysis_presentation(
    control: &InspectorControl,
    target: Option<&InspectorTarget>,
) -> Option<shrimply_inspector_core::AnalysisControlPresentation> {
    match control.action? {
        shrimply_inspector_core::InspectorControlAction::ToggleSam2Analysis {
            modifier_id,
            generation,
            prompt_signature,
            can_analyze,
        } => Some(shrimply_inspector_core::sam2_analysis_control(
            modifier_id,
            generation,
            prompt_signature,
            can_analyze,
        )),
        shrimply_inspector_core::InspectorControlAction::ToggleTransparentFillAnalysis {
            modifier_id,
        } => super::transparent_fill_analysis_control(target?, modifier_id).ok(),
        shrimply_inspector_core::InspectorControlAction::ToggleCameraAnalysis => {
            super::camera_analysis_control(target?).ok()
        }
        _ => None,
    }
}

fn analysis_tooltip(status: &shrimply_inspector_core::AnalysisControlPresentation) -> QString {
    match &status.tooltip {
        shrimply_inspector_core::AnalysisTooltip::MessageKey(message) => {
            shrimply_i18n_qt::text(message)
        }
        shrimply_inspector_core::AnalysisTooltip::RawError(error) => QString::from(error.as_str()),
    }
}

fn analysis_control_cache(
    document: &InspectorDocument,
) -> HashMap<AnalysisControlKey, CachedAnalysisControl> {
    let mut cache = HashMap::new();
    for (category_index, category) in document.categories.iter().enumerate() {
        for (item_index, item) in category.items.iter().enumerate() {
            let section = match item {
                InspectorListItem::Item(item) => &item.section,
                InspectorListItem::Flat(section) => section,
            };
            for (control_index, control) in section.controls.iter().enumerate() {
                if control.kind != ControlKind::Analysis {
                    continue;
                }
                let Some(action) = control.action else {
                    continue;
                };
                let Some(presentation) = control.analysis.clone() else {
                    continue;
                };
                cache.insert(
                    (category_index, item_index, control_index),
                    CachedAnalysisControl {
                        target: document.target.clone(),
                        action,
                        presentation,
                    },
                );
            }
        }
    }
    cache
}

#[cxx_qt::bridge]
pub(crate) mod qobject {
    #[qenum(InspectorBackend)]
    enum InspectorControlKind {
        Boolean,
        Number,
        Fraction,
        Text,
        MultilineText,
        ReadOnly,
        Color,
        LayeredColor,
        LayeredText,
        LayeredDrawing,
        FontFamilies,
        Selector,
        Vector2,
        Vector3,
        LayeredNumber,
        LayeredVector2,
        LayeredVector3,
        ProjectSettings,
        Performance,
        LayeredBoolean,
        LayeredSelector,
        AudioCache,
        AudioCachePreset,
        VisualCache,
        VisualCacheQuality,
        Analysis,
        ModifierMenu,
        TtsEditor,
        BeatDetection,
        InfoHeading,
        InfoArtwork,
        FileLocation,
        InfoLoading,
        Action,
    }

    #[qenum(InspectorBackend)]
    enum InspectorControlRowRole {
        Standalone,
        Primary,
        Auxiliary,
        TrailingAction,
    }

    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!("cxx-qt-lib/qcolor.h");
        type QColor = cxx_qt_lib::QColor;
        include!("cxx-qt-lib/qurl.h");
        type QUrl = cxx_qt_lib::QUrl;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(bool, ready)]
        #[qproperty(i32, revision)]
        #[qproperty(i32, cache_revision, cxx_name = "cacheRevision")]
        #[qproperty(i32, analysis_revision, cxx_name = "analysisRevision")]
        #[qproperty(i32, document_revision, cxx_name = "documentRevision")]
        #[qproperty(i32, expression_revision, cxx_name = "expressionRevision")]
        #[qproperty(i32, graph_revision, cxx_name = "graphRevision")]
        #[qproperty(i32, playhead_revision, cxx_name = "playheadRevision")]
        #[qproperty(i32, transform_revision, cxx_name = "transformRevision")]
        #[qproperty(i32, font_browser_revision, cxx_name = "fontBrowserRevision")]
        #[qproperty(i32, active_category, cxx_name = "activeCategory")]
        #[qproperty(f64, scroll_position, cxx_name = "scrollPosition")]
        #[qproperty(QString, title)]
        type InspectorBackend = super::InspectorBackendRust;

        #[qinvokable]
        fn poll(self: Pin<&mut InspectorBackend>, scroll_position: f64);
        #[qinvokable]
        #[cxx_name = "pollAnalysisControl"]
        fn poll_analysis_control(
            self: Pin<&mut InspectorBackend>,
            category: i32,
            item: i32,
            control: i32,
        );
        #[qinvokable]
        #[cxx_name = "targetChangePending"]
        fn target_change_pending(self: &InspectorBackend) -> bool;
        #[qinvokable]
        #[cxx_name = "minimumWidth"]
        fn minimum_width(self: &InspectorBackend) -> i32;
        #[qinvokable]
        #[cxx_name = "destructiveBackground"]
        fn destructive_background(self: &InspectorBackend) -> QColor;
        #[qinvokable]
        #[cxx_name = "destructiveForeground"]
        fn destructive_foreground(self: &InspectorBackend) -> QColor;
        #[qinvokable]
        #[cxx_name = "keyframeClipboardMarker"]
        fn keyframe_clipboard_marker(self: &InspectorBackend) -> QString;
        #[qinvokable]
        #[cxx_name = "keyframeSnappingEnabled"]
        fn keyframe_snapping_enabled(self: &InspectorBackend) -> bool;
        #[qinvokable]
        #[cxx_name = "keyframeSnappingRadius"]
        fn keyframe_snapping_radius(self: &InspectorBackend) -> f64;
        #[qinvokable]
        #[cxx_name = "categoryKeys"]
        fn category_keys(self: &InspectorBackend) -> QStringList;
        #[qinvokable]
        #[cxx_name = "categoryLabel"]
        fn category_label(self: &InspectorBackend, category: i32) -> QString;
        #[qinvokable]
        #[cxx_name = "categoryIcon"]
        fn category_icon(self: &InspectorBackend, category: i32) -> QString;
        #[qinvokable]
        #[cxx_name = "activateCategory"]
        fn activate_category(self: Pin<&mut InspectorBackend>, category: i32);

        #[qinvokable]
        #[cxx_name = "itemKeys"]
        fn item_keys(self: &InspectorBackend, category: i32) -> QStringList;
        #[qinvokable]
        #[cxx_name = "itemIsCard"]
        fn item_is_card(self: &InspectorBackend, category: i32, item: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "itemIdentity"]
        fn item_identity(self: &InspectorBackend, category: i32, item: i32) -> QString;
        #[qinvokable]
        #[cxx_name = "itemTitle"]
        fn item_title(self: &InspectorBackend, category: i32, item: i32) -> QString;
        #[qinvokable]
        #[cxx_name = "itemFocusAvailable"]
        fn item_focus_available(self: &InspectorBackend, category: i32, item: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "itemFocused"]
        fn item_focused(self: &InspectorBackend, category: i32, item: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "focusItem"]
        fn focus_item(self: Pin<&mut InspectorBackend>, category: i32, item: i32);
        #[qinvokable]
        #[cxx_name = "focusItemBody"]
        fn focus_item_body(self: Pin<&mut InspectorBackend>, category: i32, item: i32);
        #[qinvokable]
        #[cxx_name = "focusControl"]
        fn focus_control(self: Pin<&mut InspectorBackend>, category: i32, item: i32, control: i32);
        #[qinvokable]
        #[cxx_name = "itemExpanded"]
        fn item_expanded(self: &InspectorBackend, category: i32, item: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "setItemExpanded"]
        fn set_item_expanded(
            self: Pin<&mut InspectorBackend>,
            category: i32,
            item: i32,
            expanded: bool,
        );
        #[qinvokable]
        #[cxx_name = "itemResetAvailable"]
        fn item_reset_available(self: &InspectorBackend, category: i32, item: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "resetItem"]
        fn reset_item(self: Pin<&mut InspectorBackend>, category: i32, item: i32);

        #[qinvokable]
        #[cxx_name = "itemHasToggle"]
        fn item_has_toggle(self: &InspectorBackend, category: i32, item: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "itemToggleActive"]
        fn item_toggle_active(self: &InspectorBackend, category: i32, item: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "itemToggleTooltip"]
        fn item_toggle_tooltip(self: &InspectorBackend, category: i32, item: i32) -> QString;
        #[qinvokable]
        #[cxx_name = "setItemToggle"]
        fn set_item_toggle(
            self: Pin<&mut InspectorBackend>,
            category: i32,
            item: i32,
            active: bool,
        );
        #[qinvokable]
        #[cxx_name = "itemHasButtonToggle"]
        fn item_has_button_toggle(self: &InspectorBackend, category: i32, item: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "itemButtonToggleActive"]
        fn item_button_toggle_active(self: &InspectorBackend, category: i32, item: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "itemButtonToggleIcon"]
        fn item_button_toggle_icon(self: &InspectorBackend, category: i32, item: i32) -> QString;
        #[qinvokable]
        #[cxx_name = "itemButtonToggleTooltip"]
        fn item_button_toggle_tooltip(self: &InspectorBackend, category: i32, item: i32)
        -> QString;
        #[qinvokable]
        #[cxx_name = "setItemButtonToggle"]
        fn set_item_button_toggle(
            self: Pin<&mut InspectorBackend>,
            category: i32,
            item: i32,
            active: bool,
        );
        #[qinvokable]
        #[cxx_name = "itemActionCount"]
        fn item_action_count(self: &InspectorBackend, category: i32, item: i32) -> i32;
        #[qinvokable]
        #[cxx_name = "itemActionIcon"]
        fn item_action_icon(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            action: i32,
        ) -> QString;
        #[qinvokable]
        #[cxx_name = "itemActionTooltip"]
        fn item_action_tooltip(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            action: i32,
        ) -> QString;
        #[qinvokable]
        #[cxx_name = "itemActionSensitive"]
        fn item_action_sensitive(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            action: i32,
        ) -> bool;
        #[qinvokable]
        #[cxx_name = "triggerItemAction"]
        fn trigger_item_action(
            self: Pin<&mut InspectorBackend>,
            category: i32,
            item: i32,
            action: i32,
        );

        #[qinvokable]
        #[cxx_name = "controlCount"]
        fn control_count(self: &InspectorBackend, category: i32, item: i32) -> i32;
        #[qinvokable]
        #[cxx_name = "controlKind"]
        fn control_kind(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> InspectorControlKind;
        #[qinvokable]
        #[cxx_name = "controlRowRole"]
        fn control_row_role(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> InspectorControlRowRole;
        #[qinvokable]
        #[cxx_name = "controlRowMember"]
        fn control_row_member(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
            role: InspectorControlRowRole,
        ) -> i32;
        #[qinvokable]
        #[cxx_name = "controlLabel"]
        fn control_label(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> QString;
        #[qinvokable]
        #[cxx_name = "controlTransformLive"]
        fn control_transform_live(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> bool;
        #[qinvokable]
        #[cxx_name = "controlSubtitle"]
        fn control_subtitle(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> QString;
        #[qinvokable]
        #[cxx_name = "controlTooltip"]
        fn control_tooltip(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> QString;
        #[qinvokable]
        #[cxx_name = "controlValue"]
        fn control_value(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> QString;
        #[qinvokable]
        #[cxx_name = "controlComponent"]
        fn control_component(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
            component: i32,
        ) -> f64;
        #[qinvokable]
        #[cxx_name = "controlColor"]
        fn control_color(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> QStringList;
        #[qinvokable]
        #[cxx_name = "controlComponentText"]
        fn control_component_text(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
            component: i32,
        ) -> QString;
        #[qinvokable]
        #[cxx_name = "controlEditable"]
        fn control_editable(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> bool;
        #[qinvokable]
        #[cxx_name = "controlSensitive"]
        fn control_sensitive(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> bool;
        #[qinvokable]
        #[cxx_name = "controlVisible"]
        fn control_visible(self: &InspectorBackend, category: i32, item: i32, control: i32)
        -> bool;
        #[qinvokable]
        #[cxx_name = "controlBusy"]
        fn control_busy(self: &InspectorBackend, category: i32, item: i32, control: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "controlMinimum"]
        fn control_minimum(self: &InspectorBackend, category: i32, item: i32, control: i32) -> f64;
        #[qinvokable]
        #[cxx_name = "controlMaximum"]
        fn control_maximum(self: &InspectorBackend, category: i32, item: i32, control: i32) -> f64;
        #[qinvokable]
        #[cxx_name = "controlDragStep"]
        fn control_drag_step(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> f64;
        #[qinvokable]
        #[cxx_name = "controlDigits"]
        fn control_digits(self: &InspectorBackend, category: i32, item: i32, control: i32) -> i32;
        #[qinvokable]
        #[cxx_name = "controlUnit"]
        fn control_unit(self: &InspectorBackend, category: i32, item: i32, control: i32)
        -> QString;
        #[qinvokable]
        #[cxx_name = "controlWidthCharacters"]
        fn control_width_characters(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> i32;
        #[qinvokable]
        #[cxx_name = "controlPrefixIcon"]
        fn control_prefix_icon(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> QString;
        #[qinvokable]
        #[cxx_name = "controlHasAction"]
        fn control_has_action(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> bool;
        #[qinvokable]
        #[cxx_name = "controlActionIcon"]
        fn control_action_icon(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> QString;
        #[qinvokable]
        #[cxx_name = "controlActionSensitive"]
        fn control_action_sensitive(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> bool;
        #[qinvokable]
        #[cxx_name = "controlActionTooltip"]
        fn control_action_tooltip(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> QString;
        #[qinvokable]
        #[cxx_name = "controlDragPayload"]
        fn control_drag_payload(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> QString;
        #[qinvokable]
        #[cxx_name = "controlPrefixIconRotates"]
        fn control_prefix_icon_rotates(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> bool;
        #[qinvokable]
        #[cxx_name = "controlPrefixIconRotationOffset"]
        fn control_prefix_icon_rotation_offset(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> f64;
        #[qinvokable]
        #[cxx_name = "controlPrefix"]
        fn control_prefix(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
            component: i32,
        ) -> QString;
        #[qinvokable]
        #[cxx_name = "controlLock"]
        fn control_lock(self: &InspectorBackend, category: i32, item: i32, control: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "controlWithAlpha"]
        fn control_with_alpha(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> bool;
        #[qinvokable]
        #[cxx_name = "controlChoiceValues"]
        fn control_choice_values(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> QStringList;
        #[qinvokable]
        #[cxx_name = "controlChoiceLabels"]
        fn control_choice_labels(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> QStringList;
        #[qinvokable]
        #[cxx_name = "controlChoiceIcons"]
        fn control_choice_icons(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> QStringList;
        #[qinvokable]
        #[cxx_name = "controlChoiceSearchTerms"]
        fn control_choice_search_terms(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> QStringList;
        #[qinvokable]
        #[cxx_name = "openFontBrowser"]
        fn open_font_browser(self: Pin<&mut InspectorBackend>);
        #[qinvokable]
        #[cxx_name = "searchFontBrowser"]
        fn search_font_browser(self: Pin<&mut InspectorBackend>, query: &QString);
        #[qinvokable]
        #[cxx_name = "requestFontBrowserPreviews"]
        fn request_font_browser_previews(self: Pin<&mut InspectorBackend>, first: i32, count: i32);
        #[qinvokable]
        #[cxx_name = "fontBrowserCount"]
        fn font_browser_count(self: &InspectorBackend) -> i32;
        #[qinvokable]
        #[cxx_name = "fontBrowserLabel"]
        fn font_browser_label(self: &InspectorBackend, choice: i32) -> QString;
        #[qinvokable]
        #[cxx_name = "fontBrowserValue"]
        fn font_browser_value(self: &InspectorBackend, choice: i32) -> QString;
        #[qinvokable]
        #[cxx_name = "fontBrowserGoogle"]
        fn font_browser_google(self: &InspectorBackend, choice: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "fontBrowserPreviewSource"]
        fn font_browser_preview_source(self: &InspectorBackend, choice: i32) -> QUrl;
        #[qinvokable]
        #[cxx_name = "fontBrowserStatus"]
        fn font_browser_status(self: &InspectorBackend) -> QString;
        #[qinvokable]
        #[cxx_name = "fontBrowserBusy"]
        fn font_browser_busy(self: &InspectorBackend) -> bool;
        #[qinvokable]
        #[cxx_name = "fontListWithChoice"]
        fn font_list_with_choice(
            self: &InspectorBackend,
            value: &QString,
            index: i32,
            family: &QString,
        ) -> QString;
        #[qinvokable]
        #[cxx_name = "moveFontListValue"]
        fn move_font_list_value(
            self: &InspectorBackend,
            value: &QString,
            index: i32,
            offset: i32,
        ) -> QString;
        #[qinvokable]
        #[cxx_name = "removeFontListValue"]
        fn remove_font_list_value(self: &InspectorBackend, value: &QString, index: i32) -> QString;
        #[qinvokable]
        #[cxx_name = "activateControlFont"]
        fn activate_control_font(
            self: Pin<&mut InspectorBackend>,
            category: i32,
            item: i32,
            control: i32,
            family: &QString,
            value: &QString,
        );
        #[qinvokable]
        #[cxx_name = "cancelControlFontActivation"]
        fn cancel_control_font_activation(self: &InspectorBackend);
        #[qinvokable]
        #[cxx_name = "controlKeyframes"]
        fn control_keyframes(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> bool;
        #[qinvokable]
        #[cxx_name = "controlExpression"]
        fn control_expression(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> bool;
        #[qinvokable]
        #[cxx_name = "controlExpressionSource"]
        fn control_expression_source(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> QString;
        #[qinvokable]
        #[cxx_name = "controlExpressionResult"]
        fn control_expression_result(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> QStringList;
        #[qinvokable]
        #[cxx_name = "expressionDiagnostic"]
        fn expression_diagnostic(self: Pin<&mut InspectorBackend>, source: &QString)
        -> QStringList;
        #[qinvokable]
        #[cxx_name = "expressionDiagnosticDebounce"]
        fn expression_diagnostic_debounce(self: &InspectorBackend) -> i32;
        #[qinvokable]
        #[cxx_name = "expressionCompletionDebounce"]
        fn expression_completion_debounce(self: &InspectorBackend) -> i32;
        #[qinvokable]
        #[cxx_name = "controlExpressionCompletion"]
        fn control_expression_completion(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
            source: &QString,
            cursor: i32,
            automatic: bool,
        ) -> QStringList;
        #[qinvokable]
        #[cxx_name = "controlGraphPointTimes"]
        fn control_graph_point_times(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> QStringList;
        #[qinvokable]
        #[cxx_name = "controlGraphPointValues"]
        fn control_graph_point_values(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> QStringList;
        #[qinvokable]
        #[cxx_name = "controlGraphSegments"]
        fn control_graph_segments(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> QStringList;
        #[qinvokable]
        #[cxx_name = "controlGraphTiming"]
        fn control_graph_timing(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> QStringList;
        #[qinvokable]
        #[cxx_name = "controlGraphPlayhead"]
        fn control_graph_playhead(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
        ) -> QStringList;

        #[qinvokable]
        #[cxx_name = "setControlValue"]
        fn set_control_value(
            self: Pin<&mut InspectorBackend>,
            category: i32,
            item: i32,
            control: i32,
            value: &QString,
        );
        #[qinvokable]
        #[cxx_name = "setControlFraction"]
        fn set_control_fraction(
            self: Pin<&mut InspectorBackend>,
            category: i32,
            item: i32,
            control: i32,
            numerator: i64,
            denominator: i64,
        );
        #[qinvokable]
        #[cxx_name = "setControlComponents"]
        fn set_control_components(
            self: Pin<&mut InspectorBackend>,
            category: i32,
            item: i32,
            control: i32,
            first: f64,
            second: f64,
            third: f64,
            changed: i32,
        );
        #[qinvokable]
        #[cxx_name = "setControlColor"]
        fn set_control_color(
            self: Pin<&mut InspectorBackend>,
            category: i32,
            item: i32,
            control: i32,
            red: f64,
            green: f64,
            blue: f64,
            alpha: f64,
        );
        #[qinvokable]
        #[cxx_name = "commitControl"]
        fn commit_control(self: Pin<&mut InspectorBackend>, category: i32, item: i32, control: i32);
        #[qinvokable]
        #[cxx_name = "setControlKeyframes"]
        fn set_control_keyframes(
            self: Pin<&mut InspectorBackend>,
            category: i32,
            item: i32,
            control: i32,
            enabled: bool,
        );
        #[qinvokable]
        #[cxx_name = "setControlExpression"]
        fn set_control_expression(
            self: Pin<&mut InspectorBackend>,
            category: i32,
            item: i32,
            control: i32,
            enabled: bool,
        );
        #[qinvokable]
        #[cxx_name = "setControlExpressionSource"]
        fn set_control_expression_source(
            self: Pin<&mut InspectorBackend>,
            category: i32,
            item: i32,
            control: i32,
            source: &QString,
        );
        #[qinvokable]
        #[cxx_name = "seekControlGraph"]
        fn seek_control_graph(
            self: Pin<&mut InspectorBackend>,
            category: i32,
            item: i32,
            control: i32,
            numerator: i64,
            denominator: i64,
        );
        #[qinvokable]
        #[cxx_name = "moveControlGraphKeys"]
        fn move_control_graph_keys(
            self: Pin<&mut InspectorBackend>,
            category: i32,
            item: i32,
            control: i32,
            old_times: &QStringList,
            times: &QStringList,
            values: &QStringList,
        ) -> QStringList;
        #[qinvokable]
        #[cxx_name = "deleteControlGraphKeys"]
        fn delete_control_graph_keys(
            self: Pin<&mut InspectorBackend>,
            category: i32,
            item: i32,
            control: i32,
            times: &QStringList,
        );
        #[qinvokable]
        #[cxx_name = "addControlGraphKey"]
        fn add_control_graph_key(
            self: Pin<&mut InspectorBackend>,
            category: i32,
            item: i32,
            control: i32,
            numerator: i64,
            denominator: i64,
        );
        #[qinvokable]
        #[cxx_name = "copyControlGraphKeys"]
        fn copy_control_graph_keys(
            self: Pin<&mut InspectorBackend>,
            category: i32,
            item: i32,
            control: i32,
            times: &QStringList,
        ) -> bool;
        #[qinvokable]
        #[cxx_name = "pasteControlGraphKeys"]
        fn paste_control_graph_keys(
            self: Pin<&mut InspectorBackend>,
            category: i32,
            item: i32,
            control: i32,
            numerator: i64,
            denominator: i64,
        );
        #[qinvokable]
        #[cxx_name = "setControlGraphInterpolation"]
        fn set_control_graph_interpolation(
            self: Pin<&mut InspectorBackend>,
            category: i32,
            item: i32,
            control: i32,
            owner_id: &QString,
            interpolation: i32,
        );
        #[qinvokable]
        #[cxx_name = "textInterpolationLabels"]
        fn text_interpolation_labels(self: &InspectorBackend) -> QStringList;
        #[qinvokable]
        #[cxx_name = "textInterpolationTooltips"]
        fn text_interpolation_tooltips(self: &InspectorBackend) -> QStringList;
        #[qinvokable]
        #[cxx_name = "controlGraphTextInterpolation"]
        fn control_graph_text_interpolation(
            self: &InspectorBackend,
            category: i32,
            item: i32,
            control: i32,
            owner_id: &QString,
        ) -> i32;
        #[qinvokable]
        #[cxx_name = "setControlGraphTextInterpolation"]
        fn set_control_graph_text_interpolation(
            self: Pin<&mut InspectorBackend>,
            category: i32,
            item: i32,
            control: i32,
            owner_id: &QString,
            interpolation: i32,
        );
        #[qinvokable]
        #[cxx_name = "toggleControlGraphPlayback"]
        fn toggle_control_graph_playback(self: Pin<&mut InspectorBackend>);
        #[qinvokable]
        #[cxx_name = "triggerControlAction"]
        fn trigger_control_action(
            self: Pin<&mut InspectorBackend>,
            category: i32,
            item: i32,
            control: i32,
        );
        #[qinvokable]
        #[cxx_name = "triggerSecondaryControlAction"]
        fn trigger_secondary_control_action(
            self: Pin<&mut InspectorBackend>,
            category: i32,
            item: i32,
            control: i32,
        );
        #[qinvokable]
        #[cxx_name = "showControlPath"]
        fn show_control_path(
            self: Pin<&mut InspectorBackend>,
            category: i32,
            item: i32,
            control: i32,
        );
        #[qinvokable]
        #[cxx_name = "applyProjectSettings"]
        fn apply_project_settings(
            self: Pin<&mut InspectorBackend>,
            width: i32,
            height: i32,
            fps_numerator: &QString,
            fps_denominator: &QString,
        );

        #[qsignal]
        #[cxx_name = "showError"]
        fn show_error(self: Pin<&mut InspectorBackend>, body: QString);
        #[qsignal]
        #[cxx_name = "showConfirmation"]
        fn show_confirmation(self: Pin<&mut InspectorBackend>, body: QString);
        #[qsignal]
        #[cxx_name = "openPath"]
        fn open_path(self: Pin<&mut InspectorBackend>, url: QUrl);
    }

    impl cxx_qt::Initialize for InspectorBackend {}
}

#[derive(Default)]
pub struct InspectorBackendRust {
    ready: bool,
    revision: i32,
    cache_revision: i32,
    analysis_revision: i32,
    document_revision: i32,
    expression_revision: i32,
    graph_revision: i32,
    playhead_revision: i32,
    transform_revision: i32,
    font_browser_revision: i32,
    active_category: i32,
    scroll_position: f64,
    title: QString,
    document: Option<InspectorDocument>,
    list_state: InspectorListState,
    stabilization_generating: Option<bool>,
    resolved_transform: Option<shrimply_project::project::ResolvedTransform>,
    transform_live: Option<shrimply_inspector_core::transform::TransformLivePresentation>,
    analysis_controls: HashMap<AnalysisControlKey, CachedAnalysisControl>,
    expression_diagnostic_cache: shrimply_inspector_core::rhai_editor::DiagnosticCache,
}

impl cxx_qt::Initialize for qobject::InspectorBackend {
    fn initialize(self: Pin<&mut Self>) {}
}

impl qobject::InspectorBackend {
    fn transform_active(&self) -> bool {
        let Some(document) = self.document() else {
            return false;
        };
        self.category(*self.active_category())
            .is_some_and(|category| {
                category.key == "visual"
                    && category.items.iter().any(|entry| {
                        let InspectorListItem::Item(item) = entry else {
                            return false;
                        };
                        self.rust()
                            .list_state
                            .expanded(&document.target, &item.presentation.key)
                            && crate::graph_backend::has_transform_controls(&item.section)
                    })
            })
    }

    fn cached_analysis_control(
        &self,
        category: i32,
        item: i32,
        control: i32,
    ) -> Option<&shrimply_inspector_core::AnalysisControlPresentation> {
        let key = analysis_control_key(category, item, control)?;
        let document = self.document()?;
        let control = self.control(category, item, control)?;
        let cached = self.rust().analysis_controls.get(&key)?;
        (control.kind == ControlKind::Analysis
            && control.action == Some(cached.action)
            && cached.target == document.target)
            .then_some(&cached.presentation)
    }

    pub fn poll_analysis_control(mut self: Pin<&mut Self>, category: i32, item: i32, control: i32) {
        let Some(key) = analysis_control_key(category, item, control) else {
            return;
        };
        let next = self
            .control_target(category, item, control)
            .and_then(|(target, control)| {
                (control.kind == ControlKind::Analysis).then_some((target, control))
            })
            .and_then(|(target, control)| {
                Some(CachedAnalysisControl {
                    target: target.clone(),
                    action: control.action?,
                    presentation: analysis_presentation(&control, Some(&target))?,
                })
            });
        let (changed, camera_changed, refresh_analysis_output) = match next {
            Some(next) => {
                let previous = self.rust().analysis_controls.get(&key);
                let refresh_analysis_output = super::with_controller(|controller| {
                    Ok(controller.observe_analysis_transition(
                        &next.target,
                        next.action,
                        &next.presentation,
                    ))
                })
                .expect("Qt analysis control polled before inspector installation");
                let changed = previous != Some(&next);
                let camera_changed = changed
                    && next.action
                        == shrimply_inspector_core::InspectorControlAction::ToggleCameraAnalysis;
                if changed {
                    self.as_mut().rust_mut().analysis_controls.insert(key, next);
                }
                (changed, camera_changed, refresh_analysis_output)
            }
            None => (
                self.as_mut()
                    .rust_mut()
                    .analysis_controls
                    .remove(&key)
                    .is_some(),
                false,
                false,
            ),
        };
        if changed {
            let revision = self.analysis_revision().wrapping_add(1);
            self.as_mut().set_analysis_revision(revision);
        }
        if camera_changed {
            super::mark_dirty();
        }
        if refresh_analysis_output {
            super::with_controller(|controller| {
                controller.refresh_analysis_output();
                Ok(())
            })
            .expect("Qt analysis control polled before inspector installation");
        }
    }

    pub fn minimum_width(&self) -> i32 {
        shrimply_inspector_core::INSPECTOR_MIN_WIDTH
    }

    pub fn poll(mut self: Pin<&mut Self>, scroll_position: f64) {
        let (font_browser_changed, font_edits) = super::receive_font_browser();
        for (edit, activation) in font_edits {
            let result = activation.and_then(|()| super::apply_font_browser_edit(&edit));
            self.as_mut().finish(result);
        }
        if font_browser_changed {
            let revision = self.font_browser_revision().wrapping_add(1);
            self.as_mut().set_font_browser_revision(revision);
        }
        if self.target_change_pending() {
            super::mark_dirty();
        }
        let generating = self
            .document()
            .and_then(|document| super::video_stabilization_generating(&document.target));
        let previous = self.rust().stabilization_generating;
        self.as_mut().rust_mut().stabilization_generating = generating;
        if previous.is_some() && previous != generating {
            super::mark_dirty();
        }
        let document = super::take_document();
        let cache_dirty = super::take_cache_dirty();
        let expression_dirty = super::take_expression_dirty();
        let graph_dirty = super::take_graph_dirty();
        let playhead_dirty = super::take_playhead_dirty();
        let transform_dirty = super::take_transform_dirty();
        let focus_dirty = super::take_focus_dirty();
        let Some(document) = document else {
            if cache_dirty {
                let revision = self.cache_revision().wrapping_add(1);
                self.as_mut().set_cache_revision(revision);
            }
            if expression_dirty {
                let revision = self.expression_revision().wrapping_add(1);
                self.as_mut().set_expression_revision(revision);
            }
            if graph_dirty {
                let target = self.document().map(|document| document.target.clone());
                if let Some(target) = target
                    && let Some(document) = self.as_mut().rust_mut().document.as_mut()
                {
                    crate::graph_backend::update_control_graphs(document, &target);
                }
                let revision = self.graph_revision().wrapping_add(1);
                self.as_mut().set_graph_revision(revision);
            }
            let transform_active = self.transform_active();
            if (playhead_dirty || transform_dirty) && transform_active {
                let target = self.document().map(|document| document.target.clone());
                let live = target.as_ref().and_then(super::transform_live_presentation);
                self.as_mut().rust_mut().resolved_transform = live
                    .as_ref()
                    .map(|presentation| presentation.resolved)
                    .or_else(|| target.as_ref().and_then(super::resolved_transform));
                if let Some(live) = &live
                    && let Some(document) = self.as_mut().rust_mut().document.as_mut()
                {
                    crate::graph_backend::update_transform_graphs(document, live);
                }
                self.as_mut().rust_mut().transform_live = live;
                let revision = self.transform_revision().wrapping_add(1);
                self.as_mut().set_transform_revision(revision);
            }
            if playhead_dirty {
                let revision = self.playhead_revision().wrapping_add(1);
                self.as_mut().set_playhead_revision(revision);
            }
            if focus_dirty {
                let revision = self.revision().wrapping_add(1);
                self.as_mut().set_revision(revision);
            }
            return;
        };
        if let Some(target) = self.document().map(|document| document.target.clone()) {
            self.as_mut()
                .rust_mut()
                .list_state
                .set_scroll_position(&target, scroll_position);
        }
        let remembered = self.rust().list_state.active_category(&document.target);
        let active = document
            .categories
            .iter()
            .position(|category| remembered == Some(category.key))
            .unwrap_or_default();
        let scroll_position = self.rust().list_state.scroll_position(&document.target);
        let active = i32::try_from(active).expect("inspector category index exceeds Qt limits");
        let title = QString::from(document.title.as_str());
        let analysis_controls = analysis_control_cache(&document);
        let refresh_analysis_output = super::with_controller(|controller| {
            let mut refresh = false;
            for control in analysis_controls.values() {
                refresh |= controller.observe_analysis_transition(
                    &control.target,
                    control.action,
                    &control.presentation,
                );
            }
            Ok(refresh)
        })
        .expect("Qt analysis document rebuilt before inspector installation");
        let analysis_changed = self.rust().analysis_controls != analysis_controls;
        let transform_live = super::transform_live_presentation(&document.target);
        let resolved_transform = transform_live
            .as_ref()
            .map(|presentation| presentation.resolved)
            .or_else(|| super::resolved_transform(&document.target));
        let revision = self.revision().wrapping_add(1);
        let document_revision = self.document_revision().wrapping_add(1);
        self.as_mut().rust_mut().document = Some(document);
        self.as_mut().rust_mut().analysis_controls = analysis_controls;
        self.as_mut().rust_mut().resolved_transform = resolved_transform;
        self.as_mut().rust_mut().transform_live = transform_live;
        self.as_mut().set_ready(true);
        self.as_mut().set_title(title);
        self.as_mut().set_active_category(active);
        self.as_mut().set_scroll_position(scroll_position);
        self.as_mut().set_document_revision(document_revision);
        if analysis_changed {
            let analysis_revision = self.analysis_revision().wrapping_add(1);
            self.as_mut().set_analysis_revision(analysis_revision);
        }
        self.as_mut().set_revision(revision);
        if refresh_analysis_output {
            super::with_controller(|controller| {
                controller.refresh_analysis_output();
                Ok(())
            })
            .expect("Qt analysis document rebuilt before inspector installation");
        }
    }

    pub fn destructive_background(&self) -> QColor {
        let color = shrimply_cross_ui_theme::current().destructive_bg;
        QColor::from_rgba_f(color.r, color.g, color.b, color.a)
    }

    pub fn target_change_pending(&self) -> bool {
        self.document()
            .is_some_and(|document| super::target_change_pending(&document.target))
    }

    pub fn destructive_foreground(&self) -> QColor {
        let color = shrimply_cross_ui_theme::current().destructive_fg;
        QColor::from_rgba_f(color.r, color.g, color.b, color.a)
    }

    pub fn keyframe_clipboard_marker(&self) -> QString {
        QString::from(shrimply_inspector_core::keyframe_model::KEYFRAME_CLIPBOARD_MARKER)
    }

    pub fn keyframe_snapping_enabled(&self) -> bool {
        super::keyframe_snapping().0
    }

    pub fn keyframe_snapping_radius(&self) -> f64 {
        super::keyframe_snapping().1
    }

    pub fn category_keys(&self) -> QStringList {
        let Some(document) = self.document() else {
            return QStringList::default();
        };
        document
            .categories
            .iter()
            .map(|category| QString::from(format!("{:?}:{}", document.target, category.key)))
            .collect()
    }

    pub fn category_label(&self, category: i32) -> QString {
        self.category(category)
            .map_or_else(QString::default, |category| {
                shrimply_i18n_qt::text(category.label)
            })
    }

    pub fn category_icon(&self, category: i32) -> QString {
        self.category(category)
            .map_or_else(QString::default, |category| QString::from(category.icon))
    }

    pub fn activate_category(mut self: Pin<&mut Self>, category: i32) {
        let Some(category_value) = self.category(category) else {
            return;
        };
        let key = category_value.key.to_string();
        let target = self
            .document()
            .expect("category has a document")
            .target
            .clone();
        self.as_mut()
            .rust_mut()
            .list_state
            .set_active_category(&target, &key);
        self.as_mut().set_active_category(category);
        if key == "visual" {
            super::mark_transform_dirty();
        }
    }

    pub fn item_keys(&self, category: i32) -> QStringList {
        let Some(document) = self.document() else {
            return QStringList::default();
        };
        let Some(category) = index(category).and_then(|index| document.categories.get(index))
        else {
            return QStringList::default();
        };
        category
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let key = match item {
                    InspectorListItem::Item(item) => item.presentation.key.as_str(),
                    InspectorListItem::Flat(_) => "flat",
                };
                QString::from(format!(
                    "{:?}:{}:{index}:{key}",
                    document.target, category.key
                ))
            })
            .collect()
    }

    pub fn item_is_card(&self, category: i32, item: i32) -> bool {
        matches!(self.item(category, item), Some(InspectorListItem::Item(_)))
    }

    pub fn item_title(&self, category: i32, item: i32) -> QString {
        self.card(category, item)
            .map_or_else(QString::default, |item| {
                shrimply_i18n_qt::text(&item.presentation.title)
            })
    }

    pub fn item_focus_available(&self, category: i32, item: i32) -> bool {
        self.document().is_some_and(|document| {
            document.preview_item.is_some() && self.card(category, item).is_some()
        })
    }

    pub fn item_focused(&self, category: i32, item: i32) -> bool {
        self.document()
            .zip(self.card(category, item))
            .is_some_and(|(document, item)| super::item_focused(document, item))
    }

    pub fn focus_item(self: Pin<&mut Self>, category: i32, item: i32) {
        if let Some((document, item)) = self.document().zip(self.card(category, item)) {
            super::focus_item(document, item);
        }
    }

    pub fn focus_item_body(self: Pin<&mut Self>, category: i32, item: i32) {
        if let Some((document, item)) = self.document().zip(self.card(category, item)) {
            if let Some(control) = item
                .section
                .controls
                .iter()
                .find(|control| control.preview_focus.is_some())
            {
                super::focus_control(document, item, control);
            } else {
                super::focus_item(document, item);
            }
        }
    }

    pub fn focus_control(self: Pin<&mut Self>, category: i32, item: i32, control: i32) {
        if let Some((document, item, control)) = self
            .document()
            .zip(self.card(category, item))
            .zip(self.control(category, item, control))
            .map(|((document, item), control)| (document, item, control))
        {
            super::focus_control(document, item, control);
        }
    }

    pub fn item_expanded(&self, category: i32, item: i32) -> bool {
        let Some(card) = self.card(category, item) else {
            return true;
        };
        let Some(document) = self.document() else {
            return false;
        };
        self.rust()
            .list_state
            .expanded(&document.target, &card.presentation.key)
    }

    pub fn item_identity(&self, category: i32, item: i32) -> QString {
        let Some(document) = self.document() else {
            return QString::default();
        };
        let Some(category) = index(category).and_then(|index| document.categories.get(index))
        else {
            return QString::default();
        };
        let Some(InspectorListItem::Item(item)) =
            index(item).and_then(|index| category.items.get(index))
        else {
            return QString::default();
        };
        QString::from(format!(
            "{:?}:{}:{}",
            document.target, category.key, item.presentation.key
        ))
    }

    pub fn set_item_expanded(mut self: Pin<&mut Self>, category: i32, item: i32, expanded: bool) {
        let Some(card) = self.card(category, item) else {
            return;
        };
        let key = card.presentation.key.clone();
        let transform = crate::graph_backend::has_transform_controls(&card.section);
        let target = self.document().expect("card has a document").target.clone();
        if self.rust().list_state.expanded(&target, &key) == expanded {
            return;
        }
        self.as_mut()
            .rust_mut()
            .list_state
            .set_expanded(&target, &key, expanded);
        if expanded && transform {
            super::mark_transform_dirty();
        }
        let revision = self.revision().wrapping_add(1);
        self.as_mut().set_revision(revision);
    }

    pub fn item_reset_available(&self, category: i32, item: i32) -> bool {
        self.card(category, item)
            .is_some_and(|item| item.reset.is_some())
    }

    pub fn reset_item(mut self: Pin<&mut Self>, category: i32, item: i32) {
        let action = self
            .card(category, item)
            .and_then(|item| item.reset.clone());
        self.as_mut().perform(action);
    }

    pub fn item_has_toggle(&self, category: i32, item: i32) -> bool {
        self.card(category, item)
            .is_some_and(|item| item.toggle.is_some())
    }

    pub fn item_toggle_active(&self, category: i32, item: i32) -> bool {
        self.card(category, item)
            .and_then(|item| item.toggle.as_ref())
            .is_some_and(|toggle| toggle.active)
    }

    pub fn item_toggle_tooltip(&self, category: i32, item: i32) -> QString {
        self.card(category, item)
            .and_then(|item| item.toggle.as_ref())
            .map_or_else(QString::default, |toggle| {
                shrimply_i18n_qt::text(toggle.tooltip)
            })
    }

    pub fn set_item_toggle(mut self: Pin<&mut Self>, category: i32, item: i32, active: bool) {
        let action = self
            .card(category, item)
            .and_then(|item| item.toggle.as_ref())
            .map(|toggle| boolean_action(toggle.activate.clone(), active));
        self.as_mut().perform(action);
    }

    pub fn item_has_button_toggle(&self, category: i32, item: i32) -> bool {
        self.card(category, item)
            .is_some_and(|item| item.button_toggle.is_some())
    }

    pub fn item_button_toggle_active(&self, category: i32, item: i32) -> bool {
        self.card(category, item)
            .and_then(|item| item.button_toggle.as_ref())
            .is_some_and(|toggle| toggle.active)
    }

    pub fn item_button_toggle_icon(&self, category: i32, item: i32) -> QString {
        self.card(category, item)
            .and_then(|item| item.button_toggle.as_ref())
            .map_or_else(QString::default, |toggle| QString::from(toggle.icon))
    }

    pub fn item_button_toggle_tooltip(&self, category: i32, item: i32) -> QString {
        self.card(category, item)
            .and_then(|item| item.button_toggle.as_ref())
            .map_or_else(QString::default, |toggle| {
                shrimply_i18n_qt::text(toggle.tooltip)
            })
    }

    pub fn set_item_button_toggle(
        mut self: Pin<&mut Self>,
        category: i32,
        item: i32,
        active: bool,
    ) {
        let action = self
            .card(category, item)
            .and_then(|item| item.button_toggle.as_ref())
            .map(|toggle| optional_action(toggle.activate.clone(), active));
        self.as_mut().perform(action);
    }

    pub fn item_action_count(&self, category: i32, item: i32) -> i32 {
        count(self.card(category, item).map(|item| item.actions.len()))
    }

    pub fn item_action_icon(&self, category: i32, item: i32, action: i32) -> QString {
        self.action(category, item, action)
            .map_or_else(QString::default, |action| QString::from(action.icon))
    }

    pub fn item_action_tooltip(&self, category: i32, item: i32, action: i32) -> QString {
        self.action(category, item, action)
            .map_or_else(QString::default, |action| {
                shrimply_i18n_qt::text(action.tooltip)
            })
    }

    pub fn item_action_sensitive(&self, category: i32, item: i32, action: i32) -> bool {
        self.action(category, item, action)
            .is_some_and(|action| action.sensitive)
    }

    pub fn trigger_item_action(mut self: Pin<&mut Self>, category: i32, item: i32, action: i32) {
        let action = self
            .action(category, item, action)
            .map(|action| action.activate.clone());
        self.as_mut().perform(action);
    }

    pub fn control_count(&self, category: i32, item: i32) -> i32 {
        count(
            self.section(category, item)
                .map(|section| section.controls.len()),
        )
    }

    pub fn control_kind(
        &self,
        category: i32,
        item: i32,
        control: i32,
    ) -> qobject::InspectorControlKind {
        use qobject::InspectorControlKind as QtKind;
        self.control(category, item, control)
            .map_or(QtKind::ReadOnly, |control| match control.kind {
                ControlKind::Boolean => QtKind::Boolean,
                ControlKind::Number => QtKind::Number,
                ControlKind::Fraction => QtKind::Fraction,
                ControlKind::Text => QtKind::Text,
                ControlKind::MultilineText => QtKind::MultilineText,
                ControlKind::ReadOnly => QtKind::ReadOnly,
                ControlKind::Color => QtKind::Color,
                ControlKind::LayeredColor => QtKind::LayeredColor,
                ControlKind::LayeredText => QtKind::LayeredText,
                ControlKind::LayeredDrawing => QtKind::LayeredDrawing,
                ControlKind::FontFamilies => QtKind::FontFamilies,
                ControlKind::Selector
                | ControlKind::OptionalSelector
                | ControlKind::OptionalNumberSelector => QtKind::Selector,
                ControlKind::Vector2 => QtKind::Vector2,
                ControlKind::Vector3 => QtKind::Vector3,
                ControlKind::LayeredNumber => QtKind::LayeredNumber,
                ControlKind::LayeredVector2 => QtKind::LayeredVector2,
                ControlKind::LayeredVector3 => QtKind::LayeredVector3,
                ControlKind::ProjectSettings => QtKind::ProjectSettings,
                ControlKind::Performance => QtKind::Performance,
                ControlKind::LayeredBoolean => QtKind::LayeredBoolean,
                ControlKind::LayeredSelector => QtKind::LayeredSelector,
                ControlKind::AudioCache => QtKind::AudioCache,
                ControlKind::AudioCachePreset => QtKind::AudioCachePreset,
                ControlKind::VisualCache => QtKind::VisualCache,
                ControlKind::VisualCacheQuality => QtKind::VisualCacheQuality,
                ControlKind::Analysis => QtKind::Analysis,
                ControlKind::AudioModifierMenu | ControlKind::VisualModifierMenu => {
                    QtKind::ModifierMenu
                }
                ControlKind::TtsEditor => QtKind::TtsEditor,
                ControlKind::BeatDetection => QtKind::BeatDetection,
                ControlKind::InfoHeading => QtKind::InfoHeading,
                ControlKind::InfoArtwork => QtKind::InfoArtwork,
                ControlKind::FileLocation => QtKind::FileLocation,
                ControlKind::InfoLoading => QtKind::InfoLoading,
                ControlKind::Action => QtKind::Action,
            })
    }

    pub fn control_row_role(
        &self,
        category: i32,
        item: i32,
        control: i32,
    ) -> qobject::InspectorControlRowRole {
        use qobject::InspectorControlRowRole as QtRole;
        self.control(category, item, control)
            .map_or(QtRole::Standalone, |control| match control.row_role {
                ControlRowRole::Standalone => QtRole::Standalone,
                ControlRowRole::Primary => QtRole::Primary,
                ControlRowRole::Auxiliary => QtRole::Auxiliary,
                ControlRowRole::TrailingAction => QtRole::TrailingAction,
            })
    }

    pub fn control_row_member(
        &self,
        category: i32,
        item: i32,
        control: i32,
        role: qobject::InspectorControlRowRole,
    ) -> i32 {
        use qobject::InspectorControlRowRole as QtRole;
        let role = match role {
            QtRole::Standalone => ControlRowRole::Standalone,
            QtRole::Primary => ControlRowRole::Primary,
            QtRole::Auxiliary => ControlRowRole::Auxiliary,
            QtRole::TrailingAction => ControlRowRole::TrailingAction,
            _ => panic!("Qt passed an invalid inspector control row role"),
        };
        let Some((section, group)) = self.section(category, item).zip(
            self.control(category, item, control)
                .and_then(|control| control.row_group),
        ) else {
            return MISSING_CONTROL_INDEX;
        };
        section
            .controls
            .iter()
            .position(|control| control.row_group == Some(group) && control.row_role == role)
            .and_then(|index| i32::try_from(index).ok())
            .unwrap_or(MISSING_CONTROL_INDEX)
    }

    pub fn control_label(&self, category: i32, item: i32, control: i32) -> QString {
        self.control(category, item, control)
            .map_or_else(QString::default, |control| QString::from(&control.label))
    }

    pub fn control_transform_live(&self, category: i32, item: i32, control: i32) -> bool {
        self.control(category, item, control)
            .is_some_and(|control| crate::graph_backend::is_transform_path(&control.path))
    }

    pub fn control_subtitle(&self, category: i32, item: i32, control: i32) -> QString {
        self.control(category, item, control)
            .map_or_else(QString::default, |control| QString::from(&control.subtitle))
    }

    pub fn control_tooltip(&self, category: i32, item: i32, control: i32) -> QString {
        let control_index = control;
        self.control(category, item, control)
            .map_or_else(QString::default, |control| {
                if control.kind == ControlKind::Analysis {
                    return self
                        .cached_analysis_control(category, item, control_index)
                        .map_or_else(
                            || shrimply_i18n_qt::text(&control.tooltip),
                            analysis_tooltip,
                        );
                }
                control.target_id.map_or_else(
                    || shrimply_i18n_qt::text(&control.tooltip),
                    |id| {
                        if matches!(
                            control.kind,
                            ControlKind::AudioCache | ControlKind::VisualCache
                        ) {
                            QString::from(
                                super::tracked_cache_control(control.kind, id).map_or_else(
                                    || control.tooltip.clone(),
                                    |status| status.tooltip,
                                ),
                            )
                        } else {
                            shrimply_i18n_qt::text(&control.tooltip)
                        }
                    },
                )
            })
    }

    pub fn control_value(&self, category: i32, item: i32, control: i32) -> QString {
        let control_index = control;
        let target = self.document().map(|document| document.target.clone());
        self.control(category, item, control)
            .map_or_else(QString::default, |control| {
                if control.kind == ControlKind::Fraction {
                    QString::from(fraction_value(control).to_string())
                } else if control.kind == ControlKind::LayeredNumber {
                    let cached = self
                        .rust()
                        .transform_live
                        .as_ref()
                        .and_then(|live| live.number(&control.path));
                    let value = cached
                        .or_else(|| {
                            target.as_ref().and_then(|target| {
                                super::timeline_number_value(
                                    target,
                                    control.audio_modifier,
                                    control.target_id,
                                    control.timeline_id,
                                    control.timeline_path.as_deref().unwrap_or(&control.path),
                                )
                                .ok()
                            })
                        })
                        .map(|value| control.display_number(value))
                        .unwrap_or_else(|| control.value.parse::<f64>().unwrap_or_default());
                    QString::from(value.to_string())
                } else if matches!(
                    control.kind,
                    ControlKind::AudioCache | ControlKind::VisualCache
                ) {
                    control
                        .target_id
                        .and_then(|id| super::tracked_cache_control(control.kind, id))
                        .map_or_else(
                            || shrimply_i18n_qt::text(&control.value),
                            |status| shrimply_i18n_qt::text(status.label),
                        )
                } else if control.kind == ControlKind::Analysis {
                    QString::from(
                        self.cached_analysis_control(category, item, control_index)
                            .map_or(control.value.as_str(), |status| status.label.as_str()),
                    )
                } else {
                    QString::from(control.value.as_str())
                }
            })
    }

    pub fn control_component(&self, category: i32, item: i32, control: i32, component: i32) -> f64 {
        let control_index = control;
        let target = self.document().map(|document| document.target.clone());
        self.control(category, item, control)
            .and_then(|control| {
                if control.kind == ControlKind::Analysis && component >= 0 {
                    let status = self.cached_analysis_control(category, item, control_index)?;
                    return match component {
                        0 => Some(status.progress),
                        1 => Some(f64::from(u8::from(status.running))),
                        2 => Some(f64::from(u8::from(status.cancelling))),
                        3 => Some(f64::from(u8::from(status.suggested))),
                        _ => None,
                    };
                }
                if matches!(
                    control.kind,
                    ControlKind::AudioCache | ControlKind::VisualCache
                ) {
                    let status = control
                        .target_id
                        .and_then(|id| super::tracked_cache_control(control.kind, id))
                        .and_then(|status| match component {
                            0 => Some(status.progress),
                            1 => Some(f64::from(u8::from(status.baking))),
                            _ => None,
                        });
                    return status.or_else(|| {
                        control
                            .components
                            .get(usize::try_from(component).ok()?)?
                            .parse()
                            .ok()
                    });
                }
                if control.kind == ControlKind::LayeredVector2 {
                    let value = self
                        .rust()
                        .transform_live
                        .as_ref()
                        .and_then(|live| live.vector(&control.path))
                        .or_else(|| {
                            let transform = self.rust().resolved_transform?;
                            match control.path.as_str() {
                                "/transform/position" => Some(transform.position),
                                "/transform/anchor" => Some(transform.anchor),
                                "/transform/scale" => Some(transform.scale),
                                "/transform/shear" => Some(transform.shear),
                                _ => None,
                            }
                        })
                        .or_else(|| {
                            target.as_ref().and_then(|target| {
                                super::timeline_vector2_value(
                                    target,
                                    control.timeline_id,
                                    control.timeline_path.as_deref().unwrap_or(&control.path),
                                )
                                .ok()
                            })
                        })
                        .or_else(|| {
                            Some(glam::Vec2::new(
                                control.components.first()?.parse().ok()?,
                                control.components.get(1)?.parse().ok()?,
                            ))
                        })?;
                    return match component {
                        0 => Some(f64::from(value.x)),
                        1 => Some(f64::from(value.y)),
                        _ => None,
                    };
                }
                if control.kind == ControlKind::LayeredVector3 {
                    let timeline_id = control.timeline_id?;
                    let value = target
                        .as_ref()
                        .and_then(|target| {
                            super::timeline_vector3_value(
                                target,
                                timeline_id,
                                control.timeline_path.as_deref().unwrap_or(&control.path),
                            )
                            .ok()
                        })
                        .or_else(|| {
                            Some(glam::Vec3::new(
                                control.components.first()?.parse().ok()?,
                                control.components.get(1)?.parse().ok()?,
                                control.components.get(2)?.parse().ok()?,
                            ))
                        })?;
                    return match component {
                        0 => Some(f64::from(value.x)),
                        1 => Some(f64::from(value.y)),
                        2 => Some(f64::from(value.z)),
                        _ => None,
                    };
                }
                control.components.get(index(component)?)?.parse().ok()
            })
            .unwrap_or_default()
    }

    pub fn control_color(&self, category: i32, item: i32, control: i32) -> QStringList {
        let Some((target, control)) = self.control_target(category, item, control) else {
            return QStringList::default();
        };
        if control.kind != ControlKind::LayeredColor {
            return QStringList::default();
        }
        let Some(timeline_id) = control.timeline_id else {
            return QStringList::default();
        };
        super::color_value(&target, &control.path, timeline_id)
            .map(|color| [color.r, color.g, color.b, color.a])
            .unwrap_or_default()
            .into_iter()
            .map(|channel| QString::from(channel.to_string()))
            .collect()
    }

    pub fn control_component_text(
        &self,
        category: i32,
        item: i32,
        control: i32,
        component: i32,
    ) -> QString {
        self.control(category, item, control)
            .and_then(|control| control.components.get(index(component)?))
            .map_or_else(QString::default, QString::from)
    }

    pub fn control_editable(&self, category: i32, item: i32, control: i32) -> bool {
        self.control(category, item, control)
            .is_some_and(|control| control.editable)
    }

    pub fn control_sensitive(&self, category: i32, item: i32, control: i32) -> bool {
        let control_index = control;
        self.control(category, item, control)
            .is_some_and(|control| {
                if control.kind == ControlKind::Analysis {
                    return self
                        .cached_analysis_control(category, item, control_index)
                        .map_or(control.sensitive, |status| status.sensitive);
                }
                control.sensitive
                    && !(matches!(
                        control.kind,
                        ControlKind::AudioCachePreset | ControlKind::VisualCacheQuality
                    ) && control
                        .target_id
                        .and_then(|id| super::tracked_cache_control(control.kind, id))
                        .is_some_and(|status| status.baking))
            })
    }

    pub fn control_visible(&self, category: i32, item: i32, control: i32) -> bool {
        self.control(category, item, control)
            .is_some_and(|control| control.visible)
    }

    pub fn control_busy(&self, category: i32, item: i32, control: i32) -> bool {
        let control_index = control;
        self.control(category, item, control)
            .is_some_and(|control| {
                if control.kind == ControlKind::Analysis {
                    return self
                        .cached_analysis_control(category, item, control_index)
                        .map_or(control.busy, |status| status.active());
                }
                control.busy
                    || control.kind == ControlKind::BeatDetection
                        && control
                            .target_id
                            .is_some_and(shrimply_audio::beat::is_loading)
            })
    }

    pub fn show_control_path(mut self: Pin<&mut Self>, category: i32, item: i32, control: i32) {
        let result = self
            .control(category, item, control)
            .filter(|control| control.kind == ControlKind::FileLocation)
            .ok_or_else(|| "inspector control is not a file location".to_string())
            .and_then(|control| {
                shrimply_qt_components::desktop_open::prepare(
                    std::path::Path::new(&control.value),
                    None,
                )
            });
        match result {
            Ok(shrimply_qt_components::desktop_open::Action::Open(path)) => self
                .as_mut()
                .open_path(QUrl::from_local_file(&QString::from(
                    path.to_string_lossy().as_ref(),
                ))),
            Ok(shrimply_qt_components::desktop_open::Action::FocusRevealed(_)) => {}
            Err(error) => self.as_mut().show_error(QString::from(error)),
        }
    }

    pub fn control_minimum(&self, category: i32, item: i32, control: i32) -> f64 {
        self.control(category, item, control)
            .map_or(0.0, |control| control.number.minimum)
    }

    pub fn control_maximum(&self, category: i32, item: i32, control: i32) -> f64 {
        self.control(category, item, control)
            .map_or(0.0, |control| control.number.maximum)
    }

    pub fn control_drag_step(&self, category: i32, item: i32, control: i32) -> f64 {
        self.control(category, item, control)
            .map_or(1.0, |control| control.number.drag_step)
    }

    pub fn control_digits(&self, category: i32, item: i32, control: i32) -> i32 {
        self.control(category, item, control)
            .map_or(2, |control| control.number.digits)
    }

    pub fn control_unit(&self, category: i32, item: i32, control: i32) -> QString {
        self.control(category, item, control)
            .map_or_else(QString::default, |control| {
                QString::from(control.number.unit)
            })
    }

    pub fn control_width_characters(&self, category: i32, item: i32, control: i32) -> i32 {
        self.control(category, item, control)
            .map_or(8, |control| control.width_characters)
    }

    pub fn control_prefix_icon(&self, category: i32, item: i32, control: i32) -> QString {
        self.control(category, item, control)
            .map_or_else(QString::default, |control| {
                QString::from(control.prefix_icon.as_str())
            })
    }

    pub fn control_has_action(&self, category: i32, item: i32, control: i32) -> bool {
        self.control(category, item, control)
            .is_some_and(|control| control.action.is_some())
    }

    pub fn control_action_icon(&self, category: i32, item: i32, control: i32) -> QString {
        self.control(category, item, control)
            .map_or_else(QString::default, |control| {
                QString::from(control.action_icon.as_str())
            })
    }

    pub fn control_action_sensitive(&self, category: i32, item: i32, control: i32) -> bool {
        self.control(category, item, control)
            .is_some_and(|control| control.action_sensitive)
    }

    pub fn control_action_tooltip(&self, category: i32, item: i32, control: i32) -> QString {
        self.control(category, item, control)
            .map_or_else(QString::default, |control| {
                shrimply_i18n_qt::text(&control.action_tooltip)
            })
    }

    pub fn control_drag_payload(&self, category: i32, item: i32, control: i32) -> QString {
        self.control(category, item, control)
            .map_or_else(QString::default, |control| {
                QString::from(control.drag_payload.as_str())
            })
    }

    pub fn control_prefix_icon_rotates(&self, category: i32, item: i32, control: i32) -> bool {
        self.control(category, item, control)
            .is_some_and(|control| control.prefix_icon_rotates)
    }

    pub fn control_prefix_icon_rotation_offset(
        &self,
        category: i32,
        item: i32,
        control: i32,
    ) -> f64 {
        self.control(category, item, control)
            .map_or(0.0, |control| control.prefix_icon_rotation_offset_degrees)
    }

    pub fn control_prefix(
        &self,
        category: i32,
        item: i32,
        control: i32,
        component: i32,
    ) -> QString {
        let defaults = ["X", "Y", "Z"];
        self.control(category, item, control)
            .and_then(|control| control.prefixes.get(index(component)?))
            .map_or_else(
                || {
                    index(component)
                        .and_then(|component| defaults.get(component))
                        .map_or_else(QString::default, |prefix| QString::from(*prefix))
                },
                |prefix| QString::from(prefix.as_str()),
            )
    }

    pub fn control_lock(&self, category: i32, item: i32, control: i32) -> bool {
        self.control(category, item, control)
            .is_some_and(|control| control.lock)
    }

    pub fn control_with_alpha(&self, category: i32, item: i32, control: i32) -> bool {
        self.control(category, item, control)
            .is_some_and(|control| control.with_alpha)
    }

    pub fn control_choice_values(&self, category: i32, item: i32, control: i32) -> QStringList {
        strings(
            self.control(category, item, control)
                .map(|control| control.values.as_slice()),
        )
    }

    pub fn control_choice_labels(&self, category: i32, item: i32, control: i32) -> QStringList {
        self.control(category, item, control)
            .map(|control| {
                control
                    .labels
                    .iter()
                    .map(|label| shrimply_i18n_qt::text(label))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn control_choice_icons(&self, category: i32, item: i32, control: i32) -> QStringList {
        strings(
            self.control(category, item, control)
                .map(|control| control.icons.as_slice()),
        )
    }

    pub fn control_choice_search_terms(
        &self,
        category: i32,
        item: i32,
        control: i32,
    ) -> QStringList {
        strings(
            self.control(category, item, control)
                .map(|control| control.search_terms.as_slice()),
        )
    }

    pub fn open_font_browser(mut self: Pin<&mut Self>) {
        if super::open_font_browser() {
            let revision = self.font_browser_revision().wrapping_add(1);
            self.as_mut().set_font_browser_revision(revision);
        }
    }

    pub fn search_font_browser(mut self: Pin<&mut Self>, query: &QString) {
        if super::search_font_browser(query.to_string()) {
            let revision = self.font_browser_revision().wrapping_add(1);
            self.as_mut().set_font_browser_revision(revision);
        }
    }

    pub fn request_font_browser_previews(self: Pin<&mut Self>, first: i32, count: i32) {
        let (Ok(first), Ok(count)) = (usize::try_from(first), usize::try_from(count)) else {
            return;
        };
        let Some(end) = first.checked_add(count) else {
            return;
        };
        super::request_font_browser_previews(first..end);
    }

    pub fn font_browser_count(&self) -> i32 {
        count(Some(super::font_browser_count()))
    }

    pub fn font_browser_label(&self, choice: i32) -> QString {
        index(choice)
            .and_then(super::font_browser_choice)
            .map_or_else(QString::default, |family| QString::from(&family.name))
    }

    pub fn font_browser_value(&self, choice: i32) -> QString {
        index(choice)
            .and_then(super::font_browser_choice)
            .map_or_else(QString::default, |family| {
                QString::from(
                    serde_json::to_string(&shrimply_inspector_core::font_cache::project_family(
                        &family,
                    ))
                    .expect("font family must serialize"),
                )
            })
    }

    pub fn font_browser_google(&self, choice: i32) -> bool {
        index(choice)
            .and_then(super::font_browser_choice)
            .is_some_and(|family| {
                family.source == shrimply_inspector_core::font_cache::FontSource::Google
            })
    }

    pub fn font_browser_preview_source(&self, choice: i32) -> QUrl {
        let Some(family) = index(choice).and_then(super::font_browser_choice) else {
            return QUrl::default();
        };
        match shrimply_inspector_core::font_cache::preview_source(
            &family,
            super::font_browser_lookup().as_ref(),
        ) {
            Ok(shrimply_inspector_core::font_cache::FontPreviewSource::Installed) | Err(_) => {
                QUrl::default()
            }
            Ok(shrimply_inspector_core::font_cache::FontPreviewSource::File(path)) => {
                QUrl::from_local_file(&QString::from(path.to_string_lossy().as_ref()))
            }
            Ok(shrimply_inspector_core::font_cache::FontPreviewSource::Remote(url)) => {
                QUrl::from(url.as_str())
            }
        }
    }

    pub fn font_browser_status(&self) -> QString {
        QString::from(super::font_browser_status())
    }

    pub fn font_browser_busy(&self) -> bool {
        super::font_browser_busy()
    }

    pub fn font_list_with_choice(&self, value: &QString, index: i32, family: &QString) -> QString {
        let Ok(families) =
            serde_json::from_str::<Vec<shrimply_core::FontFamily>>(&value.to_string())
        else {
            return QString::default();
        };
        let Ok(family) = serde_json::from_str::<shrimply_core::FontFamily>(&family.to_string())
        else {
            return QString::default();
        };
        let next = if index < 0 {
            shrimply_inspector_core::font_selector::append_family(&families, family)
        } else {
            usize::try_from(index).ok().and_then(|index| {
                shrimply_inspector_core::font_selector::replace_family(&families, index, family)
            })
        };
        next.map_or_else(QString::default, |next| {
            QString::from(serde_json::to_string(&next).expect("font families must serialize"))
        })
    }

    pub fn move_font_list_value(&self, value: &QString, index: i32, offset: i32) -> QString {
        let Ok(families) =
            serde_json::from_str::<Vec<shrimply_core::FontFamily>>(&value.to_string())
        else {
            return QString::default();
        };
        let next = usize::try_from(index).ok().and_then(|index| {
            isize::try_from(offset).ok().and_then(|offset| {
                shrimply_inspector_core::font_selector::move_family(&families, index, offset)
            })
        });
        next.map_or_else(QString::default, |next| {
            QString::from(serde_json::to_string(&next).expect("font families must serialize"))
        })
    }

    pub fn remove_font_list_value(&self, value: &QString, index: i32) -> QString {
        let Ok(families) =
            serde_json::from_str::<Vec<shrimply_core::FontFamily>>(&value.to_string())
        else {
            return QString::default();
        };
        let next = usize::try_from(index).ok().and_then(|index| {
            shrimply_inspector_core::font_selector::remove_family(&families, index)
        });
        next.map_or_else(QString::default, |next| {
            QString::from(serde_json::to_string(&next).expect("font families must serialize"))
        })
    }

    pub fn activate_control_font(
        mut self: Pin<&mut Self>,
        category: i32,
        item: i32,
        control: i32,
        family: &QString,
        value: &QString,
    ) {
        let result = self
            .control_target(category, item, control)
            .ok_or_else(|| "font control is no longer available".to_string())
            .and_then(|(target, control)| {
                if control.kind != ControlKind::FontFamilies {
                    return Err("inspector control is not a font family list".to_string());
                }
                let chosen: shrimply_core::FontFamily =
                    serde_json::from_str(&family.to_string())
                        .map_err(|error| format!("invalid font family: {error}"))?;
                let next: Vec<shrimply_core::FontFamily> = serde_json::from_str(&value.to_string())
                    .map_err(|error| format!("invalid font family list: {error}"))?;
                if !next.iter().any(|candidate| candidate == &chosen) {
                    return Err("selected font is missing from the font list".to_string());
                }
                let source = match &chosen {
                    shrimply_core::FontFamily::Local { .. } => {
                        shrimply_inspector_core::font_cache::FontSource::Local
                    }
                    shrimply_core::FontFamily::GoogleFonts { .. } => {
                        shrimply_inspector_core::font_cache::FontSource::Google
                    }
                };
                let available = super::find_font_browser_choice(chosen.name(), source)
                    .ok_or_else(|| "selected font is no longer available".to_string())?;
                let modifier_id = control.target_id;
                super::with_controller(|controller| {
                    super::ensure_font_control(controller, &target, &control.path, modifier_id)
                })?;
                super::activate_font_browser_family(
                    available,
                    super::PendingFontEdit {
                        target,
                        modifier_id,
                        path: control.path,
                        commit_name: control.commit_name,
                        source_value: control.value,
                        value: value.to_string(),
                    },
                )
            });
        if let Err(error) = result {
            self.as_mut().finish(Err(error));
        }
    }

    pub fn cancel_control_font_activation(&self) {
        super::cancel_font_browser_edit();
    }

    pub fn control_keyframes(&self, category: i32, item: i32, control: i32) -> bool {
        self.control(category, item, control)
            .is_some_and(|control| control.layered.keyframes)
    }

    pub fn control_expression(&self, category: i32, item: i32, control: i32) -> bool {
        self.control(category, item, control)
            .is_some_and(|control| control.layered.expression)
    }

    pub fn control_expression_source(&self, category: i32, item: i32, control: i32) -> QString {
        self.control(category, item, control)
            .map_or_else(QString::default, |control| {
                QString::from(control.layered.expression_source.as_str())
            })
    }

    pub fn control_expression_result(&self, category: i32, item: i32, control: i32) -> QStringList {
        let Some(document) = self.document() else {
            return QStringList::default();
        };
        let Some(control) = self.control(category, item, control) else {
            return QStringList::default();
        };
        if !control.layered.expression {
            return QStringList::default();
        }
        let path = control.timeline_path.as_deref().unwrap_or(&control.path);
        if let Some(field) = shrimply_inspector_core::transform::TransformField::from_path(path) {
            let Some(timeline_id) = control.timeline_id else {
                return QStringList::default();
            };
            let output = match field {
                shrimply_inspector_core::transform::TransformField::Vec2(field) => {
                    let Ok(Some(output)) = super::transform_vec2_expression_output(
                        &document.target,
                        field,
                        timeline_id,
                    ) else {
                        return QStringList::default();
                    };
                    [
                        shrimply_inspector_core::transform::expressions::format_vec2(
                            field,
                            output.value,
                        ),
                        output.error.unwrap_or_default(),
                    ]
                }
                shrimply_inspector_core::transform::TransformField::Scalar(field) => {
                    let Ok(Some(output)) = super::transform_scalar_expression_output(
                        &document.target,
                        field,
                        timeline_id,
                    ) else {
                        return QStringList::default();
                    };
                    [
                        shrimply_inspector_core::transform::expressions::format_scalar(
                            field,
                            output.value,
                        ),
                        output.error.unwrap_or_default(),
                    ]
                }
            };
            return output.into_iter().map(QString::from).collect();
        }
        if control.kind == ControlKind::LayeredBoolean {
            let Ok(output) = super::bool_expression_output(&document.target, path) else {
                return QStringList::default();
            };
            return [output.value.to_string(), output.error.unwrap_or_default()]
                .into_iter()
                .map(QString::from)
                .collect();
        }
        if control.kind == ControlKind::LayeredSelector {
            let Ok(output) =
                super::step_expression_output(&document.target, path, control.timeline_id)
            else {
                return QStringList::default();
            };
            return [output.value, output.error.unwrap_or_default()]
                .into_iter()
                .map(QString::from)
                .collect();
        }
        if control.kind == ControlKind::LayeredVector2 {
            let Ok(output) =
                super::vector2_expression_output(&document.target, path, control.timeline_id)
            else {
                return QStringList::default();
            };
            let digits = usize::try_from(control.number.digits).unwrap_or_default();
            let first_prefix = control.prefixes.first().map_or("X", String::as_str);
            let second_prefix = control.prefixes.get(1).map_or("Y", String::as_str);
            return [
                shrimply_inspector_core::timeline_value::vector::vec2::format_value(
                    output.value,
                    first_prefix,
                    second_prefix,
                    digits,
                    control.number.unit,
                ),
                output.error.unwrap_or_default(),
            ]
            .into_iter()
            .map(QString::from)
            .collect();
        }
        if control.kind == ControlKind::LayeredVector3 {
            let Some(timeline_id) = control.timeline_id else {
                return QStringList::default();
            };
            let Ok(output) = super::vector3_expression_output(&document.target, path, timeline_id)
            else {
                return QStringList::default();
            };
            let digits = usize::try_from(control.number.digits).unwrap_or_default();
            let first_prefix = control.prefixes.first().map_or("X", String::as_str);
            let second_prefix = control.prefixes.get(1).map_or("Y", String::as_str);
            let third_prefix = control.prefixes.get(2).map_or("Z", String::as_str);
            return [
                shrimply_inspector_core::timeline_value::vector::vec3::format_value(
                    output.value,
                    [first_prefix, second_prefix, third_prefix],
                    digits,
                    control.number.unit,
                ),
                output.error.unwrap_or_default(),
            ]
            .into_iter()
            .map(QString::from)
            .collect();
        }
        if control.kind == ControlKind::LayeredColor {
            let Some(timeline_id) = control.timeline_id else {
                return QStringList::default();
            };
            let Ok(output) = super::color_expression_output(&document.target, path, timeline_id)
            else {
                return QStringList::default();
            };
            return [
                format!(
                    "#{:02X}{:02X}{:02X}{:02X}",
                    output.value.r, output.value.g, output.value.b, output.value.a,
                ),
                output.error.unwrap_or_default(),
            ]
            .into_iter()
            .map(QString::from)
            .collect();
        }
        if control.kind == ControlKind::LayeredText {
            let Some(timeline_id) = control.timeline_id else {
                return QStringList::default();
            };
            let Ok(output) = super::text_expression_output(&document.target, path, timeline_id)
            else {
                return QStringList::default();
            };
            return [output.value, output.error.unwrap_or_default()]
                .into_iter()
                .map(QString::from)
                .collect();
        }
        if control.kind == ControlKind::LayeredDrawing {
            let Some(timeline_id) = control.timeline_id else {
                return QStringList::default();
            };
            let Ok(output) = super::paint_drawing_expression_output(&document.target, timeline_id)
            else {
                return QStringList::default();
            };
            return [output.value, output.error.unwrap_or_default()]
                .into_iter()
                .map(QString::from)
                .collect();
        }
        if crate::graph_backend::background_integer(&control) {
            let Some(timeline_id) = control.timeline_id else {
                return QStringList::default();
            };
            let Ok(output) =
                super::background_integer_expression_output(&document.target, path, timeline_id)
            else {
                return QStringList::default();
            };
            return [output.value.to_string(), output.error.unwrap_or_default()]
                .into_iter()
                .map(QString::from)
                .collect();
        }
        let output = if control.audio_modifier {
            control
                .target_id
                .zip(control.timeline_id)
                .ok_or_else(|| "audio modifier expression target is unavailable".to_string())
                .and_then(|(modifier_id, timeline_id)| {
                    super::audio_modifier_expression_output(
                        &document.target,
                        modifier_id,
                        timeline_id,
                    )
                })
        } else {
            super::scalar_expression_output(&document.target, path, control.timeline_id)
        };
        let Ok(output) = output else {
            return QStringList::default();
        };
        [
            format!(
                "{:.*}{}",
                usize::try_from(control.number.digits).unwrap_or_default(),
                control.display_number(f64::from(output.value)),
                control.number.unit,
            ),
            output.error.unwrap_or_default(),
        ]
        .into_iter()
        .map(QString::from)
        .collect()
    }

    pub fn expression_diagnostic(mut self: Pin<&mut Self>, source: &QString) -> QStringList {
        let source = source.to_string();
        let diagnostic = self
            .as_mut()
            .rust_mut()
            .expression_diagnostic_cache
            .diagnostic(&source)
            .cloned();
        let Some(diagnostic) = diagnostic else {
            return QStringList::default();
        };
        [
            diagnostic.message,
            diagnostic
                .line
                .map_or_else(String::new, |line| line.to_string()),
            diagnostic
                .column
                .map_or_else(String::new, |column| column.to_string()),
        ]
        .into_iter()
        .map(QString::from)
        .collect()
    }

    pub fn expression_diagnostic_debounce(&self) -> i32 {
        shrimply_inspector_core::rhai_editor::DIAGNOSTIC_DEBOUNCE_MILLISECONDS
    }

    pub fn expression_completion_debounce(&self) -> i32 {
        shrimply_inspector_core::rhai_editor::COMPLETION_DEBOUNCE_MILLISECONDS
    }

    pub fn control_expression_completion(
        &self,
        category: i32,
        item: i32,
        control: i32,
        source: &QString,
        cursor: i32,
        automatic: bool,
    ) -> QStringList {
        let Some(control) = self.control(category, item, control) else {
            return QStringList::default();
        };
        let value = match control.kind {
            ControlKind::LayeredBoolean => {
                shrimply_inspector_core::rhai_editor::ExpressionValue::Bool
            }
            ControlKind::LayeredSelector => {
                shrimply_inspector_core::rhai_editor::ExpressionValue::Step
            }
            ControlKind::LayeredText => shrimply_inspector_core::rhai_editor::ExpressionValue::Text,
            ControlKind::LayeredDrawing => {
                shrimply_inspector_core::rhai_editor::ExpressionValue::Drawing
            }
            ControlKind::LayeredVector2 => {
                shrimply_inspector_core::rhai_editor::ExpressionValue::Vec2
            }
            ControlKind::LayeredVector3 => {
                shrimply_inspector_core::rhai_editor::ExpressionValue::Vec3
            }
            ControlKind::LayeredColor => {
                shrimply_inspector_core::rhai_editor::ExpressionValue::Color
            }
            _ => shrimply_inspector_core::rhai_editor::ExpressionValue::Scalar,
        };
        let source = source.to_string();
        let utf16_cursor = usize::try_from(cursor).unwrap_or_default();
        let cursor = shrimply_inspector_core::rhai_editor::utf16_offset_to_char_offset(
            &source,
            utf16_cursor,
        );
        let completion = if automatic {
            shrimply_inspector_core::rhai_editor::automatic_completion(&source, value, cursor)
        } else {
            shrimply_inspector_core::rhai_editor::completion(&source, value, cursor)
        };
        let Some(completion) = completion else {
            return QStringList::default();
        };
        let start = shrimply_inspector_core::rhai_editor::char_offset_to_utf16_offset(
            &source,
            completion.start,
        );
        let end = shrimply_inspector_core::rhai_editor::char_offset_to_utf16_offset(
            &source,
            completion.end,
        );
        [start.to_string(), end.to_string()]
            .into_iter()
            .chain(completion.candidates)
            .map(QString::from)
            .collect()
    }

    pub fn set_control_value(
        mut self: Pin<&mut Self>,
        category: i32,
        item: i32,
        control: i32,
        value: &QString,
    ) {
        let Some((target, control)) = self.control_target(category, item, control) else {
            return;
        };
        if let Err(error) = super::ensure_control_timeline(&target, &control) {
            self.as_mut().finish(Err(error));
            return;
        }
        let value = value.to_string();
        if let Some(shrimply_inspector_core::InspectorControlAction::SetSam2Model { modifier_id }) =
            control.action
        {
            self.as_mut()
                .finish(super::set_sam2_model(&target, modifier_id, &value));
            return;
        }
        if let Some(shrimply_inspector_core::InspectorControlAction::SetSam2PointLabel {
            modifier_id,
            point_id,
        }) = control.action
        {
            self.as_mut().finish(super::set_sam2_point_label(
                &target,
                modifier_id,
                point_id,
                &value,
            ));
            return;
        }
        if control.kind == ControlKind::AudioModifierMenu {
            if value == "__paste__" {
                let result = super::paste_audio_modifiers(&target).map(|count| {
                    (count > 0).then(|| {
                        if count == 1 {
                            "1 effect pasted".to_string()
                        } else {
                            format!("{count} effects pasted")
                        }
                    })
                });
                self.as_mut().finish_confirmation(result);
            } else {
                self.as_mut()
                    .finish(super::add_audio_modifier(&target, &value));
            }
            return;
        }
        if control.kind == ControlKind::VisualModifierMenu {
            if value == "__paste__" {
                let result = super::paste_visual_modifiers(&target).map(|count| {
                    (count > 0).then(|| {
                        if count == 1 {
                            "1 effect pasted".to_string()
                        } else {
                            format!("{count} effects pasted")
                        }
                    })
                });
                self.as_mut().finish_confirmation(result);
            } else {
                let result = super::add_visual_modifier(&target, &value).map(|id| {
                    self.as_mut().rust_mut().list_state.set_expanded(
                        &target,
                        &format!("modifier:{id}"),
                        true,
                    );
                });
                self.as_mut().finish(result);
            }
            return;
        }
        if control.kind == ControlKind::LayeredText {
            let result = control
                .timeline_id
                .ok_or_else(|| "text timeline ID is unavailable".to_string())
                .and_then(|timeline_id| {
                    super::set_text_value(
                        &target,
                        &control.path,
                        timeline_id,
                        value,
                        &control.commit_name,
                    )
                });
            self.as_mut().finish(result);
            return;
        }
        if let Some(result) = super::named_control_edit(&target, &control, value.clone()) {
            self.as_mut().finish(result);
            return;
        }
        let result = if control.kind == ControlKind::AudioCachePreset {
            control
                .target_id
                .ok_or_else(|| "audio cache control has no modifier target".to_string())
                .and_then(|id| super::set_audio_cache_preset(&target, id, &value))
        } else if control.kind == ControlKind::VisualCacheQuality {
            control
                .target_id
                .ok_or_else(|| "visual cache control has no modifier target".to_string())
                .and_then(|id| super::set_visual_cache_quality(&target, id, &value))
        } else if control.kind == ControlKind::LayeredBoolean {
            value
                .parse::<bool>()
                .map_err(|_| format!("invalid timeline boolean: {value}"))
                .and_then(|value| super::set_bool_value(&target, &control.path, value))
        } else if matches!(
            control.kind,
            ControlKind::LayeredNumber | ControlKind::LayeredSelector
        ) {
            control_value(&control, &value).and_then(|value| {
                if crate::graph_backend::background_integer(&control) {
                    let value = serde_json::from_value::<u32>(value)
                        .map_err(|error| format!("invalid background integer: {error}"))?;
                    super::set_background_integer_value(
                        &target,
                        &control.path,
                        control.timeline_id.ok_or_else(|| {
                            "background integer timeline ID is unavailable".to_string()
                        })?,
                        value,
                    )
                } else if control.audio_modifier {
                    control
                        .target_id
                        .ok_or_else(|| "audio modifier target is unavailable".to_string())
                        .and_then(|id| {
                            control
                                .timeline_id
                                .ok_or_else(|| {
                                    "audio modifier timeline ID is unavailable".to_string()
                                })
                                .and_then(|timeline_id| {
                                    super::set_audio_modifier_timeline_base(
                                        &target,
                                        id,
                                        timeline_id,
                                        value,
                                    )
                                })
                        })
                } else {
                    super::set_timeline_base(
                        &target,
                        &control.path,
                        value,
                        &control.commit_name,
                        control.commit_immediately,
                    )
                }
            })
        } else if control.kind == ControlKind::OptionalSelector {
            super::set_optional_field(
                &target,
                &control.path,
                (!value.is_empty()).then_some(value.as_str()),
            )
        } else if control.kind == ControlKind::OptionalNumberSelector {
            super::set_optional_number_field(
                &target,
                &control.path,
                (!value.is_empty()).then_some(value.as_str()),
            )
        } else {
            let value = if control.kind == ControlKind::Number
                && (control.store_multiplier != 1.0
                    || control.number_mapping != shrimply_inspector_core::NumberMapping::Linear)
            {
                value
                    .parse::<f64>()
                    .map(|value| control.store_number(value).to_string())
                    .map_err(|_| format!("invalid numeric inspector value: {value}"))
            } else {
                Ok(value)
            };
            value.and_then(|value| {
                if control.audio_modifier {
                    control
                        .target_id
                        .ok_or_else(|| "audio modifier target is unavailable".to_string())
                        .and_then(|id| {
                            if control.kind == ControlKind::Number {
                                super::set_audio_modifier_live_field(
                                    &target,
                                    id,
                                    &control.path,
                                    &value,
                                )
                            } else {
                                super::set_audio_modifier_field(&target, id, &control.path, &value)
                            }
                        })
                } else {
                    super::set_field(&target, &control.path, &value)
                }
            })
        };
        self.as_mut().finish(result);
    }

    pub fn trigger_control_action(
        mut self: Pin<&mut Self>,
        category: i32,
        item: i32,
        control: i32,
    ) {
        let control_index = control;
        let Some((target, control)) = self.control_target(category, item, control) else {
            return;
        };
        if control.action.is_some() && !control.action_sensitive {
            return;
        }
        let result = if let Some(action) = control.action {
            match action {
                shrimply_inspector_core::InspectorControlAction::SelectObject3dModel {
                    modifier_id,
                } => super::select_object_3d_model(&target, modifier_id),
                shrimply_inspector_core::InspectorControlAction::SelectScene3dEnvironment => {
                    super::select_scene_3d_environment(&target)
                }
                shrimply_inspector_core::InspectorControlAction::SelectPaintTexture {
                    color_id,
                } => super::select_paint_texture(&target, color_id),
                shrimply_inspector_core::InspectorControlAction::ToggleSam2Analysis {
                    modifier_id,
                    ..
                } => super::toggle_sam2_analysis(&target, modifier_id),
                shrimply_inspector_core::InspectorControlAction::ToggleTransparentFillAnalysis {
                    modifier_id,
                } => super::toggle_transparent_fill_analysis(&target, modifier_id),
                shrimply_inspector_core::InspectorControlAction::ToggleCameraAnalysis => {
                    super::toggle_camera_analysis(&target)
                }
                action => super::trigger_video_control_action(&target, action),
            }
        } else {
            match control.kind {
                ControlKind::AudioCache => control
                    .target_id
                    .ok_or_else(|| "audio cache control has no modifier target".to_string())
                    .and_then(|id| super::toggle_audio_cache(&target, id)),
                ControlKind::VisualCache => control
                    .target_id
                    .ok_or_else(|| "visual cache control has no modifier target".to_string())
                    .and_then(|id| super::toggle_visual_cache(&target, id)),
                ControlKind::Analysis => Err("analysis control has no action".to_string()),
                ControlKind::Action => Err("inspector action control has no action".to_string()),
                _ => Err("inspector control does not have an action".to_string()),
            }
        };
        let refresh_analysis = result.is_ok() && control.kind == ControlKind::Analysis;
        self.as_mut().finish(result);
        if refresh_analysis {
            self.as_mut()
                .poll_analysis_control(category, item, control_index);
        }
    }

    pub fn trigger_secondary_control_action(
        mut self: Pin<&mut Self>,
        category: i32,
        item: i32,
        control: i32,
    ) {
        let Some((target, control)) = self.control_target(category, item, control) else {
            return;
        };
        let result = control
            .secondary_action
            .ok_or_else(|| "inspector control does not have a secondary action".to_string())
            .and_then(|action| super::trigger_video_control_action(&target, action));
        self.as_mut().finish(result);
    }

    pub fn set_control_fraction(
        mut self: Pin<&mut Self>,
        category: i32,
        item: i32,
        control: i32,
        numerator: i64,
        denominator: i64,
    ) {
        let Some((target, control)) = self.control_target(category, item, control) else {
            return;
        };
        let result = super::set_control_fraction(&target, &control, numerator, denominator);
        self.as_mut().finish(result);
    }

    pub fn set_control_components(
        mut self: Pin<&mut Self>,
        category: i32,
        item: i32,
        control: i32,
        first: f64,
        second: f64,
        third: f64,
        _changed: i32,
    ) {
        let Some((target, control)) = self.control_target(category, item, control) else {
            return;
        };
        if let Err(error) = super::ensure_control_timeline(&target, &control) {
            self.as_mut().finish(Err(error));
            return;
        }
        if let Some(shrimply_inspector_core::InspectorControlAction::SetSam2PointPosition {
            modifier_id,
            point_id,
        }) = control.action
        {
            self.as_mut().finish(super::set_sam2_point_position(
                &target,
                modifier_id,
                point_id,
                first,
                second,
            ));
            return;
        }
        let count = if matches!(
            control.kind,
            ControlKind::Vector3 | ControlKind::LayeredVector3
        ) {
            3
        } else {
            2
        };
        if control.kind == ControlKind::LayeredVector2 {
            self.as_mut().finish(super::set_vector2_value(
                &target,
                &control.path,
                first * control.store_multiplier,
                second * control.store_multiplier,
                &control.commit_name,
            ));
            return;
        }
        if control.kind == ControlKind::LayeredVector3 {
            self.as_mut().finish(super::set_vector3_value(
                &target,
                &control.path,
                first * control.store_multiplier,
                second * control.store_multiplier,
                third * control.store_multiplier,
                &control.commit_name,
            ));
            return;
        }
        let values = [first, second, third]
            .into_iter()
            .take(count)
            .enumerate()
            .map(|(component, value)| (component, (value * control.store_multiplier).to_string()))
            .collect::<Vec<_>>();
        self.as_mut()
            .finish(super::set_components(&target, &control.path, &values));
    }

    pub fn set_control_color(
        mut self: Pin<&mut Self>,
        category: i32,
        item: i32,
        control: i32,
        red: f64,
        green: f64,
        blue: f64,
        alpha: f64,
    ) {
        let Some((target, control)) = self.control_target(category, item, control) else {
            return;
        };
        let alpha = if control.with_alpha { alpha } else { 1.0 };
        let channels =
            [red, green, blue, alpha].map(|value| (value.clamp(0.0, 1.0) * 255.0).round() as u8);
        if control.kind == ControlKind::LayeredColor {
            let result = control
                .timeline_id
                .ok_or_else(|| "timeline color ID is unavailable".to_string())
                .and_then(|timeline_id| {
                    super::set_color_value(
                        &target,
                        &control.path,
                        timeline_id,
                        shrimply_core::Color::new(
                            channels[0],
                            channels[1],
                            channels[2],
                            channels[3],
                        ),
                        &control.commit_name,
                    )
                });
            self.as_mut().finish(result);
            return;
        }
        let values = channels
            .into_iter()
            .enumerate()
            .map(|(component, value)| (component, value.to_string()))
            .collect::<Vec<_>>();
        let result = if matches!(target, InspectorTarget::Transition { .. })
            && !control.commit_name.is_empty()
        {
            super::set_transition_components(
                &target,
                &control.path,
                &values,
                &control.commit_name,
                control.commit_immediately,
            )
        } else {
            super::set_components(&target, &control.path, &values)
        };
        self.as_mut().finish(result);
    }

    pub fn commit_control(mut self: Pin<&mut Self>, category: i32, item: i32, control: i32) {
        if let Some((target, control)) = self.control_target(category, item, control) {
            self.as_mut()
                .finish(super::commit_control_edit(&target, &control));
        }
        super::mark_dirty();
    }

    pub fn set_control_keyframes(
        mut self: Pin<&mut Self>,
        category: i32,
        item: i32,
        control: i32,
        enabled: bool,
    ) {
        let Some((target, control)) = self.control_target(category, item, control) else {
            return;
        };
        if let Err(error) = super::ensure_control_timeline(&target, &control) {
            self.as_mut().finish(Err(error));
            return;
        }
        let path = control.timeline_path.as_deref().unwrap_or(&control.path);
        let result = if control.kind == ControlKind::LayeredDrawing {
            control
                .timeline_id
                .ok_or_else(|| "paint drawing timeline ID is unavailable".to_string())
                .and_then(|timeline_id| {
                    super::set_paint_drawing_keyframes_enabled(&target, timeline_id, enabled)
                })
        } else if control.audio_modifier {
            control
                .target_id
                .ok_or_else(|| "audio modifier target is unavailable".to_string())
                .and_then(|id| {
                    control
                        .timeline_id
                        .ok_or_else(|| "audio modifier timeline ID is unavailable".to_string())
                        .and_then(|timeline_id| {
                            super::set_audio_modifier_timeline_mode(
                                &target,
                                id,
                                timeline_id,
                                path,
                                shrimply_inspector_core::TimelineModeChange {
                                    keyframes: true,
                                    enabled,
                                    current: serde_json::Value::Null,
                                    default_expression: control.kind.default_expression(),
                                },
                            )
                        })
                })
        } else if control.kind == ControlKind::LayeredNumber {
            if crate::graph_backend::background_integer(&control) {
                control
                    .timeline_id
                    .ok_or_else(|| "background integer timeline ID is unavailable".to_string())
                    .and_then(|timeline_id| {
                        super::set_background_integer_keyframes_enabled(
                            &target,
                            path,
                            timeline_id,
                            enabled,
                        )
                    })
            } else {
                super::set_scalar_keyframes_enabled(
                    &target,
                    path,
                    enabled,
                    control.number_constraint,
                    &control.keyframe_commit_name,
                )
            }
        } else if control.kind == ControlKind::LayeredVector2 {
            super::set_vector2_keyframes_enabled(
                &target,
                path,
                enabled,
                &control.keyframe_commit_name,
            )
        } else if control.kind == ControlKind::LayeredVector3 {
            super::set_vector3_keyframes_enabled(
                &target,
                path,
                enabled,
                &control.keyframe_commit_name,
            )
        } else if control.kind == ControlKind::LayeredColor {
            control
                .timeline_id
                .ok_or_else(|| "timeline color ID is unavailable".to_string())
                .and_then(|timeline_id| {
                    super::set_color_keyframes_enabled(
                        &target,
                        path,
                        timeline_id,
                        enabled,
                        &control.keyframe_commit_name,
                    )
                })
        } else if control.kind == ControlKind::LayeredBoolean {
            super::set_bool_keyframes_enabled(&target, path, enabled, &control.keyframe_commit_name)
        } else if control.kind == ControlKind::LayeredText {
            control
                .timeline_id
                .ok_or_else(|| "text timeline ID is unavailable".to_string())
                .and_then(|timeline_id| {
                    super::set_text_keyframes_enabled(
                        &target,
                        path,
                        timeline_id,
                        enabled,
                        &control.keyframe_commit_name,
                    )
                })
        } else if control.kind == ControlKind::LayeredSelector {
            super::set_step_keyframes_enabled(&target, path, enabled, &control.keyframe_commit_name)
        } else {
            timeline_value(&control).and_then(|current| {
                super::set_timeline_mode(
                    &target,
                    path,
                    true,
                    enabled,
                    current,
                    control.kind.default_expression(),
                    &control.keyframe_commit_name,
                )
            })
        };
        self.as_mut().finish(result);
    }

    pub fn set_control_expression(
        mut self: Pin<&mut Self>,
        category: i32,
        item: i32,
        control: i32,
        enabled: bool,
    ) {
        let Some((target, control)) = self.control_target(category, item, control) else {
            return;
        };
        if let Err(error) = super::ensure_control_timeline(&target, &control) {
            self.as_mut().finish(Err(error));
            return;
        }
        let path = control.timeline_path.as_deref().unwrap_or(&control.path);
        let result = if control.kind == ControlKind::LayeredDrawing {
            super::set_timeline_mode(
                &target,
                path,
                false,
                enabled,
                serde_json::Value::Null,
                control.kind.default_expression(),
                &control.expression_commit_name,
            )
        } else if control.audio_modifier {
            control
                .target_id
                .ok_or_else(|| "audio modifier target is unavailable".to_string())
                .and_then(|id| {
                    control
                        .timeline_id
                        .ok_or_else(|| "audio modifier timeline ID is unavailable".to_string())
                        .and_then(|timeline_id| {
                            super::set_audio_modifier_timeline_mode(
                                &target,
                                id,
                                timeline_id,
                                path,
                                shrimply_inspector_core::TimelineModeChange {
                                    keyframes: false,
                                    enabled,
                                    current: serde_json::Value::Null,
                                    default_expression: control.kind.default_expression(),
                                },
                            )
                        })
                })
        } else if crate::graph_backend::background_integer(&control) {
            control
                .timeline_id
                .ok_or_else(|| "background integer timeline ID is unavailable".to_string())
                .and_then(|timeline_id| {
                    super::set_background_integer_expression_enabled(
                        &target,
                        path,
                        timeline_id,
                        enabled,
                    )
                })
        } else if let Some(field) =
            shrimply_inspector_core::transform::TransformField::from_path(path)
        {
            control
                .timeline_id
                .ok_or_else(|| "transform timeline ID is unavailable".to_string())
                .and_then(|timeline_id| {
                    super::set_transform_expression_enabled(&target, field, timeline_id, enabled)
                })
        } else if control.kind == ControlKind::LayeredVector2 {
            super::set_vector2_expression_enabled(
                &target,
                path,
                enabled,
                &control.expression_commit_name,
            )
        } else if control.kind == ControlKind::LayeredText {
            control
                .timeline_id
                .ok_or_else(|| "text timeline ID is unavailable".to_string())
                .and_then(|timeline_id| {
                    super::set_text_expression_enabled(
                        &target,
                        path,
                        timeline_id,
                        enabled,
                        &control.expression_commit_name,
                    )
                })
        } else {
            timeline_value(&control).and_then(|current| {
                super::set_timeline_mode(
                    &target,
                    path,
                    false,
                    enabled,
                    current,
                    control.kind.default_expression(),
                    &control.expression_commit_name,
                )
            })
        };
        self.as_mut().finish(result);
    }

    pub fn set_control_expression_source(
        mut self: Pin<&mut Self>,
        category: i32,
        item: i32,
        control: i32,
        source: &QString,
    ) {
        let Some((target, control)) = self.control_target(category, item, control) else {
            return;
        };
        if let Err(error) = super::ensure_control_timeline(&target, &control) {
            self.as_mut().finish(Err(error));
            return;
        }
        let path = control.timeline_path.as_deref().unwrap_or(&control.path);
        let source = source.to_string();
        let result = if control.audio_modifier {
            control
                .target_id
                .ok_or_else(|| "audio modifier target is unavailable".to_string())
                .and_then(|id| {
                    super::set_audio_modifier_expression_source(&target, id, path, &source)
                })
        } else if crate::graph_backend::background_integer(&control) {
            control
                .timeline_id
                .ok_or_else(|| "background integer timeline ID is unavailable".to_string())
                .and_then(|timeline_id| {
                    super::set_background_integer_expression_source(
                        &target,
                        path,
                        timeline_id,
                        source,
                    )
                })
        } else if let Some(field) =
            shrimply_inspector_core::transform::TransformField::from_path(path)
        {
            control
                .timeline_id
                .ok_or_else(|| "transform timeline ID is unavailable".to_string())
                .and_then(|timeline_id| {
                    super::set_transform_expression_source(&target, field, timeline_id, source)
                })
        } else if control.kind == ControlKind::LayeredVector2 {
            super::set_vector2_expression_source(
                &target,
                path,
                source,
                &control.expression_commit_name,
            )
        } else if control.kind == ControlKind::LayeredText {
            control
                .timeline_id
                .ok_or_else(|| "text timeline ID is unavailable".to_string())
                .and_then(|timeline_id| {
                    super::set_text_expression_source(
                        &target,
                        path,
                        timeline_id,
                        &source,
                        &control.expression_commit_name,
                    )
                })
        } else {
            super::set_expression_source(&target, path, &source, &control.expression_commit_name)
        };
        self.as_mut().finish(result);
    }

    pub fn apply_project_settings(
        mut self: Pin<&mut Self>,
        width: i32,
        height: i32,
        fps_numerator: &QString,
        fps_denominator: &QString,
    ) {
        self.as_mut().finish(super::apply_project_settings(
            width,
            height,
            &fps_numerator.to_string(),
            &fps_denominator.to_string(),
        ));
    }

    fn perform(mut self: Pin<&mut Self>, action: Option<InspectorAction>) {
        let Some(action) = action else {
            return;
        };
        let Some(target) = self.document().map(|document| document.target.clone()) else {
            return;
        };
        self.as_mut()
            .finish_confirmation(super::perform_action(&target, action));
    }

    pub(crate) fn finish_confirmation(
        mut self: Pin<&mut Self>,
        result: Result<Option<String>, String>,
    ) {
        match result {
            Ok(Some(message)) => self.as_mut().show_confirmation(QString::from(message)),
            Ok(None) => {}
            Err(error) => self.as_mut().show_error(QString::from(error)),
        }
    }

    pub(crate) fn finish(mut self: Pin<&mut Self>, result: Result<(), String>) {
        if let Err(error) = result {
            self.as_mut().show_error(QString::from(error));
        }
    }

    fn document(&self) -> Option<&InspectorDocument> {
        self.rust().document.as_ref()
    }

    fn category(&self, category: i32) -> Option<&crate::list::InspectorCategory> {
        self.document()?.categories.get(index(category)?)
    }

    fn item(&self, category: i32, item: i32) -> Option<&InspectorListItem> {
        self.category(category)?.items.get(index(item)?)
    }

    fn card(&self, category: i32, item: i32) -> Option<&crate::item::InspectorItem> {
        match self.item(category, item)? {
            InspectorListItem::Item(item) => Some(item.as_ref()),
            InspectorListItem::Flat(_) => None,
        }
    }

    fn section(&self, category: i32, item: i32) -> Option<&crate::section::InspectorSection> {
        match self.item(category, item)? {
            InspectorListItem::Item(item) => Some(&item.section),
            InspectorListItem::Flat(section) => Some(section),
        }
    }

    pub(crate) fn control(
        &self,
        category: i32,
        item: i32,
        control: i32,
    ) -> Option<&InspectorControl> {
        self.section(category, item)?.controls.get(index(control)?)
    }

    pub(crate) fn control_target(
        &self,
        category: i32,
        item: i32,
        control: i32,
    ) -> Option<(shrimply_inspector_core::InspectorTarget, InspectorControl)> {
        Some((
            self.document()?.target.clone(),
            self.control(category, item, control)?.clone(),
        ))
    }

    fn action(&self, category: i32, item: i32, action: i32) -> Option<&crate::item::HeaderAction> {
        self.card(category, item)?.actions.get(index(action)?)
    }
}

fn index(index: i32) -> Option<usize> {
    usize::try_from(index).ok()
}

fn analysis_control_key(category: i32, item: i32, control: i32) -> Option<AnalysisControlKey> {
    Some((index(category)?, index(item)?, index(control)?))
}

fn count(value: Option<usize>) -> i32 {
    value
        .and_then(|value| i32::try_from(value).ok())
        .unwrap_or_default()
}

fn strings(values: Option<&[String]>) -> QStringList {
    values
        .unwrap_or_default()
        .iter()
        .map(|value| QString::from(value.as_str()))
        .collect()
}

fn time_parts(time: shrimply_project::project::Time) -> [i64; 2] {
    [
        shrimply_core::timeline_value::fraction_numerator(time.seconds),
        shrimply_core::timeline_value::fraction_denominator(time.seconds),
    ]
}

pub(crate) fn time_text(time: shrimply_project::project::Time) -> String {
    let [numerator, denominator] = time_parts(time);
    format!("{numerator}/{denominator}")
}
