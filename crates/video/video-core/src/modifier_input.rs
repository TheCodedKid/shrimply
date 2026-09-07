use shrimply_project::project::{ItemAddress, Project, TrackMut, VideoItem};
use std::borrow::Cow;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CaptureBranch {
    #[default]
    Normal,
    Item,
    Host,
}

pub fn capture_branch(target: &ItemAddress, address: &ItemAddress) -> CaptureBranch {
    if target == address {
        CaptureBranch::Item
    } else if target.sequence_path().starts_with(address.sequence_path())
        && target.sequence_path().get(address.sequence_path().len()) == Some(&address.item_id())
    {
        CaptureBranch::Host
    } else {
        CaptureBranch::Normal
    }
}

/// Modifier inputs include the source transform and preceding modifiers, but
/// exclude clip compositing and transitions. Ancestor hosts only carry timing.
/// Normalize a local render item, retaining the original project for expressions
/// and external masks. Preserve property IDs so repeated captures remain stable.
pub fn capture_item(mut item: Cow<'_, VideoItem>, branch: CaptureBranch) -> Cow<'_, VideoItem> {
    if branch == CaptureBranch::Normal {
        return item;
    }
    use shrimply_core::timeline_value::TimelineBase;
    let value = item.to_mut();
    value.compositing.opacity.base = TimelineBase::Const(1.0);
    value.compositing.opacity.expression = None;
    value.compositing.blend_mode.base = TimelineBase::Const(Default::default());
    value.compositing.blend_mode.expression = None;
    value.compositing.alpha_mask = None;
    value.motion_blur.enabled = false;
    value.transitions = Default::default();
    if branch == CaptureBranch::Host {
        value.modifiers.clear();
        value.alpha_mask_video = None;
    }
    item
}

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
