package main

import "core:fmt"
import "core:math"
import "core:os"
import "core:strings"
import "core:sys/posix"
import "core:time"
import "./audio"
import "./dsp"
import "./physics"
import "./tui"

FFT_SIZE :: 2048

main :: proc() {
	if !audio.init(44100) {
		fmt.eprintln("audio: init failed")
		os.exit(1)
	}
	defer audio.shutdown()

	tui.enter_alt_screen()
	defer tui.exit_alt_screen()

	orig_termios: posix.termios
	posix.tcgetattr(posix.STDIN_FILENO, &orig_termios)

	raw_termios := orig_termios
	raw_termios.c_lflag -= { .ECHO, .ICANON, .ISIG, .IEXTEN }
	raw_termios.c_iflag -= { .IXON, .ICRNL }
	raw_termios.c_cc[.VMIN] = 0
	raw_termios.c_cc[.VTIME] = 0
	posix.tcsetattr(posix.STDIN_FILENO, .TCSAFLUSH, &raw_termios)

	defer posix.tcsetattr(posix.STDIN_FILENO, .TCSAFLUSH, &orig_termios)
	window := make([]f32, FFT_SIZE)
	defer delete(window)
	dsp.hann_window_init(window)

	samples := make([]f32, FFT_SIZE)
	defer delete(samples)

	windowed := make([]f32, FFT_SIZE)
	defer delete(windowed)

	fft_data := make([]complex64, FFT_SIZE)
	defer delete(fft_data)

	term_size := tui.get_terminal_size()
	num_bars := term_size.width
	if num_bars <= 0 do num_bars = 40
	bars_state := make([]physics.Bar_State, num_bars)
	defer delete(bars_state)

	physics_config := physics.Physics_Config{
		gravity          = 0.03,
		peak_gravity     = 0.008,
		peak_hold_frames = 15,
		rise_smoothing   = 0.7,
		fall_smoothing   = 0.15,
	}

	bin_lows := make([]int, num_bars)
	bin_highs := make([]int, num_bars)
	defer delete(bin_lows)
	defer delete(bin_highs)

	dsp.calculate_bins(num_bars, FFT_SIZE, 44100.0, 20.0, 20000.0, bin_lows, bin_highs)

	raw_bars := make([]f32, num_bars)
	defer delete(raw_bars)

	normalized_bars := make([]f32, num_bars)
	defer delete(normalized_bars)

	builder: strings.Builder
	strings.builder_init(&builder)
	defer strings.builder_destroy(&builder)

	max_val_seen: f32 = 0.05
	last_width, last_height := term_size.width, term_size.height

	for {
		key: u8 = 0
		bytes_read := posix.read(posix.STDIN_FILENO, &key, 1)
		if bytes_read > 0 && (key == 'q' || key == 'Q' || key == 27) {
			break
		}

		term_size = tui.get_terminal_size()
		if term_size.width != last_width || term_size.height != last_height {
			tui.clear_screen()
			last_width = term_size.width
			last_height = term_size.height

			num_bars = term_size.width
			if num_bars <= 0 do num_bars = 40

			delete(bars_state)
			bars_state = make([]physics.Bar_State, num_bars)

			delete(bin_lows)
			delete(bin_highs)
			bin_lows = make([]int, num_bars)
			bin_highs = make([]int, num_bars)
			dsp.calculate_bins(num_bars, FFT_SIZE, 44100.0, 20.0, 20000.0, bin_lows, bin_highs)

			delete(raw_bars)
			raw_bars = make([]f32, num_bars)

			delete(normalized_bars)
			normalized_bars = make([]f32, num_bars)
		}
		audio.get_latest_samples(samples)
		dsp.apply_window(samples, window, windowed)

		for i in 0..<FFT_SIZE {
			fft_data[i] = complex(windowed[i], f32(0.0))
		}

		dsp.fft(fft_data)

		dsp.bin_fft_data(fft_data, bin_lows, bin_highs, raw_bars)

		max_val_seen = math.max(max_val_seen * 0.995, 0.05)
		for val in raw_bars {
			if val > max_val_seen {
				max_val_seen = val
			}
		}

		for i in 0..<num_bars {
			normalized_bars[i] = raw_bars[i] / max_val_seen
		}

		physics.update_physics(bars_state, normalized_bars, physics_config)

		tui.render_frame(bars_state, term_size.width, term_size.height, &builder)

		time.sleep(16 * time.Millisecond)
	}

}
