use nih_plug::prelude::*;
use std::sync::Arc;

mod dsp;
use dsp::{Bitcrush, Delay, Distortion, DistortionMode, Filter, FilterMode, RingMod};

// ---------------------------------------------------------------------------
//  MESSR — brutal multi-effect audio destruction
// ---------------------------------------------------------------------------

const MAX_DELAY_SEC: f32 = 1.0;

struct Messr {
    params: Arc<MessrParams>,
    sample_rate: f32,

    // Per-channel DSP state (we support up to stereo)
    bitcrush: [Bitcrush; 2],
    distortion: [Distortion; 2],
    ringmod: [RingMod; 2],
    filter: [Filter; 2],
    delay: [Delay; 2],
}

// ---------------------------------------------------------------------------
//  Parameters
// ---------------------------------------------------------------------------

#[derive(Params)]
struct MessrParams {
    // -- global --
    #[id = "input_gain"]
    input_gain: FloatParam,
    #[id = "output_gain"]
    output_gain: FloatParam,
    #[id = "dry_wet"]
    dry_wet: FloatParam,

    // -- bitcrush --
    #[id = "crush_bits"]
    crush_bits: FloatParam,
    #[id = "crush_rate"]
    crush_rate: FloatParam,
    #[id = "crush_mix"]
    crush_mix: FloatParam,

    // -- distortion --
    #[id = "dist_mode"]
    dist_mode: EnumParam<DistortionMode>,
    #[id = "dist_drive"]
    dist_drive: FloatParam,
    #[id = "dist_mix"]
    dist_mix: FloatParam,

    // -- ring mod --
    #[id = "ring_freq"]
    ring_freq: FloatParam,
    #[id = "ring_depth"]
    ring_depth: FloatParam,

    // -- filter --
    #[id = "filt_mode"]
    filt_mode: EnumParam<FilterMode>,
    #[id = "filt_cutoff"]
    filt_cutoff: FloatParam,
    #[id = "filt_reso"]
    filt_reso: FloatParam,

    // -- delay --
    #[id = "delay_time"]
    delay_time: FloatParam,
    #[id = "delay_feedback"]
    delay_feedback: FloatParam,
    #[id = "delay_mix"]
    delay_mix: FloatParam,
}

impl Default for MessrParams {
    fn default() -> Self {
        Self {
            // --- Global ---
            input_gain: FloatParam::new(
                "Input Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-12.0),
                    max: util::db_to_gain(36.0),
                    factor: FloatRange::gain_skew_factor(-12.0, 36.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(20.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            output_gain: FloatParam::new(
                "Output Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-48.0),
                    max: util::db_to_gain(6.0),
                    factor: FloatRange::gain_skew_factor(-48.0, 6.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(20.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            dry_wet: FloatParam::new("Dry/Wet", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_smoother(SmoothingStyle::Linear(20.0))
                .with_unit(" %")
                .with_value_to_string(formatters::v2s_f32_percentage(0))
                .with_string_to_value(formatters::s2v_f32_percentage()),

            // --- Bitcrush ---
            crush_bits: FloatParam::new(
                "Crush Bits",
                32.0,
                FloatRange::Linear {
                    min: 1.0,
                    max: 32.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(20.0))
            .with_unit(" bits"),

            crush_rate: FloatParam::new(
                "Crush Rate",
                1.0,
                FloatRange::Skewed {
                    min: 1.0,
                    max: 100.0,
                    factor: FloatRange::skew_factor(-1.5),
                },
            )
            .with_smoother(SmoothingStyle::Linear(20.0))
            .with_unit("x"),

            crush_mix: FloatParam::new(
                "Crush Mix",
                1.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(20.0))
            .with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            // --- Distortion ---
            dist_mode: EnumParam::new("Distortion Mode", DistortionMode::HardClip),

            dist_drive: FloatParam::new(
                "Distortion Drive",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 1.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(20.0))
            .with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            dist_mix: FloatParam::new(
                "Distortion Mix",
                1.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(20.0))
            .with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            // --- Ring Mod ---
            ring_freq: FloatParam::new(
                "Ring Mod Freq",
                440.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20000.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(20.0))
            .with_unit(" Hz")
            .with_value_to_string(formatters::v2s_f32_rounded(1)),

            ring_depth: FloatParam::new(
                "Ring Mod Depth",
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(20.0))
            .with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            // --- Filter ---
            filt_mode: EnumParam::new("Filter Mode", FilterMode::LowPass),

            filt_cutoff: FloatParam::new(
                "Filter Cutoff",
                20000.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20000.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(20.0))
            .with_unit(" Hz")
            .with_value_to_string(formatters::v2s_f32_rounded(0)),

            filt_reso: FloatParam::new(
                "Filter Resonance",
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(20.0))
            .with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            // --- Delay ---
            delay_time: FloatParam::new(
                "Delay Time",
                0.0,
                FloatRange::Skewed {
                    min: 0.0,
                    max: 1000.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" ms"),

            delay_feedback: FloatParam::new(
                "Delay Feedback",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 1.1,
                },
            )
            .with_smoother(SmoothingStyle::Linear(20.0))
            .with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            delay_mix: FloatParam::new(
                "Delay Mix",
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(20.0))
            .with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),
        }
    }
}

// ---------------------------------------------------------------------------
//  Plugin impl
// ---------------------------------------------------------------------------

impl Default for Messr {
    fn default() -> Self {
        Self {
            params: Arc::new(MessrParams::default()),
            sample_rate: 44100.0,
            bitcrush: Default::default(),
            distortion: Default::default(),
            ringmod: Default::default(),
            filter: Default::default(),
            delay: [Delay::new(44100, MAX_DELAY_SEC), Delay::new(44100, MAX_DELAY_SEC)],
        }
    }
}

impl Plugin for Messr {
    const NAME: &'static str = "MESSR";
    const VENDOR: &'static str = "SickSoSlick";
    const URL: &'static str = "https://github.com/SickSoSlick/CAN-MIT-MESSR-shrine";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        // Stereo
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        // Mono
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate;
        for ch in 0..2 {
            self.delay[ch] = Delay::new(buffer_config.sample_rate as usize, MAX_DELAY_SEC);
            self.filter[ch].set_sample_rate(buffer_config.sample_rate);
        }
        true
    }

    fn reset(&mut self) {
        for ch in 0..2 {
            self.bitcrush[ch].reset();
            self.distortion[ch].reset();
            self.ringmod[ch].reset();
            self.filter[ch].reset();
            self.delay[ch].reset();
        }
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let num_channels = buffer.channels();

        for mut channel_samples in buffer.iter_samples() {
            // Read smoothed params once per sample
            let in_gain = self.params.input_gain.smoothed.next();
            let out_gain = self.params.output_gain.smoothed.next();
            let dry_wet = self.params.dry_wet.smoothed.next();

            let crush_bits = self.params.crush_bits.smoothed.next();
            let crush_rate = self.params.crush_rate.smoothed.next();
            let crush_mix = self.params.crush_mix.smoothed.next();

            let dist_mode = self.params.dist_mode.value();
            let dist_drive = self.params.dist_drive.smoothed.next();
            let dist_mix = self.params.dist_mix.smoothed.next();

            let ring_freq = self.params.ring_freq.smoothed.next();
            let ring_depth = self.params.ring_depth.smoothed.next();

            let filt_mode = self.params.filt_mode.value();
            let filt_cutoff = self.params.filt_cutoff.smoothed.next();
            let filt_reso = self.params.filt_reso.smoothed.next();

            let delay_time_ms = self.params.delay_time.smoothed.next();
            let delay_feedback = self.params.delay_feedback.smoothed.next();
            let delay_mix = self.params.delay_mix.smoothed.next();

            for (ch, sample) in channel_samples.iter_mut().enumerate() {
                if ch >= num_channels {
                    break;
                }

                let dry = *sample;

                // Input gain
                let mut s = dry * in_gain;

                // Bitcrush
                let crushed = self.bitcrush[ch].process(s, crush_bits, crush_rate);
                s = s + (crushed - s) * crush_mix;

                // Distortion
                let distorted = self.distortion[ch].process(s, dist_drive, dist_mode);
                s = s + (distorted - s) * dist_mix;

                // Ring Mod
                s = self.ringmod[ch].process(s, ring_freq, ring_depth, self.sample_rate);

                // Filter
                self.filter[ch].set_params(filt_cutoff, filt_reso, filt_mode);
                s = self.filter[ch].process(s);

                // Delay (feedback goes through a soft-clip to prevent blow-up)
                let delay_samples = (delay_time_ms / 1000.0) * self.sample_rate;
                let delayed = self.delay[ch].process(s, delay_samples, delay_feedback);
                s = s + (delayed - s) * delay_mix;

                // Dry/Wet mix
                s = dry + (s - dry) * dry_wet;

                // Output gain
                *sample = s * out_gain;
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for Messr {
    const CLAP_ID: &'static str = "com.sickSoSlick.messr";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Brutal multi-effect audio destruction: bitcrush, distortion, ring mod, filter, delay");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Distortion,
        ClapFeature::Filter,
    ];
}

impl Vst3Plugin for Messr {
    const VST3_CLASS_ID: [u8; 16] = *b"MESSR_SickSoSlic";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,
        Vst3SubCategory::Distortion,
        Vst3SubCategory::Filter,
    ];
}

nih_export_clap!(Messr);
nih_export_vst3!(Messr);
