use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex, Weak};

use hashbrown::HashMap;
use uuid::Uuid;

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

static STATUSES: LazyLock<Mutex<HashMap<Uuid, AnalysisState>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static CANCELLATIONS: LazyLock<
    Mutex<HashMap<Uuid, (RunId, shrimply_server_client::CancellationToken)>>,
> = LazyLock::new(|| Mutex::new(HashMap::new()));
#[derive(Default)]
struct Claims {
    owners: HashMap<Uuid, RunId>,
    waiters: HashMap<Uuid, HashMap<Uuid, Weak<ClaimWake>>>,
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
    modifier_id: Uuid,
    run_id: RunId,
}

impl Drop for Claim {
    fn drop(&mut self) {
        let mut claims = CLAIMS.lock().expect("SAM2 analysis claim lock is poisoned");
        if claims.owners.get(&self.modifier_id) != Some(&self.run_id) {
            return;
        }
        claims.owners.remove(&self.modifier_id);
        let waiters = claims
            .waiters
            .remove(&self.modifier_id)
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

pub fn try_claim(modifier_id: Uuid, run_id: RunId, waiter: &ClaimWaiter) -> Option<Claim> {
    let statuses = STATUSES
        .lock()
        .expect("SAM2 analysis status lock is poisoned");
    if !statuses.get(&modifier_id).is_some_and(|state| {
        state.run_id == run_id && matches!(&state.status, Status::Running { .. })
    }) {
        return None;
    }
    let mut claims = CLAIMS.lock().expect("SAM2 analysis claim lock is poisoned");
    let waiters = claims.waiters.entry(modifier_id).or_default();
    waiters.retain(|_, waiter| waiter.strong_count() > 0);
    waiters.insert(waiter.id, Arc::downgrade(&waiter.wake));
    if claims.owners.contains_key(&modifier_id) {
        return None;
    }
    claims.owners.insert(modifier_id, run_id);
    Some(Claim {
        modifier_id,
        run_id,
    })
}

pub fn start(modifier_id: Uuid, generation: u64, status: Status) {
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
            modifier_id,
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
        take_cancellation(modifier_id, replaced.run_id)
    };
    if let Some((_, cancellation)) = cancellation {
        cancellation.cancel();
    }
}

pub fn update(modifier_id: Uuid, run_id: RunId, status: Status) -> bool {
    let mut statuses = STATUSES
        .lock()
        .expect("SAM2 analysis status lock is poisoned");
    let Some(state) = statuses.get_mut(&modifier_id) else {
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

pub fn is_current(modifier_id: Uuid, run_id: RunId) -> bool {
    STATUSES
        .lock()
        .expect("SAM2 analysis status lock is poisoned")
        .get(&modifier_id)
        .is_some_and(|state| {
            state.run_id == run_id && matches!(&state.status, Status::Running { .. })
        })
}

pub fn cancel(modifier_id: Uuid, run_id: RunId) -> bool {
    let mut statuses = STATUSES
        .lock()
        .expect("SAM2 analysis status lock is poisoned");
    let Some(state) = statuses.get_mut(&modifier_id) else {
        return false;
    };
    if state.run_id != run_id || !matches!(&state.status, Status::Running { .. }) {
        return false;
    }
    let cancellation = take_cancellation(modifier_id, run_id);
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
        for (modifier_id, run_id) in stale {
            if let Some((_, cancellation)) = take_cancellation(modifier_id, run_id) {
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
    modifier_id: Uuid,
    run_id: RunId,
    cancellation: shrimply_server_client::CancellationToken,
) -> bool {
    let statuses = STATUSES
        .lock()
        .expect("SAM2 analysis status lock is poisoned");
    let current = statuses.get(&modifier_id).is_some_and(|state| {
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
        .insert(modifier_id, (run_id, cancellation));
    true
}

pub fn clear_cancellation(modifier_id: Uuid, run_id: RunId) {
    let mut cancellations = CANCELLATIONS
        .lock()
        .expect("SAM2 cancellation lock is poisoned");
    if cancellations
        .get(&modifier_id)
        .is_some_and(|(stored_run_id, _)| *stored_run_id == run_id)
    {
        cancellations.remove(&modifier_id);
    }
    drop(cancellations);
    let mut statuses = STATUSES
        .lock()
        .expect("SAM2 analysis status lock is poisoned");
    if let Some(state) = statuses
        .get_mut(&modifier_id)
        .filter(|state| state.run_id == run_id)
        && matches!(&state.status, Status::Cancelling)
    {
        state.status = Status::Cancelled;
    }
}

pub fn get_for_prompt(modifier_id: Uuid, generation: u64, prompt_signature: u64) -> Option<Status> {
    STATUSES
        .lock()
        .expect("SAM2 analysis status lock is poisoned")
        .get(&modifier_id)
        .filter(|state| {
            state.generation == generation && state.prompt_signature == prompt_signature
        })
        .map(|state| state.status.clone())
}

pub fn invalidate_if_stale(modifier_id: Uuid, generation: u64, prompt_signature: u64) -> bool {
    let cancellation = {
        let mut statuses = STATUSES
            .lock()
            .expect("SAM2 analysis status lock is poisoned");
        let Some(run_id) = statuses.get(&modifier_id).and_then(|state| {
            (state.generation != generation || state.prompt_signature != prompt_signature)
                .then_some(state.run_id)
        }) else {
            return false;
        };
        statuses
            .remove(&modifier_id)
            .expect("checked SAM2 analysis state must still exist");
        take_cancellation(modifier_id, run_id)
    };
    if let Some((_, cancellation)) = cancellation {
        cancellation.cancel();
    }
    true
}

fn take_cancellation(
    modifier_id: Uuid,
    run_id: RunId,
) -> Option<(RunId, shrimply_server_client::CancellationToken)> {
    let mut cancellations = CANCELLATIONS
        .lock()
        .expect("SAM2 cancellation lock is poisoned");
    cancellations
        .get(&modifier_id)
        .is_some_and(|(stored_run_id, _)| *stored_run_id == run_id)
        .then(|| {
            cancellations
                .remove(&modifier_id)
                .expect("checked SAM2 cancellation must still exist")
        })
}

pub fn active_run(
    modifier_id: Uuid,
    generation: u64,
    prompt_signature: u64,
) -> Option<(RunId, String)> {
    let statuses = STATUSES
        .lock()
        .expect("SAM2 analysis status lock is poisoned");
    let state = statuses.get(&modifier_id)?;
    if state.generation != generation || state.prompt_signature != prompt_signature {
        return None;
    }
    match &state.status {
        Status::Running { server_url, .. } => Some((state.run_id, server_url.clone())),
        _ => None,
    }
}
