use std::fmt::Write as _;
use std::io::{self, Write};
use crate::physics::BarState;

pub struct TerminalSize {
    pub width: usize,
    pub height: usize,
}

pub fn get_terminal_size() -> TerminalSize {
    let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
    let ret = unsafe { libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws) };
    if ret >= 0 && ws.ws_col > 0 && ws.ws_row > 0 {
        TerminalSize { width: ws.ws_col as usize, height: ws.ws_row as usize }
    } else {
        TerminalSize { width: 80, height: 24 }
    }
}

pub fn enter_alt_screen() {
    print!("\x1b[?1049h\x1b[?25l\x1b[2J");
    let _ = io::stdout().flush();
}

pub fn exit_alt_screen() {
    print!("\x1b[?25h\x1b[?1049l");
    let _ = io::stdout().flush();
}

pub fn clear_screen() {
    print!("\x1b[2J");
    let _ = io::stdout().flush();
}

fn waveform_color(dist: f32) -> (u8, u8, u8) {
    let d = dist.clamp(0.0, 1.0);
    if d < 0.12 {
        (245, 255, 255)
    } else if d < 0.35 {
        let t = (d - 0.12) / 0.23;
        (
            (120.0 * (1.0 - t)) as u8,
            (220.0 + (150.0 - 220.0) * t) as u8,
            255,
        )
    } else if d < 0.70 {
        let t = (d - 0.35) / 0.35;
        (
            0,
            (150.0 * (1.0 - t)) as u8,
            (255.0 + (180.0 - 255.0) * t) as u8,
        )
    } else {
        let t = (d - 0.70) / 0.30;
        (0, 0, (180.0 * (1.0 - t) + 60.0) as u8)
    }
}

pub fn render_frame(bars: &[BarState], width: usize, height: usize, buf: &mut String) {
    buf.clear();
    buf.push_str("\x1b[H");

    let num_bars = bars.len();
    if num_bars == 0 || width == 0 || height == 0 {
        return;
    }

    let mid_y = height as f32 / 2.0;
    let half_h = mid_y;

    for y in (0..height).rev() {
        let dist_y = (y as f32 - mid_y + 0.5).abs() / half_h;
        let (r, g, b) = waveform_color(dist_y);
        let _ = write!(buf, "\x1b[38;2;{r};{g};{b}m");

        for x in 0..width {
            let norm_x = (x as f32 / width as f32) * 2.0 - 1.0;
            let envelope = (1.0 - norm_x.abs().powf(1.25)).clamp(0.05, 1.0);

            let bar_idx = ((x as f32 / width as f32) * num_bars as f32) as usize;
            let bar_idx = bar_idx.min(num_bars - 1);

            let bar_amp = (bars[bar_idx].amplitude * envelope).clamp(0.0, 1.0);
            if bar_amp >= dist_y {
                buf.push('│');
            } else if bar_amp >= dist_y - (0.4 / half_h) {
                buf.push('·');
            } else {
                buf.push(' ');
            }
        }
        buf.push('\n');
    }

    buf.push_str("\x1b[0m");
    let mut stdout = io::stdout().lock();
    let _ = stdout.write_all(buf.as_bytes());
    let _ = stdout.flush();
}
