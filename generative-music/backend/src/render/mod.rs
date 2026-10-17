pub mod midi;
pub mod wav;

use crate::compose::Song;
use crate::error::AppError;

pub trait Renderer {
    fn content_type(&self) -> &'static str;
    fn render(&self, song: &Song) -> Result<Vec<u8>, AppError>;
}
