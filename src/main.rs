mod adsr;
mod visualizer;
mod waveform;

use adsr::ADSR;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use visualizer::Visualizer;
use waveform::Waveform;

fn main() -> Result<(), eframe::Error> {
    let audio_data = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let adsr = ADSR::new(0.8, 1.2, 0.5, 0.8);
    let adsr = Arc::new(Mutex::new(adsr));

    let waveform = Arc::new(Mutex::new(Waveform::Sine));

    let audio_data_clone = audio_data.clone();
    let adsr_data_clone = adsr.clone();
    let waveform_data_clone = waveform.clone();
    std::thread::spawn(move || {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("No output device available");
        let config = device.default_output_config().unwrap();
        let sample_rate = config.sample_rate().0 as f32;

        let mut phase = 0.0;
        let frequency = 440.0;
        let mut time_sec = 0.0;

        let mut note_on = true;
        let mut note_off_time = None;
        let mut last_note_time = 0.0;

        let stream = device
            .build_output_stream(
                &config.into(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut local_data = audio_data_clone.lock().unwrap();
                    let local_adsr = adsr_data_clone.lock().unwrap();
                    let local_waveform = waveform_data_clone.lock().unwrap();
                    local_data.clear();
                    for sample in data.iter_mut() {
                        let time = time_sec;

                        let sine_wave = local_waveform.gen(phase);

                        let amplitude = local_adsr.apply(time, note_on, note_off_time);
                        *sample = sine_wave * amplitude;

                        local_data.push(*sample);

                        phase = (phase + frequency / sample_rate) % 1.0;
                        time_sec += 1.0 / sample_rate;

                        if note_on && (time - last_note_time) > 2.0 {
                            note_on = false;
                            note_off_time = Some(time_sec);
                        }

                        if !note_on && (time - last_note_time) > 5.0 {
                            note_on = true;
                            note_off_time = None;
                            last_note_time = time_sec;
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
                waveform,
            }))
        }),
    )
}
