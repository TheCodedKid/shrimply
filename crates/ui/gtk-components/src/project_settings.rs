use std::cell::Cell;
use std::rc::Rc;

use adw::prelude::*;
use shrimply_component_core::project_settings::{CUSTOM_PRESET_INDEX, ProjectSettings};
use shrimply_project_core::{COMMON_FRAME_RATES, PROJECT_PRESETS};

pub struct ProjectSettingsSelector {
    pub preset: adw::ComboRow,
    pub width: adw::SpinRow,
    pub height: adw::SpinRow,
    pub fps: adw::ComboRow,
    model: Rc<Cell<ProjectSettings>>,
}

impl ProjectSettingsSelector {
    pub fn new() -> Self {
        let initial = ProjectSettings::default();
        let mut preset_labels = PROJECT_PRESETS
            .iter()
            .map(|preset| preset.label)
            .collect::<Vec<_>>();
        preset_labels.push("Custom");
        let preset_labels = preset_labels
            .iter()
            .map(|label| crate::i18n::text(label))
            .collect::<Vec<_>>();
        let preset_labels = preset_labels
            .iter()
            .map(|label| label.as_ref())
            .collect::<Vec<_>>();
        let preset = adw::ComboRow::builder()
            .title(crate::i18n::text("Preset").as_ref())
            .model(&gtk::StringList::new(&preset_labels))
            .selected(initial.preset as u32)
            .build();
        let width = adw::SpinRow::with_range(1.0, 16_384.0, 1.0);
        width.set_title(crate::i18n::text("Width").as_ref());
        width.set_value(f64::from(initial.width));
        width.set_digits(0);
        let height = adw::SpinRow::with_range(1.0, 16_384.0, 1.0);
        height.set_title(crate::i18n::text("Height").as_ref());
        height.set_value(f64::from(initial.height));
        height.set_digits(0);
        let labels = COMMON_FRAME_RATES
            .iter()
            .map(|rate| rate.label)
            .collect::<Vec<_>>();
        let fps = adw::ComboRow::builder()
            .title(crate::i18n::text("Frame Rate").as_ref())
            .model(&gtk::StringList::new(&labels))
            .selected(initial.frame_rate as u32)
            .build();

        let updating = Rc::new(Cell::new(false));
        let model = Rc::new(Cell::new(initial));
        preset.connect_selected_notify({
            let width = width.clone();
            let height = height.clone();
            let fps = fps.clone();
            let updating = updating.clone();
            let model = model.clone();
            move |row| {
                let index = row.selected() as usize;
                if index == CUSTOM_PRESET_INDEX {
                    return;
                }
                let mut settings = model.get();
                settings.select_preset(index);
                model.set(settings);
                updating.set(true);
                width.set_value(f64::from(settings.width));
                height.set_value(f64::from(settings.height));
                fps.set_selected(settings.frame_rate as u32);
                updating.set(false);
            }
        });
        width.connect_value_notify({
            let preset = preset.clone();
            let updating = updating.clone();
            let model = model.clone();
            move |width| {
                if updating.get() {
                    return;
                }
                let mut settings = model.get();
                settings.set_width(width.value().round() as u32);
                model.set(settings);
                preset.set_selected(CUSTOM_PRESET_INDEX as u32);
            }
        });
        height.connect_value_notify({
            let preset = preset.clone();
            let updating = updating.clone();
            let model = model.clone();
            move |height| {
                if updating.get() {
                    return;
                }
                let mut settings = model.get();
                settings.set_height(height.value().round() as u32);
                model.set(settings);
                preset.set_selected(CUSTOM_PRESET_INDEX as u32);
            }
        });
        fps.connect_selected_notify({
            let preset = preset.clone();
            let updating = updating.clone();
            let model = model.clone();
            move |fps| {
                if updating.get() {
                    return;
                }
                let mut settings = model.get();
                settings.set_frame_rate(fps.selected() as usize);
                model.set(settings);
                preset.set_selected(CUSTOM_PRESET_INDEX as u32);
            }
        });
        Self {
            preset,
            width,
            height,
            fps,
            model,
        }
    }

    pub fn settings(
        &self,
    ) -> Option<(
        shrimply_project_core::CanvasSize,
        shrimply_math_core::Fraction,
    )> {
        self.model.get().settings()
    }
}

impl Default for ProjectSettingsSelector {
    fn default() -> Self {
        Self::new()
    }
}
