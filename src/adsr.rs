#[allow(clippy::upper_case_acronyms)]
#[derive(Clone)]
pub struct ADSR {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
}

impl ADSR {
    pub fn new(attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        Self {
            attack,
            decay,
            sustain,
            release,
        }
    }

    pub fn apply(&self, time: f32, note_on: bool, note_off_time: Option<f32>) -> f32 {
        if note_on {
            if time < self.attack {
                time / self.attack
            } else if time < self.attack + self.decay {
                self.sustain + (1.0 - self.sustain) * (1.0 - (time - self.attack) / self.decay)
            } else {
                self.sustain
            }
        } else if let Some(note_off_time) = note_off_time {
            let release_time = time - note_off_time;
            if release_time < self.release {
                self.sustain * (1.0 - release_time / self.release)
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
}
