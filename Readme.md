# skilja

Fast, terminal-native stem separation and audio visualization in Rust.

## Features

- **Audio Capture**: Real-time loopback/monitor and input device capture powered by `cpal`.
- **DSP & FFT**: In-place Cooley-Tukey radix-2 FFT with Hann windowing and logarithmic frequency binning.
- **Physics Simulation**: Falloff smoothing, gravity, and peak-hold mechanics for fluid audio metering.
- **Terminal UI**: High-throughput TrueColor rendering over raw ANSI with zero-allocation frame buffering and clean RAII terminal restoration.
- **ML Separation**: Embedded ONNX Runtime (`ort`) stem demuxing and RNNoise (`nnnoiseless`) denoising.

## Prerequisites

- **Rust**: 1.75+ (`cargo`, `rustc`)
- **Linux Audio Libraries**: `libasound2-dev` (Debian/Ubuntu) or `alsa-lib` (Arch/Fedora) for `cpal` ALSA backend

## Build & Run

### Run the Visualizer

```bash
cargo run --release
```

Controls: Press `q`, `Q`, or `Esc` to exit.

### Run Tests

```bash
cargo test
```

## Architecture

- [`src/audio`](src/audio/mod.rs): Device discovery, stream lifecycle, lock-free ring buffering, ONNX demuxer, and RNNoise denoiser.
- [`src/dsp`](src/dsp/mod.rs): Hann window generation, Cooley-Tukey FFT, magnitude calculations, and logarithmic frequency binning.
- [`src/physics`](src/physics/mod.rs): Interpolation, gravity decay, peak holding, and rise/fall dynamics for spectrum bars.
- [`src/tui`](src/tui/mod.rs): Terminal geometry detection via ioctl, 24-bit TrueColor gradient mapper, and alternate screen rendering.
- [`src/main.rs`](src/main.rs): Application loop, resize handling, raw mode terminal management, and pipeline orchestration.
