use std::cell::RefCell;
use std::rc::Rc;

use shrimply_core::timeline_value::TimelineValue;
use shrimply_inspector_core::audio_generator::{AudioGeneratorControl, AudioGeneratorNumber};
use shrimply_project::project::{AudioGenerator, AudioSource, Project};

use crate::player_state::{self, ProjectChange, SharedPlayerState};
use crate::{
    InspectedItem, InspectorContext,
    item::{DefaultInspectorItem, InspectorListItem},
    modifiers::{ScalarOptions, audio_item_integer_scalar_row, audio_item_scalar_row},
    section::InspectorSection,
    ui::dropdown,
};

pub(super) fn item(generator: &AudioGenerator) -> InspectorListItem {
    DefaultInspectorItem::new(
        "audio-generator",
        "Generator",
        generator.clone(),
        controls,
        |context, _: AudioGenerator| {
            edit(context, "reset-audio-generator", |generator| {
                *generator = AudioGenerator::default();
            });
        },
    )
    .boxed()
}

fn controls(generator: &AudioGenerator, context: &InspectorContext) -> Vec<gtk::Widget> {
    let section = InspectorSection::controls();
    for control in shrimply_inspector_core::audio_generator::controls(generator) {
        match control {
            AudioGeneratorControl::Waveform { value, choices } => {
                let project = context.project.clone();
                let player = context.player_state.clone();
                let key = context.selected_item.clone();
                section.add_control_row(
                    "Waveform",
                    &dropdown(
                        value,
                        choices.iter().map(|choice| (choice.value, choice.label)),
                        move |waveform| {
                            edit_parts(
                                &project,
                                &player,
                                &key,
                                "change-audio-generator-waveform",
                                |generator| {
                                    if generator.waveform == waveform {
                                        return false;
                                    }
                                    generator.waveform = waveform;
                                    true
                                },
                            );
                        },
                    ),
                );
            }
            AudioGeneratorControl::Number {
                value,
                presentation,
            } => {
                let options = ScalarOptions {
                    minimum: Some(presentation.minimum * presentation.store_multiplier),
                    maximum: Some(presentation.maximum * presentation.store_multiplier),
                    unit: (!presentation.unit.is_empty()).then_some(presentation.unit),
                    rotating: false,
                };
                let (get, get_mut): (
                    crate::timeline_value::scalar::ScalarGet,
                    crate::timeline_value::scalar::ScalarGetMut,
                ) = match presentation.field {
                    AudioGeneratorNumber::Frequency => (frequency, frequency_mut),
                    AudioGeneratorNumber::PulseWidth => (pulse_width, pulse_width_mut),
                    AudioGeneratorNumber::Seed => (seed, seed_mut),
                };
                let row = if presentation.integer {
                    audio_item_integer_scalar_row(
                        presentation.label,
                        value,
                        get,
                        get_mut,
                        options,
                        context,
                    )
                } else {
                    audio_item_scalar_row(presentation.label, value, get, get_mut, options, context)
                };
                section.add_wide_control(&row);
            }
        }
    }
    vec![section.into_widget()]
}

fn frequency(project: &Project, key: InspectedItem) -> Option<&TimelineValue<f32>> {
    Some(&generator(project, &key)?.frequency_hz)
}

fn frequency_mut(project: &mut Project, key: InspectedItem) -> Option<&mut TimelineValue<f32>> {
    Some(&mut generator_mut(project, &key)?.frequency_hz)
}

fn pulse_width(project: &Project, key: InspectedItem) -> Option<&TimelineValue<f32>> {
    Some(&generator(project, &key)?.pulse_width)
}

fn pulse_width_mut(project: &mut Project, key: InspectedItem) -> Option<&mut TimelineValue<f32>> {
    Some(&mut generator_mut(project, &key)?.pulse_width)
}

fn seed(project: &Project, key: InspectedItem) -> Option<&TimelineValue<f32>> {
    Some(&generator(project, &key)?.seed)
}

fn seed_mut(project: &mut Project, key: InspectedItem) -> Option<&mut TimelineValue<f32>> {
    Some(&mut generator_mut(project, &key)?.seed)
}

fn generator<'a>(project: &'a Project, key: &InspectedItem) -> Option<&'a AudioGenerator> {
    let AudioSource::Generator(generator) = &project.audio_item(key)?.source else {
        return None;
    };
    Some(generator)
}

fn generator_mut<'a>(
    project: &'a mut Project,
    key: &InspectedItem,
) -> Option<&'a mut AudioGenerator> {
    let AudioSource::Generator(generator) = &mut project.audio_item_mut(key)?.source else {
        return None;
    };
    Some(generator)
}

fn edit(context: &InspectorContext, tag: &'static str, change: impl FnOnce(&mut AudioGenerator)) {
    edit_parts(
        &context.project,
        &context.player_state,
        &context.selected_item,
        tag,
        |generator| {
            change(generator);
            true
        },
    );
}

fn edit_parts(
    project: &Rc<RefCell<Project>>,
    player: &SharedPlayerState,
    key: &Option<InspectedItem>,
    tag: &'static str,
    change: impl FnOnce(&mut AudioGenerator) -> bool,
) {
    let Some(key) = key else { return };
    let mut project = project.borrow_mut();
    let Some(generator) = generator_mut(&mut project, key) else {
        return;
    };
    if !change(generator) {
        return;
    }
    shrimply_project::project::commit_edit(&project, tag);
    drop(project);
    player_state::refresh_project(
        player,
        ProjectChange {
            audio: true,
            audio_waveforms: true,
            inspector: true,
            ..Default::default()
        },
    );
}
