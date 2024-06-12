use rustfft::{num_complex::Complex, FftPlanner};

pub struct Analizer<'a> {
    samples: &'a [f32],
    sample_rate: u32,
    chunk_size: usize,
    num_channels: usize,
}
impl<'a> Analizer<'a> {
    pub fn new(
        samples: &'a [f32],
        chunk_size: usize,
        sample_rate: u32,
        num_channels: usize,
    ) -> Self {
        Self {
            samples,
            chunk_size,
            sample_rate,
            num_channels,
        }
    }

    pub fn get_frequencies(&self) -> Vec<Vec<f32>> {
        let mut channels = self.partition_samples(self.samples, self.num_channels);
        let mut left_channel: Vec<f32> = channels.pop().expect("At least 2 channels are required");
        let mut right_channel: Vec<f32> = channels.pop().expect("At least 2 channels are required");

        let mut l_sample_chunks = left_channel.chunks_mut(self.chunk_size);
        let mut r_sample_chunks = right_channel.chunks_mut(self.chunk_size);

        let mut freqs: Vec<Vec<f32>> = Vec::new();
        while let Some(mut l_chunk) = l_sample_chunks.next() {
            let mut r_chunk = r_sample_chunks
                .next()
                .expect("Channels must have the same length");
            self.apply_hann_window(&mut l_chunk);
            self.apply_hann_window(&mut r_chunk);
            let l_frequencies = self.analyze_frequencies(&l_chunk, self.sample_rate);
            let r_frequencies = self.analyze_frequencies(&r_chunk, self.sample_rate);
            let frequencies: Vec<f32> = l_frequencies
                .iter()
                .zip(r_frequencies.iter())
                .map(|(l, r)| (l + r) / 2.0)
                .collect();
            freqs.push(frequencies);
        }
        self.normalize_freqs(&mut freqs);
        freqs
    }

    fn partition_samples(&'a self, samples: &'a [f32], num_channels: usize) -> Vec<Vec<f32>> {
        let mut channels: Vec<Vec<f32>> = vec![Vec::new(); num_channels];

        for (index, sample) in samples.iter().enumerate() {
            let channel = index % num_channels;
            channels[channel].push(*sample);
        }
        channels
    }

    fn hann_window(&self, n: usize) -> Vec<f32> {
        let mut window = Vec::with_capacity(n);
        for i in 0..n {
            let value =
                0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (n as f32 - 1.0)).cos());
            window.push(value);
        }
        window
    }
    fn apply_hann_window(&self, samples: &mut [f32]) {
        let n = samples.len();
        let window = self.hann_window(n);

        for (i, sample) in samples.iter_mut().enumerate() {
            *sample *= window[i];
        }
    }

    fn analyze_frequencies(&self, samples: &[f32], _sample_rate: u32) -> Vec<f32> {
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

    fn normalize_freqs(&self, freqs: &mut Vec<Vec<f32>>) {
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
}
