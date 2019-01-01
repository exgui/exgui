use std::any::Any;
use std::convert::AsRef;

#[derive(Default)]
pub struct Word {
    pub content: String,
    pub modifier: Option<fn(&mut Word, &dyn Any)>,
}

impl AsRef<str> for Word {
    fn as_ref(&self) -> &str {
        self.content.as_str()
    }
}