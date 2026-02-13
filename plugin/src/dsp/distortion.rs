use nih_plug::prelude::*;
use std::f32::consts::PI;

/// Distortion waveshaping modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum DistortionMode {
    /// Classic hard clipping at ±1
    #[name = "Hard Clip"]
    HardClip,
    /// Warm tanh soft clipping
    #[name = "Soft Clip"]
    SoftClip,
    /// Foldback distortion — folds signal back on itself
    #[name = "Foldback"]
    Foldback,
    /// Sine waveshaper — warps amplitude through sin()
    #[name = "Sine Wrap"]
    SineWrap,
    /// Asymmetric — different positive/negative shaping
    #[name = "Asymmetric"]
    Asymmetric,
}

#[derive(Default)]
pub struct Distortion;

impl Distortion {
    pub fn reset(&mut self) {}

    /// `drive` 0.0–1.0 where 0 = clean, 1 = maximum destruction.
    pub fn process(&self, input: f32, drive: f32, mode: DistortionMode) -> f32 {
        if drive < 1e-6 {
            return input;
        }

        // Map drive 0..1 to a gain multiplier 1..50
        let gain = 1.0 + drive * 49.0;
        let driven = input * gain;

        let shaped = match mode {
            DistortionMode::HardClip => driven.clamp(-1.0, 1.0),

            DistortionMode::SoftClip => driven.tanh(),

            DistortionMode::Foldback => foldback(driven, 1.0),

            DistortionMode::SineWrap => (driven * PI * 0.5).sin(),

            DistortionMode::Asymmetric => {
                if driven >= 0.0 {
                    driven.tanh()
                } else {
                    // Harder clip on negative half — asymmetric harmonics
                    -((-driven).powf(0.7).min(1.0))
                }
            }
        };

        // Compensate output level (roughly)
        shaped / gain.sqrt()
    }
}

/// Foldback distortion: when signal exceeds ±threshold it folds back.
fn foldback(mut x: f32, threshold: f32) -> f32 {
    if threshold < 1e-6 {
        return 0.0;
    }
    // Iterative fold — up to 8 folds to handle extreme drive
    for _ in 0..8 {
        if x > threshold {
            x = 2.0 * threshold - x;
        } else if x < -threshold {
            x = -2.0 * threshold - x;
        } else {
            break;
        }
    }
    x
}
