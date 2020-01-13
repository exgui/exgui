/// Converts property and attach lazy components to it.
pub trait ConvertTo<T> {
    /// Convert one type to another.
    fn convert(self) -> T;
}

impl<T> ConvertTo<T> for T {
    fn convert(self) -> T {
        self
    }
}

impl<T> ConvertTo<Option<T>> for T {
    fn convert(self) -> Option<T> {
        Some(self)
    }
}

impl<'a, T: Clone> ConvertTo<T> for &'a T {
    fn convert(self) -> T {
        self.clone()
    }
}

impl<'a, T: Clone> ConvertTo<Option<T>> for &'a T {
    fn convert(self) -> Option<T> {
        Some(self.clone())
    }
}

impl<'a> ConvertTo<String> for &'a str {
    fn convert(self) -> String {
        self.to_owned()
    }
}

impl<'a> ConvertTo<Option<String>> for &'a str {
    fn convert(self) -> Option<String> {
        Some(self.to_owned())
    }
}