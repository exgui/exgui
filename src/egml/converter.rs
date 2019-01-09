/// Converts property and attach lazy components to it.
pub trait Converter<TO> {
    /// Convert one type to another.
    fn convert(self) -> TO;
}

impl<T> Converter<T> for T {
    fn convert(self) -> T {
        self
    }
}

impl<T> Converter<Option<T>> for T {
    fn convert(self) -> Option<T> {
        Some(self)
    }
}

impl<'a, T: Clone> Converter<T> for &'a T {
    fn convert(self) -> T {
        self.clone()
    }
}

impl<'a, T: Clone> Converter<Option<T>> for &'a T {
    fn convert(self) -> Option<T> {
        Some(self.clone())
    }
}

impl<'a> Converter<String> for &'a str {
    fn convert(self) -> String {
        self.to_owned()
    }
}

impl<'a> Converter<Option<String>> for &'a str {
    fn convert(self) -> Option<String> {
        Some(self.to_owned())
    }
}