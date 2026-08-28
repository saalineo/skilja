package dsp

import "core:math"

calculate_bins :: proc(
	num_bars: int,
	fft_size: int,
	sample_rate: f32,
	min_freq: f32,
	max_freq: f32,
	bin_lows: []int,
	bin_highs: []int,
) {
	half_fft := fft_size / 2
	for i in 0..<num_bars {
		freq_low  := min_freq * math.pow(max_freq / min_freq, f32(i) / f32(num_bars))
		freq_high := min_freq * math.pow(max_freq / min_freq, f32(i + 1) / f32(num_bars))

		bin_start := int(freq_low * f32(fft_size) / sample_rate)
		bin_end   := int(freq_high * f32(fft_size) / sample_rate)

		if bin_start < 1 do bin_start = 1
		if bin_end <= bin_start do bin_end = bin_start + 1
		if bin_start >= half_fft do bin_start = half_fft - 1
		if bin_end > half_fft do bin_end = half_fft

		bin_lows[i]  = bin_start
		bin_highs[i] = bin_end
	}
}

bin_fft_data :: proc(
	fft_data: []complex64,
	bin_lows: []int,
	bin_highs: []int,
	bars: []f32,
) {
	num_bars := len(bars)
	for i in 0..<num_bars {
		low := bin_lows[i]
		high := bin_highs[i]

		sum: f32 = 0.0
		for k in low..<high {
			re := real(fft_data[k])
			im := imag(fft_data[k])
			sum += math.sqrt(re*re + im*im)
		}

		avg := sum / f32(high - low)
		boost := 1.0 + 3.0 * (f32(i) / f32(num_bars))
		bars[i] = avg * boost
	}
}

