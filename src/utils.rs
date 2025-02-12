use regex::Regex;
use std::fmt::Debug;
use std::time::Duration;

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

/// Scale a Duration by a factor
pub fn scale_duration(duration: Duration, scale: f32) -> Duration {
    let sec = (duration.as_secs() as f32 + duration.subsec_nanos() as f32 * 1e-9) * scale;
    Duration::new(sec.trunc() as u64, (sec.fract() * 1e9) as u32)
}
