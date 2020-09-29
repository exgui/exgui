use std::fmt::Debug;

use crate::CompositeShape;

pub trait Render {
    type Error: Debug;

    fn init(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn set_dimensions(&mut self, width: u32, height: u32, device_pixel_ratio: f64) {}

    fn render(&self, node: &mut dyn CompositeShape) -> Result<(), Self::Error>;
}
