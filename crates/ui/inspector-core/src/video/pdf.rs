use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock, mpsc};

use shrimply_core::timeline_value::TimelineBase;
use shrimply_pdf::PageSize;
use shrimply_project::project::{Asset, AssetSnapshot, Transform, VideoItemContent};
use shrimply_state::player_state;

use crate::{
    ControlKind, InspectorControl, InspectorController, InspectorSection, InspectorTarget,
    NumberSpec, VideoCard, VideoReset,
};

pub const PAGE_PATH: &str = "/content/page";

#[derive(Clone, Debug, PartialEq)]
pub enum PdfPages {
    Loading,
    Ready(Arc<[PageSize]>),
    Failed(String),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct CacheKey {
    path: PathBuf,
    revision: u64,
}

struct LoadedPages {
    key: CacheKey,
    snapshot: AssetSnapshot,
    result: Result<Arc<[PageSize]>, String>,
}

struct PageCache {
    results: HashMap<CacheKey, Result<Arc<[PageSize]>, String>>,
    pending: HashSet<CacheKey>,
    sender: mpsc::Sender<LoadedPages>,
    receiver: mpsc::Receiver<LoadedPages>,
}

impl Default for PageCache {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            results: HashMap::new(),
            pending: HashSet::new(),
            sender,
            receiver,
        }
    }
}

pub fn pages(source: &Asset) -> PdfPages {
    let snapshot = match source.snapshot() {
        Ok(snapshot) => snapshot,
        Err(error) => return PdfPages::Failed(error),
    };
    let key = CacheKey {
        path: snapshot.path().to_path_buf(),
        revision: snapshot.revision(),
    };
    let cache = cache();
    let mut cache = cache.lock().expect("PDF page cache mutex poisoned");
    if let Some(result) = cache.results.get(&key) {
        return match result {
            Ok(pages) => PdfPages::Ready(Arc::clone(pages)),
            Err(error) => PdfPages::Failed(error.clone()),
        };
    }
    if cache.pending.insert(key.clone()) {
        let sender = cache.sender.clone();
        std::thread::spawn(move || {
            let result = snapshot
                .read()
                .and_then(shrimply_pdf::page_sizes)
                .map(Arc::<[PageSize]>::from);
            let _ = sender.send(LoadedPages {
                key,
                snapshot,
                result,
            });
        });
    }
    PdfPages::Loading
}

pub fn poll_pages() -> bool {
    let cache = cache();
    let mut cache = cache.lock().expect("PDF page cache mutex poisoned");
    let mut changed = false;
    while let Ok(loaded) = cache.receiver.try_recv() {
        cache.pending.remove(&loaded.key);
        changed = true;
        if !loaded.snapshot.is_current() {
            continue;
        }
        cache
            .results
            .retain(|key, _| key.path != loaded.key.path || key == &loaded.key);
        cache.results.insert(loaded.key, loaded.result);
    }
    changed
}

pub fn card(item: &shrimply_project::project::VideoItem) -> VideoCard {
    let VideoItemContent::Pdf(pdf) = &item.content else {
        panic!("PDF inspector card requires a PDF video item")
    };
    let mut section = InspectorSection::default();
    match pages(&item.file) {
        PdfPages::Loading => section
            .add(InspectorControl::new(ControlKind::InfoLoading, "", "").value("Loading pages…")),
        PdfPages::Failed(error) => {
            section.add(InspectorControl::new(ControlKind::ReadOnly, "", "Error").value(error))
        }
        PdfPages::Ready(pages) => {
            let last_page = u32::try_from(pages.len() - 1).expect("PDF page count must fit u32");
            let selected_page = pdf.page.min(last_page);
            let page_count = if pages.len() == 1 {
                shrimply_i18n_core::text("1 page").into_owned()
            } else {
                shrimply_i18n_core::text_args(
                    "%{count} pages",
                    &[("count", pages.len().to_string())],
                )
            };
            section.add(
                InspectorControl::new(ControlKind::Number, PAGE_PATH, "Page")
                    .subtitle(page_count)
                    .value((selected_page + 1).to_string())
                    .number(NumberSpec {
                        minimum: 1.0,
                        maximum: pages.len() as f64,
                        drag_step: 1.0,
                        digits: 0,
                        ..NumberSpec::default()
                    })
                    .integer()
                    .immediate_commit("pdf-page"),
            );
        }
    }
    let mut card = VideoCard::new("pdf", "PDF", section);
    card.reset = Some(VideoReset {
        values: vec![(PAGE_PATH.to_string(), serde_json::Value::from(0))],
        fraction: None,
        commit_name: "reset-pdf-page",
        cancel_stabilization: false,
        paint_palette: false,
    });
    card
}

impl InspectorController {
    pub fn normalize_pdf_page(&self, target: &InspectorTarget) -> Result<(), String> {
        let InspectorTarget::Item(address) = target else {
            return Ok(());
        };
        let (source, selected_page) = {
            let project = self.project.borrow();
            let Some(item) = project.video_item(address) else {
                return Ok(());
            };
            let VideoItemContent::Pdf(pdf) = &item.content else {
                return Ok(());
            };
            (item.file.clone(), pdf.page)
        };
        let PdfPages::Ready(pages) = pages(&source) else {
            return Ok(());
        };
        let last_page = u32::try_from(pages.len() - 1).expect("PDF page count must fit u32");
        self.set_pdf_page(target, selected_page.min(last_page), "normalize-pdf-page")
    }

    pub fn set_pdf_page(
        &self,
        target: &InspectorTarget,
        page: u32,
        commit_name: &str,
    ) -> Result<(), String> {
        super::validate_video_edit(target, commit_name)?;
        let InspectorTarget::Item(address) = target else {
            unreachable!("validated PDF target must be a video item")
        };
        let source = {
            let project = self.project.borrow();
            let item = project
                .video_item(address)
                .ok_or_else(|| "video item is no longer available".to_string())?;
            if !matches!(item.content, VideoItemContent::Pdf(_)) {
                return Err("video item is not a PDF".to_string());
            }
            item.file.clone()
        };
        let pages = match pages(&source) {
            PdfPages::Ready(pages) => pages,
            PdfPages::Loading => return Err("PDF pages are still loading".to_string()),
            PdfPages::Failed(error) => return Err(error),
        };
        let size = pages
            .get(page as usize)
            .copied()
            .ok_or_else(|| format!("PDF page {} does not exist", page.saturating_add(1)))?;

        let mut project = self.project.borrow_mut();
        let item = project
            .video_item_mut(address)
            .ok_or_else(|| "video item is no longer available".to_string())?;
        let VideoItemContent::Pdf(pdf) = &item.content else {
            return Err("video item is not a PDF".to_string());
        };
        if pdf.page == page && item.source_width == size.width && item.source_height == size.height
        {
            return Ok(());
        }
        let old_center = glam::Vec2::new(item.source_width as f32, item.source_height as f32) * 0.5;
        let new_center = glam::Vec2::new(size.width as f32, size.height as f32) * 0.5;
        recenter_default_anchor(&mut item.transform, old_center, new_center);
        if let Some(transform) = &mut item.default_transform {
            recenter_default_anchor(transform, old_center, new_center);
        }
        let VideoItemContent::Pdf(pdf) = &mut item.content else {
            unreachable!("PDF item changed content while updating its page")
        };
        pdf.page = page;
        item.source_width = size.width;
        item.source_height = size.height;
        shrimply_project::project::commit_edit(&project, commit_name);
        drop(project);
        player_state::refresh_project(
            &self.player_state,
            player_state::ProjectChange {
                video: true,
                inspector: true,
                ..player_state::ProjectChange::default()
            },
        );
        Ok(())
    }
}

fn recenter_default_anchor(transform: &mut Transform, old: glam::Vec2, new: glam::Vec2) {
    if transform.anchor.expression.is_none()
        && let TimelineBase::Const(anchor) = &mut transform.anchor.base
        && *anchor == old
    {
        *anchor = new;
    }
}

fn cache() -> &'static Mutex<PageCache> {
    static CACHE: OnceLock<Mutex<PageCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(PageCache::default()))
}
