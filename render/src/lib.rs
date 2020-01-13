use exgui_core::CompositeShape;

pub trait Render {
    type Error;

    fn init(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn render(&self, node: &mut dyn CompositeShape) -> Result<(), Self::Error>;
}