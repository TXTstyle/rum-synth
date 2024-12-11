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

pub struct NoteState {
    pub note_on: bool,
    pub note_off_time: Option<f32>,
}

impl Default for NoteState {
    fn default() -> Self {
        Self {
            note_on: false,
            note_off_time: None,
        }
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
                        let time = time_sec;

                        *sample = local_wave.apply(phase);

                        *sample *= local_adsr.apply(time, local_note.note_on, local_note.note_off_time);
                        for filter in local_filters.iter_mut() {
                            *sample = filter.apply(*sample);
                        }

                        local_data.push(*sample);

                        phase = (phase + local_wave.frequency / sample_rate) % 1.0;
                        time_sec += 1.0 / sample_rate;
                        
                        if let Some(off_time) = local_note.note_off_time {
                            if off_time == 1.0 {
                                local_note.note_off_time = Some(time_sec);
                            }
                        }
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
            }))
        }),
    )
}
