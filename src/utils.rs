use regex::Regex;
use std::fmt::Debug;

/// Trait to get the text of an enum variant
pub trait NameFromEnum {
    fn as_string(&self) -> String;
}

impl<T: Debug> NameFromEnum for T {
    fn as_string(&self) -> String {
        let text = format!("{:?}", self);

        let re = Regex::new(r"([a-z])([A-Z])").unwrap();
        let spaced = re.replace_all(&text, "$1 $2");
        let mut result = spaced.to_lowercase().to_string();

        // Capitalize only the first letter
        result.replace_range(0..1, &result[0..1].to_uppercase());

        result
    }
}
