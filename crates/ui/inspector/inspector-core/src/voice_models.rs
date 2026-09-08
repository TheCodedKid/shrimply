use crate::{InspectorControl, model_catalog::ModelCatalog};

#[derive(Default)]
pub struct VoiceModels {
    catalog: ModelCatalog,
}

impl VoiceModels {
    pub fn poll(&mut self) -> bool {
        self.catalog.poll()
    }

    pub fn populate(&mut self, server_url: &str, control: &mut InspectorControl) {
        let state = self.catalog.request(
            server_url,
            crate::audio_modifiers::cached_voice_change_model_catalog(server_url).map(Ok),
            crate::voice_change_model_catalog,
        );
        let mut models = match state {
            Some(Ok(models)) => models,
            None => {
                control.sensitive = false;
                control.subtitle = "Loading Pneuma models…".into();
                Vec::new()
            }
            Some(Err(error)) => {
                control.sensitive = false;
                control.subtitle = error;
                Vec::new()
            }
        };
        if !models.contains(&control.value) {
            models.insert(0, control.value.clone());
        }
        control.labels = models.clone();
        control.values = models;
    }
}
