use std::f64::consts::PI;

pub const RATE: u32 = 44100;
pub const FRAME: usize = 2048;

pub fn sine(frequency: f64, amplitude: f64) -> Vec<f32> {
    (0..FRAME)
        .map(|i| (amplitude * (2.0 * PI * frequency * i as f64 / f64::from(RATE)).sin()) as f32)
        .collect()
}

pub fn saw(frequency: f64, amplitude: f64) -> Vec<f32> {
    (0..FRAME)
        .map(|i| {
            let phase = (frequency * i as f64 / f64::from(RATE)).fract();
            (amplitude * (2.0 * phase - 1.0)) as f32
        })
        .collect()
}

pub fn noise(amplitude: f64) -> Vec<f32> {
    let mut state: u32 = 0x1234_5678;
    (0..FRAME)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            (amplitude * (f64::from(state) / f64::from(u32::MAX) * 2.0 - 1.0)) as f32
        })
        .collect()
}
