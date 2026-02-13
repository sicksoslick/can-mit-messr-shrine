/// Bitcrusher + sample-rate reducer.
///
/// Reduces bit depth (quantisation noise) and effective sample rate
/// (staircase aliasing). Both are continuously variable.
#[derive(Default)]
pub struct Bitcrush {
    hold_counter: f32,
    held_sample: f32,
}

impl Bitcrush {
    pub fn reset(&mut self) {
        self.hold_counter = 0.0;
        self.held_sample = 0.0;
    }

    /// `bits` — effective bit depth (1.0 .. 32.0)
    /// `rate_div` — sample-rate divisor  (1.0 = no reduction, 100.0 = heavy)
    pub fn process(&mut self, input: f32, bits: f32, rate_div: f32) -> f32 {
        // --- sample-rate reduction ---
        self.hold_counter += 1.0;
        if self.hold_counter >= rate_div {
            self.hold_counter -= rate_div; // preserve fractional part for smooth sweep
            self.held_sample = input;
        }
        let s = self.held_sample;

        // --- bit-depth reduction ---
        if bits >= 31.5 {
            return s; // bypass quantisation at full depth
        }
        let levels = (2.0_f32).powf(bits); // e.g. 8 bits → 256 levels
        (s * levels).round() / levels
    }
}
