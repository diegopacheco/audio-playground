pub const SAMPLE_RATE: u32 = 44_100;

pub struct Mono {
    pub sample_rate: u32,
    pub samples: Vec<f32>,
}

pub struct Stereo {
    pub sample_rate: u32,
    pub left: Vec<f32>,
    pub right: Vec<f32>,
}

impl Mono {
    pub fn duration(&self) -> f64 {
        self.samples.len() as f64 / self.sample_rate as f64
    }
}
