use analizer::Analizer;
use draw::draw_window;
use minifb::{Key, Window, WindowOptions};
use player::Player;
use std::time::SystemTime;
use visualizer::Visualizer;

pub mod analizer;
pub mod draw;
pub mod player;
pub mod visualizer;

const WIDTH: usize = 1024;
const HEIGHT: usize = WIDTH;
const DELTA: f32 = 2.0;
const CHUNK_SIZE: usize = 1024;
const SHRINK_FACTOR: usize = 4;
const SCALE_FACTOR: usize = 2;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //Get the player
    let player = Player::new("bach.wav")?;
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
    let analizer = Analizer::new(&samples, CHUNK_SIZE, sample_rate, num_channels);
    let frequencies: Vec<Vec<f32>> = analizer.get_frequencies();
    let frequencies_len = frequencies.len();

    //Get the visual processor
    let visualizer = Visualizer::new(
        frequencies,
        WIDTH,
        HEIGHT,
        SCALE_FACTOR,
        SHRINK_FACTOR,
        DELTA,
    );

    //setup
    let all_start = SystemTime::now();

    // Window
    let chunks_per_milisecond = frequencies_len as f64 / (duration * 1000.0);
    let window = Window::new(
        "Frequency Spectrum",
        WIDTH / SCALE_FACTOR,
        HEIGHT / SCALE_FACTOR,
        WindowOptions::default(),
    )?;
    // Play the audio
    player.play()?;
    // Draw window
    draw_window(
        window,
        WIDTH,
        HEIGHT,
        SCALE_FACTOR,
        CHUNK_SIZE,
        SHRINK_FACTOR,
        chunks_per_milisecond,
        frequencies_len,
        visualizer,
    )?;

    let end = SystemTime::now();
    let elapsed = end.duration_since(all_start).unwrap();
    println!(
        "Total time: {} duration {}",
        elapsed.as_millis() as f32 / 1_000.,
        duration
    );
    Ok(())
}

fn visualize_frequencies_plot(frequencies: &[f32], colors: &Vec<u32>) -> Vec<u32> {
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
