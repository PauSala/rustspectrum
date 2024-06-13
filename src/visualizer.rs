use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{analizer::Analizer, SHRINK_FACTOR};

pub struct Visualizer {
    frequencies: Vec<Vec<f32>>,
    width: usize,
    height: usize,
    scale_factor: usize,
    delta: f32,
    colors: Vec<u32>,
    visualization: Visualization,
    circles: HashMap<usize, Vec<usize>>,
}

pub enum Visualization {
    CircleGod,
    SquaredGod,
    CircularPlot,
    Plot,
}

impl Visualizer {
    pub fn new(
        frequencies: Vec<Vec<f32>>,
        width: usize,
        height: usize,
        scale_factor: usize,
        delta: f32,
        visualization: Visualization,
    ) -> Self {
        let colors = Visualizer::gradient(frequencies[0].len() / SHRINK_FACTOR);
        Self {
            frequencies,
            width,
            height,
            scale_factor,
            delta,
            colors,
            visualization,
            circles: Visualizer::classify_circles(width, height),
        }
    }

    pub fn classify_circles(width: usize, height: usize) -> HashMap<usize, Vec<usize>> {
        let circles = 128;
        let max_radius = width.min(height) / 2;
        let radius_step = max_radius / circles;

        let mut all_circles: Vec<usize> = Vec::new();

        for i in 0..128 {
            let radius = (circles - i) * radius_step;
            all_circles.push(radius);
        }

        let mut res: HashMap<usize, Vec<usize>> = HashMap::new();

        let cx = width / 2;
        let cy = height / 2;
        for y in 0..height {
            for x in 0..width {
                let dx = x as isize - cx as isize;
                let dy = y as isize - cy as isize;
                let dist_sq = dx * dx + dy * dy;
                let index = y * width + x;
                for circle in all_circles.iter().rev() {
                    if dist_sq <= (circle * circle) as isize {
                        res.entry(*circle).and_modify(|v| v.push(index));
                        break;
                    }
                }
            }
        }
        res
    }

    pub fn get_live_buffer(
        &self,
        prev_buffer: &mut Vec<f32>,
        freqs: Arc<Mutex<Vec<f32>>>,
        elapsed_milis: f64,
    ) -> Option<Vec<u32>> {
        let ff = freqs.lock().unwrap();
        if ff.len() < 1 {
            return None;
        }
        for i in 0..prev_buffer.len() {
            prev_buffer[i] +=
                (ff[i] - prev_buffer[i]) as f32 * (elapsed_milis / 1000.0) as f32 * self.delta;
        }
        //println!("Buffer {:?}", prev_buffer);
        match self.visualization {
            Visualization::CircleGod => return Some(self.draw_circles(&prev_buffer, &self.colors)),
            Visualization::SquaredGod => {
                return Some(self.draw_squares(&prev_buffer, &self.colors))
            }
            Visualization::CircularPlot => {
                return Some(self.circle_plot(&prev_buffer, &self.colors))
            }
            Visualization::Plot => {
                return Some(self.visualize_frequencies_plot(&prev_buffer, &self.colors))
            }
        }
    }

    pub fn get_buffer(
        &self,
        prev_buffer: &mut Vec<f32>,
        current_chunk: usize,
        elapsed_milis: f64,
    ) -> Option<Vec<u32>> {
        if current_chunk < self.frequencies.len() {
            let freq = self.frequencies.get(current_chunk).unwrap();
            //println!("buff len: {} freq len: {}", prev_buffer.len(), freq.len());
            for i in 0..prev_buffer.len() {
                prev_buffer[i] += (freq[i] - prev_buffer[i]) as f32
                    * (elapsed_milis / 1000.0) as f32
                    * self.delta;
            }
            match self.visualization {
                Visualization::CircleGod => {
                    return Some(self.draw_circles(&prev_buffer, &self.colors))
                }
                Visualization::SquaredGod => {
                    return Some(self.draw_squares(&prev_buffer, &self.colors))
                }
                Visualization::CircularPlot => {
                    return Some(self.circle_plot(&prev_buffer, &self.colors))
                }
                Visualization::Plot => {
                    return Some(self.visualize_frequencies_plot(&prev_buffer, &self.colors))
                }
            }
        }
        None
    }

    fn draw_circles(&self, freqs: &[f32], colors: &Vec<u32>) -> Vec<u32> {
        let mut buffer: Vec<u32> = vec![0x000000; self.width * self.height];
        let circles = freqs.len();
        let max_radius = self.width.min(self.height) / 2;
        let radius_step = max_radius / circles;

        let mut all_circles: Vec<(usize, u32)> = Vec::new();

        for (i, &sample) in freqs.iter().rev().enumerate() {
            let radius = (circles - i) * radius_step;
            let color_index = ((sample * (colors.len() as f32)).round() as usize) % colors.len();
            let color = colors[color_index];
            all_circles.push((radius, color));
        }
        self.fill_circles(&mut buffer, all_circles);
        self.downscale(&buffer)
    }

    fn fill_circles(&self, buffer: &mut Vec<u32>, circles: Vec<(usize, u32)>) {
        let cx = self.width / 2;
        let cy = self.height / 2;
        for y in 0..self.height {
            for x in 0..self.width {
                let dx = x as isize - cx as isize;
                let dy = y as isize - cy as isize;
                let dist_sq = dx * dx + dy * dy;
                let index = y * self.width + x;
                for circle in circles.iter().rev() {
                    if dist_sq <= circle.0.pow(2) as isize {
                        buffer[index] = circle.1;
                        break;
                    }
                }
            }
        }
    }

    fn average_colors(&self, colors: &[u32]) -> u32 {
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

    fn downscale(&self, buffer: &[u32]) -> Vec<u32> {
        let mut new_buffer =
            vec![0xFFFFFF; self.width * self.height / (self.scale_factor * self.scale_factor)];

        for r in 0..self.width / self.scale_factor {
            for c in 0..self.height / self.scale_factor {
                let mut colors = Vec::new();
                for dy in 0..4 {
                    for dx in 0..4 {
                        let orig_row = r * self.scale_factor + dx;
                        let orig_col = c * self.scale_factor + dy;
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
                new_buffer[r * self.width / self.scale_factor + c] =
                    self.average_colors(colors.as_slice());
            }
        }
        new_buffer
    }

    fn draw_squares(&self, freqs: &[f32], colors: &Vec<u32>) -> Vec<u32> {
        let mut buffer: Vec<u32> =
            vec![0xFFFFFF; (self.width / self.scale_factor) * (self.height / self.scale_factor)];
        let mut start = 0;
        let mut end = self.width / self.scale_factor;
        let squares = freqs.len();
        let square_width = (self.width / self.scale_factor) / squares / 2;
        for &sample in freqs.iter().rev() {
            self.fill_square(
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

    fn fill_square(&self, buffer: &mut Vec<u32>, start: usize, end: usize, color: u32) {
        for i in start..end {
            for j in start..end {
                buffer[j * (self.width / self.scale_factor) + i] = color;
            }
        }
    }

    pub fn circle_plot(&self, frequencies: &[f32], colors: &Vec<u32>) -> Vec<u32> {
        const WIDTH: usize = 512;
        const HEIGHT: usize = 512;
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

    pub fn visualize_frequencies_plot(&self, frequencies: &[f32], colors: &Vec<u32>) -> Vec<u32> {
        const WIDTH: usize = 512;
        const HEIGHT: usize = 512;
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
}
