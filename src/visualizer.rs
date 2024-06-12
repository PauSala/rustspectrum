pub struct Visualizer {
    frequencies: Vec<Vec<f32>>,
    shrink_factor: usize,
    width: usize,
    height: usize,
    scale_factor: usize,
    delta: f32,
    colors: Vec<u32>,
}

impl Visualizer {
    pub fn new(
        frequencies: Vec<Vec<f32>>,
        width: usize,
        height: usize,
        scale_factor: usize,
        shrink_factor: usize,
        delta: f32,
    ) -> Self {
        let colors = Visualizer::gradient(frequencies[0].len());
        Self {
            frequencies,
            width,
            height,
            scale_factor,
            shrink_factor,
            delta,
            colors,
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
            for i in 0..&freq.len() / self.shrink_factor {
                prev_buffer[i] += (freq[i] - prev_buffer[i]) as f32
                    * (elapsed_milis / 1000.0) as f32
                    * self.delta;
            }
            return Some(self.draw_circles(&prev_buffer, &self.colors));
        }
        None
    }

    fn draw_circles(&self, freqs: &[f32], colors: &Vec<u32>) -> Vec<u32> {
        let mut buffer: Vec<u32> = vec![0x000000; self.width * self.height];
        let circles = freqs.len();
        let max_radius = self.width.min(self.height) / 2;
        let radius_step = max_radius / circles;

        for (i, &sample) in freqs.iter().rev().enumerate() {
            let radius = (circles - i) * radius_step;
            let color_index = ((sample * (colors.len() as f32)).round() as usize) % colors.len();
            let color = colors[color_index];
            self.fill_circle(&mut buffer, self.width / 2, self.height / 2, radius, color);
        }
        self.downscale(&buffer)
    }

    fn fill_circle(&self, buffer: &mut Vec<u32>, cx: usize, cy: usize, radius: usize, color: u32) {
        for y in 0..self.height {
            for x in 0..self.width {
                let dx = x as isize - cx as isize;
                let dy = y as isize - cy as isize;
                if dx * dx + dy * dy <= (radius as isize) * (radius as isize) {
                    buffer[y * self.width + x] = color;
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
