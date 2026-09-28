use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Host, Stream, StreamConfig};
use ort::session::{builder::GraphOptimizationLevel, Session};
use ort::value::TensorRef;

const BUFFER_SIZE: usize = 16384;

struct RingBuffer {
    buf: Box<UnsafeCell<[f32; BUFFER_SIZE]>>,
    write_pos: AtomicUsize,
}

unsafe impl Sync for RingBuffer {}

static STREAM: Mutex<Option<Stream>> = Mutex::new(None);
static RING: Mutex<Option<Arc<RingBuffer>>> = Mutex::new(None);

pub fn init(sample_rate: u32) -> bool {
    let host = cpal::default_host();

    let device = find_monitor(&host).unwrap_or_else(|| {
        eprintln!("audio: monitor unavailable, using default input");
        host.default_input_device().expect("no input device")
    });

    let config = StreamConfig {
        channels: 1,
        sample_rate,
        buffer_size: cpal::BufferSize::Default,
    };

    let ring = Arc::new(RingBuffer {
        buf: Box::new(UnsafeCell::new([0f32; BUFFER_SIZE])),
        write_pos: AtomicUsize::new(0),
    });
    *RING.lock().unwrap() = Some(Arc::clone(&ring));

    let ring_capture = Arc::clone(&ring);
    let stream = match device.build_input_stream(
        config,
        move |data: &[f32], _| {
            let buf = unsafe { &mut *ring_capture.buf.get() };
            let mut pos = ring_capture.write_pos.load(Ordering::Relaxed);
            for &sample in data {
                buf[pos] = sample;
                pos = (pos + 1) % BUFFER_SIZE;
            }
            ring_capture.write_pos.store(pos, Ordering::Release);
        },
        |err| eprintln!("audio: stream error: {err}"),
        None,
    ) {
        Ok(stream) => stream,
        Err(err) => {
            eprintln!("audio: failed to build stream: {err}");
            return false;
        }
    };

    if stream.play().is_err() {
        return false;
    }
    *STREAM.lock().unwrap() = Some(stream);
    true
}

pub fn shutdown() {
    *STREAM.lock().unwrap() = None;
    *RING.lock().unwrap() = None;
}

pub fn get_latest_samples(dest: &mut [f32]) {
    let guard = RING.lock().unwrap();
    let Some(ring) = guard.as_ref() else { return };
    let n = dest.len().min(BUFFER_SIZE);
    let write_pos = ring.write_pos.load(Ordering::Acquire);
    let start = (write_pos + BUFFER_SIZE - n) % BUFFER_SIZE;
    let buf = unsafe { &*ring.buf.get() };
    for (i, target) in dest[..n].iter_mut().enumerate() {
        *target = buf[(start + i) % BUFFER_SIZE];
    }
}

fn find_monitor(host: &Host) -> Option<cpal::Device> {
    host.input_devices().ok()?.find(|device| {
        device
            .description()
            .map(|desc| desc.name().to_lowercase().contains("monitor"))
            .unwrap_or(false)
    })
}

#[allow(dead_code)]
pub const RNNOISE_FRAME_SIZE: usize = 480;

#[allow(dead_code)]
pub struct Denoiser(Box<nnnoiseless::DenoiseState<'static>>);

impl Denoiser {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self(nnnoiseless::DenoiseState::new())
    }

    #[allow(dead_code)]
    pub fn process(&mut self, input: &[f32], speech: &mut [f32], noise_residual: &mut [f32]) -> bool {
        if input.len() < RNNOISE_FRAME_SIZE
            || speech.len() < RNNOISE_FRAME_SIZE
            || noise_residual.len() < RNNOISE_FRAME_SIZE
        {
            return false;
        }
        let mut scaled = [0f32; RNNOISE_FRAME_SIZE];
        let mut out_scaled = [0f32; RNNOISE_FRAME_SIZE];
        for (scaled_sample, &raw_sample) in scaled.iter_mut().zip(input) {
            *scaled_sample = raw_sample * 32767.0;
        }
        self.0.process_frame(&mut out_scaled, &scaled);
        for i in 0..RNNOISE_FRAME_SIZE {
            speech[i] = out_scaled[i] / 32767.0;
            noise_residual[i] = input[i] - speech[i];
        }
        true
    }
}

#[allow(dead_code)]
pub struct Demuxer(Session);

impl Demuxer {
    #[allow(dead_code)]
    pub fn new(model_path: &str) -> ort::Result<Self> {
        let session = Session::builder()?
            .with_intra_threads(1)?
            .with_optimization_level(GraphOptimizationLevel::All)?
            .commit_from_file(model_path)?;
        Ok(Self(session))
    }

    #[allow(dead_code)]
    pub fn separate(
        &mut self,
        raw: &[f32],
        vocals: &mut [f32],
        bgm: &mut [f32],
        noise: Option<&mut [f32]>,
    ) -> ort::Result<()> {
        let n = raw.len();
        let input = TensorRef::from_array_view(([1, 1, n], raw))?;
        let outputs = self.0.run(ort::inputs!["input_audio" => input])?;

        let (_, data) = outputs["stems_output"].try_extract_tensor::<f32>()?;

        vocals[..n].copy_from_slice(&data[..n]);
        bgm[..n].copy_from_slice(&data[n..2 * n]);
        if let Some(noise_buf) = noise {
            for i in 0..n {
                noise_buf[i] = raw[i] - vocals[i] - bgm[i];
            }
        }
        Ok(())
    }
}
