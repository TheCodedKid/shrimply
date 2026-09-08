pub use shrimply_manim_state::{Update, error, parameters, set_parameters};

pub fn apply(
    project: &std::rc::Rc<std::cell::RefCell<shrimply_project_document::project::Project>>,
    player: &crate::player_state::SharedPlayerState,
    update: Update,
) {
    let result = shrimply_manim_state::apply(&mut project.borrow_mut(), update);
    if let Some(result) = result {
        crate::player_state::refresh_project(
            player,
            crate::player_state::ProjectChange {
                duration: result.duration,
                video: result.video,
                inspector: result.inspector,
                ..Default::default()
            },
        );
    }
}
