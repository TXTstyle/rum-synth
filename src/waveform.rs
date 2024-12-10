use std::f32::consts::PI;

#[derive(Debug)]
pub struct Wave {
    pub waveform: Waveform,
    pub frequency: f32,
    pub sample_rate: f32,
    pub amplitude: f32,
}

impl Wave {
    pub fn new(frequency: f32, sample_rate: f32, waveform: Waveform) -> Self {
        Self {
            waveform,
            frequency,
            sample_rate,
            amplitude: 1.0,
        }
    }

    pub fn apply(&self, phase: f32) -> f32 {
        self.waveform.gen(phase)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Waveform {
    Sine,
    Square,
    Sawtooth,
}

impl Waveform {
    pub fn gen(&self, phase: f32) -> f32 {
        match self {
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
}
