use crate::compositor::Compositor;
use shrimply_project::project::{CanvasSize, ItemAddress, Project, Time};
use shrimply_video_core::transparent_fill::analysis::{self, FrameSource, FrameStatus, RunId};
use uuid::Uuid;

impl FrameSource for Compositor {
    fn frame(
        &mut self,
        project: &Project,
        position: Time,
        address: &ItemAddress,
        width: u32,
        height: u32,
    ) -> Result<FrameStatus, String> {
        self.set_capture_target(shrimply_preview_render_core::CaptureTarget::ModifierInput {
            address: address.clone(),
            snap_content: true,
        });
        objc2::rc::autoreleasepool(|_| {
            self.poll_accurate_image(project, position)?
                .map_or(Ok(FrameStatus::Pending), |image| {
                    crate::capture::rgba(&image, CanvasSize { width, height })
                        .map(FrameStatus::Ready)
                })
        })
    }
}

pub fn analyze(
    project: Project,
    address: &ItemAddress,
    modifier_id: Uuid,
) -> Result<RunId, String> {
    analysis::analyze_with(project, address, modifier_id, || Ok(Compositor::default()))
}
