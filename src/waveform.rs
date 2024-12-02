use std::f32::consts::PI;

#[derive(Debug, PartialEq)]
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
