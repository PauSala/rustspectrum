use minifb::{Key, Window, WindowOptions};
use std::time::SystemTime;
use visualizer::Visualizer;

pub mod analizer;
pub mod player;
pub mod visualizer;

const WIDTH: usize = 1024;
const HEIGHT: usize = WIDTH;
const DELTA: f32 = 2.0;
const CHUNK_SIZE: usize = 2048;
const SHRINK_FACTOR: usize = 4;
const SCALE_FACTOR: usize = 2;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let player = player::Player::new("bach.wav")?;

    // Collect samples
    let samples: Vec<f32> = player.samples();

    //Audio Spec
    let sample_rate = player.spec().sample_rate;
    println!("Sample rate: {}", sample_rate);
    let num_channels = player.spec().channels as usize;
    println!("Num channels: {}", num_channels);
    let duration = player.spec().duration;
    println!("Duration: {}", duration);

    //Get frequencies
    let analizer = analizer::Analizer::new(&samples, CHUNK_SIZE, sample_rate, num_channels);
    let freqs: Vec<Vec<f32>> = analizer.get_frequencies();
    let freqs_len = freqs.len();
    let chunk_size = freqs[0].len();

    // Create a window for visualization
    let mut window = Window::new(
        "Frequency Spectrum",
        WIDTH / SCALE_FACTOR,
        HEIGHT / SCALE_FACTOR,
        WindowOptions::default(),
    )?;

    let chunks_per_milisecond = freqs.len() as f64 / (duration * 1000.0);

    //Get the visual processor
    let visualizer = Visualizer::new(freqs, WIDTH, HEIGHT, SCALE_FACTOR, SHRINK_FACTOR, DELTA);

    //window.set_target_fps(left_channel.len() / duration as usize / chunk_size as usize);
    let mut current_chunk = 0;
    let all_start = SystemTime::now();
    let mut start = SystemTime::now();
    let mut chunk_count = 0.0;
    // Play the audio
    player.play()?;
    //stream_handle.play_once(source)?;
    let mut curr = vec![0.0; chunk_size / SHRINK_FACTOR];
    while window.is_open() && !window.is_key_down(Key::Escape) && current_chunk < freqs_len {
        let end = SystemTime::now();
        let elapsed = end.duration_since(start).unwrap();
        let millis = elapsed.as_nanos() as f64 / 1_000_000.0;
        chunk_count += millis * chunks_per_milisecond;

        let b = visualizer.get_buffer(&mut curr, current_chunk, millis);
        if let Some(b) = b {
            window.update_with_buffer(&b, WIDTH / SCALE_FACTOR, HEIGHT / SCALE_FACTOR)?;
        }
        current_chunk += 1;
        if current_chunk != chunk_count.round() as usize {
            current_chunk = chunk_count.round() as usize;
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
