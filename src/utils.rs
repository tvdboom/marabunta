use regex::Regex;
use std::fmt::Debug;
use std::net::{IpAddr, UdpSocket};

/// Get the local IP address
#[cfg(not(target_arch = "wasm32"))]
pub fn get_local_ip() -> IpAddr {
    let socket = UdpSocket::bind("0.0.0.0:0")
        .ok()
        .expect("Socket not found.");
    socket
        .connect("8.8.8.8:80")
        .ok()
        .expect("Failed to connect to socket."); // Doesn't send data
    socket.local_addr().ok().map(|addr| addr.ip()).unwrap()
}

#[cfg(target_arch = "wasm32")]
pub fn get_local_ip() -> IpAddr {
    // WebAssembly in browsers cannot access local network interfaces
    "127.0.0.1".parse().unwrap()
}

/// Helper function to extract only the variant name (removes tuple/struct fields)
fn extract_variant_name(text: String) -> String {
    text.split_once('(')
        .or_else(|| text.split_once('{'))
        .map(|(variant, _)| variant)
        .unwrap_or(&text)
        .trim_matches(&['"', ' '][..])
        .to_string()
}

/// Trait to get the text of an enum variant
pub trait NameFromEnum {
    fn to_name(&self) -> String;
    fn to_lowername(&self) -> String;
    fn to_title(&self) -> String;
    fn to_snake(&self) -> String;
}

impl<T: Debug> NameFromEnum for T {
    fn to_name(&self) -> String {
        let re = Regex::new(r"([a-z])([A-Z])").unwrap();

        let text = extract_variant_name(format!("{:?}", self));
        re.replace_all(&text, "$1 $2").to_string()
    }

    fn to_lowername(&self) -> String {
        self.to_name().to_lowercase()
    }

    fn to_title(&self) -> String {
        let mut name = self.to_lowername();

        // Capitalize only the first letter
        name.replace_range(0..1, &name[0..1].to_uppercase());

        name
    }

    fn to_snake(&self) -> String {
        let re = Regex::new(r"([a-z])([A-Z])").unwrap();

        let text = extract_variant_name(format!("{:?}", self));
        re.replace_all(&text, "${1}_${2}").to_lowercase()
    }
}
