use std::f32::consts::TAU;

/// Ring modulator with internal sine oscillator.
///
/// Multiplies the input by a sine wave, producing sum and difference
/// frequencies — metallic, atonal, bell-like.  At audio rates it goes
/// full Dalek.
pub struct RingMod {
    phase: f32,
}

impl Default for RingMod {
    fn default() -> Self {
        Self { phase: 0.0 }
    }
}

impl RingMod {
    pub fn reset(&mut self) {
        self.phase = 0.0;
    }

    /// `freq`  — modulation frequency in Hz
    /// `depth` — 0.0 = bypass, 1.0 = full ring mod
    pub fn process(&mut self, input: f32, freq: f32, depth: f32, sample_rate: f32) -> f32 {
        if depth < 1e-6 {
            return input;
        }

        let mod_signal = (self.phase * TAU).sin();
        self.phase += freq / sample_rate;
        // Keep phase in [0, 1) to avoid precision loss over long runs
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        // Crossfade between dry and ring-modulated
        let wet = input * mod_signal;
        input + (wet - input) * depth
    }
}
