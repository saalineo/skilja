package dsp

import "core:math"

AudioStems :: struct($T: typeid) {
	vocals: []T,
	bgm:    []T,
	noise:  []T,
}

hann_window_init :: proc(window: []f32) {
	n := len(window)
	if n <= 1 do return
	for i in 0..<n {
		window[i] = 0.5 * (1.0 - math.cos(2.0 * math.PI * f32(i) / f32(n - 1)))
	}
}

apply_window_single :: proc(input: []f32, window: []f32, output: []f32) {
	n := len(input)
	for i in 0..<n {
		output[i] = input[i] * window[i]
	}
}

apply_window_stems :: proc(input: AudioStems(f32), window: []f32, output: AudioStems(f32)) {
	apply_window_single(input.vocals, window, output.vocals)
	apply_window_single(input.bgm, window, output.bgm)
	apply_window_single(input.noise, window, output.noise)
}

apply_window :: proc{
	apply_window_single,
	apply_window_stems,
}

