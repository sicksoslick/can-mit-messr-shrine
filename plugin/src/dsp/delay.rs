/// Simple feedback delay line.
///
/// Features:
/// - Continuously variable delay time with linear interpolation
/// - Feedback path runs through a soft-clip to prevent blow-up
///   (but 100%+ feedback still self-oscillates beautifully)
/// - Zero-allocation after init
pub struct Delay {
    buffer: Vec<f32>,
    write_pos: usize,
}

impl Default for Delay {
    fn default() -> Self {
        Self::new(44100, 1.0)
    }
}

impl Delay {
    /// `sample_rate` and `max_seconds` determine buffer size.
    pub fn new(sample_rate: usize, max_seconds: f32) -> Self {
        let size = (sample_rate as f32 * max_seconds) as usize + 2;
        Self {
            buffer: vec![0.0; size],
            write_pos: 0,
        }
    }

    pub fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
    }

    /// `delay_samples` — delay length in fractional samples
    /// `feedback`      — 0.0–1.1 (past 1.0 = runaway, soft-clipped)
    pub fn process(&mut self, input: f32, delay_samples: f32, feedback: f32) -> f32 {
        let len = self.buffer.len();
        if len == 0 || delay_samples < 0.5 {
            return input; // no delay
        }

        // Read with linear interpolation
        let delay_clamped = delay_samples.min((len - 1) as f32);
        let read_pos = self.write_pos as f32 - delay_clamped;
        let read_pos = if read_pos < 0.0 {
            read_pos + len as f32
        } else {
            read_pos
        };

        let idx0 = read_pos.floor() as usize % len;
        let idx1 = (idx0 + 1) % len;
        let frac = read_pos.fract();
        let delayed = self.buffer[idx0] * (1.0 - frac) + self.buffer[idx1] * frac;

        // Write: input + soft-clipped feedback
        let fb_signal = soft_clip(delayed * feedback);
        self.buffer[self.write_pos] = input + fb_signal;
        self.write_pos = (self.write_pos + 1) % len;

        delayed
    }
}

/// Gentle tanh soft-clip to tame feedback runaway without hard limiting.
#[inline]
fn soft_clip(x: f32) -> f32 {
    x.tanh()
}
