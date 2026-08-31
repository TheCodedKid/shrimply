use rhai::{Engine, Scope};
use shrimply_core::modifier_model::ModifierModel;
use shrimply_video_modifiers::{ModifierEffect, ModifierSource, ModifierState, VisualKind};

pub const EXPRESSION_SOURCE: &str = "value * 2.0";
pub const EXPRESSION_INPUT: f64 = 42.0;

pub fn modifier_names() -> Vec<&'static str> {
    ModifierEffect::catalog()
        .filter_map(|effect| effect.adapted_for(demo_modifier_state()))
        .map(|effect| effect.display_name())
        .collect()
}

fn demo_modifier_state() -> ModifierState {
    ModifierState {
        source: ModifierSource::Image,
        kind: VisualKind::Raster,
        pristine: true,
    }
}

pub fn expression_output(source: &str) -> String {
    let mut engine = Engine::new();
    engine.set_max_call_levels(16).set_max_operations(10_000);
    let mut scope = Scope::new();
    scope.push("value", EXPRESSION_INPUT);
    match engine.eval_with_scope::<f64>(&mut scope, source) {
        Ok(value) => format!("Output · {value:.1}"),
        Err(error) => format!("Error · {error}"),
    }
}
