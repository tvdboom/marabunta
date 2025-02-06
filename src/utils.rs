use std::fmt::Debug;

/// Trait to get the text of an enum variant
pub trait NameFromEnum {
    fn as_string(&self) -> String;
}

impl<T: Debug> NameFromEnum for T {
    fn as_string(&self) -> String {
        format!("{:?}", self)
    }
}
