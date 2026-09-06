use shrimply_project::project::{ItemAddress, Project, TrackMut};

pub fn render_input_project(
    project: &Project,
    address: &ItemAddress,
    modifier_index: usize,
) -> Result<Project, String> {
    let mut render_project = project.clone();
    render_project.format_version = 0;
    render_project.name.clear();
    render_project.expanded_sequence_paths.clear();
    render_project.cursor_position = None;
    render_project.timeline_zoom = None;
    render_project.preview_guides = Box::default();
    let target_item_id = address.item_id();
    let TrackMut::Video(track) = render_project
        .track_mut(&address.track())
        .ok_or_else(|| "modifier input track no longer exists".to_string())?
    else {
        return Err("modifier input requires a video track".to_string());
    };
    for item in &mut track.items {
        if item
            .transitions
            .to_next
            .as_ref()
            .is_some_and(|transition| transition.target_item_id == target_item_id)
        {
            item.transitions.to_next = None;
        }
    }
    render_project
        .video_item_mut(address)
        .ok_or_else(|| "modifier input item no longer exists".to_string())?
        .modifiers
        .truncate(modifier_index);
    Ok(render_project)
}
