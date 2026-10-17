pub mod frame;
pub mod lsb;
pub mod ultrasonic;

use crate::error::{Result, StegoError};
use crate::wav::Audio;

pub trait Stego {
    fn capacity(&self, carrier: &Audio) -> Result<usize>;
    fn encode(&self, carrier: &Audio, payload: &[u8]) -> Result<Audio>;
    fn decode(&self, stego: &Audio) -> Result<Vec<u8>>;
}

pub fn check_capacity(payload: &[u8], available: usize) -> Result<()> {
    if payload.len() > available {
        return Err(StegoError::CapacityExceeded {
            needed: payload.len(),
            available,
        });
    }
    Ok(())
}

pub fn build(method: &str, bits: u8) -> Result<Box<dyn Stego + Send + Sync>> {
    match method {
        "lsb" => Ok(Box::new(lsb::LsbStego::new(bits)?)),
        "ultrasonic" => Ok(Box::new(ultrasonic::UltrasonicStego::default())),
        other => Err(StegoError::UnknownMethod(other.to_string())),
    }
}
