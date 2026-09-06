use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex, Weak};
use std::thread;

use hashbrown::{HashMap, HashSet};
use shrimply_project::project::{ItemAddress, Project, Time};
use shrimply_video_modifiers::{ModifierEffect, RasterModifierEffect, sam2::Sam2Modifier};
use uuid::Uuid;

use super::{Sam2MaskCache, cache_key};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct AnalysisTarget {
    pub address: ItemAddress,
    pub modifier_id: Uuid,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Status {
    Running {
        message: String,
        completed_frames: u64,
        total_frames: u64,
        prompt_signature: u64,
        server_url: String,
    },
    Complete {
        prompt_signature: u64,
    },
    Cancelling,
    Cancelled,
    Failed(String),
}

struct AnalysisState {
    run_id: RunId,
    generation: u64,
    prompt_signature: u64,
    status: Status,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RunId(Uuid);

static STATUSES: LazyLock<Mutex<HashMap<AnalysisTarget, AnalysisState>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static CANCELLATIONS: LazyLock<
    Mutex<HashMap<AnalysisTarget, (RunId, shrimply_server_client::CancellationToken)>>,
> = LazyLock::new(|| Mutex::new(HashMap::new()));
#[derive(Default)]
struct Claims {
    owners: HashMap<AnalysisTarget, RunId>,
    waiters: HashMap<AnalysisTarget, HashMap<Uuid, Weak<ClaimWake>>>,
}

struct ClaimWake {
    notification_pending: AtomicBool,
    notify: Box<dyn Fn() + Send + Sync>,
}

pub struct ClaimWaiter {
    id: Uuid,
    wake: Arc<ClaimWake>,
}

impl ClaimWaiter {
    pub fn new(notify: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            id: Uuid::new_v4(),
            wake: Arc::new(ClaimWake {
                notification_pending: AtomicBool::new(false),
                notify: Box::new(notify),
            }),
        }
    }

    pub fn consume_notification(&self) {
        self.wake
            .notification_pending
            .store(false, Ordering::Release);
    }
}

impl ClaimWake {
    fn notify(&self) {
        if self
            .notification_pending
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            (self.notify)();
        }
    }
}

static CLAIMS: LazyLock<Mutex<Claims>> = LazyLock::new(|| Mutex::new(Claims::default()));

pub struct Claim {
    target: AnalysisTarget,
    run_id: RunId,
}

impl Drop for Claim {
    fn drop(&mut self) {
        let mut claims = CLAIMS.lock().expect("SAM2 analysis claim lock is poisoned");
        if claims.owners.get(&self.target) != Some(&self.run_id) {
            return;
        }
        claims.owners.remove(&self.target);
        let waiters = claims
            .waiters
            .remove(&self.target)
            .into_iter()
            .flatten()
            .filter_map(|(_, waiter)| waiter.upgrade())
            .collect::<Vec<_>>();
        drop(claims);
        for waiter in waiters {
            waiter.notify();
        }
    }
}

pub fn try_claim(target: &AnalysisTarget, run_id: RunId, waiter: &ClaimWaiter) -> Option<Claim> {
    let statuses = STATUSES
        .lock()
        .expect("SAM2 analysis status lock is poisoned");
    if !statuses.get(target).is_some_and(|state| {
        state.run_id == run_id && matches!(&state.status, Status::Running { .. })
    }) {
        return None;
    }
    let mut claims = CLAIMS.lock().expect("SAM2 analysis claim lock is poisoned");
    let waiters = claims.waiters.entry(target.clone()).or_default();
    waiters.retain(|_, waiter| waiter.strong_count() > 0);
    waiters.insert(waiter.id, Arc::downgrade(&waiter.wake));
    if claims.owners.contains_key(target) {
        return None;
    }
    claims.owners.insert(target.clone(), run_id);
    Some(Claim {
        target: target.clone(),
        run_id,
    })
}

pub fn start(target: AnalysisTarget, generation: u64, status: Status) {
    let prompt_signature = match &status {
        Status::Running {
            prompt_signature, ..
        }
        | Status::Complete { prompt_signature } => *prompt_signature,
        Status::Cancelling | Status::Cancelled | Status::Failed(_) => {
            panic!("a SAM2 analysis must start with its prompt signature")
        }
    };
    let run_id = RunId(Uuid::new_v4());
    let cancellation = {
        let mut statuses = STATUSES
            .lock()
            .expect("SAM2 analysis status lock is poisoned");
        let replaced = statuses.insert(
            target.clone(),
            AnalysisState {
                run_id,
                generation,
                prompt_signature,
                status,
            },
        );
        let Some(replaced) = replaced else {
            return;
        };
        take_cancellation(&target, replaced.run_id)
    };
    if let Some((_, cancellation)) = cancellation {
        cancellation.cancel();
    }
}

pub fn update(target: &AnalysisTarget, run_id: RunId, status: Status) -> bool {
    let mut statuses = STATUSES
        .lock()
        .expect("SAM2 analysis status lock is poisoned");
    let Some(state) = statuses.get_mut(target) else {
        return false;
    };
    if state.run_id != run_id || matches!(&state.status, Status::Cancelling | Status::Cancelled) {
        return false;
    }
    if let Status::Running {
        prompt_signature, ..
    }
    | Status::Complete { prompt_signature } = &status
        && *prompt_signature != state.prompt_signature
    {
        return false;
    }
    state.status = status;
    true
}

pub fn is_current(target: &AnalysisTarget, run_id: RunId) -> bool {
    STATUSES
        .lock()
        .expect("SAM2 analysis status lock is poisoned")
        .get(target)
        .is_some_and(|state| {
            state.run_id == run_id && matches!(&state.status, Status::Running { .. })
        })
}

pub fn cancel(target: &AnalysisTarget, run_id: RunId) -> bool {
    let mut statuses = STATUSES
        .lock()
        .expect("SAM2 analysis status lock is poisoned");
    let Some(state) = statuses.get_mut(target) else {
        return false;
    };
    if state.run_id != run_id || !matches!(&state.status, Status::Running { .. }) {
        return false;
    }
    let cancellation = take_cancellation(target, run_id);
    state.status = if cancellation.is_some() {
        Status::Cancelling
    } else {
        Status::Cancelled
    };
    if let Some((_, cancellation)) = cancellation {
        cancellation.cancel();
    }
    true
}

pub fn clear() {
    let cancellations = {
        let mut statuses = STATUSES
            .lock()
            .expect("SAM2 analysis status lock is poisoned");
        if statuses.is_empty() {
            return;
        }
        let stale = statuses
            .drain()
            .map(|(modifier_id, state)| (modifier_id, state.run_id))
            .collect::<Vec<_>>();
        let mut cancellations = Vec::with_capacity(stale.len());
        for (target, run_id) in stale {
            if let Some((_, cancellation)) = take_cancellation(&target, run_id) {
                cancellations.push(cancellation);
            }
        }
        cancellations
    };
    for cancellation in cancellations {
        cancellation.cancel();
    }
}

pub fn set_cancellation(
    target: &AnalysisTarget,
    run_id: RunId,
    cancellation: shrimply_server_client::CancellationToken,
) -> bool {
    let statuses = STATUSES
        .lock()
        .expect("SAM2 analysis status lock is poisoned");
    let current = statuses.get(target).is_some_and(|state| {
        state.run_id == run_id && matches!(&state.status, Status::Running { .. })
    });
    if !current {
        drop(statuses);
        cancellation.cancel();
        return false;
    }
    CANCELLATIONS
        .lock()
        .expect("SAM2 cancellation lock is poisoned")
        .insert(target.clone(), (run_id, cancellation));
    true
}

pub fn clear_cancellation(target: &AnalysisTarget, run_id: RunId) {
    let mut cancellations = CANCELLATIONS
        .lock()
        .expect("SAM2 cancellation lock is poisoned");
    if cancellations
        .get(target)
        .is_some_and(|(stored_run_id, _)| *stored_run_id == run_id)
    {
        cancellations.remove(target);
    }
    drop(cancellations);
    let mut statuses = STATUSES
        .lock()
        .expect("SAM2 analysis status lock is poisoned");
    if let Some(state) = statuses
        .get_mut(target)
        .filter(|state| state.run_id == run_id)
        && matches!(&state.status, Status::Cancelling)
    {
        state.status = Status::Cancelled;
    }
}

pub fn get_for_prompt(
    target: &AnalysisTarget,
    generation: u64,
    prompt_signature: u64,
) -> Option<Status> {
    STATUSES
        .lock()
        .expect("SAM2 analysis status lock is poisoned")
        .get(target)
        .filter(|state| {
            state.generation == generation && state.prompt_signature == prompt_signature
        })
        .map(|state| state.status.clone())
}

pub fn invalidate_if_stale(
    target: &AnalysisTarget,
    generation: u64,
    prompt_signature: u64,
) -> bool {
    let cancellation = {
        let mut statuses = STATUSES
            .lock()
            .expect("SAM2 analysis status lock is poisoned");
        let Some(run_id) = statuses.get(target).and_then(|state| {
            (state.generation != generation || state.prompt_signature != prompt_signature)
                .then_some(state.run_id)
        }) else {
            return false;
        };
        statuses
            .remove(target)
            .expect("checked SAM2 analysis state must still exist");
        take_cancellation(target, run_id)
    };
    if let Some((_, cancellation)) = cancellation {
        cancellation.cancel();
    }
    true
}

fn take_cancellation(
    target: &AnalysisTarget,
    run_id: RunId,
) -> Option<(RunId, shrimply_server_client::CancellationToken)> {
    let mut cancellations = CANCELLATIONS
        .lock()
        .expect("SAM2 cancellation lock is poisoned");
    cancellations
        .get(target)
        .is_some_and(|(stored_run_id, _)| *stored_run_id == run_id)
        .then(|| {
            cancellations
                .remove(target)
                .expect("checked SAM2 cancellation must still exist")
        })
}

pub fn active_run(
    target: &AnalysisTarget,
    generation: u64,
    prompt_signature: u64,
) -> Option<(RunId, String)> {
    let statuses = STATUSES
        .lock()
        .expect("SAM2 analysis status lock is poisoned");
    let state = statuses.get(target)?;
    if state.generation != generation || state.prompt_signature != prompt_signature {
        return None;
    }
    match &state.status {
        Status::Running { server_url, .. } => Some((state.run_id, server_url.clone())),
        _ => None,
    }
}

const PROXY_JPEG_QUALITY: u32 = 95;

pub enum ProxyFrameStatus {
    Pending,
    Ready(Vec<u8>),
}

pub struct ProxyFrameRequest<'a> {
    pub project: &'a Project,
    pub target: &'a AnalysisTarget,
    pub timeline_position: Time,
    pub sequence_position: Time,
}

pub trait ProxyFrameSource {
    fn frame(&mut self, request: ProxyFrameRequest<'_>) -> Result<ProxyFrameStatus, String>;
}

pub struct Scheduler {
    waiter: ClaimWaiter,
    scheduled: HashMap<AnalysisTarget, RunId>,
}

impl Scheduler {
    pub fn new(notify: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            waiter: ClaimWaiter::new(notify),
            scheduled: HashMap::new(),
        }
    }

    pub fn consume_notification(&self) {
        self.waiter.consume_notification();
    }

    pub fn schedule_next_with<F, S, E>(
        &mut self,
        project: &Project,
        source: F,
        on_error: E,
    ) -> Result<bool, String>
    where
        F: FnOnce() -> Result<S, String> + Send + 'static,
        S: ProxyFrameSource + 'static,
        E: FnOnce(String) + Send + 'static,
    {
        self.scheduled
            .retain(|target, _| project.video_item(&target.address).is_some());
        let Some(job) = pending_analysis(project, &self.scheduled)? else {
            return Ok(false);
        };
        let target = job.target.clone();
        let run_id = job.run_id;
        if !spawn_analysis_with(project, job, &self.waiter, source, on_error) {
            return Ok(false);
        }
        self.scheduled.insert(target, run_id);
        Ok(true)
    }
}

#[derive(Clone, Copy)]
struct AnalysisFrame {
    timeline_position: Time,
    sequence_position: Time,
    cache_frame: i64,
}

#[derive(Clone)]
pub struct AnalysisJob {
    target: AnalysisTarget,
    run_id: RunId,
    prompt_signature: u64,
    cache_key: String,
    server_url: String,
    modifier: Sam2Modifier,
    frames: Vec<AnalysisFrame>,
    seed_frame: u64,
}

impl AnalysisJob {
    pub fn target(&self) -> &AnalysisTarget {
        &self.target
    }

    pub fn run_id(&self) -> RunId {
        self.run_id
    }
}

pub fn pending_analysis(
    project: &Project,
    scheduled: &HashMap<AnalysisTarget, RunId>,
) -> Result<Option<AnalysisJob>, String> {
    for address in crate::sequence::video_item_addresses(project)? {
        let item = project
            .video_item(&address)
            .ok_or_else(|| format!("SAM2 item {} no longer exists", address.item_id()))?;
        for (modifier_index, modifier) in item.modifiers.iter().enumerate() {
            if !modifier.enabled {
                continue;
            }
            let ModifierEffect::Raster(effect) = &modifier.effect else {
                continue;
            };
            let RasterModifierEffect::Sam2(sam2) = &**effect else {
                continue;
            };
            if sam2.analysis_generation == 0 || sam2.points.is_empty() && sam2.box_prompt.is_none()
            {
                continue;
            }
            let target = AnalysisTarget {
                address: address.clone(),
                modifier_id: modifier.id,
            };
            let prompt_signature = sam2.prompt_signature();
            let Some((run_id, server_url)) =
                active_run(&target, sam2.analysis_generation, prompt_signature)
            else {
                continue;
            };
            if scheduled.get(&target).copied() == Some(run_id) {
                continue;
            }
            let frames = analysis_frames(project, &address)?;
            let seed_position = sam2.seed_position.unwrap_or(frames[0].timeline_position);
            let seed_sequence_position = project
                .timeline_time_to_sequence(&address.track(), seed_position)
                .unwrap_or(frames[0].sequence_position);
            let seed_frame = frames
                .iter()
                .enumerate()
                .min_by_key(|(_, frame)| frame.sequence_position.abs_diff(seed_sequence_position))
                .and_then(|(index, _)| u64::try_from(index).ok())
                .ok_or("SAM2 analysis has too many frames")?;
            return Ok(Some(AnalysisJob {
                target,
                run_id,
                prompt_signature,
                cache_key: cache_key(project, &address, modifier.id, modifier_index, sam2)?,
                server_url,
                modifier: sam2.clone(),
                frames,
                seed_frame,
            }));
        }
    }
    Ok(None)
}

pub(super) fn analysis_frame_count(
    project: &Project,
    address: &ItemAddress,
) -> Result<usize, String> {
    analysis_frames(project, address).map(|frames| frames.len())
}

fn analysis_frames(project: &Project, address: &ItemAddress) -> Result<Vec<AnalysisFrame>, String> {
    let item = project
        .video_item(address)
        .ok_or_else(|| "SAM2 source item no longer exists".to_string())?;
    let (start, end) = project
        .projected_item_times(address)
        .ok_or("SAM2 source item is not visible on the timeline")?;
    let timeline_frames = shrimply_math_core::frame_range(start, end, project.fps)
        .ok_or("project frame rate must be positive for SAM2 analysis")?;
    let mut cache_frames = HashSet::new();
    let mut frames = Vec::new();
    for timeline_frame in timeline_frames {
        let timeline_position = shrimply_math_core::time_from_frame(timeline_frame, project.fps)
            .ok_or("project frame rate must be positive for SAM2 analysis")?;
        let Some(sequence_position) =
            project.timeline_time_to_sequence(&address.track(), timeline_position)
        else {
            continue;
        };
        if sequence_position < item.start || sequence_position >= item.end {
            continue;
        }
        let cache_frame = shrimply_math_core::frame_index(sequence_position, project.fps)
            .ok_or("project frame rate must be positive for SAM2 analysis")?;
        if cache_frames.insert(cache_frame) {
            frames.push(AnalysisFrame {
                timeline_position,
                sequence_position,
                cache_frame,
            });
        }
    }
    frames.sort_unstable_by_key(|frame| frame.cache_frame);
    if frames.is_empty() {
        return Err("cannot analyze an item shorter than one visible project frame".to_string());
    }
    Ok(frames)
}

fn update_progress(
    job: &AnalysisJob,
    message: &str,
    completed_frames: u64,
    total_frames: u64,
) -> bool {
    update(
        &job.target,
        job.run_id,
        Status::Running {
            message: message.to_string(),
            completed_frames,
            total_frames,
            prompt_signature: job.prompt_signature,
            server_url: job.server_url.clone(),
        },
    )
}

fn encode_proxy(rgba: &[u8]) -> Result<Vec<u8>, String> {
    let row_bytes = usize::try_from(super::MODEL_SIZE)
        .ok()
        .and_then(|width| width.checked_mul(4))
        .ok_or("SAM2 proxy row size overflow")?;
    let expected = row_bytes
        .checked_mul(usize::try_from(super::MODEL_SIZE).map_err(|_| "SAM2 model size overflow")?)
        .ok_or("SAM2 proxy size overflow")?;
    if rgba.len() != expected {
        return Err(format!(
            "SAM2 proxy has {} RGBA bytes; expected {expected}",
            rgba.len()
        ));
    }
    let size = i32::try_from(super::MODEL_SIZE).map_err(|_| "SAM2 model size is too large")?;
    let image = skia_safe::images::raster_from_data(
        &skia_safe::ImageInfo::new(
            (size, size),
            skia_safe::ColorType::RGBA8888,
            skia_safe::AlphaType::Opaque,
            None,
        ),
        skia_safe::Data::new_copy(rgba),
        row_bytes,
    )
    .ok_or_else(|| "create SAM2 proxy image".to_string())?;
    image
        .encode(
            None,
            skia_safe::EncodedImageFormat::JPEG,
            Some(PROXY_JPEG_QUALITY),
        )
        .map(|encoded| encoded.as_bytes().to_vec())
        .ok_or_else(|| "encode SAM2 proxy JPEG".to_string())
}

fn analyze_clip(
    project: &Project,
    job: &AnalysisJob,
    source: &mut impl ProxyFrameSource,
    mask_cache: &Sam2MaskCache,
) -> Result<bool, String> {
    let total_frames = u64::try_from(job.frames.len()).map_err(|_| "too many SAM2 frames")?;
    let item = project
        .video_item(&job.target.address)
        .ok_or_else(|| "SAM2 source item no longer exists".to_string())?;
    let seed_index = usize::try_from(job.seed_frame).map_err(|_| "SAM2 seed frame is too large")?;
    let seed = job.frames[seed_index];
    let prompt_time = shrimply_project::project::generated_item_time(item, seed.sequence_position)
        .unwrap_or(Time::ZERO);
    let request = shrimply_server_client::Sam2AnalysisRequest::new(
        match job.modifier.model {
            shrimply_video_modifiers::sam2::Sam2Model::Tiny => {
                shrimply_server_client::Sam2Model::Tiny
            }
            shrimply_video_modifiers::sam2::Sam2Model::Small => {
                shrimply_server_client::Sam2Model::Small
            }
            shrimply_video_modifiers::sam2::Sam2Model::BasePlus => {
                shrimply_server_client::Sam2Model::BasePlus
            }
            shrimply_video_modifiers::sam2::Sam2Model::Large => {
                shrimply_server_client::Sam2Model::Large
            }
        },
        total_frames,
        job.seed_frame,
        job.modifier
            .points
            .iter()
            .map(|point| {
                let position = point
                    .position
                    .value_at(prompt_time)
                    .clamp(glam::Vec2::ZERO, glam::Vec2::ONE);
                shrimply_server_client::Sam2Point {
                    position,
                    label: match point.label {
                        shrimply_video_modifiers::sam2::Sam2PointLabel::Foreground => 1,
                        shrimply_video_modifiers::sam2::Sam2PointLabel::Background => 0,
                    },
                }
            })
            .collect(),
        job.modifier
            .box_prompt
            .map(|box_prompt| shrimply_server_client::Sam2Box {
                minimum: box_prompt.min,
                maximum: box_prompt.max,
            }),
    );
    let mut archive = tempfile::NamedTempFile::new()
        .map_err(|error| format!("create SAM2 proxy archive: {error}"))?;
    shrimply_server_client::write_sam2_archive_header(archive.as_file_mut(), &request)?;
    let progress_total = total_frames.saturating_mul(2);
    if !update_progress(job, "Preparing frames…", 0, progress_total) {
        return Ok(false);
    }
    for (completed, frame) in job.frames.iter().enumerate() {
        if !is_current(&job.target, job.run_id) {
            return Ok(false);
        }
        let rgba = loop {
            match source.frame(ProxyFrameRequest {
                project,
                target: &job.target,
                timeline_position: frame.timeline_position,
                sequence_position: frame.sequence_position,
            })? {
                ProxyFrameStatus::Pending => {
                    if !is_current(&job.target, job.run_id) {
                        return Ok(false);
                    }
                    thread::yield_now();
                }
                ProxyFrameStatus::Ready(rgba) => break rgba,
            }
        };
        let jpeg = encode_proxy(&rgba)?;
        shrimply_server_client::write_sam2_archive_frame(archive.as_file_mut(), &jpeg)?;
        let completed = u64::try_from(completed)
            .map_err(|_| "SAM2 frame count is too large")?
            .saturating_add(1);
        if !update_progress(job, "Preparing frames…", completed, progress_total) {
            return Ok(false);
        }
    }
    archive
        .as_file_mut()
        .flush()
        .map_err(|error| format!("flush SAM2 proxy archive: {error}"))?;
    mask_cache.begin_analysis(&job.cache_key);
    let cancellation = shrimply_server_client::CancellationToken::new(&job.server_url)?;
    if !set_cancellation(&job.target, job.run_id, cancellation.clone()) {
        return Ok(false);
    }
    if !update_progress(job, "Sending request…", total_frames, progress_total) {
        cancellation.cancel();
        return Ok(false);
    }
    let mut server_error = None;
    let mut result_frames = None;
    let mut received = HashSet::new();
    shrimply_server_client::analyze_sam2(
        &job.server_url,
        &cancellation,
        archive.path(),
        |event| {
            if !is_current(&job.target, job.run_id) {
                return false;
            }
            match event {
                shrimply_server_client::Sam2Event::Queued { position } => update_progress(
                    job,
                    &shrimply_server_client::queued_status(position),
                    total_frames,
                    progress_total,
                ),
                shrimply_server_client::Sam2Event::Progress {
                    message,
                    completed_frames,
                    ..
                } => update_progress(
                    job,
                    &message,
                    total_frames.saturating_add(completed_frames.min(total_frames)),
                    progress_total,
                ),
                shrimply_server_client::Sam2Event::Mask { frame_index, mask } => {
                    let Some(frame) = usize::try_from(frame_index)
                        .ok()
                        .and_then(|frame_index| job.frames.get(frame_index))
                    else {
                        server_error = Some(format!(
                            "SAM2 server returned out-of-range frame {frame_index}"
                        ));
                        return false;
                    };
                    if !received.insert(frame_index) {
                        server_error = Some(format!(
                            "SAM2 server returned duplicate frame {frame_index}"
                        ));
                        return false;
                    }
                    if let Err(error) =
                        mask_cache.insert_staged(&job.cache_key, frame.cache_frame, &mask)
                    {
                        server_error = Some(error);
                        return false;
                    }
                    true
                }
                shrimply_server_client::Sam2Event::Result { completed_frames } => {
                    result_frames = Some(completed_frames);
                    true
                }
                shrimply_server_client::Sam2Event::Error { code, message } => {
                    server_error = Some(format!("SAM2 server error {code}: {message}"));
                    false
                }
            }
        },
    )?;
    if let Some(error) = server_error {
        return Err(error);
    }
    if !is_current(&job.target, job.run_id) {
        return Ok(false);
    }
    let received_frames = u64::try_from(received.len()).map_err(|_| "too many SAM2 masks")?;
    if result_frames != Some(total_frames) || received_frames != total_frames {
        return Err(format!(
            "SAM2 server returned {received_frames} unique masks and declared {}; expected {total_frames}",
            result_frames.unwrap_or(0)
        ));
    }
    Ok(true)
}

pub fn spawn_analysis_with<F, S, E>(
    project: &Project,
    job: AnalysisJob,
    claim_waiter: &ClaimWaiter,
    source: F,
    on_error: E,
) -> bool
where
    F: FnOnce() -> Result<S, String> + Send + 'static,
    S: ProxyFrameSource + 'static,
    E: FnOnce(String) + Send + 'static,
{
    let Some(claim) = try_claim(&job.target, job.run_id, claim_waiter) else {
        return false;
    };
    let project = project.clone();
    thread::Builder::new()
        .name(format!("sam2-analysis-{}", job.target.modifier_id))
        .spawn(move || {
            let _claim = claim;
            if !is_current(&job.target, job.run_id) {
                return;
            }
            let mask_cache = Sam2MaskCache::shared();
            if mask_cache.analysis_complete(&job.cache_key, job.frames.len()) {
                update(
                    &job.target,
                    job.run_id,
                    Status::Complete {
                        prompt_signature: job.prompt_signature,
                    },
                );
                return;
            }
            let result = source().and_then(|mut source| {
                if !is_current(&job.target, job.run_id) {
                    return Ok(false);
                }
                analyze_clip(&project, &job, &mut source, &mask_cache)
            });
            match result {
                Ok(true) => {
                    mask_cache.complete_analysis(&job.cache_key);
                    if !update(
                        &job.target,
                        job.run_id,
                        Status::Complete {
                            prompt_signature: job.prompt_signature,
                        },
                    ) {
                        mask_cache.abort_analysis(&job.cache_key);
                    }
                }
                Ok(false) => mask_cache.abort_analysis(&job.cache_key),
                Err(error) => {
                    mask_cache.abort_analysis(&job.cache_key);
                    let display_error = if error.starts_with("Compute server connection failed") {
                        tracing::error!(%error, "SAM2 compute connection failed");
                        "Compute server connection failed".to_string()
                    } else {
                        error.clone()
                    };
                    if update(&job.target, job.run_id, Status::Failed(display_error)) {
                        on_error(error);
                    }
                }
            }
            clear_cancellation(&job.target, job.run_id);
        })
        .expect("spawn SAM2 analysis worker");
    true
}
