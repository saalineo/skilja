use num_complex::Complex32;
use std::f32::consts::PI;

#[allow(dead_code)]
pub struct AudioStems<T> {
    pub vocals: Vec<T>,
    pub bgm: Vec<T>,
    pub noise: Vec<T>,
}

pub fn hann_window_init(window: &mut [f32]) {
    let n = window.len();
    if n <= 1 {
        return;
    }
    for (i, w) in window.iter_mut().enumerate() {
        *w = 0.5 * (1.0 - (2.0 * PI * i as f32 / (n - 1) as f32).cos());
    }
}

pub fn apply_window(input: &[f32], window: &[f32], output: &mut [f32]) {
    for ((out, &inp), &win) in output.iter_mut().zip(input).zip(window) {
        *out = inp * win;
    }
}

#[allow(dead_code)]
pub fn apply_window_stems(input: &AudioStems<f32>, window: &[f32], output: &mut AudioStems<f32>) {
    apply_window(&input.vocals, window, &mut output.vocals);
    apply_window(&input.bgm, window, &mut output.bgm);
    apply_window(&input.noise, window, &mut output.noise);
}

pub fn fft(data: &mut [Complex32]) {
    let n = data.len();
    if n <= 1 {
        return;
    }

    let mut j = 0usize;
    for i in 0..n {
        if i < j {
            data.swap(i, j);
        }
        let mut m = n >> 1;
        while m >= 1 && j >= m {
            j -= m;
            m >>= 1;
        }
        j += m;
    }

    let mut len_stage = 2;
    while len_stage <= n {
        let half = len_stage >> 1;
        let angle = -2.0 * PI / len_stage as f32;
        let w_step = Complex32::from_polar(1.0, angle);

        let mut base = 0;
        while base < n {
            let mut w = Complex32::new(1.0, 0.0);
            for k in 0..half {
                let u = data[base + k];
                let t = w * data[base + k + half];
                data[base + k] = u + t;
                data[base + k + half] = u - t;
                w *= w_step;
            }
            base += len_stage;
        }

        len_stage <<= 1;
    }
}

#[allow(dead_code)]
pub fn fft_stems(stems: &mut AudioStems<Complex32>) {
    fft(&mut stems.vocals);
    fft(&mut stems.bgm);
    fft(&mut stems.noise);
}

#[allow(dead_code)]
pub fn fft_magnitude(fft_data: &[Complex32], magnitude: &mut [f32]) {
    for (mag, c) in magnitude.iter_mut().zip(fft_data) {
        *mag = c.norm();
    }
}

#[allow(dead_code)]
pub fn fft_magnitude_stems(stems: &AudioStems<Complex32>, magnitude: &mut AudioStems<f32>) {
    fft_magnitude(&stems.vocals, &mut magnitude.vocals);
    fft_magnitude(&stems.bgm, &mut magnitude.bgm);
    fft_magnitude(&stems.noise, &mut magnitude.noise);
}

#[allow(dead_code)]
pub fn batch_windowed_fft(
    input: &AudioStems<f32>,
    window: &[f32],
    fft_buf: &mut AudioStems<Complex32>,
    mag_out: &mut AudioStems<f32>,
) {
    process_stem(
        &input.vocals,
        window,
        &mut fft_buf.vocals,
        &mut mag_out.vocals,
    );
    process_stem(&input.bgm, window, &mut fft_buf.bgm, &mut mag_out.bgm);
    process_stem(&input.noise, window, &mut fft_buf.noise, &mut mag_out.noise);
}

#[allow(dead_code)]
fn process_stem(input: &[f32], window: &[f32], fft_buf: &mut [Complex32], mag_out: &mut [f32]) {
    for ((c, s), w) in fft_buf.iter_mut().zip(input).zip(window) {
        *c = Complex32::new(s * w, 0.0);
    }
    fft(fft_buf);
    fft_magnitude(fft_buf, mag_out);
}

pub fn calculate_bins(
    num_bars: usize,
    fft_size: usize,
    sample_rate: f32,
    min_freq: f32,
    max_freq: f32,
    bin_lows: &mut [usize],
    bin_highs: &mut [usize],
) {
    let half_fft = fft_size / 2;
    let ratio = max_freq / min_freq;
    for i in 0..num_bars {
        let freq_low = min_freq * ratio.powf(i as f32 / num_bars as f32);
        let freq_high = min_freq * ratio.powf((i + 1) as f32 / num_bars as f32);

        let mut low = (freq_low * fft_size as f32 / sample_rate) as usize;
        let mut high = (freq_high * fft_size as f32 / sample_rate) as usize;

        low = low.max(1);
        if high <= low {
            high = low + 1;
        }
        low = low.min(half_fft - 1);
        high = high.min(half_fft);

        bin_lows[i] = low;
        bin_highs[i] = high;
    }
}

pub fn bin_fft_data(
    fft_data: &[Complex32],
    bin_lows: &[usize],
    bin_highs: &[usize],
    bars: &mut [f32],
) {
    let num_bars = bars.len();
    for (i, (bar_amp, (&low, &high))) in bars
        .iter_mut()
        .zip(bin_lows.iter().zip(bin_highs))
        .enumerate()
    {
        let sum: f32 = fft_data[low..high].iter().map(|c| c.norm()).sum();
        let avg = sum / (high - low) as f32;
        let boost = 1.0 + 3.0 * (i as f32 / num_bars as f32);
        *bar_amp = avg * boost;
    }
}

// tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_stems_windowing() {
        let n = 16;
        let mut win = vec![0f32; n];
        hann_window_init(&mut win);

        let stems_in = AudioStems {
            vocals: vec![1.0f32; n],
            bgm: vec![2.0f32; n],
            noise: vec![0.5f32; n],
        };
        let mut stems_out = AudioStems {
            vocals: vec![0f32; n],
            bgm: vec![0f32; n],
            noise: vec![0f32; n],
        };
        apply_window_stems(&stems_in, &win, &mut stems_out);

        for i in 0..n {
            assert!((stems_out.vocals[i] - stems_in.vocals[i] * win[i]).abs() < 1e-5);
            assert!((stems_out.bgm[i] - stems_in.bgm[i] * win[i]).abs() < 1e-5);
            assert!((stems_out.noise[i] - stems_in.noise[i] * win[i]).abs() < 1e-5);
        }
    }

    #[test]
    fn test_stems_fft_magnitude() {
        let n = 16;
        let mut vocals = vec![Complex32::new(0.0, 0.0); n];
        let mut bgm = vec![Complex32::new(0.0, 0.0); n];
        let mut noise_buf = vec![Complex32::new(0.0, 0.0); n];
        vocals[0] = Complex32::new(3.0, 4.0);
        bgm[0] = Complex32::new(1.0, 1.0);
        noise_buf[0] = Complex32::new(0.0, 5.0);

        let mut v_mag = vec![0f32; n];
        let mut b_mag = vec![0f32; n];
        let mut n_mag = vec![0f32; n];
        fft_magnitude(&vocals, &mut v_mag);
        fft_magnitude(&bgm, &mut b_mag);
        fft_magnitude(&noise_buf, &mut n_mag);

        assert!((v_mag[0] - 5.0).abs() < 1e-5);
        assert!((b_mag[0] - 2f32.sqrt()).abs() < 1e-5);
        assert!((n_mag[0] - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_batch_windowed_fft_peak() {
        let n = 32;
        let mut win = vec![0f32; n];
        hann_window_init(&mut win);

        let vocals: Vec<f32> = (0..n)
            .map(|i| (2.0 * PI * 4.0 * i as f32 / n as f32).cos())
            .collect();
        let bgm: Vec<f32> = (0..n)
            .map(|i| (2.0 * PI * 2.0 * i as f32 / n as f32).sin())
            .collect();
        let noise = vec![0.1f32; n];

        let in_stems = AudioStems { vocals, bgm, noise };
        let zero_c = Complex32::new(0.0, 0.0);
        let mut fft_buf = AudioStems {
            vocals: vec![zero_c; n],
            bgm: vec![zero_c; n],
            noise: vec![zero_c; n],
        };
        let mut mag = AudioStems {
            vocals: vec![0f32; n],
            bgm: vec![0f32; n],
            noise: vec![0f32; n],
        };

        batch_windowed_fft(&in_stems, &win, &mut fft_buf, &mut mag);
        assert!(mag.vocals[4] > 0.1, "expected peak at bin 4");
    }
}
