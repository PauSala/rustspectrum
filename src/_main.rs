use analizer::Analizer;
use draw::draw_window;
use minifb::{Window, WindowOptions};
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
const SHRINK_FACTOR: usize = 8;
const SCALE_FACTOR: usize = 2;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //Get the player
    let player = Player::new("moz.mp3")?;
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
    let frequencies: Vec<Vec<f32>> = analizer.get_2c_frequencies();
    let frequencies_len = frequencies.len();
    let chunk_len = frequencies[0].len();

    dbg!(chunk_len);
    dbg!(chunk_len / SHRINK_FACTOR);

    //Get the visual processor
    let visualizer = Visualizer::new(
        frequencies,
        WIDTH,
        HEIGHT,
        SCALE_FACTOR,
        DELTA,
        visualizer::Visualization::CircularPlot,
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
        chunk_len / SHRINK_FACTOR,
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
