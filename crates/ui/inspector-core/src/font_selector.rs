use std::collections::{HashMap, VecDeque};
use std::ops::Range;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;

use shrimply_project::project::FontFamily as ProjectFontFamily;

use crate::font_cache::{self, FontCatalog, FontFamily, FontSource, GoogleFamily};

enum Message {
    Catalog(FontCatalog),
    Preview {
        family: FontFamily,
        generation: u64,
        result: Option<Result<(), String>>,
    },
    Lookup {
        generation: u64,
        result: Result<GoogleFamily, String>,
    },
    Activation(Result<FontFamily, String>),
}

#[derive(Default)]
pub struct Poll {
    pub changed: bool,
    pub visible_changed: bool,
    pub activations: Vec<Result<FontFamily, String>>,
}

pub struct Browser {
    sender: mpsc::Sender<Message>,
    receiver: mpsc::Receiver<Message>,
    families: Vec<FontFamily>,
    visible: Vec<FontFamily>,
    lookup: Option<GoogleFamily>,
    query: String,
    status: String,
    generation: u64,
    current_generation: Arc<AtomicU64>,
    lookup_generation: Option<u64>,
    pending_previews: HashMap<(String, i64), u64>,
    preview_queue: VecDeque<FontFamily>,
    active_previews: usize,
    catalog_loaded: bool,
    loading_catalog: bool,
    looking_up: bool,
    activating: bool,
}

impl Default for Browser {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver,
            families: Vec::new(),
            visible: Vec::new(),
            lookup: None,
            query: String::new(),
            status: String::new(),
            generation: 0,
            current_generation: Arc::new(AtomicU64::new(0)),
            lookup_generation: None,
            pending_previews: HashMap::new(),
            preview_queue: VecDeque::new(),
            active_previews: 0,
            catalog_loaded: false,
            loading_catalog: false,
            looking_up: false,
            activating: false,
        }
    }
}

impl Browser {
    pub fn open(&mut self) -> bool {
        if self.catalog_loaded || self.loading_catalog {
            return false;
        }
        self.loading_catalog = true;
        if self.families.is_empty() {
            self.status = "Loading installed fonts…".to_string();
        }
        let sender = self.sender.clone();
        thread::spawn(move || {
            let _ = sender.send(Message::Catalog(font_cache::available_families()));
        });
        true
    }

    pub fn search(&mut self, query: impl Into<String>) -> bool {
        let previous_generation = self.generation;
        let generation = self.set_query(query);
        let lookup_started = self.begin_lookup(generation);
        previous_generation != generation || lookup_started
    }

    pub fn set_query(&mut self, query: impl Into<String>) -> u64 {
        let query = query.into();
        if self.query == query {
            return self.generation;
        }
        self.query = query;
        self.lookup = None;
        self.generation = self.generation.wrapping_add(1);
        self.current_generation
            .store(self.generation, Ordering::Release);
        self.lookup_generation = None;
        self.looking_up = false;
        self.pending_previews.clear();
        self.preview_queue.clear();
        if self.loading_catalog && self.families.is_empty() {
            self.status = "Loading installed fonts…".to_string();
        } else {
            self.status.clear();
        }
        self.rebuild();
        self.generation
    }

    pub fn begin_lookup(&mut self, generation: u64) -> bool {
        if generation != self.generation || self.lookup_generation == Some(generation) {
            return false;
        }
        if self.loading_catalog && self.families.is_empty() {
            return false;
        }
        if !font_cache::google_lookup_needed(&self.families, &self.query) {
            return false;
        }
        self.lookup_generation = Some(generation);
        self.looking_up = true;
        self.status = "Searching Google Fonts…".to_string();
        let generation = self.generation;
        let query = self.query.clone();
        let sender = self.sender.clone();
        thread::spawn(move || {
            let _ = sender.send(Message::Lookup {
                generation,
                result: font_cache::lookup_google_family(&query),
            });
        });
        true
    }

    pub fn request_previews(&mut self, range: Range<usize>) {
        let end = range.end.min(self.visible.len());
        for family in self
            .visible
            .get(range.start.min(end)..end)
            .into_iter()
            .flatten()
        {
            if family.source != FontSource::Google
                || family.revision < 0
                || font_cache::preview_source(family, self.lookup.as_ref()).is_ok()
            {
                continue;
            }
            let key = preview_key(family);
            if let Some(generation) = self.pending_previews.get_mut(&key) {
                *generation = self.generation;
            } else {
                self.pending_previews.insert(key, self.generation);
                self.preview_queue.push_back(family.clone());
            }
        }
        self.start_preview_jobs();
    }

    pub fn activate(&mut self, family: FontFamily) -> Result<(), String> {
        if self.activating {
            return Err("A font is already being activated".to_string());
        }
        self.activating = true;
        self.status = if family.revision < 0 {
            "Downloading font family…"
        } else {
            "Loading cached font…"
        }
        .to_string();
        let lookup = self.lookup.clone();
        let sender = self.sender.clone();
        thread::spawn(move || {
            let _ = sender.send(Message::Activation(font_cache::activate_selection(
                family,
                lookup.as_ref(),
            )));
        });
        Ok(())
    }

    pub fn poll(&mut self) -> Poll {
        let mut poll = Poll::default();
        while let Ok(message) = self.receiver.try_recv() {
            match message {
                Message::Catalog(catalog) => {
                    poll.changed = true;
                    poll.visible_changed = true;
                    self.loading_catalog = false;
                    self.catalog_loaded = true;
                    self.families = catalog.families;
                    if let Some(error) = catalog.cache_error {
                        self.status = error;
                    } else if !self.looking_up {
                        self.status.clear();
                    }
                    self.rebuild();
                    self.begin_lookup(self.generation);
                }
                Message::Preview {
                    family,
                    generation,
                    result,
                } => {
                    self.active_previews = self.active_previews.saturating_sub(1);
                    let key = preview_key(&family);
                    let requested_generation = self.pending_previews.get(&key).copied();
                    if requested_generation == Some(generation) {
                        self.pending_previews.remove(&key);
                    }
                    let current = self.visible.iter().any(|candidate| {
                        candidate.source == family.source
                            && candidate.revision == family.revision
                            && candidate.name.eq_ignore_ascii_case(&family.name)
                    });
                    match result {
                        Some(Ok(()))
                            if generation == self.generation
                                && current
                                && requested_generation == Some(generation) =>
                        {
                            poll.changed = true;
                        }
                        Some(Ok(())) | None => {}
                        Some(Err(error)) if generation == self.generation => tracing::warn!(
                            family = family.name,
                            revision = family.revision,
                            "Could not prepare Google font preview: {error}"
                        ),
                        Some(Err(_)) => {}
                    }
                    self.start_preview_jobs();
                }
                Message::Lookup { generation, result } if generation == self.generation => {
                    poll.changed = true;
                    poll.visible_changed = true;
                    self.looking_up = false;
                    match result {
                        Ok(family) => {
                            self.lookup = Some(family);
                            self.status.clear();
                        }
                        Err(error) => self.status = error,
                    }
                    self.rebuild();
                }
                Message::Lookup { .. } => {}
                Message::Activation(result) => {
                    poll.changed = true;
                    poll.visible_changed = true;
                    self.activating = false;
                    match &result {
                        Ok(family) => {
                            if let Some(existing) = self.families.iter_mut().find(|candidate| {
                                candidate.source == family.source
                                    && candidate.name.eq_ignore_ascii_case(&family.name)
                            }) {
                                existing.clone_from(family);
                            } else {
                                self.families.push(family.clone());
                            }
                            self.status.clear();
                            self.rebuild();
                        }
                        Err(error) => self.status.clone_from(error),
                    }
                    poll.activations.push(result);
                }
            }
        }
        poll
    }

    pub fn visible(&self) -> &[FontFamily] {
        &self.visible
    }

    pub fn lookup(&self) -> Option<&GoogleFamily> {
        self.lookup.as_ref()
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn busy(&self) -> bool {
        self.loading_catalog || self.looking_up || self.activating
    }

    fn rebuild(&mut self) {
        self.visible =
            font_cache::matching_families(&self.families, &self.query, self.lookup.as_ref());
    }

    fn start_preview_jobs(&mut self) {
        const MAX_PREVIEW_WORKERS: usize = 2;
        while self.active_previews < MAX_PREVIEW_WORKERS {
            let Some(family) = self.preview_queue.pop_front() else {
                break;
            };
            if !self.pending_previews.contains_key(&preview_key(&family)) {
                continue;
            }
            let generation = self.generation;
            self.active_previews += 1;
            let sender = self.sender.clone();
            let current_generation = Arc::clone(&self.current_generation);
            thread::spawn(move || {
                let result = (current_generation.load(Ordering::Acquire) == generation)
                    .then(|| font_cache::prepare_cached_preview(&family))
                    .filter(|_| current_generation.load(Ordering::Acquire) == generation);
                let _ = sender.send(Message::Preview {
                    family,
                    generation,
                    result,
                });
            });
        }
    }
}

fn preview_key(family: &FontFamily) -> (String, i64) {
    (family.name.to_lowercase(), family.revision)
}

pub fn normalized_families(families: &[ProjectFontFamily]) -> Vec<ProjectFontFamily> {
    families
        .iter()
        .filter_map(|family| {
            let name = family.name().trim();
            (!name.is_empty()).then(|| with_name(family, name.to_string()))
        })
        .collect()
}

pub fn replace_family(
    families: &[ProjectFontFamily],
    index: usize,
    family: ProjectFontFamily,
) -> Option<Vec<ProjectFontFamily>> {
    let mut next = normalized_families(families);
    if index >= next.len()
        || next.iter().enumerate().any(|(candidate_index, candidate)| {
            candidate_index != index && candidate.name().eq_ignore_ascii_case(family.name())
        })
    {
        return None;
    }
    next[index] = family;
    Some(next)
}

pub fn append_family(
    families: &[ProjectFontFamily],
    family: ProjectFontFamily,
) -> Option<Vec<ProjectFontFamily>> {
    let mut next = normalized_families(families);
    if next
        .iter()
        .any(|candidate| candidate.name().eq_ignore_ascii_case(family.name()))
    {
        return None;
    }
    next.push(family);
    Some(next)
}

pub fn move_family(
    families: &[ProjectFontFamily],
    index: usize,
    offset: isize,
) -> Option<Vec<ProjectFontFamily>> {
    let mut next = normalized_families(families);
    let destination = index.checked_add_signed(offset)?;
    if index >= next.len() || destination >= next.len() {
        return None;
    }
    next.swap(index, destination);
    Some(next)
}

pub fn remove_family(
    families: &[ProjectFontFamily],
    index: usize,
) -> Option<Vec<ProjectFontFamily>> {
    let mut next = normalized_families(families);
    (index < next.len()).then(|| {
        next.remove(index);
        next
    })
}

fn with_name(family: &ProjectFontFamily, name: String) -> ProjectFontFamily {
    match family {
        ProjectFontFamily::Local { .. } => ProjectFontFamily::Local { name },
        ProjectFontFamily::GoogleFonts { .. } => ProjectFontFamily::GoogleFonts { name },
    }
}
