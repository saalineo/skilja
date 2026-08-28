package dsp

import "core:math"
import "core:testing"

@(test)
test_audio_stems_windowing :: proc(t: ^testing.T) {
	n := 16
	win := make([]f32, n)
	defer delete(win)
	hann_window_init(win)

	v_in := make([]f32, n)
	b_in := make([]f32, n)
	n_in := make([]f32, n)
	defer delete(v_in)
	defer delete(b_in)
	defer delete(n_in)

	v_out := make([]f32, n)
	b_out := make([]f32, n)
	n_out := make([]f32, n)
	defer delete(v_out)
	defer delete(b_out)
	defer delete(n_out)

	for i in 0..<n {
		v_in[i] = 1.0
		b_in[i] = 2.0
		n_in[i] = 0.5
	}

	stems_in := AudioStems(f32){
		vocals = v_in,
		bgm    = b_in,
		noise  = n_in,
	}

	stems_out := AudioStems(f32){
		vocals = v_out,
		bgm    = b_out,
		noise  = n_out,
	}

	apply_window(stems_in, win, stems_out)

	for i in 0..<n {
		testing.expect(t, math.abs(stems_out.vocals[i] - v_in[i]*win[i]) < 1e-5, "Vocals windowing mismatch")
		testing.expect(t, math.abs(stems_out.bgm[i] - b_in[i]*win[i]) < 1e-5, "BGM windowing mismatch")
		testing.expect(t, math.abs(stems_out.noise[i] - n_in[i]*win[i]) < 1e-5, "Noise windowing mismatch")
	}
}

@(test)
test_stems_fft_and_magnitude :: proc(t: ^testing.T) {
	n := 16
	v_data := make([]complex64, n)
	b_data := make([]complex64, n)
	n_data := make([]complex64, n)
	defer delete(v_data)
	defer delete(b_data)
	defer delete(n_data)

	v_mag := make([]f32, n)
	b_mag := make([]f32, n)
	n_mag := make([]f32, n)
	defer delete(v_mag)
	defer delete(b_mag)
	defer delete(n_mag)

	v_data[0] = complex(3.0, 4.0)
	b_data[0] = complex(1.0, 1.0)
	n_data[0] = complex(0.0, 5.0)

	c_stems := AudioStems(complex64){
		vocals = v_data,
		bgm    = b_data,
		noise  = n_data,
	}

	m_stems := AudioStems(f32){
		vocals = v_mag,
		bgm    = b_mag,
		noise  = n_mag,
	}

	fft_magnitude(c_stems, m_stems)

	testing.expect(t, math.abs(m_stems.vocals[0] - 5.0) < 1e-5, "Vocals magnitude mismatch")
	testing.expect(t, math.abs(m_stems.bgm[0] - math.sqrt(f32(2.0))) < 1e-5, "BGM magnitude mismatch")
	testing.expect(t, math.abs(m_stems.noise[0] - 5.0) < 1e-5, "Noise magnitude mismatch")
}

@(test)
test_batch_windowed_fft :: proc(t: ^testing.T) {
	n := 32
	win := make([]f32, n)
	defer delete(win)
	hann_window_init(win)

	v_in := make([]f32, n)
	b_in := make([]f32, n)
	n_in := make([]f32, n)
	defer delete(v_in)
	defer delete(b_in)
	defer delete(n_in)

	for i in 0..<n {
		v_in[i] = math.cos(2.0 * math.PI * 4.0 * f32(i) / f32(n))
		b_in[i] = math.sin(2.0 * math.PI * 2.0 * f32(i) / f32(n))
		n_in[i] = 0.1
	}

	in_stems := AudioStems(f32){vocals = v_in, bgm = b_in, noise = n_in}

	v_fft := make([]complex64, n)
	b_fft := make([]complex64, n)
	n_fft := make([]complex64, n)
	defer delete(v_fft)
	defer delete(b_fft)
	defer delete(n_fft)

	fft_stems := AudioStems(complex64){vocals = v_fft, bgm = b_fft, noise = n_fft}

	v_mag := make([]f32, n)
	b_mag := make([]f32, n)
	n_mag := make([]f32, n)
	defer delete(v_mag)
	defer delete(b_mag)
	defer delete(n_mag)

	mag_stems := AudioStems(f32){vocals = v_mag, bgm = b_mag, noise = n_mag}

	batch_windowed_fft(in_stems, win, fft_stems, mag_stems)

	testing.expect(t, mag_stems.vocals[4] > 0.1, "Expected peak in vocals stem at bin 4")
}
