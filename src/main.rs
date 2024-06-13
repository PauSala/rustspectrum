use analizer::Analizer;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use minifb::{Key, Window, WindowOptions};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::SystemTime;
use visualizer::Visualizer;

pub mod analizer;
pub mod visualizer;

const WIDTH: usize = 1024;
const HEIGHT: usize = WIDTH;
const DELTA: f32 = 2.0;
const CHUNK_SIZE: usize = 1024;
const SHRINK_FACTOR: usize = 8;
const SCALE_FACTOR: usize = 2;
const BUF_LEN: usize = 1024;
const DB_LEN: usize = 128;

fn main() {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .expect("Failed to get default input device");
    let config = device
        .default_input_config()
        .expect("Failed to get default input config");

    let sample_format = config.sample_format();
    dbg!(&config);
    let config = config.into();
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    //samles buffer
    let shared_buffer = Arc::new(Mutex::new(Vec::new()));
    let shared_buffer_clone = Arc::clone(&shared_buffer);

    //Frequencies buffer
    let shrd_ff = Arc::new(Mutex::new(Vec::new()));
    let shrd_ff_cln = Arc::clone(&shrd_ff);

    //input stream
    let stream = match sample_format {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config,
            move |data: &[f32], _| {
                let mut buffer = shared_buffer_clone.lock().unwrap();
                if buffer.len() >= BUF_LEN {
                    buffer.clear();
                }
                buffer.extend_from_slice(data);
            },
            err_fn,
            None,
        ),
        cpal::SampleFormat::I8 => todo!(),
        cpal::SampleFormat::I32 => todo!(),
        cpal::SampleFormat::I64 => todo!(),
        cpal::SampleFormat::U8 => todo!(),
        cpal::SampleFormat::U32 => todo!(),
        cpal::SampleFormat::U64 => todo!(),
        cpal::SampleFormat::F64 => todo!(),
        _ => todo!(),
    }
    .expect("Failed to build input stream");

    stream.play().expect("Failed to play stream");

    // Simulate processing in another thread
    let shared_buffer_clone = Arc::clone(&shared_buffer);

    thread::spawn(move || loop {
        let buffer = {
            let buffer = shared_buffer_clone.lock().unwrap();
            if buffer.len() > 0 {
                Some(buffer)
            } else {
                None
            }
        };
        if let Some(buffer) = buffer {
            if buffer.len() >= BUF_LEN {
                let analizer = Analizer::new(&buffer, CHUNK_SIZE, 44100, 1);
                let ff = analizer.get_1c_live_frequencies();
                let lock = shrd_ff_cln.lock();
                match lock {
                    Ok(mut buffer) => {
                        buffer.clear();
                        buffer.extend_from_slice(&ff);
                    }
                    Err(e) => {
                        panic!("{}", e)
                    }
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    });

    let mut window = Window::new(
        "Frequency Spectrum",
        WIDTH / SCALE_FACTOR,
        HEIGHT / SCALE_FACTOR,
        WindowOptions::default(),
    )
    .unwrap();
    let visualizer = Visualizer::new(
        vec![vec![0.0; DB_LEN]],
        WIDTH,
        HEIGHT,
        SCALE_FACTOR,
        DELTA,
        visualizer::Visualization::CircleGod,
    );
    window.set_target_fps(30);
    let mut start = SystemTime::now();
    let mut curr = vec![0.0; DB_LEN];
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let end = SystemTime::now();
        let elapsed = end.duration_since(start).unwrap();
        //println!("Elapsed: {:?}", elapsed.as_millis());
        // precission issues
        let millis = elapsed.as_nanos() as f64 / 1_000_000.0;

        //Get visualiation buffer
        let shrd_ff_cln = Arc::clone(&shrd_ff);
        let b = visualizer.get_live_buffer(&mut curr, shrd_ff_cln, millis);
        if let Some(b) = b {
            window
                .update_with_buffer(&b, WIDTH / SCALE_FACTOR, HEIGHT / SCALE_FACTOR)
                .unwrap();
        }

        start = end;
    }
}
