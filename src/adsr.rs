use crate::NoteState;
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone)]
/// An ADSR envelope. Currently only using linear interpolation.
pub struct ADSR {
    pub attack: f32, //time to reach peak volume
    pub decay: f32, //time to reach sustain volume
    pub sustain: f32, //volume during the main part of the sound (WHAT UNIT?)
    pub release: f32, //time to reach 0 volume
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

    pub fn apply(&self, note_state: &NoteState) -> f32 {
        if note_state.active {
            if note_state.time < self.attack {
                note_state.time / self.attack
            } else if note_state.time < self.attack + self.decay {
                self.sustain + (1.0 - self.sustain) * (1.0 - (note_state.time - self.attack) / self.decay)
            } else {
                self.sustain
            }
        } else {
            if note_state.time < self.release {
                self.sustain * (1.0 - note_state.time / self.release)
            } else {
                0.0
            }
        }
    }
}
