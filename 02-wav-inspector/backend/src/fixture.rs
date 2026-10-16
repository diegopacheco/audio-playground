use crate::riff::{FORMAT_FLOAT, FORMAT_PCM};

pub struct WavBuilder {
    format_tag: u16,
    bits: u16,
    channels: u16,
    sample_rate: u32,
    extras: Vec<([u8; 4], Vec<u8>)>,
    data: Vec<u8>,
    data_first: bool,
    extensible: bool,
    with_fmt: bool,
    with_data: bool,
}

impl WavBuilder {
    pub fn new(format_tag: u16, bits: u16, sample_rate: u32, channels: u16) -> Self {
        WavBuilder {
            format_tag,
            bits,
            channels,
            sample_rate,
            extras: Vec::new(),
            data: Vec::new(),
            data_first: false,
            extensible: false,
            with_fmt: true,
            with_data: true,
        }
    }

    pub fn pcm8(sample_rate: u32, channels: u16) -> Self {
        Self::new(FORMAT_PCM, 8, sample_rate, channels)
    }

    pub fn pcm16(sample_rate: u32, channels: u16) -> Self {
        Self::new(FORMAT_PCM, 16, sample_rate, channels)
    }

    pub fn pcm24(sample_rate: u32, channels: u16) -> Self {
        Self::new(FORMAT_PCM, 24, sample_rate, channels)
    }

    pub fn pcm32(sample_rate: u32, channels: u16) -> Self {
        Self::new(FORMAT_PCM, 32, sample_rate, channels)
    }

    pub fn float32(sample_rate: u32, channels: u16) -> Self {
        Self::new(FORMAT_FLOAT, 32, sample_rate, channels)
    }

    pub fn extra_chunk(mut self, id: &[u8; 4], body: &[u8]) -> Self {
        self.extras.push((*id, body.to_vec()));
        self
    }

    pub fn data_first(mut self) -> Self {
        self.data_first = true;
        self
    }

    pub fn extensible(mut self) -> Self {
        self.extensible = true;
        self
    }

    pub fn without_fmt(mut self) -> Self {
        self.with_fmt = false;
        self
    }

    pub fn without_data(mut self) -> Self {
        self.with_data = false;
        self
    }

    pub fn raw_data(mut self, data: &[u8]) -> Self {
        self.data = data.to_vec();
        self
    }

    pub fn samples(mut self, samples: &[f32]) -> Self {
        self.data = samples.iter().flat_map(|&s| self.encode(s)).collect();
        self
    }

    fn encode(&self, s: f32) -> Vec<u8> {
        match (self.format_tag, self.bits) {
            (FORMAT_FLOAT, 32) => s.to_le_bytes().to_vec(),
            (_, 8) => vec![((s * 127.0).round() + 128.0) as u8],
            (_, 16) => ((s * 32767.0).round() as i16).to_le_bytes().to_vec(),
            (_, 24) => ((s * 8_388_607.0).round() as i32).to_le_bytes()[0..3].to_vec(),
            (_, 32) => ((f64::from(s) * 2_147_483_647.0).round() as i32)
                .to_le_bytes()
                .to_vec(),
            _ => vec![0; usize::from(self.bits.div_ceil(8))],
        }
    }

    fn fmt_body(&self) -> Vec<u8> {
        let bytes_per_sample = self.bits.div_ceil(8);
        let block_align = self.channels * bytes_per_sample;
        let tag = if self.extensible {
            0xFFFE
        } else {
            self.format_tag
        };
        let mut b = Vec::new();
        b.extend_from_slice(&tag.to_le_bytes());
        b.extend_from_slice(&self.channels.to_le_bytes());
        b.extend_from_slice(&self.sample_rate.to_le_bytes());
        b.extend_from_slice(&(self.sample_rate * u32::from(block_align)).to_le_bytes());
        b.extend_from_slice(&block_align.to_le_bytes());
        b.extend_from_slice(&self.bits.to_le_bytes());
        if self.extensible {
            b.extend_from_slice(&22u16.to_le_bytes());
            b.extend_from_slice(&self.bits.to_le_bytes());
            b.extend_from_slice(&0u32.to_le_bytes());
            b.extend_from_slice(&self.format_tag.to_le_bytes());
            b.extend_from_slice(&[0; 14]);
        }
        b
    }

    pub fn build(self) -> Vec<u8> {
        let mut parts: Vec<([u8; 4], Vec<u8>)> = Vec::new();
        let data = (*b"data", self.data.clone());
        if self.data_first && self.with_data {
            parts.push(data.clone());
        }
        if self.with_fmt {
            parts.push((*b"fmt ", self.fmt_body()));
        }
        parts.extend(self.extras.iter().cloned());
        if !self.data_first && self.with_data {
            parts.push(data);
        }
        let mut body = b"WAVE".to_vec();
        for (id, chunk) in parts {
            body.extend_from_slice(&id);
            body.extend_from_slice(&(chunk.len() as u32).to_le_bytes());
            body.extend_from_slice(&chunk);
            if chunk.len() % 2 == 1 {
                body.push(0);
            }
        }
        let mut out = b"RIFF".to_vec();
        out.extend_from_slice(&(body.len() as u32).to_le_bytes());
        out.extend_from_slice(&body);
        out
    }
}
