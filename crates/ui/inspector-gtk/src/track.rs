use adw::prelude::*;
use shrimply_gtk_components::{
    tr,
    ui::{StringChoice, labeled_string_selector, switch_row},
};

use shrimply_inspector_core::TrackPresentation;
use shrimply_project::project::{ItemKind, Project, TrackAddress, caption_languages};

use super::{Inspectable, InspectorContext, item::flat, list, section::InspectorSection};

pub(super) struct TrackInspection(TrackPresentation);

impl TrackInspection {
    pub(super) fn resolve(project: &Project, address: TrackAddress) -> Option<Self> {
        shrimply_inspector_core::track::presentation(project, address).map(Self)
    }
}

impl Inspectable for TrackInspection {
    fn title(&self) -> &'static str {
        self.0.title()
    }

    fn add_rows(&self, _section: &InspectorSection, _context: &InspectorContext) {}

    fn inspect(&self, context: &InspectorContext) -> Vec<gtk::Widget> {
        let track = InspectorSection::controls();
        let controller = context.inspector_core.clone();
        let address = self.0.address.clone();
        let enabled = switch_row(
            "Enabled",
            Some("Include this track in playback and export"),
            self.0.enabled,
            move |next| {
                if let Err(error) = controller.set_track_enabled(&address, next) {
                    tracing::error!(%error, "Could not change GTK track enabled state");
                }
            },
        );
        track.add_wide_control(&enabled);

        if self.0.kind == ItemKind::Caption {
            let language = labeled_string_selector(
                "Language",
                self.0.language.as_deref().unwrap_or_default(),
                std::iter::once(StringChoice {
                    value: String::new(),
                    label: tr!("None").into_owned(),
                })
                .chain(caption_languages().iter().map(|language| StringChoice {
                    value: language.clone(),
                    label: language.clone(),
                }))
                .collect(),
                {
                    let controller = context.inspector_core.clone();
                    let address = self.0.address.clone();
                    move |language| {
                        let language = (!language.is_empty()).then_some(language);
                        if let Err(error) =
                            controller.set_caption_track_language(&address, language.as_deref())
                        {
                            tracing::error!(%error, "Could not change GTK caption track language");
                        }
                    }
                },
            );
            track.add_wide_control(language.widget());
        }

        let info = adw::PreferencesGroup::new();
        for detail in self.0.details() {
            info.add(
                &adw::ActionRow::builder()
                    .title(tr!(detail.label).as_ref())
                    .subtitle(detail.value)
                    .build(),
            );
        }

        list::render_categories(
            vec![
                list::InspectorCategory {
                    key: "track",
                    label: "Track",
                    icon: "sliders-horizontal-symbolic",
                    items: vec![flat(track.into_widget())],
                },
                list::InspectorCategory {
                    key: "info",
                    label: "Info",
                    icon: "info-outline-symbolic",
                    items: vec![flat(info)],
                },
            ],
            context,
        )
    }
}
