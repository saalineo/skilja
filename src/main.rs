mod audio;
mod dsp;
mod physics;
mod tui;

use num_complex::Complex32;
use physics::{BarState, PhysicsConfig};
use std::time::Duration;

const FFT_SIZE: usize = 2048;

fn main() {
    if !audio::init(44100) {
        eprintln!("audio: init failed");
        std::process::exit(1);
    }
    let _audio_guard = AudioGuard;

    tui::enter_alt_screen();
    let _tui_guard = TuiGuard;
    let _raw_guard = enter_raw_mode();

    let mut window = vec![0f32; FFT_SIZE];
    dsp::hann_window_init(&mut window);

    let mut samples = vec![0f32; FFT_SIZE];
    let mut windowed = vec![0f32; FFT_SIZE];
    let mut fft_data = vec![Complex32::new(0.0, 0.0); FFT_SIZE];

    let mut term = tui::get_terminal_size();
    let mut last_size = (term.width, term.height);
    let mut num_bars = term.width.max(1);

    let (mut bars_state, mut bin_lows, mut bin_highs, mut raw_bars, mut norm_bars) =
        allocate_bars(num_bars);

    let physics_config = PhysicsConfig {
        gravity: 0.03,
        peak_gravity: 0.008,
        peak_hold_frames: 15,
        rise_smoothing: 0.7,
        fall_smoothing: 0.15,
    };

    let mut render_buf = String::with_capacity(term.width * term.height * 16);
    let mut max_val_seen: f32 = 0.05;

    loop {
        let mut key = 0u8;
        let bytes_read = unsafe { libc::read(libc::STDIN_FILENO, &mut key as *mut u8 as *mut _, 1) };
        if bytes_read > 0 && (key == b'q' || key == b'Q' || key == 27) {
            break;
        }

        term = tui::get_terminal_size();
        if (term.width, term.height) != last_size {
            tui::clear_screen();
            last_size = (term.width, term.height);
            num_bars = term.width.max(1);
            (bars_state, bin_lows, bin_highs, raw_bars, norm_bars) = allocate_bars(num_bars);
        }

        audio::get_latest_samples(&mut samples);
        dsp::apply_window(&samples, &window, &mut windowed);

        for (fft_bin, &sample) in fft_data.iter_mut().zip(&windowed) {
            *fft_bin = Complex32::new(sample, 0.0);
        }
        dsp::fft(&mut fft_data);
        dsp::bin_fft_data(&fft_data, &bin_lows, &bin_highs, &mut raw_bars);

        max_val_seen = (max_val_seen * 0.995).max(0.05);
        for &bar in &raw_bars {
            max_val_seen = max_val_seen.max(bar);
        }

        for (norm, &raw) in norm_bars.iter_mut().zip(&raw_bars) {
            *norm = raw / max_val_seen;
        }

        physics::update_physics(&mut bars_state, &norm_bars, &physics_config);
        tui::render_frame(&bars_state, term.width, term.height, &mut render_buf);

        std::thread::sleep(Duration::from_millis(16));
    }
}

fn allocate_bars(
    num_bars: usize,
) -> (Vec<BarState>, Vec<usize>, Vec<usize>, Vec<f32>, Vec<f32>) {
    let mut bin_lows = vec![0usize; num_bars];
    let mut bin_highs = vec![0usize; num_bars];
    dsp::calculate_bins(num_bars, FFT_SIZE, 44100.0, 20.0, 20000.0, &mut bin_lows, &mut bin_highs);
    (
        vec![BarState::default(); num_bars],
        bin_lows,
        bin_highs,
        vec![0f32; num_bars],
        vec![0f32; num_bars],
    )
}

struct RawModeGuard(libc::termios);

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &self.0) };
    }
}

fn enter_raw_mode() -> RawModeGuard {
    let mut orig: libc::termios = unsafe { std::mem::zeroed() };
    unsafe { libc::tcgetattr(libc::STDIN_FILENO, &mut orig) };

    let mut raw = orig;
    raw.c_lflag &= !(libc::ECHO | libc::ICANON | libc::ISIG | libc::IEXTEN);
    raw.c_iflag &= !(libc::IXON | libc::ICRNL);
    raw.c_cc[libc::VMIN] = 0;
    raw.c_cc[libc::VTIME] = 0;
    unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &raw) };
    RawModeGuard(orig)
}

struct AudioGuard;
impl Drop for AudioGuard {
    fn drop(&mut self) {
        audio::shutdown();
    }
}

struct TuiGuard;
impl Drop for TuiGuard {
    fn drop(&mut self) {
        tui::exit_alt_screen();
    }
}
