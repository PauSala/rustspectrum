use anyhow::Result;
use rodio::{source::Buffered, Decoder, OutputStream, OutputStreamHandle, Source};
use std::{fs::File, io::BufReader};

pub struct Player {
    buffered_decoder: Buffered<Decoder<BufReader<File>>>,
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
}

pub struct AudioSpec {
    pub sample_rate: u32,
    pub channels: usize,
    pub duration: f64,
}

impl Player {
    pub fn new(file_path: &str) -> Result<Self> {
        let file = BufReader::new(File::open(file_path)?);
        let decoder = Decoder::new(file)?.buffered();
        let (_stream, stream_handle) = OutputStream::try_default()?;
        Ok(Self {
            buffered_decoder: decoder,
            _stream,
            stream_handle,
        })
    }

    pub fn spec(&self) -> AudioSpec {
        let sample_rate = self.buffered_decoder.sample_rate();
        let channels = self.buffered_decoder.channels() as usize;
        AudioSpec {
            sample_rate,
            channels,
            duration: (self.samples().len() / channels) as f64 / sample_rate as f64,
        }
    }

    pub fn samples(&self) -> Vec<f32> {
        self.buffered_decoder.clone().convert_samples().collect()
    }

    pub fn play(&self) -> Result<()> {
        self.stream_handle
            .play_raw(self.buffered_decoder.clone().convert_samples())?;
        Ok(())
    }
}
