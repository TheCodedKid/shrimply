use std::{collections::HashMap, sync::mpsc};

type Models<T> = Result<Vec<T>, String>;
type Response<T> = (String, Models<T>);

/// URL-scoped asynchronous catalogs survive inspector document rebuilds.
pub(crate) struct ModelCatalog<T = String> {
    states: HashMap<String, Option<Models<T>>>,
    sender: mpsc::Sender<Response<T>>,
    receiver: mpsc::Receiver<Response<T>>,
}

impl<T> Default for ModelCatalog<T> {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            states: HashMap::new(),
            sender,
            receiver,
        }
    }
}

impl<T: Clone + Send + 'static> ModelCatalog<T> {
    pub fn retry_failed(&mut self, server_url: &str) {
        if matches!(self.states.get(server_url), Some(Some(Err(_)))) {
            self.states.remove(server_url);
        }
    }

    pub fn poll(&mut self) -> bool {
        let mut changed = false;
        while let Ok((url, result)) = self.receiver.try_recv() {
            self.states.insert(url, Some(result));
            changed = true;
        }
        changed
    }

    pub fn request(
        &mut self,
        server_url: &str,
        cached: Option<Models<T>>,
        load: fn(&str) -> Models<T>,
    ) -> Option<Models<T>> {
        self.states
            .entry(server_url.to_string())
            .or_insert_with(|| {
                if cached.is_some() {
                    return cached;
                }
                let url = server_url.to_string();
                let sender = self.sender.clone();
                std::thread::spawn(move || {
                    let result = load(&url);
                    let _ = sender.send((url, result));
                });
                None
            })
            .clone()
    }
}
