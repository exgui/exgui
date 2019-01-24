use crate::egml::Drawable;

pub trait Renderer {
    type Error;

    fn init(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn render(&self, node: &mut dyn Drawable) -> Result<(), Self::Error>;
}