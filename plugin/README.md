# MESSR

Brutal multi-effect audio destruction plugin. VST3 + CLAP.

Drop it on a kick, snare, vocal, synth — whatever — and watch it get obliterated.

## Signal Chain

```
INPUT -> [Input Gain] -> [Bitcrush] -> [Distortion] -> [Ring Mod] -> [Filter] -> [Delay] -> [Dry/Wet] -> [Output Gain] -> OUTPUT
```

## Effects

### Bitcrusher
- **Crush Bits** (1–32) — bit depth reduction. Low values = pure digital grime
- **Crush Rate** (1x–100x) — sample rate reduction. Staircase aliasing artifacts
- **Crush Mix** — blend crushed with clean

### Distortion (5 modes)
- **Hard Clip** — classic brick-wall clipping
- **Soft Clip** — warm tanh saturation
- **Foldback** — signal folds back on itself for gnarly harmonic content
- **Sine Wrap** — warps amplitude through sin() for metallic overtones
- **Asymmetric** — different shaping on positive/negative halves (odd + even harmonics)
- **Drive** (0–100%) — mapped to 1x–50x gain into the waveshaper
- **Mix** — parallel blend

### Ring Modulator
- **Frequency** (20 Hz–20 kHz) — internal sine oscillator
- **Depth** (0–100%) — crossfade into ring-modulated signal. Go full Dalek

### State Variable Filter
- **Mode** — Low Pass / High Pass / Band Pass / Notch
- **Cutoff** (20 Hz–20 kHz) — log-scaled
- **Resonance** (0–100%) — goes into self-oscillation at max. Screams

### Feedback Delay
- **Time** (0–1000 ms) — interpolated for smooth sweeps
- **Feedback** (0–110%) — past 100% it self-oscillates through a soft-clip, building into chaos without instant blow-up
- **Mix** — blend delayed signal

### Global
- **Input Gain** (-12 dB to +36 dB) — drive the whole chain hot
- **Dry/Wet** (0–100%) — parallel processing blend
- **Output Gain** (-48 dB to +6 dB) — tame the beast

## Building

Requires [Rust](https://rustup.rs/).

```bash
cd plugin
cargo xtask bundle messr --release
```

Outputs:
- `target/bundled/messr.clap` — CLAP plugin
- `target/bundled/messr.vst3/` — VST3 plugin bundle

## Installing

### macOS (Logic Pro, Ableton, etc.)
- **VST3**: Copy `messr.vst3` to `~/Library/Audio/Plug-Ins/VST3/`
- **CLAP**: Copy `messr.clap` to `~/Library/Audio/Plug-Ins/CLAP/`

Cross-compile for macOS with:
```bash
rustup target add x86_64-apple-darwin aarch64-apple-darwin
cargo xtask bundle messr --release --target x86_64-apple-darwin
cargo xtask bundle messr --release --target aarch64-apple-darwin
```

### Windows (Ableton, FL Studio, etc.)
- **VST3**: Copy `messr.vst3` to `C:\Program Files\Common Files\VST3\`
- **CLAP**: Copy `messr.clap` to `C:\Program Files\Common Files\CLAP\`

### Linux
- **VST3**: Copy `messr.vst3` to `~/.vst3/`
- **CLAP**: Copy `messr.clap` to `~/.clap/`

## License

MIT
