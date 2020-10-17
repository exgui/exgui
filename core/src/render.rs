use std::fmt::Debug;

use crate::{Color, CompositeShape};

pub trait Render {
    type Error: Debug;

    fn init(&mut self, _background_color: Color) -> Result<(), Self::Error> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn set_dimensions(&mut self, physical_width: u32, physical_height: u32, device_pixel_ratio: f64) {}

    fn render(&mut self, node: &mut dyn CompositeShape) -> Result<(), Self::Error>;
}
