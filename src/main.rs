use std::collections::VecDeque;
use std::fs::File;
use std::io::BufReader;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, StreamConfig};
use minifb::{Key, Window, WindowOptions};
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;

const WIDTH: usize = 1024;
const HEIGHT: usize = 812;
const DELTA: f32 = 8.0;
const CHUNK_SIZE: usize = 8192;
const FREQS_SIZE: usize = 128;

fn main() {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no output device available");
    let mut supported_configs_range = device
        .supported_output_configs()
        .expect("error while querying configs");
    let config = supported_configs_range
        .next()
        .expect("no supported config?!")
        .with_max_sample_rate();
    println!("{:?}", &config);
    let config: StreamConfig = StreamConfig {
        channels: config.channels(),
        sample_rate: config.sample_rate(),
        buffer_size: BufferSize::Default,
    };

    let file = BufReader::new(File::open("bach.wav").expect("Failed to open audio file"));
    let mut reader = hound::WavReader::new(file).expect("Failed to read WAV file");
    let mut original_samples: Vec<f32> = reader
        .samples::<i16>()
        .map(|s| s.expect("Failed to read sample") as f32 / i16::MAX as f32)
        .collect();

    apply_hann_window(&mut original_samples);
    let mut freqs: Vec<f32> = Vec::new();
    for chunk in original_samples.chunks(CHUNK_SIZE) {
        let frequencies = analyze_frequencies(chunk);
        for _ in 0..CHUNK_SIZE / FREQS_SIZE {
            for f in &frequencies[0..FREQS_SIZE] {
                freqs.push(*f);
            }
        }
    }
    normalize_freqs(&mut freqs);
    let samples_len = original_samples.len();
    println!("samples len: {} | freqs: {}", samples_len, freqs.len());
    let freqs_index = Arc::new(AtomicUsize::new(0));
    let freqs_index_clone = Arc::clone(&freqs_index);

    // Arc and Mutex for thread-safe data sharing
    // Use VecDeque for efficient front removal
    let samples = Arc::new(Mutex::new(VecDeque::from(original_samples.clone())));
    let samples_clone = Arc::clone(&samples);

    //let mut now = std::time::Instant::now();

    let output_stream = device
        .build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut samples = samples_clone.lock().unwrap();
                for sample in data.iter_mut() {
                    if let Some(s) = samples.pop_front() {
                        *sample = s;
                    } else {
                        *sample = 0.0;
                    }
                }
                freqs_index_clone.fetch_add(data.len(), Ordering::Relaxed);
            },
            move |err| {
                eprintln!("an error occurred on stream: {}", err);
            },
            Some(Duration::from_millis(0)),
        )
        .expect("error while building stream");
    output_stream.play().expect("error while playing stream");

    println!("Playing audio");
    // Create a window for visualization
    let mut window = Window::new(
        "Frequency Spectrum",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap();

    let mut start = SystemTime::now();
    let mut curr = vec![0.0; FREQS_SIZE];
    let colors = gradient(FREQS_SIZE);

    let mut prev_index = 0;

    while window.is_open()
        && !window.is_key_down(Key::Escape)
        && freqs_index.load(Ordering::Relaxed) < freqs.len() - 1024
    {
        let end = SystemTime::now();
        let index = freqs_index.load(Ordering::Acquire);
        if index == prev_index {
            continue;
        }
        let millis = end.duration_since(start).unwrap().as_millis() as f64;
        let freqs = freqs[index..index + FREQS_SIZE].to_vec();
        // Update the curr vector based on freqs and millis
        for i in 0..freqs.len() {
            curr[i] += (freqs[i] - curr[i]) as f32 * (millis / 1000.0) as f32 * DELTA;
        }
        // println!("Freqs {:?}", &curr);
        // Visualize the curr vector
        let v = visualize_bars(&curr, &colors);
        // Update the window's buffer with the visualization
        window.update_with_buffer(&v, WIDTH, HEIGHT).unwrap();

        // Update the start time for the next iteration
        start = end;
        prev_index = index;
    }

    println!("Finished playing");
}

fn hann_window(n: usize) -> Vec<f32> {
    let mut window = Vec::with_capacity(n);
    for i in 0..n {
        let value = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (n as f32 - 1.0)).cos());
        window.push(value);
    }
    window
}

fn apply_hann_window(samples: &mut [f32]) {
    let n = samples.len();
    let window = hann_window(n);

    for (i, sample) in samples.iter_mut().enumerate() {
        *sample *= window[i];
    }
}

fn analyze_frequencies(samples: &[f32]) -> Vec<f32> {
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(samples.len());
    let mut spectrum: Vec<Complex<f32>> = samples
        .iter()
        .map(|&sample| Complex::new(sample, 0.0))
        .collect();
    fft.process(&mut spectrum);
    let half = spectrum.len() / 2;
    spectrum
        .iter()
        .skip(1)
        .take(half)
        .map(|sample| sample.norm())
        .collect() // Only take the first half of the spectrum
}

fn visualize_bars(frequencies: &[f32], colors: &Vec<u32>) -> Vec<u32> {
    let mut buffer: Vec<u32> = vec![0x0f0015; WIDTH * HEIGHT];
    let margin = 1;
    let sample_width = (WIDTH / frequencies.len()) - margin;
    for (i, &sample) in frequencies.iter().enumerate() {
        let x = i * (sample_width + margin);
        let y = ((1.0 - sample) * HEIGHT as f32) as usize; // flip the visualization
        if y == 0 {
            println!("y: {}", y);
        }
        let color = colors[i % colors.len()];
        for j in x..x + sample_width {
            for k in y..HEIGHT {
                let index = j + k * WIDTH;
                if index < buffer.len() {
                    buffer[index] = color;
                }
            }
        }
    }
    buffer
}

fn visualize_circle(frequencies: &[f32], colors: &Vec<u32>) -> Vec<u32> {
    let mut buffer: Vec<u32> = vec![0x0f0015; WIDTH * HEIGHT];
    let center_x = WIDTH / 2;
    let center_y = HEIGHT / 2;
    let radius = center_x.min(center_y) as f32;

    for (i, &sample) in frequencies.iter().enumerate() {
        let angle = (i as f32 / frequencies.len() as f32) * std::f32::consts::PI;
        let r = radius * sample;
        for r in 0..(r * 1000.) as usize {
            let x = (center_x as f32 + (r / 1000) as f32 * angle.cos()) as usize; // convert polar to cartesian coordinates
            let y = (center_y as f32 + (r / 1000) as f32 * angle.sin()) as usize;
            let color = colors[i % colors.len()];

            if x < WIDTH && y < HEIGHT {
                let index = x + y * WIDTH;
                buffer[index] = color;
            }
        }
    }

    buffer
}

fn normalize_freqs(freqs: &mut Vec<f32>) {
    let mut min = f32::MAX;
    let mut max = f32::MIN;

    // for value in freqs.iter_mut() {
    //     *value = (*value + 1.0).ln();
    // }

    // Find the minimum and maximum values in freqs
    for &value in freqs.iter() {
        if value > max {
            max = value;
        }
        if value < min {
            min = value;
        }
    }

    println!("Min: {} | Max: {}", min, max);

    // Normalize the values in freqs to be between 0 and 1
    for value in freqs.iter_mut() {
        *value = (*value - min) / (max - min);
    }
}

fn gradient(len: usize) -> Vec<u32> {
    let g = colorgrad::magma();
    let c = g.colors(len);
    c.iter()
        .map(|color| {
            let r = (color.r * 255.0) as u32;
            let g = (color.g * 255.0) as u32;
            let b = (color.b * 255.0) as u32;
            (r << 16) | (g << 8) | b
        })
        .collect::<Vec<u32>>()
}
