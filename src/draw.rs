use anyhow::Result;
use minifb::{Key, Window};
use std::time::SystemTime;

use crate::visualizer::Visualizer;

pub fn draw_window(
    mut window: Window,
    width: usize,
    height: usize,
    scale_factor: usize,
    chunk_size: usize,
    shrink_factor: usize,
    chunks_per_milisecond: f64,
    freqs_len: usize,
    visualizer: Visualizer,
) -> Result<()> {
    //setup
    let mut current_chunk = 0;
    let mut start = SystemTime::now();
    let mut chunk_count = 0.0;
    let mut curr = vec![0.0; chunk_size / shrink_factor];

    while window.is_open() && !window.is_key_down(Key::Escape) && current_chunk < freqs_len {
        //get chunk count
        let end = SystemTime::now();
        let elapsed = end.duration_since(start).unwrap();
        let millis = elapsed.as_nanos() as f64 / 1_000_000.0;
        chunk_count += millis * chunks_per_milisecond;

        //Get visualiation buffer
        let b = visualizer.get_buffer(&mut curr, current_chunk, millis);
        if let Some(b) = b {
            window.update_with_buffer(&b, width / scale_factor, height / scale_factor)?;
        }

        //Adjust frames and chunks
        current_chunk += 1;
        if current_chunk > chunk_count.round() as usize && chunk_count.round() > 1.0 {
            current_chunk = chunk_count.round() as usize - 1;
        } else if current_chunk < chunk_count.round() as usize {
            current_chunk = chunk_count.round() as usize + 1;
        }
        start = end;
    }
    Ok(())
}
