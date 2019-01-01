use std::any::Any;
use std::convert::AsRef;

#[derive(Default)]
pub struct Text {
    pub content: String,
    pub modifier: Option<fn(&mut Text, &dyn Any)>,
}

impl AsRef<str> for Text {
    fn as_ref(&self) -> &str {
        self.content.as_str()
    }
}