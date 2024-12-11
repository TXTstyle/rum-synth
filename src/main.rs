mod adsr;
mod filter;
mod visualizer;
mod waveform;

use adsr::ADSR;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use filter::Filter;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use visualizer::{Visualizer, WindowState};
use waveform::{Wave, Waveform};

/// The state of a note.
/// time represents the time since the note changed state.
#[derive(Debug, Clone)]
pub struct NoteState {
    active: bool,
    time: f32,
}

impl Default for NoteState {
    fn default() -> Self {
        Self {
            active: false,
            time: 0.0,
        }
    }
}

impl NoteState {
    pub fn note_on(&mut self) {
        self.active = true;
        self.time = 0.0;
    }

    pub fn note_off(&mut self) {
        self.active = false;
        self.time = 0.0;
    }

    pub fn update(&mut self, delta_time: f32) {
        self.time += delta_time;
    }
}

fn main() -> Result<(), eframe::Error> {
    let audio_data = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let adsr = ADSR::new(0.8, 1.2, 0.5, 0.8);
    let adsr = Arc::new(Mutex::new(adsr));

    let filters: Arc<Mutex<Vec<Box<dyn Filter + Send + Sync>>>> = Arc::new(Mutex::new(Vec::new()));

    let audio_data_clone = audio_data.clone();
    let adsr_data_clone = adsr.clone();
    let filters_data_clone = filters.clone();

    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("No output device available");
    let config = device.default_output_config().unwrap();
    let sample_rate = config.sample_rate().0 as f32;

    let wave = Wave::new(440., sample_rate, Waveform::Sine);
    let wave = Arc::new(Mutex::new(wave));
    let wave_data_clone = wave.clone();

    let note_state = NoteState::default();
    let note_state = Arc::new(Mutex::new(note_state));
    let note_data_clone = note_state.clone();

    std::thread::spawn(move || {
        let mut phase = 0.0;
        let mut time_sec = 0.0;

        let stream = device
            .build_output_stream(
                &config.into(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut local_data = audio_data_clone.lock().unwrap();
                    let local_adsr = adsr_data_clone.lock().unwrap();
                    let local_wave = wave_data_clone.lock().unwrap();
                    let mut local_filters = filters_data_clone.lock().unwrap();
                    let mut local_note = note_data_clone.lock().unwrap();

                    local_data.clear();
                    for sample in data.iter_mut() {
                        let _time = time_sec;

                        *sample = local_wave.apply(phase);
                        
                        let adsr = local_adsr.apply(&*local_note);
                        *sample *= adsr;
                        for filter in local_filters.iter_mut() {
                            *sample = filter.apply(*sample);
                        }

                        local_data.push(*sample);

                        phase = (phase + local_wave.frequency / sample_rate) % 1.0;
                        let delta_time = 1.0 / sample_rate;
                        time_sec += delta_time;
                        local_note.update(delta_time);
                    }
                },
                |err| eprintln!("Error occurred: {}", err),
                None,
            )
            .unwrap();

        stream.play().unwrap();
        loop {
            thread::sleep(Duration::from_micros(100));
        }
    });

    eframe::run_native(
        "Waveform Visualizer",
        eframe::NativeOptions::default(),
        Box::new(|_| {
            Ok(Box::new(Visualizer {
                audio_data,
                adsr,
                wave,
                filter_select: filter::FilterTypes::High,
                filters,
                filter_cutoff: 0.0,
                window_state: WindowState::default(),
                note_state,
                key_down: false,
            }))
        }),
    )
}
