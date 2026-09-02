use serde_json::Value;
use shrimply_project::project::{
    Interpolation, ItemAddress, ItemMut, Project, TransitionSide, VideoItemContent,
    VisualClipTransitionKind, VisualTransitionKind,
};
use shrimply_state::player_state::{self, ProjectChange};

use crate::{
    ControlKind, InspectorCapabilities, InspectorControl, InspectorController, InspectorSection,
    InspectorTarget, NumberSpec,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransitionType {
    Visual,
    Audio,
    VisualClip,
    AudioClip,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TransitionPresentation {
    pub title: &'static str,
    pub side: TransitionSide,
    pub kind: TransitionType,
    pub value: Value,
    pub capabilities: InspectorCapabilities,
}

impl TransitionPresentation {
    pub fn section(&self) -> InspectorSection {
        match self.kind {
            TransitionType::Visual => visual_section(&self.value, self.side, self.capabilities),
            TransitionType::Audio => audio_section(&self.value),
            TransitionType::VisualClip => visual_clip_section(&self.value),
            TransitionType::AudioClip => audio_clip_section(&self.value),
        }
    }
}

pub fn presentation(
    project: &Project,
    item: &ItemAddress,
    side: TransitionSide,
) -> Option<TransitionPresentation> {
    match item {
        ItemAddress::Video { .. } => {
            let item = project.video_item(item)?;
            if side == TransitionSide::Outro
                && let Some(transition) = item.transitions.to_next.as_ref()
            {
                return Some(TransitionPresentation {
                    title: "Transition",
                    side,
                    kind: TransitionType::VisualClip,
                    value: serialize(transition),
                    capabilities: InspectorCapabilities::default(),
                });
            }
            let transition = match side {
                TransitionSide::Intro => item.transitions.intro.as_ref(),
                TransitionSide::Outro => item.transitions.outro.as_ref(),
            }?;
            Some(TransitionPresentation {
                title: side_title(side),
                side,
                kind: TransitionType::Visual,
                value: serialize(transition),
                capabilities: InspectorCapabilities {
                    vector_transitions: item.supports_vector_transitions(),
                    text: matches!(&item.content, VideoItemContent::Text(_)),
                    drawing: matches!(&item.content, VideoItemContent::Paint(_)),
                },
            })
        }
        ItemAddress::Audio { .. } => {
            let item = project.audio_item(item)?;
            if side == TransitionSide::Outro
                && let Some(transition) = item.transitions.to_next.as_ref()
            {
                return Some(TransitionPresentation {
                    title: "Transition",
                    side,
                    kind: TransitionType::AudioClip,
                    value: serialize(transition.as_ref()),
                    capabilities: InspectorCapabilities::default(),
                });
            }
            let transition = match side {
                TransitionSide::Intro => item.transitions.intro.as_ref(),
                TransitionSide::Outro => item.transitions.outro.as_ref(),
            }?;
            Some(TransitionPresentation {
                title: side_title(side),
                side,
                kind: TransitionType::Audio,
                value: serialize(transition),
                capabilities: InspectorCapabilities::default(),
            })
        }
        ItemAddress::Caption { .. } => None,
    }
}

fn side_title(side: TransitionSide) -> &'static str {
    match side {
        TransitionSide::Intro => "Intro",
        TransitionSide::Outro => "Outro",
    }
}

fn serialize(value: &impl serde::Serialize) -> Value {
    serde_json::to_value(value).expect("inspector transition must serialize")
}

impl InspectorController {
    pub(crate) fn set_transition_kind(
        &self,
        target: &InspectorTarget,
        text: &str,
    ) -> Result<(), String> {
        self.set_transition_field(target, "/kind", text, "transition-kind", true)
    }

    pub fn set_transition_field(
        &self,
        target: &InspectorTarget,
        path: &str,
        text: &str,
        commit_name: &str,
        commit_immediately: bool,
    ) -> Result<(), String> {
        self.edit_transition(target, commit_name, commit_immediately, |value| {
            let current = value
                .pointer_mut(path)
                .ok_or_else(|| format!("transition field is no longer available: {path}"))?;
            let replacement = crate::model::parsed_value(current, text)?;
            if *current == replacement {
                return Ok((false, None));
            }
            *current = replacement.clone();
            Ok((true, (path == "/kind").then_some(replacement)))
        })
    }

    pub fn set_transition_components(
        &self,
        target: &InspectorTarget,
        path: &str,
        components: &[(usize, String)],
        commit_name: &str,
        commit_immediately: bool,
    ) -> Result<(), String> {
        self.edit_transition(target, commit_name, commit_immediately, |value| {
            let current = value
                .pointer_mut(path)
                .and_then(Value::as_object_mut)
                .ok_or_else(|| "transition color is no longer available".to_string())?;
            let mut changed = false;
            for (component, text) in components {
                let key = *["r", "g", "b", "a"]
                    .get(*component)
                    .ok_or_else(|| "transition color component is invalid".to_string())?;
                let replacement = text
                    .parse::<serde_json::Number>()
                    .map(Value::Number)
                    .map_err(|_| format!("invalid transition color component: {text}"))?;
                let value = current
                    .get_mut(key)
                    .ok_or_else(|| "transition color component is unavailable".to_string())?;
                if *value != replacement {
                    *value = replacement;
                    changed = true;
                }
            }
            Ok((changed, None))
        })
    }

    pub fn commit_transition_field(
        &self,
        target: &InspectorTarget,
        commit_name: &str,
    ) -> Result<(), String> {
        validate_commit_name(commit_name)?;
        if !matches!(target, InspectorTarget::Transition { .. }) {
            return Err("inspector target is not a transition".to_string());
        }
        let project = self.project.borrow();
        crate::model::target_value(&project, target)
            .ok_or_else(|| "inspector transition is no longer available".to_string())?;
        shrimply_project::project::commit_edit(&project, commit_name);
        drop(project);
        player_state::refresh_project(
            &self.player_state,
            ProjectChange {
                inspector: true,
                ..ProjectChange::default()
            },
        );
        Ok(())
    }

    fn edit_transition(
        &self,
        target: &InspectorTarget,
        commit_name: &str,
        commit_immediately: bool,
        edit: impl FnOnce(&mut Value) -> Result<(bool, Option<Value>), String>,
    ) -> Result<(), String> {
        validate_commit_name(commit_name)?;
        let InspectorTarget::Transition { item, side } = target else {
            return Err("inspector target is not a transition".to_string());
        };
        let mut project = self.project.borrow_mut();
        let mut value = crate::model::target_value(&project, target)
            .ok_or_else(|| "inspector transition is no longer available".to_string())?
            .1;
        let (changed, kind) = edit(&mut value)?;
        if !changed {
            return Ok(());
        }
        if let Some(kind) = kind {
            set_kind(&mut project, item, *side, kind)?;
        } else {
            crate::model::replace_target(&mut project, target, value)?;
        }
        if commit_immediately {
            shrimply_project::project::commit_edit(&project, commit_name);
        }
        drop(project);
        player_state::refresh_project(
            &self.player_state,
            crate::refresh::target_change(target, None, commit_immediately),
        );
        Ok(())
    }
}

fn set_kind(
    project: &mut Project,
    item: &ItemAddress,
    side: TransitionSide,
    kind: Value,
) -> Result<(), String> {
    match project
        .item_mut(item)
        .ok_or_else(|| "transition is no longer available".to_string())?
    {
        ItemMut::Video(item) => {
            if side == TransitionSide::Outro
                && let Some(transition) = item.transitions.to_next.as_mut()
            {
                transition.set_kind(deserialize::<VisualClipTransitionKind>(kind)?);
            } else {
                let kind = deserialize::<VisualTransitionKind>(kind)?;
                match side {
                    TransitionSide::Intro => item.transitions.intro.as_mut(),
                    TransitionSide::Outro => item.transitions.outro.as_mut(),
                }
                .ok_or_else(|| "visual transition is no longer available".to_string())?
                .set_kind(side, kind);
            }
        }
        ItemMut::Audio(item) => {
            let transition = match side {
                TransitionSide::Intro => item.transitions.intro.as_mut(),
                TransitionSide::Outro => item.transitions.outro.as_mut(),
            }
            .ok_or_else(|| "audio transition is no longer available".to_string())?;
            transition.kind = deserialize(kind)?;
        }
        ItemMut::Caption(_) => {
            return Err("captions do not have inspector transitions".to_string());
        }
    }
    Ok(())
}

fn deserialize<T: serde::de::DeserializeOwned>(value: Value) -> Result<T, String> {
    serde_json::from_value(value).map_err(|error| format!("invalid inspector value: {error}"))
}

fn validate_commit_name(commit_name: &str) -> Result<(), String> {
    if commit_name.is_empty() {
        Err("transition edit has no commit name".to_string())
    } else {
        Ok(())
    }
}

fn visual_clip_section(transition: &Value) -> InspectorSection {
    let kind = text(transition, "/kind");
    let mut section = InspectorSection::default();
    section.add(crate::selector::selector(
        "/kind",
        "Kind",
        kind,
        [
            ("cross_fade", "Cross Fade"),
            ("fade_through_color", "Fade Through Color"),
            ("wipe", "Wipe"),
            ("morph", "Morph"),
            ("iris", "Iris"),
            ("clock_wipe", "Clock Wipe"),
            ("dissolve", "Dissolve"),
            ("slide", "Slide"),
            ("push", "Push"),
            ("zoom", "Zoom"),
        ]
        .map(string_choice),
    ));
    section.add(interpolation_control(transition, "/interpolation", "Curve"));
    match kind {
        "fade_through_color" => section.add(
            color_control(transition, "/fade_color", "Color", false).tooltip("Fade-through color"),
        ),
        "wipe" => {
            section.add(number_control(
                transition,
                "/direction_degrees",
                "Direction",
                NumberSpec {
                    minimum: -180.0,
                    maximum: 180.0,
                    drag_step: 1.0,
                    digits: 1,
                    unit: "°",
                },
            ));
            section.add(number_control(
                transition,
                "/softness",
                "Softness",
                NumberSpec {
                    minimum: 0.0,
                    maximum: f64::from(
                        shrimply_project::project::MAX_VISUAL_CLIP_TRANSITION_SOFTNESS,
                    ),
                    drag_step: 0.01,
                    digits: 2,
                    unit: "",
                },
            ));
        }
        "iris" => {
            add_center(&mut section, transition, "/center");
            section.add(crate::selector::selector(
                "/iris_from_inside",
                "Direction",
                boolean(transition, "/iris_from_inside").to_string(),
                [("true", "From inside"), ("false", "From outside")].map(string_choice),
            ));
            section.add(number_control(
                transition,
                "/softness",
                "Softness",
                NumberSpec {
                    minimum: 0.0,
                    maximum: f64::from(
                        shrimply_project::project::MAX_VISUAL_CLIP_TRANSITION_SOFTNESS,
                    ),
                    drag_step: 0.01,
                    digits: 2,
                    unit: "",
                },
            ));
        }
        "clock_wipe" => {
            add_center(&mut section, transition, "/center");
            section.add(number_control(
                transition,
                "/direction_degrees",
                "Starting angle",
                NumberSpec {
                    minimum: -180.0,
                    maximum: 180.0,
                    drag_step: 1.0,
                    digits: 1,
                    unit: "°",
                },
            ));
            section.add(crate::selector::selector(
                "/clockwise",
                "Direction",
                boolean(transition, "/clockwise").to_string(),
                [("true", "Clockwise"), ("false", "Counterclockwise")].map(string_choice),
            ));
            section.add(number_control(
                transition,
                "/softness",
                "Softness",
                NumberSpec {
                    minimum: 0.0,
                    maximum: f64::from(
                        shrimply_project::project::MAX_VISUAL_CLIP_TRANSITION_CLOCK_SOFTNESS,
                    ),
                    drag_step: 0.01,
                    digits: 2,
                    unit: "",
                },
            ));
        }
        "dissolve" => section.add(number_control(
            transition,
            "/dissolve_grain_size",
            "Grain size",
            NumberSpec {
                minimum: 1.0,
                maximum: f64::from(
                    shrimply_project::project::MAX_VISUAL_CLIP_TRANSITION_DISSOLVE_GRAIN_SIZE,
                ),
                drag_step: 1.0,
                digits: 0,
                unit: "px",
            },
        )),
        "slide" | "push" => {
            section.add(number_control(
                transition,
                "/direction_degrees",
                "Direction",
                NumberSpec {
                    minimum: -180.0,
                    maximum: 180.0,
                    drag_step: 1.0,
                    digits: 1,
                    unit: "°",
                },
            ));
            section.add(boolean_control(transition, "/fade_opacity", "Fade opacity"));
        }
        "zoom" => {
            add_center(&mut section, transition, "/center");
            section.add(number_control(
                transition,
                "/zoom_start_scale",
                "Starting scale",
                NumberSpec {
                    minimum: 0.0,
                    maximum: f64::from(
                        shrimply_project::project::MAX_VISUAL_CLIP_TRANSITION_ZOOM_SCALE,
                    ),
                    drag_step: 0.01,
                    digits: 2,
                    unit: "x",
                },
            ));
            section.add(boolean_control(transition, "/fade_opacity", "Fade opacity"));
        }
        _ => {}
    }
    edit_controls(
        section,
        "edit-visual-clip-transition",
        "visual-clip-transition-config",
    )
}

fn audio_clip_section(transition: &Value) -> InspectorSection {
    let curve = text(transition, "/curve");
    let mut section = InspectorSection::default();
    section.add(crate::selector::selector(
        "/curve",
        "Curve",
        curve,
        [("equal_power", "Equal Power"), ("linear", "Linear")].map(string_choice),
    ));
    edit_controls(
        section,
        "edit-audio-clip-transition",
        "edit-audio-clip-transition",
    )
}

fn visual_section(
    transition: &Value,
    side: TransitionSide,
    capabilities: InspectorCapabilities,
) -> InspectorSection {
    let kind = text(transition, "/kind");
    let mut kinds = vec![
        string_choice(("fade", "Fade")),
        string_choice(("slide", "Slide")),
        string_choice(("slide_fade", "Slide + Fade")),
        string_choice(("wipe", "Wipe")),
        string_choice(("iris", "Iris")),
        string_choice(("clock_wipe", "Clock Wipe")),
        string_choice(("zoom", "Zoom")),
        string_choice(("spin", "Spin")),
        string_choice(("blur", "Blur")),
        string_choice(("pixelate", "Pixelate")),
        string_choice(("dissolve", "Dissolve")),
        string_choice(("triangular_fold", "Triangular Fold")),
        string_choice(("origami", "Origami")),
        string_choice(("streak_wipe", "Streak Wipe")),
    ];
    if capabilities.text {
        kinds.push(string_choice(("morph", "Morph")));
    }
    if capabilities.drawing {
        kinds.push(string_choice(("drawing", "Drawing")));
    }
    if capabilities.vector_transitions {
        kinds.extend(vector_transition_choices(side));
    }

    let mut section = InspectorSection::default();
    section.add(crate::selector::selector("/kind", "Kind", kind, kinds));
    section.add(interpolation_control(transition, "/interpolation", "Curve"));
    if capabilities.text && kind == "morph" {
        section.add(crate::selector::selector(
            "/morph_unit",
            "Morph by",
            text(transition, "/morph_unit"),
            [("letter", "Letter"), ("word", "Word")].map(string_choice),
        ));
    }
    if capabilities.drawing && kind == "drawing" {
        section.add(
            scaled_number_control(
                transition,
                "/drawing_stroke_overlap",
                "Stroke overlap",
                100.0,
                NumberSpec {
                    minimum: -100.0,
                    maximum: 100.0,
                    drag_step: 1.0,
                    digits: 0,
                    unit: "%",
                },
            )
            .store_multiplier(0.01)
            .live_commit("transition-drawing-overlap"),
        );
        section.add(
            scaled_number_control(
                transition,
                "/drawing_stroke_length_weight",
                "Length timing",
                100.0,
                NumberSpec {
                    minimum: 0.0,
                    maximum: 100.0,
                    drag_step: 1.0,
                    digits: 0,
                    unit: "%",
                },
            )
            .store_multiplier(0.01)
            .live_commit("transition-drawing-length"),
        );
        section.add(crate::selector::selector(
            "/drawing_fill_mode",
            "Fill",
            text(transition, "/drawing_fill_mode"),
            [
                ("fade_together", "Fade together"),
                ("fade_sequentially", "Fade one by one"),
                ("direct", "Direct"),
            ]
            .map(string_choice),
        ));
    }
    if matches!(kind, "slide" | "slide_fade") {
        section.add(
            number_control(
                transition,
                "/slide_rotation_degrees",
                "Rotation",
                NumberSpec {
                    drag_step: 1.0,
                    digits: 1,
                    unit: "°",
                    ..NumberSpec::default()
                },
            )
            .live_commit("transition-slide-rotation"),
        );
        section.add(
            number_control(
                transition,
                "/slide_distance",
                "Distance",
                NumberSpec {
                    minimum: 0.0,
                    drag_step: 1.0,
                    digits: 1,
                    unit: "px",
                    ..NumberSpec::default()
                },
            )
            .live_commit("transition-slide-distance"),
        );
    }
    match kind {
        "wipe" => {
            section.add(effect_number(
                transition,
                "/effect_angle_degrees",
                "Direction",
                NumberSpec {
                    minimum: -180.0,
                    maximum: 180.0,
                    drag_step: 1.0,
                    digits: 1,
                    unit: "°",
                },
            ));
            section.add(effect_number(
                transition,
                "/effect_detail",
                "Softness",
                NumberSpec {
                    minimum: 0.0,
                    maximum: 0.5,
                    drag_step: 0.01,
                    digits: 2,
                    unit: "",
                },
            ));
        }
        "iris" => {
            let amount = number(transition, "/effect_amount");
            let (inside, outside) = match side {
                TransitionSide::Intro => ("1", "0"),
                TransitionSide::Outro => ("0", "1"),
            };
            section.add(crate::selector::selector(
                "/effect_amount",
                "Direction",
                if match side {
                    TransitionSide::Intro => amount >= 0.5,
                    TransitionSide::Outro => amount < 0.5,
                } {
                    inside
                } else {
                    outside
                },
                [
                    (inside.to_string(), "From inside".to_string()),
                    (outside.to_string(), "From outside".to_string()),
                ],
            ));
            section.add(effect_number(
                transition,
                "/iris_center/0",
                "Center X",
                NumberSpec {
                    minimum: 0.0,
                    maximum: 1.0,
                    drag_step: 0.01,
                    digits: 2,
                    unit: "",
                },
            ));
            section.add(effect_number(
                transition,
                "/iris_center/1",
                "Center Y",
                NumberSpec {
                    minimum: 0.0,
                    maximum: 1.0,
                    drag_step: 0.01,
                    digits: 2,
                    unit: "",
                },
            ));
            section.add(effect_number(
                transition,
                "/effect_detail",
                "Softness",
                NumberSpec {
                    minimum: 0.0,
                    maximum: 0.5,
                    drag_step: 0.01,
                    digits: 2,
                    unit: "",
                },
            ));
        }
        "clock_wipe" => {
            section.add(effect_number(
                transition,
                "/effect_angle_degrees",
                "Starting angle",
                NumberSpec {
                    minimum: -180.0,
                    maximum: 180.0,
                    drag_step: 1.0,
                    digits: 1,
                    unit: "°",
                },
            ));
            section.add(effect_number(
                transition,
                "/effect_detail",
                "Softness",
                NumberSpec {
                    minimum: 0.0,
                    maximum: 0.25,
                    drag_step: 0.01,
                    digits: 2,
                    unit: "",
                },
            ));
            section.add(crate::selector::selector(
                "/effect_amount",
                "Direction",
                if number(transition, "/effect_amount") >= 0.5 {
                    "1"
                } else {
                    "0"
                },
                [("1", "Clockwise"), ("0", "Counterclockwise")].map(string_choice),
            ));
        }
        "zoom" => section.add(effect_number(
            transition,
            "/effect_amount",
            "Starting scale",
            NumberSpec {
                minimum: 0.0,
                maximum: 2.0,
                drag_step: 0.01,
                digits: 2,
                unit: "x",
            },
        )),
        "spin" => {
            section.add(effect_number(
                transition,
                "/effect_amount",
                "Starting scale",
                NumberSpec {
                    minimum: 0.0,
                    maximum: 2.0,
                    drag_step: 0.01,
                    digits: 2,
                    unit: "x",
                },
            ));
            section.add(effect_number(
                transition,
                "/effect_angle_degrees",
                "Rotation",
                NumberSpec {
                    minimum: -1440.0,
                    maximum: 1440.0,
                    drag_step: 1.0,
                    digits: 1,
                    unit: "°",
                },
            ));
        }
        "blur" | "pixelate" => {
            let (label, maximum) = if kind == "blur" {
                ("Maximum radius", 100.0)
            } else {
                ("Maximum block size", 512.0)
            };
            section.add(effect_number(
                transition,
                "/effect_amount",
                label,
                NumberSpec {
                    minimum: 1.0,
                    maximum,
                    drag_step: 1.0,
                    digits: 0,
                    unit: "px",
                },
            ));
            section.add(boolean_control(transition, "/effect_fade", "Fade opacity"));
        }
        "dissolve" => section.add(effect_number(
            transition,
            "/effect_detail",
            "Grain size",
            NumberSpec {
                minimum: 1.0,
                maximum: 64.0,
                drag_step: 1.0,
                digits: 0,
                unit: "px",
            },
        )),
        "triangular_fold" => {
            section.add(effect_number(
                transition,
                "/effect_detail",
                "Fold size",
                NumberSpec {
                    minimum: 32.0,
                    maximum: 512.0,
                    drag_step: 1.0,
                    digits: 0,
                    unit: "px",
                },
            ));
            section.add(effect_number(
                transition,
                "/effect_amount",
                "Fold depth",
                NumberSpec {
                    minimum: 0.0,
                    maximum: 1.0,
                    drag_step: 0.01,
                    digits: 2,
                    unit: "",
                },
            ));
            section.add(effect_number(
                transition,
                "/effect_angle_degrees",
                "Direction",
                NumberSpec {
                    minimum: -180.0,
                    maximum: 180.0,
                    drag_step: 1.0,
                    digits: 1,
                    unit: "°",
                },
            ));
        }
        "origami" => {
            section.add(effect_number(
                transition,
                "/effect_detail",
                "Complexity",
                NumberSpec {
                    minimum: 2.0,
                    maximum: 6.0,
                    drag_step: 1.0,
                    digits: 0,
                    unit: "",
                },
            ));
            section.add(effect_number(
                transition,
                "/effect_amount",
                "Fold depth",
                NumberSpec {
                    minimum: 0.0,
                    maximum: 1.0,
                    drag_step: 0.01,
                    digits: 2,
                    unit: "",
                },
            ));
            section.add(effect_number(
                transition,
                "/effect_angle_degrees",
                "Direction",
                NumberSpec {
                    minimum: -180.0,
                    maximum: 180.0,
                    drag_step: 1.0,
                    digits: 1,
                    unit: "°",
                },
            ));
        }
        "streak_wipe" => {
            section.add(effect_number(
                transition,
                "/effect_angle_degrees",
                "Direction",
                NumberSpec {
                    minimum: -180.0,
                    maximum: 180.0,
                    drag_step: 1.0,
                    digits: 1,
                    unit: "°",
                },
            ));
            section.add(effect_number(
                transition,
                "/effect_detail",
                "Line width",
                NumberSpec {
                    minimum: 1.0,
                    maximum: 256.0,
                    drag_step: 1.0,
                    digits: 0,
                    unit: "px",
                },
            ));
            section.add(effect_number(
                transition,
                "/effect_amount",
                "Variation",
                NumberSpec {
                    minimum: 0.0,
                    maximum: 1.0,
                    drag_step: 0.01,
                    digits: 2,
                    unit: "",
                },
            ));
            section.add(effect_number(
                transition,
                "/effect_softness",
                "Edge softness",
                NumberSpec {
                    minimum: 0.0,
                    maximum: 128.0,
                    drag_step: 1.0,
                    digits: 1,
                    unit: "px",
                },
            ));
        }
        "coalesce" => {
            section.add(effect_number(
                transition,
                "/effect_amount",
                "Softness",
                NumberSpec {
                    minimum: 0.25,
                    maximum: 2.5,
                    drag_step: 0.05,
                    digits: 2,
                    unit: "",
                },
            ));
            section.add(effect_number(
                transition,
                "/effect_detail",
                "Pools",
                NumberSpec {
                    minimum: 2.0,
                    maximum: 5.0,
                    drag_step: 1.0,
                    digits: 0,
                    unit: "",
                },
            ));
        }
        "contour_current" => {
            section.add(effect_number(
                transition,
                "/effect_amount",
                "Line width",
                NumberSpec {
                    minimum: 0.25,
                    maximum: 4.0,
                    drag_step: 0.05,
                    digits: 2,
                    unit: "",
                },
            ));
            section.add(effect_number(
                transition,
                "/effect_detail",
                "Trail length",
                NumberSpec {
                    minimum: 0.04,
                    maximum: 0.7,
                    drag_step: 0.01,
                    digits: 2,
                    unit: "",
                },
            ));
        }
        "soft_refraction" => {
            section.add(effect_number(
                transition,
                "/effect_amount",
                "Strength",
                NumberSpec {
                    minimum: 0.0,
                    maximum: 3.0,
                    drag_step: 0.05,
                    digits: 2,
                    unit: "",
                },
            ));
            section.add(effect_number(
                transition,
                "/effect_detail",
                "Texture scale",
                NumberSpec {
                    minimum: 0.25,
                    maximum: 3.0,
                    drag_step: 0.05,
                    digits: 2,
                    unit: "",
                },
            ));
        }
        "morphological_resolve" => {
            section.add(effect_number(
                transition,
                "/effect_amount",
                "Amount",
                NumberSpec {
                    minimum: 0.0,
                    maximum: 3.0,
                    drag_step: 0.05,
                    digits: 2,
                    unit: "",
                },
            ));
            section.add(effect_number(
                transition,
                "/effect_detail",
                "Softness",
                NumberSpec {
                    minimum: 0.0,
                    maximum: 2.0,
                    drag_step: 0.05,
                    digits: 2,
                    unit: "",
                },
            ));
        }
        "living_fill" => {
            section.add(effect_number(
                transition,
                "/effect_amount",
                "Band width",
                NumberSpec {
                    minimum: 0.03,
                    maximum: 0.6,
                    drag_step: 0.01,
                    digits: 2,
                    unit: "",
                },
            ));
            section.add(effect_number(
                transition,
                "/effect_detail",
                "Softness",
                NumberSpec {
                    minimum: 0.05,
                    maximum: 1.0,
                    drag_step: 0.01,
                    digits: 2,
                    unit: "",
                },
            ));
            section.add(effect_number(
                transition,
                "/effect_angle_degrees",
                "Direction",
                NumberSpec {
                    minimum: -180.0,
                    maximum: 180.0,
                    drag_step: 1.0,
                    digits: 1,
                    unit: "°",
                },
            ));
        }
        "diffusion" | "reverse_diffusion" => {
            section.add(effect_number(
                transition,
                "/effect_amount",
                "Amount",
                NumberSpec {
                    minimum: 0.0,
                    maximum: 3.0,
                    drag_step: 0.05,
                    digits: 2,
                    unit: "",
                },
            ));
            section.add(effect_number(
                transition,
                "/effect_detail",
                "Detail",
                NumberSpec {
                    minimum: 0.25,
                    maximum: 3.0,
                    drag_step: 0.05,
                    digits: 2,
                    unit: "",
                },
            ));
            section.add(boolean_control(transition, "/effect_fade", "Fade opacity"));
            section.add(boolean_control(
                transition,
                "/effect_evolve_seed",
                "Evolve seed",
            ));
            if boolean(transition, "/effect_evolve_seed") {
                section.add(
                    effect_number(
                        transition,
                        "/effect_seed_frequency",
                        "Seed frequency",
                        NumberSpec {
                            minimum: 1.0,
                            maximum: 60.0,
                            drag_step: 1.0,
                            digits: 0,
                            unit: "Hz",
                        },
                    )
                    .live_commit("transition-seed-frequency"),
                );
            }
        }
        _ => {}
    }
    if matches!(
        kind,
        "write"
            | "create"
            | "facet_assembly"
            | "coalesce"
            | "contour_current"
            | "soft_refraction"
            | "morphological_resolve"
            | "living_fill"
            | "diffusion"
            | "reverse_diffusion"
    ) {
        section.add(crate::selector::selector(
            "/write_ordering",
            "Ordering",
            text(transition, "/write_ordering"),
            [
                ("sequential", "Sequential"),
                ("simultaneous", "Simultaneous"),
            ]
            .map(string_choice),
        ));
    }
    edit_controls(
        section,
        "edit-visual-transition",
        "transition-effect-config",
    )
}

fn audio_section(transition: &Value) -> InspectorSection {
    let mut section = InspectorSection::default();
    section.add(crate::selector::selector(
        "/kind",
        "Kind",
        text(transition, "/kind"),
        [string_choice(("fade", "Fade"))],
    ));
    section.add(interpolation_control(transition, "/interpolation", "Curve"));
    edit_controls(section, "edit-audio-transition", "edit-audio-transition")
}

fn edit_controls(
    mut section: InspectorSection,
    immediate_name: &'static str,
    live_name: &'static str,
) -> InspectorSection {
    for control in &mut section.controls {
        if control.commit_name.is_empty() {
            control.commit_name = match control.kind {
                ControlKind::Number => live_name,
                ControlKind::Boolean | ControlKind::Color | ControlKind::Selector => {
                    control.commit_immediately = true;
                    immediate_name
                }
                _ => panic!("unsupported transition control kind"),
            }
            .to_string();
        }
    }
    section
}

fn interpolation_control(transition: &Value, path: &str, label: &str) -> InspectorControl {
    crate::selector::selector(
        path,
        label,
        text(transition, path),
        Interpolation::CONTINUOUS.into_iter().map(|value| {
            (
                serde_json::to_value(value)
                    .expect("interpolation must serialize")
                    .as_str()
                    .expect("interpolation must serialize as text")
                    .to_string(),
                value.label().to_string(),
            )
        }),
    )
}

fn vector_transition_choices(side: TransitionSide) -> Vec<(String, String)> {
    let write = match side {
        TransitionSide::Intro => "Write",
        TransitionSide::Outro => "Unwrite",
    };
    let diffusion = match side {
        TransitionSide::Intro => ("reverse_diffusion", "Reverse Diffusion"),
        TransitionSide::Outro => ("diffusion", "Diffusion"),
    };
    [
        ("write", write),
        ("create", "Create"),
        ("facet_assembly", "Facet Assembly"),
        ("coalesce", "Coalesce"),
        ("contour_current", "Contour Current"),
        ("soft_refraction", "Soft Refraction"),
        ("morphological_resolve", "Morphological Resolve"),
        ("living_fill", "Living Fill"),
        diffusion,
    ]
    .map(string_choice)
    .into()
}

fn text<'a>(value: &'a Value, path: &str) -> &'a str {
    value
        .pointer(path)
        .and_then(Value::as_str)
        .expect("transition selector value must be text")
}

fn string_choice((value, label): (&str, &str)) -> (String, String) {
    (value.to_string(), label.to_string())
}

fn number_control(value: &Value, path: &str, label: &str, number: NumberSpec) -> InspectorControl {
    let value = value
        .pointer(path)
        .and_then(Value::as_f64)
        .expect("transition number must be numeric");
    InspectorControl::new(ControlKind::Number, path, label)
        .value(value.to_string())
        .number(number)
}

fn scaled_number_control(
    value: &Value,
    path: &str,
    label: &str,
    display_multiplier: f64,
    spec: NumberSpec,
) -> InspectorControl {
    InspectorControl::new(ControlKind::Number, path, label)
        .value((number(value, path) * display_multiplier).to_string())
        .number(spec)
}

fn effect_number(value: &Value, path: &str, label: &str, spec: NumberSpec) -> InspectorControl {
    number_control(value, path, label, spec)
}

fn number(value: &Value, path: &str) -> f64 {
    value
        .pointer(path)
        .and_then(Value::as_f64)
        .expect("transition number must be numeric")
}

fn color_control(value: &Value, path: &str, label: &str, with_alpha: bool) -> InspectorControl {
    let value = value
        .pointer(path)
        .and_then(Value::as_object)
        .expect("transition color must be an object");
    let mut components = ["r", "g", "b", "a"]
        .map(|component| {
            value
                .get(component)
                .and_then(Value::as_u64)
                .expect("transition color component must be an integer")
                .to_string()
        })
        .to_vec();
    if !with_alpha {
        components[3] = u8::MAX.to_string();
    }
    let control = InspectorControl::new(ControlKind::Color, path, label).components(components);
    if with_alpha {
        control
    } else {
        control.without_alpha()
    }
}

fn add_center(section: &mut InspectorSection, value: &Value, path: &str) {
    for (index, axis) in ["X", "Y"].into_iter().enumerate() {
        section.add(number_control(
            value,
            &format!("{path}/{index}"),
            &format!("Center {axis}"),
            NumberSpec {
                minimum: 0.0,
                maximum: 1.0,
                drag_step: 0.01,
                digits: 2,
                unit: "",
            },
        ));
    }
}

fn boolean(value: &Value, path: &str) -> bool {
    value
        .pointer(path)
        .and_then(Value::as_bool)
        .expect("transition toggle must be boolean")
}

fn boolean_control(value: &Value, path: &str, label: &str) -> InspectorControl {
    InspectorControl::new(ControlKind::Boolean, path, label).value(boolean(value, path).to_string())
}
