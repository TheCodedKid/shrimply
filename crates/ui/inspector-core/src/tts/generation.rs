use std::sync::mpsc::{self, Receiver, TryRecvError};

use shrimply_server_client::CancellationToken;
use shrimply_tts::{TtsModel, TtsSettings};

use super::TtsGeneration;

pub enum GenerationEvent {
    Progress(String),
    Done(Result<TtsGeneration, String>),
}

enum Message {
    Progress(String),
    Done(Result<UnclaimedGeneration, String>),
}

// A result abandoned in the channel must not leave an orphaned audio file.
struct UnclaimedGeneration(Option<TtsGeneration>);

impl Drop for UnclaimedGeneration {
    fn drop(&mut self) {
        if let Some(generation) = &self.0 {
            super::remove_generated(&generation.path);
        }
    }
}

/// UI-independent generation worker. Keep this alive across inspector rebuilds
/// and poll it on the platform's normal UI timer.
pub struct GenerationTask {
    cancellation: CancellationToken,
    receiver: Option<Receiver<Message>>,
}

impl GenerationTask {
    /// Call before dispatching work: the output directory belongs to the
    /// project active at invocation, even if another project is opened later.
    pub fn start(server_url: &str, model: TtsModel, settings: TtsSettings) -> Result<Self, String> {
        let directory = shrimply_project::project::project_directory();
        let cancellation = CancellationToken::new(server_url)?;
        let worker_cancellation = cancellation.clone();
        let server_url = server_url.to_string();
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let result = super::generate_in(
                &directory,
                &server_url,
                &worker_cancellation,
                &model,
                &settings,
                |message| {
                    sender.send(Message::Progress(message.to_string())).is_ok()
                        && !worker_cancellation.is_cancelled()
                },
            )
            .map(|generation| UnclaimedGeneration(Some(generation)));
            let _ = sender.send(Message::Done(result));
        });
        Ok(Self {
            cancellation,
            receiver: Some(receiver),
        })
    }

    pub fn cancel(&self) {
        self.cancellation.cancel();
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancellation.is_cancelled()
    }

    /// Empty means the worker is still running, or its one terminal event has
    /// already been consumed. A disconnected worker produces a terminal error.
    pub fn poll(&mut self) -> Option<GenerationEvent> {
        let message = self.receiver.as_ref()?.try_recv();
        match message {
            Ok(Message::Progress(message)) => Some(GenerationEvent::Progress(message)),
            Ok(Message::Done(result)) => {
                self.receiver = None;
                let result = result.and_then(|mut pending| {
                    if self.is_cancelled() {
                        Err("Cancelled".to_string())
                    } else {
                        Ok(pending.0.take().expect("unclaimed TTS generation"))
                    }
                });
                Some(GenerationEvent::Done(result))
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                self.receiver = None;
                Some(GenerationEvent::Done(Err(
                    "Generation worker stopped unexpectedly".to_string(),
                )))
            }
        }
    }
}

impl Drop for GenerationTask {
    fn drop(&mut self) {
        if self.receiver.is_some() {
            self.cancel();
        }
    }
}
