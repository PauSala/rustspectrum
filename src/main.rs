use minifb::{Key, Window, WindowOptions};
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use std::time::SystemTime;

pub mod analizer;
pub mod player;

const WIDTH: usize = 1024;
const HEIGHT: usize = WIDTH;
const DELTA: f32 = 2.0;
const CHUNK_SIZE: usize = 2048;
const SHRINK_FACTOR: usize = 4;
const SCALE_FACTOR: usize = 2;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let player = player::Player::new("moz.mp3")?;

    // Collect samples
    let sample_rate = player.spec().sample_rate;
    println!("Sample rate: {}", sample_rate);
    let samples: Vec<f32> = player.samples();
    let num_channels = player.spec().channels as usize;
    let duration = player.spec().duration;
    println!("Duration: {}", duration);

    let analizer = analizer::Analizer::new(&samples, CHUNK_SIZE, sample_rate, num_channels);
    let freqs: Vec<Vec<f32>> = analizer.get_frequencies();

    let colors = gradient(CHUNK_SIZE);

    // Create a window for visualization
    let mut window = Window::new(
        "Frequency Spectrum",
        WIDTH / SCALE_FACTOR,
        HEIGHT / SCALE_FACTOR,
        WindowOptions::default(),
    )?;

    let chunks_per_milisecond = freqs.len() as f64 / (duration * 1000.0);

    //window.set_target_fps(left_channel.len() / duration as usize / chunk_size as usize);
    let mut i = 0;
    let all_start = SystemTime::now();
    let mut start = SystemTime::now();
    let mut chunk_count = 0.0;
    // Play the audio
    player.play()?;
    //stream_handle.play_once(source)?;
    let mut curr = vec![0.0; freqs[0].len() / SHRINK_FACTOR];
    while window.is_open() && !window.is_key_down(Key::Escape) && i < freqs.len() {
        let end = SystemTime::now();
        let elapsed = end.duration_since(start).unwrap();
        let millis = elapsed.as_nanos() as f64 / 1_000_000.0;
        chunk_count += millis * chunks_per_milisecond;

        if i < freqs.len() {
            let freq = freqs.get(i).unwrap();
            for i in 0..&freq.len() / SHRINK_FACTOR {
                curr[i] += (freq[i] - curr[i]) as f32 * (millis / 1000.0) as f32 * DELTA;
            }
            let b = draw_circles(&curr, &colors);
            window.update_with_buffer(&b, WIDTH / SCALE_FACTOR, HEIGHT / SCALE_FACTOR)?;
        }
        i += 1;
        if i != chunk_count.round() as usize {
            i = chunk_count.round() as usize;
        }
        start = end;
    }

    let end = SystemTime::now();
    match end.duration_since(all_start) {
        Ok(elapsed) => {
            println!(
                "Total time: {} duration {}",
                elapsed.as_millis() as f32 / 1_000.,
                duration
            );
        }
        Err(e) => {
            // An error occurred!
            println!("Error: {:?}", e);
        }
    }
    Ok(())
}

fn normalize_freqs(freqs: &mut Vec<Vec<f32>>) {
    let mut min = f32::MAX;
    let mut max = f32::MIN;

    // Apply a logarithmic function to compress the range
    for row in freqs.iter_mut() {
        for value in row.iter_mut() {
            *value = (*value + 1.0).ln();
        }
    }
    for row in freqs.iter() {
        for &value in row.iter() {
            if value > max {
                max = value;
            }
            if value < min {
                min = value;
            }
        }
    }
    for row in freqs.iter_mut() {
        for value in row.iter_mut() {
            *value = (*value - min) / (max - min);
        }
    }
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

fn analyze_frequencies(samples: &[f32], _sample_rate: u32) -> Vec<f32> {
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
        .take(half)
        .map(|sample| sample.norm())
        .collect()
}

fn _visualize_frequencies(frequencies: &[f32], colors: &Vec<u32>) -> Vec<u32> {
    let mut buffer: Vec<u32> = vec![0x1c0424; WIDTH * HEIGHT];
    let margin = 1;
    let sample_width = (WIDTH / frequencies.len()) - margin;
    for (i, &sample) in frequencies.iter().enumerate() {
        let x = i * (sample_width + margin);
        let y = ((1.0 - sample) * HEIGHT as f32) as usize; // flip the visualization
        let color = colors[i % colors.len()]; // create a color based on the index
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

fn draw_squares(freqs: &[f32], colors: &Vec<u32>) -> Vec<u32> {
    let mut buffer: Vec<u32> = vec![0xFFFFFF; WIDTH * HEIGHT];
    let mut start = 0;
    let mut end = WIDTH;
    let squares = freqs.len();
    let square_width = WIDTH / squares / 2;
    for &sample in freqs.iter().rev() {
        fill_square(
            &mut buffer,
            start,
            end,
            colors[((sample * (colors.len() as f32)).round() as usize) % colors.len()],
        );
        start += square_width;
        end -= square_width;
    }
    buffer
}

fn fill_square(buffer: &mut Vec<u32>, start: usize, end: usize, color: u32) {
    for i in start..end {
        for j in start..end {
            buffer[j * WIDTH + i] = color;
        }
    }
}

fn draw_circles(freqs: &[f32], colors: &Vec<u32>) -> Vec<u32> {
    let mut buffer: Vec<u32> = vec![0x000000; WIDTH * HEIGHT];
    let circles = freqs.len();
    let max_radius = WIDTH.min(HEIGHT) / 2;
    let radius_step = max_radius / circles;

    for (i, &sample) in freqs.iter().rev().enumerate() {
        let radius = (circles - i) * radius_step;
        let color_index = ((sample * (colors.len() as f32)).round() as usize) % colors.len();
        let color = colors[color_index];
        fill_circle(&mut buffer, WIDTH / 2, HEIGHT / 2, radius, color);
    }
    downscale(&buffer)
}

fn average_colors(colors: &[u32]) -> u32 {
    let mut sum_r = 0u32;
    let mut sum_g = 0u32;
    let mut sum_b = 0u32;
    let count = colors.len() as u32;

    for &color in colors {
        sum_r += (color >> 16) & 0xFF;
        sum_g += (color >> 8) & 0xFF;
        sum_b += color & 0xFF;
    }

    let avg_r = sum_r / count;
    let avg_g = sum_g / count;
    let avg_b = sum_b / count;

    (avg_r << 16) | (avg_g << 8) | avg_b
}

fn downscale(buffer: &[u32]) -> Vec<u32> {
    let mut new_buffer = vec![0xFFFFFF; WIDTH * HEIGHT / (SCALE_FACTOR * SCALE_FACTOR)];

    for r in 0..WIDTH / SCALE_FACTOR {
        for c in 0..HEIGHT / SCALE_FACTOR {
            let mut colors = Vec::new();
            for dy in 0..4 {
                for dx in 0..4 {
                    let orig_row = r * SCALE_FACTOR + dx;
                    let orig_col = c * SCALE_FACTOR + dy;
                    let length = buffer.len() as f64;
                    let width = length.sqrt() as usize;
                    let orig_index = orig_row * width + orig_col;
                    if orig_index >= buffer.len() {
                        continue;
                    }
                    let c = buffer[orig_index].clone();
                    colors.push(c);
                }
            }
            new_buffer[r * WIDTH / SCALE_FACTOR + c] = average_colors(colors.as_slice());
        }
    }
    new_buffer
}

fn fill_circle(buffer: &mut Vec<u32>, cx: usize, cy: usize, radius: usize, color: u32) {
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let dx = x as isize - cx as isize;
            let dy = y as isize - cy as isize;
            if dx * dx + dy * dy <= (radius as isize) * (radius as isize) {
                buffer[y * WIDTH + x] = color;
            }
        }
    }
}

pub fn visualize_frequencies(frequencies: &[f32], colors: &Vec<u32>) -> Vec<u32> {
    let mut buffer: Vec<u32> = vec![0x1c0424; WIDTH * HEIGHT];
    let center_x = WIDTH / 2;
    let center_y = HEIGHT / 2;
    let radius = center_x.min(center_y) as f32;

    for (i, &sample) in frequencies.iter().enumerate() {
        let angle = (i as f32 / frequencies.len() as f32) * 2.0 * std::f32::consts::PI;
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
