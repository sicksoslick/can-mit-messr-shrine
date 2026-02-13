use nih_plug::prelude::*;
use std::f32::consts::PI;

/// State-variable filter modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum FilterMode {
    #[name = "Low Pass"]
    LowPass,
    #[name = "High Pass"]
    HighPass,
    #[name = "Band Pass"]
    BandPass,
    #[name = "Notch"]
    Notch,
}

/// Chamberlin 2-pole state-variable filter.
///
/// Resonance can go into self-oscillation for screaming tones.
/// Cutoff and resonance are continuously variable.
pub struct Filter {
    sample_rate: f32,
    low: f32,
    band: f32,
    cutoff: f32,
    q: f32,
    mode: FilterMode,
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            sample_rate: 44100.0,
            low: 0.0,
            band: 0.0,
            cutoff: 20000.0,
            q: 0.5,
            mode: FilterMode::LowPass,
        }
    }
}

impl Filter {
    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr;
    }

    pub fn reset(&mut self) {
        self.low = 0.0;
        self.band = 0.0;
    }

    /// `cutoff` in Hz, `reso` 0.0–1.0 (1.0 = self-oscillation).
    pub fn set_params(&mut self, cutoff: f32, reso: f32, mode: FilterMode) {
        self.cutoff = cutoff;
        // Map 0..1 to Q range: 2.0 (gentle) down to 0.01 (screaming)
        self.q = 2.0 - reso * 1.99;
        self.mode = mode;
    }

    pub fn process(&mut self, input: f32) -> f32 {
        // Frequency coefficient (2 * sin(pi * fc / sr))
        // Clamp to prevent instability near Nyquist
        let f = (2.0 * (PI * self.cutoff / self.sample_rate).sin()).min(0.99);

        // Two-pass for 2x oversampling (improves stability at high freqs)
        for _ in 0..2 {
            self.low += f * self.band;
            let high = input - self.low - self.q * self.band;
            self.band += f * high;
        }

        match self.mode {
            FilterMode::LowPass => self.low,
            FilterMode::HighPass => input - self.low - self.q * self.band,
            FilterMode::BandPass => self.band,
            FilterMode::Notch => {
                let high = input - self.low - self.q * self.band;
                self.low + high
            }
        }
    }
}
