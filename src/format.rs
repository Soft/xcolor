use std::str::FromStr;
use failure::{Error, err_msg};

use x11::RGB;

pub enum Format {
    LowercaseHex,
    UppercaseHex,
    Plain,
    RGB
}

impl FromStr for Format {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "hex" => Ok(Format::LowercaseHex),
            "HEX" => Ok(Format::UppercaseHex),
            "plain" => Ok(Format::Plain),
            "rgb" => Ok(Format::RGB),
            _ => Err(err_msg("Invalid format"))
        }
    }
}

impl Format {
    pub fn format_color(&self, (r, g, b): RGB) -> String {
        match self {
            &Format::LowercaseHex => format!("#{:02x}{:02x}{:02x}", r, g, b),
            &Format::UppercaseHex => format!("#{:02X}{:02X}{:02X}", r, g, b),
            &Format::Plain => format!("{};{};{}", r, g, b),
            &Format::RGB => format!("rgb({}, {}, {})", r, g, b),
        }
    }
}

