use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::f32::consts::PI;

enum Waveform {
    Sine,
    Square,
    Sawtooth,
}

fn generate_wave(waveform: Waveform, phase: f32) -> f32 {
    match waveform {
        Waveform::Sine => (phase * 2.0 * PI).sin(),
        Waveform::Square => {
            if phase < 0.5 {
                1.0
            } else {
                -1.0
            }
        }
        Waveform::Sawtooth => 2.0 * phase - 1.0,
    }
}

#[allow(clippy::upper_case_acronyms)]
struct ADSR {
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
}

fn apply_adsr(adsr: &ADSR, time: f32, note_on: bool, note_off_time: Option<f32>) -> f32 {
    if note_on {
        // Note is active
        if time < adsr.attack {
            // Attack phase
            time / adsr.attack
        } else if time < adsr.attack + adsr.decay {
            // Decay phase
            adsr.sustain + (1.0 - adsr.sustain) * (1.0 - (time - adsr.attack) / adsr.decay)
        } else {
            // Sustain phase
            adsr.sustain
        }
    } else if let Some(note_off_time) = note_off_time {
        // Note is released
        let release_time = time - note_off_time;
        if release_time < adsr.release {
            // Release phase
            adsr.sustain * (1.0 - release_time / adsr.release)
        } else {
            // After release phase
            0.0
        }
    } else {
        // Note is off and no release time provided
        0.0
    }
}

fn main() {
    // Initialize audio host and default output device
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("No output device available");
    let config = device.default_output_config().unwrap();

    // Configure sample rate and buffer size
    let sample_rate = config.sample_rate().0 as f32;
    let mut phase = 0.0;
    let frequency = 440.0; // A4 note
    let mut time_sec = 0.0;

    let adsr = ADSR {
        attack: 0.8,
        decay: 1.2,
        sustain: 0.5,
        release: 0.8,
    };

    let mut note_on = true; // Simulate a note being played
    let mut note_off_time = None; // Time when the note was released

    // Generate a sine wave
    let stream = device
        .build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for sample in data.iter_mut() {
                    // Increment time
                    let time = time_sec;

                    // Generate sine wave
                    let sine_wave = generate_wave(Waveform::Sine, phase);

                    // Apply ADSR envelope
                    let amplitude = apply_adsr(&adsr, time, note_on, note_off_time);
                    *sample = sine_wave * amplitude;

                    // Update phase and time
                    phase = (phase + frequency / sample_rate) % 1.0;
                    time_sec += 1.0 / sample_rate;

                    // Simulate note-off after 2 seconds
                    if time_sec > 5.0 && note_on {
                        note_on = false;
                        note_off_time = Some(time_sec);
                    }
                }
            },
            |err| eprintln!("Error occurred: {}", err),
            None,
        )
        .unwrap();

    // Play the stream
    stream.play().unwrap();
    std::thread::sleep(std::time::Duration::from_secs(5));
}

