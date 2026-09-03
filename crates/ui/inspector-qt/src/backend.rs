use core::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QColor, QString, QStringList, QUrl};
use shrimply_inspector_core::InspectorTarget;

use crate::item::{InspectorAction, InspectorListItem};
use crate::list::{InspectorDocument, InspectorListState};
use crate::section::{ControlKind, InspectorControl};
use crate::value_backend::{
    boolean_action, control_value, default_expression, fraction_value, optional_action,
    timeline_value,
};

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
        ModifierMenu,
        TtsEditor,
        BeatDetection,
        InfoHeading,
        InfoArtwork,
        FileLocation,
        InfoLoading,
        Action,
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
        #[qproperty(i32, document_revision, cxx_name = "documentRevision")]
        #[qproperty(i32, expression_revision, cxx_name = "expressionRevision")]
        #[qproperty(i32, graph_revision, cxx_name = "graphRevision")]
        #[qproperty(i32, playhead_revision, cxx_name = "playheadRevision")]
        #[qproperty(i32, transform_revision, cxx_name = "transformRevision")]
        #[qproperty(i32, active_category, cxx_name = "activeCategory")]
        #[qproperty(f64, scroll_position, cxx_name = "scrollPosition")]
        #[qproperty(QString, title)]
        type InspectorBackend = super::InspectorBackendRust;

        #[qinvokable]
        fn poll(self: Pin<&mut InspectorBackend>, scroll_position: f64);
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
    document_revision: i32,
    expression_revision: i32,
    graph_revision: i32,
    playhead_revision: i32,
    transform_revision: i32,
    active_category: i32,
    scroll_position: f64,
    title: QString,
    document: Option<InspectorDocument>,
    list_state: InspectorListState,
    stabilization_generating: Option<bool>,
    resolved_transform: Option<shrimply_project::project::ResolvedTransform>,
    transform_live: Option<shrimply_inspector_core::transform::TransformLivePresentation>,
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

    pub fn minimum_width(&self) -> i32 {
        shrimply_inspector_core::INSPECTOR_MIN_WIDTH
    }

    pub fn poll(mut self: Pin<&mut Self>, scroll_position: f64) {
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
                    crate::graph_backend::update_visual_modifier_graphs(document, &target);
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
        let transform_live = super::transform_live_presentation(&document.target);
        let resolved_transform = transform_live
            .as_ref()
            .map(|presentation| presentation.resolved)
            .or_else(|| super::resolved_transform(&document.target));
        let revision = self.revision().wrapping_add(1);
        let document_revision = self.document_revision().wrapping_add(1);
        self.as_mut().rust_mut().document = Some(document);
        self.as_mut().rust_mut().resolved_transform = resolved_transform;
        self.as_mut().rust_mut().transform_live = transform_live;
        self.as_mut().set_ready(true);
        self.as_mut().set_title(title);
        self.as_mut().set_active_category(active);
        self.as_mut().set_scroll_position(scroll_position);
        self.as_mut().set_document_revision(document_revision);
        self.as_mut().set_revision(revision);
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
        self.control(category, item, control)
            .map_or_else(QString::default, |control| {
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
                                    control.target_id,
                                    control.timeline_id,
                                    control.timeline_path.as_deref().unwrap_or(&control.path),
                                )
                                .ok()
                            })
                        })
                        .map(|value| {
                            value
                                * if control.store_multiplier == 0.0 {
                                    1.0
                                } else {
                                    control.store_multiplier.recip()
                                }
                        })
                        .unwrap_or_else(|| control.value.parse::<f64>().unwrap_or_default());
                    QString::from(value.to_string())
                } else if matches!(
                    control.kind,
                    ControlKind::AudioCache | ControlKind::VisualCache
                ) {
                    QString::from(
                        control
                            .target_id
                            .and_then(|id| super::tracked_cache_control(control.kind, id))
                            .map_or(control.value.as_str(), |status| status.label),
                    )
                } else {
                    QString::from(control.value.as_str())
                }
            })
    }

    pub fn control_component(&self, category: i32, item: i32, control: i32, component: i32) -> f64 {
        let target = self.document().map(|document| document.target.clone());
        self.control(category, item, control)
            .and_then(|control| {
                if matches!(
                    control.kind,
                    ControlKind::AudioCache | ControlKind::VisualCache
                ) && component == 0
                {
                    return control
                        .target_id
                        .and_then(|id| super::tracked_cache_control(control.kind, id))
                        .map(|status| status.progress)
                        .or_else(|| control.components.first()?.parse().ok());
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
        self.control(category, item, control)
            .is_some_and(|control| {
                control.sensitive
                    && !(matches!(
                        control.kind,
                        ControlKind::AudioCachePreset | ControlKind::VisualCacheQuality
                    )
                        && control
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
        self.control(category, item, control)
            .is_some_and(|control| {
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
                format!(
                    "{} {:.*}{}  {} {:.*}{}",
                    first_prefix,
                    digits,
                    output.value.x,
                    control.number.unit,
                    second_prefix,
                    digits,
                    output.value.y,
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
            let Ok(output) =
                super::vector3_expression_output(&document.target, path, timeline_id)
            else {
                return QStringList::default();
            };
            let digits = usize::try_from(control.number.digits).unwrap_or_default();
            let first_prefix = control.prefixes.first().map_or("X", String::as_str);
            let second_prefix = control.prefixes.get(1).map_or("Y", String::as_str);
            let third_prefix = control.prefixes.get(2).map_or("Z", String::as_str);
            return [
                format!(
                    "{} {:.*}{}  {} {:.*}{}  {} {:.*}{}",
                    first_prefix,
                    digits,
                    output.value.x,
                    control.number.unit,
                    second_prefix,
                    digits,
                    output.value.y,
                    control.number.unit,
                    third_prefix,
                    digits,
                    output.value.z,
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
        let display_multiplier = if control.store_multiplier == 0.0 {
            1.0
        } else {
            control.store_multiplier.recip()
        };
        [
            format!(
                "{:.*}{}",
                usize::try_from(control.number.digits).unwrap_or_default(),
                f64::from(output.value) * display_multiplier,
                control.number.unit,
            ),
            output.error.unwrap_or_default(),
        ]
        .into_iter()
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
                if control.audio_modifier {
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
                    super::set_timeline_base(&target, &control.path, value)
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
            let value = if control.kind == ControlKind::Number && control.store_multiplier != 1.0 {
                value
                    .parse::<f64>()
                    .map(|value| (value * control.store_multiplier).to_string())
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
        let Some((target, control)) = self.control_target(category, item, control) else {
            return;
        };
        let result = match control.kind {
            ControlKind::AudioCache => control
                .target_id
                .ok_or_else(|| "audio cache control has no modifier target".to_string())
                .and_then(|id| super::toggle_audio_cache(&target, id)),
            ControlKind::VisualCache => control
                .target_id
                .ok_or_else(|| "visual cache control has no modifier target".to_string())
                .and_then(|id| super::toggle_visual_cache(&target, id)),
            ControlKind::Action => control
                .action
                .ok_or_else(|| "inspector action control has no action".to_string())
                .and_then(|action| super::trigger_video_control_action(&target, action)),
            _ => Err("inspector control does not have an action".to_string()),
        };
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
        let result = if control.audio_modifier {
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
                                    default_expression: default_expression(&control),
                                },
                            )
                        })
                })
        } else if control.kind == ControlKind::LayeredNumber {
            super::set_scalar_keyframes_enabled(&target, path, enabled)
        } else if control.kind == ControlKind::LayeredVector2 {
            super::set_vector2_keyframes_enabled(&target, path, enabled)
        } else if control.kind == ControlKind::LayeredVector3 {
            super::set_vector3_keyframes_enabled(&target, path, enabled)
        } else if control.kind == ControlKind::LayeredColor {
            control
                .timeline_id
                .ok_or_else(|| "timeline color ID is unavailable".to_string())
                .and_then(|timeline_id| {
                    super::set_color_keyframes_enabled(&target, path, timeline_id, enabled)
                })
        } else if control.kind == ControlKind::LayeredBoolean {
            super::set_bool_keyframes_enabled(&target, path, enabled)
        } else if control.kind == ControlKind::LayeredSelector {
            super::set_step_keyframes_enabled(&target, path, enabled)
        } else {
            timeline_value(&control).and_then(|current| {
                super::set_timeline_mode(
                    &target,
                    path,
                    true,
                    enabled,
                    current,
                    default_expression(&control),
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
        let result = if control.audio_modifier {
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
                                    default_expression: default_expression(&control),
                                },
                            )
                        })
                })
        } else {
            timeline_value(&control).and_then(|current| {
                super::set_timeline_mode(
                    &target,
                    path,
                    false,
                    enabled,
                    current,
                    default_expression(&control),
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
        } else {
            super::set_expression_source(&target, path, &source)
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
