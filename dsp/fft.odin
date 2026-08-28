package dsp

import "core:math"

fft_single :: proc(data: []complex64) {
	n := len(data)
	if n <= 1 do return
	j := 0
	for i in 0..<n {
		if i < j {
			data[i], data[j] = data[j], data[i]
		}
		m := n >> 1
		for m >= 1 && j >= m {
			j -= m
			m >>= 1
		}
		j += m
	}
	for len_stage := 2; len_stage <= n; len_stage <<= 1 {
		half_len := len_stage >> 1
		angle := -2.0 * math.PI / f32(len_stage)
		w_step := complex(math.cos(angle), math.sin(angle))

		for i := 0; i < n; i += len_stage {
			w: complex64 = 1.0
			for k in 0..<half_len {
				u := data[i + k]
				t := w * data[i + k + half_len]
				data[i + k] = u + t
				data[i + k + half_len] = u - t
				w = w * w_step
			}
		}
	}
}

fft_stems :: proc(stems: AudioStems(complex64)) {
	fft_single(stems.vocals)
	fft_single(stems.bgm)
	fft_single(stems.noise)
}

fft :: proc{
	fft_single,
	fft_stems,
}

fft_magnitude_single :: proc(fft_data: []complex64, magnitude: []f32) {
	n := len(fft_data)
	for i in 0..<n {
		r := real(fft_data[i])
		im := imag(fft_data[i])
		magnitude[i] = math.sqrt(r * r + im * im)
	}
}

fft_magnitude_stems :: proc(stems: AudioStems(complex64), magnitude: AudioStems(f32)) {
	fft_magnitude_single(stems.vocals, magnitude.vocals)
	fft_magnitude_single(stems.bgm, magnitude.bgm)
	fft_magnitude_single(stems.noise, magnitude.noise)
}

fft_magnitude :: proc{
	fft_magnitude_single,
	fft_magnitude_stems,
}

batch_windowed_fft :: proc(
	input: AudioStems(f32),
	window: []f32,
	fft_buffer: AudioStems(complex64),
	magnitude_out: AudioStems(f32),
) {
	process_stem :: proc(in_buf: []f32, win: []f32, fft_buf: []complex64, mag_out: []f32) {
		n := len(win)
		for i in 0..<n {
			fft_buf[i] = complex(in_buf[i] * win[i], f32(0.0))
		}
		fft_single(fft_buf)
		fft_magnitude_single(fft_buf, mag_out)
	}

	process_stem(input.vocals, window, fft_buffer.vocals, magnitude_out.vocals)
	process_stem(input.bgm, window, fft_buffer.bgm, magnitude_out.bgm)
	process_stem(input.noise, window, fft_buffer.noise, magnitude_out.noise)
}

