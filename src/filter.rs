#[derive(Debug, PartialEq)]
pub enum FilterTypes {
    High,
    Low,
}

pub trait Filter {
    fn new(cutoff: f32, smaple_rate: f32) -> Self
    where
        Self: Sized + Send + Sync;

    fn set_cutoff(&mut self, cutoff: f32);

    fn apply(&mut self, input: f32) -> f32;
}

#[derive(Debug)]
pub struct HighPassFilter {
    cutoff: f32,
    sample_rate: f32,
    prev_input: f32,
    prev_output: f32,
    alpha: f32,
}

impl Filter for HighPassFilter {
    fn new(cutoff: f32, sample_rate: f32) -> Self {
        let alpha = Self::calculate_alpha(cutoff, sample_rate);
        Self {
            cutoff,
            sample_rate,
            prev_input: 0.0,
            prev_output: 0.0,
            alpha,
        }
    }

    fn set_cutoff(&mut self, cutoff: f32) {
        self.cutoff = cutoff;
        self.alpha = Self::calculate_alpha(self.cutoff, self.sample_rate);
    }

    fn apply(&mut self, input: f32) -> f32 {
        let output = self.alpha * (self.prev_output + input - self.prev_input);
        self.prev_input = input;
        self.prev_output = output;
        output
    }
}

impl HighPassFilter {
    fn calculate_alpha(cutoff: f32, sample_rate: f32) -> f32 {
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff);
        let dt = 1.0 / sample_rate;
        dt / (rc + dt)
    }
}

#[derive(Debug)]
pub struct LowPassFilter {
    cutoff: f32,
    sample_rate: f32,
    prev_input: f32,
    prev_output: f32,
    alpha: f32,
}

impl Filter for LowPassFilter {
    fn new(cutoff: f32, sample_rate: f32) -> Self {
        let alpha = Self::calculate_alpha(cutoff, sample_rate);
        Self {
            cutoff,
            sample_rate,
            prev_input: 0.0,
            prev_output: 0.0,
            alpha,
        }
    }

    fn set_cutoff(&mut self, cutoff: f32) {
        self.cutoff = cutoff;
        self.alpha = Self::calculate_alpha(self.cutoff, self.sample_rate);
    }

    fn apply(&mut self, input: f32) -> f32 {
        let output = self.alpha * input + (1.0 - self.alpha) * self.prev_output;
        self.prev_output = output;
        self.prev_input = input;
        output
    }
}

impl LowPassFilter {
    fn calculate_alpha(cutoff: f32, sample_rate: f32) -> f32 {
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff);
        let dt = 1.0 / sample_rate;
        dt / (rc + dt)
    }
}
