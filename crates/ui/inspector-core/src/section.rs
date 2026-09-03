const DEFAULT_MINIMUM: f64 = -1_000_000.0;
const DEFAULT_MAXIMUM: f64 = 1_000_000.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraphPoint {
    pub time: shrimply_project::project::Time,
    pub value: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraphSegment {
    pub owner_id: uuid::Uuid,
    pub start: shrimply_project::project::Time,
    pub end: shrimply_project::project::Time,
    pub start_value: f64,
    pub end_value: f64,
    pub interpolation: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScalarGraph {
    pub points: Vec<GraphPoint>,
    pub segments: Vec<GraphSegment>,
    pub range: (
        shrimply_project::project::Time,
        shrimply_project::project::Time,
    ),
    pub frame_step: shrimply_project::project::Time,
    pub playhead: shrimply_project::project::Time,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlKind {
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
    OptionalSelector,
    OptionalNumberSelector,
    Vector2,
    Vector3,
    LayeredNumber,
    LayeredBoolean,
    LayeredSelector,
    AudioCache,
    AudioCachePreset,
    VisualCache,
    VisualCacheQuality,
    AudioModifierMenu,
    VisualModifierMenu,
    TtsEditor,
    BeatDetection,
    LayeredVector2,
    LayeredVector3,
    ProjectSettings,
    Performance,
    InfoHeading,
    InfoArtwork,
    FileLocation,
    InfoLoading,
    Action,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InspectorControlAction {
    RebuildVideoStabilization,
    AddDitheringPaletteColor {
        modifier_id: uuid::Uuid,
    },
    RemoveDitheringPaletteColor {
        modifier_id: uuid::Uuid,
        color_id: uuid::Uuid,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct NumberSpec {
    pub minimum: f64,
    pub maximum: f64,
    pub drag_step: f64,
    pub digits: i32,
    pub unit: &'static str,
}

impl Default for NumberSpec {
    fn default() -> Self {
        Self {
            minimum: DEFAULT_MINIMUM,
            maximum: DEFAULT_MAXIMUM,
            drag_step: 1.0,
            digits: 2,
            unit: "",
        }
    }
}

use crate::LayeredState;

#[derive(Clone, Debug, PartialEq)]
pub struct InspectorControl {
    pub path: String,
    pub timeline_path: Option<String>,
    pub label: String,
    pub subtitle: String,
    pub tooltip: String,
    pub value: String,
    pub components: Vec<String>,
    pub kind: ControlKind,
    pub editable: bool,
    pub sensitive: bool,
    pub visible: bool,
    pub number: NumberSpec,
    pub accepted_range: Option<(f64, f64)>,
    pub integer: bool,
    pub width_characters: i32,
    pub prefixes: Vec<String>,
    pub suffix: String,
    pub prefix_icon: String,
    pub prefix_icon_rotates: bool,
    pub prefix_icon_rotation_offset_degrees: f64,
    pub values: Vec<String>,
    pub labels: Vec<String>,
    pub icons: Vec<String>,
    pub search_terms: Vec<String>,
    pub layered: LayeredState,
    pub lock: bool,
    pub with_alpha: bool,
    pub store_multiplier: f64,
    pub target_id: Option<uuid::Uuid>,
    pub audio_modifier: bool,
    pub commit_name: String,
    pub commit_immediately: bool,
    pub timeline_id: Option<uuid::Uuid>,
    pub scalar_graph: Option<ScalarGraph>,
    pub action: Option<InspectorControlAction>,
    pub busy: bool,
}

impl InspectorControl {
    pub fn new(kind: ControlKind, path: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            timeline_path: None,
            label: label.into(),
            subtitle: String::new(),
            tooltip: String::new(),
            value: String::new(),
            components: Vec::new(),
            kind,
            editable: true,
            sensitive: true,
            visible: true,
            number: NumberSpec::default(),
            accepted_range: None,
            integer: false,
            width_characters: 8,
            prefixes: Vec::new(),
            suffix: String::new(),
            prefix_icon: String::new(),
            prefix_icon_rotates: false,
            prefix_icon_rotation_offset_degrees: 0.0,
            values: Vec::new(),
            labels: Vec::new(),
            icons: Vec::new(),
            search_terms: Vec::new(),
            layered: LayeredState::default(),
            lock: false,
            with_alpha: true,
            store_multiplier: 1.0,
            target_id: None,
            audio_modifier: false,
            commit_name: String::new(),
            commit_immediately: false,
            timeline_id: None,
            scalar_graph: None,
            action: None,
            busy: false,
        }
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    pub fn immediate_commit(mut self, name: impl Into<String>) -> Self {
        self.commit_name = name.into();
        self.commit_immediately = true;
        self
    }

    pub fn live_commit(mut self, name: impl Into<String>) -> Self {
        self.commit_name = name.into();
        self.commit_immediately = false;
        self
    }

    pub fn action(mut self, action: InspectorControlAction) -> Self {
        self.action = Some(action);
        self
    }

    pub fn busy(mut self, busy: bool) -> Self {
        self.busy = busy;
        self
    }

    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = subtitle.into();
        self
    }

    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = tooltip.into();
        self
    }

    pub fn components(mut self, components: Vec<String>) -> Self {
        self.components = components;
        self
    }

    pub fn read_only(mut self) -> Self {
        self.editable = false;
        self
    }

    pub fn sensitive(mut self, sensitive: bool) -> Self {
        self.sensitive = sensitive;
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn number(mut self, number: NumberSpec) -> Self {
        self.number = number;
        self
    }

    pub fn accepted_range(mut self, minimum: f64, maximum: f64) -> Self {
        assert!(minimum <= maximum, "accepted range must be ordered");
        self.accepted_range = Some((minimum, maximum));
        self
    }

    pub fn integer(mut self) -> Self {
        self.integer = true;
        self
    }

    pub fn width_characters(mut self, width: i32) -> Self {
        assert!(width > 0, "control width must be positive");
        self.width_characters = width;
        self
    }

    pub fn prefixes(mut self, prefixes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.prefixes = prefixes.into_iter().map(Into::into).collect();
        self
    }

    pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = suffix.into();
        self
    }

    pub fn rotating_icon(mut self, icon: impl Into<String>, rotation_offset_degrees: f64) -> Self {
        assert!(
            rotation_offset_degrees.is_finite(),
            "icon rotation offset must be finite",
        );
        self.prefix_icon = icon.into();
        self.prefix_icon_rotates = true;
        self.prefix_icon_rotation_offset_degrees = rotation_offset_degrees;
        self
    }

    pub fn choices(mut self, values: Vec<String>, labels: Vec<String>) -> Self {
        assert_eq!(
            values.len(),
            labels.len(),
            "selector values must have labels"
        );
        self.values = values;
        self.labels = labels;
        self
    }

    pub fn choice_icons(mut self, icons: Vec<String>) -> Self {
        assert_eq!(
            self.values.len(),
            icons.len(),
            "selector values must have icon entries",
        );
        self.icons = icons;
        self
    }

    pub fn choice_search_terms(mut self, search_terms: Vec<String>) -> Self {
        assert_eq!(
            self.values.len(),
            search_terms.len(),
            "selector values must have search entries",
        );
        self.search_terms = search_terms;
        self
    }

    pub fn layered(mut self, timeline_path: impl Into<String>, layered: LayeredState) -> Self {
        self.timeline_path = Some(timeline_path.into());
        self.layered = layered;
        self
    }

    pub fn lock(mut self) -> Self {
        self.lock = true;
        self
    }

    pub fn without_alpha(mut self) -> Self {
        self.with_alpha = false;
        self
    }

    pub fn store_multiplier(mut self, multiplier: f64) -> Self {
        assert!(multiplier.is_finite(), "storage multiplier must be finite");
        self.store_multiplier = multiplier;
        self
    }

    pub fn target(mut self, id: uuid::Uuid) -> Self {
        self.target_id = Some(id);
        self
    }

    pub fn timeline(mut self, id: uuid::Uuid, graph: Option<ScalarGraph>) -> Self {
        self.timeline_id = Some(id);
        self.scalar_graph = graph;
        self
    }

    pub fn graph(mut self, graph: Option<ScalarGraph>) -> Self {
        self.scalar_graph = graph;
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct InspectorSection {
    pub controls: Vec<InspectorControl>,
}

impl InspectorSection {
    pub fn add(&mut self, control: InspectorControl) {
        self.controls.push(control);
    }

    pub fn set_sensitive(&mut self, sensitive: bool) {
        for control in &mut self.controls {
            control.sensitive = sensitive;
        }
    }
}
