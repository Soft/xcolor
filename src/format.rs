use std::{iter, fmt};
use std::str::FromStr;
use failure::{Error, err_msg};
use nom::{IError, digit, anychar};

use x11::RGB;

pub struct FormatString(Vec<FormatPart>);

enum Channel {
    R, G, B
}

struct Pad {
    char: char,
    len: u16
}

enum NumberFormat {
    LowercaseHex,
    UppercaseHex,
    Decimal,
    Octal,
    Binary
}

enum FormatPart {
    Literal(String),
    Expansion {
        channel: Channel,
        format: NumberFormat,
        pad: Option<Pad>
    }
}

named!(literal<&str, FormatPart>,
       map!(take_till1_s!(|c: char| c == '%'),
            |s: &str| FormatPart::Literal(s.to_owned())));

named!(channel<&str, Channel>,
       alt_complete!(value!(Channel::R, tag_s!("r")) |
                     value!(Channel::G, tag_s!("g")) |
                     value!(Channel::B, tag_s!("b"))));

named!(format<&str, NumberFormat>,
       alt_complete!(value!(NumberFormat::LowercaseHex, tag_s!("h")) |
                     value!(NumberFormat::UppercaseHex, tag_s!("H")) |
                     value!(NumberFormat::Octal,        tag_s!("o")) |
                     value!(NumberFormat::Binary,       tag_s!("B")) |
                     value!(NumberFormat::Decimal,      tag_s!("d"))));

named!(pad<&str, Pad>,
       do_parse!(char: anychar >> // Should this be more restricted?
                 len: map_res!(digit, FromStr::from_str) >>
                 (Pad { char, len })));

named!(expansion<&str, FormatPart>,
       alt_complete!(
           value!(FormatPart::Literal("%".to_owned()), tag_s!("%%")) |
           do_parse!(tag_s!("%{") >>
                     pad: opt!(pad) >>
                     format: opt!(format) >>
                     channel: channel >>
                     tag_s!("}") >>
                     (FormatPart::Expansion {
                         channel,
                         pad,
                         format: format.unwrap_or(NumberFormat::Decimal)
                     }))));

named!(parse_format_string<&str, FormatString>,
       do_parse!(parts: many0!(alt_complete!(literal | expansion)) >>
                 eof!() >>
                 (FormatString(parts))));

impl FromStr for FormatString {
    type Err = IError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_format_string(s).to_full_result()
    }
}

pub trait FormatColor {
    fn format(&self, color: RGB) -> String;
}

impl Channel {
    fn extract(&self, (r, g, b): RGB) -> u8 {
        match *self {
            Channel::R => r,
            Channel::G => g,
            Channel::B => b
        }
    }
}

impl NumberFormat {
    fn format<T>(&self, value: T) -> String
        where T: fmt::LowerHex + fmt::UpperHex + fmt::Octal + fmt::Binary + fmt::Display {
        match *self {
            NumberFormat::LowercaseHex => format!("{:x}", value),
            NumberFormat::UppercaseHex => format!("{:X}", value),
            NumberFormat::Octal => format!("{:o}", value),
            NumberFormat::Binary => format!("{:b}", value),
            NumberFormat::Decimal => format!("{}", value)
        }
    }
}

impl FormatColor for FormatPart {
    fn format(&self, color: RGB) -> String {
        match self {
            &FormatPart::Literal(ref s) => s.clone(),
            &FormatPart::Expansion { ref channel, ref format, ref pad } => {
                let value = channel.extract(color);
                let base = format.format(value);
                if let Some(Pad { char, len }) = *pad {
                    let base_len = base.chars().count();
                    if let Some(pad_len) = (len as usize).checked_sub(base_len) {
                        let mut padded: String = iter::repeat(char).take(pad_len).collect();
                        padded.push_str(&base);
                        return padded
                    }
                }
                base
            }
        }
    }
}

impl FormatColor for FormatString {
    fn format(&self, color: RGB) -> String {
        self.0.iter().map(|part| part.format(color)).collect()
    }
}

// Formatting Shortcuts

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

impl FormatColor for Format {
    fn format(&self, (r, g, b): RGB) -> String {
        match *self {
            Format::LowercaseHex => format!("#{:02x}{:02x}{:02x}", r, g, b),
            Format::UppercaseHex => format!("#{:02X}{:02X}{:02X}", r, g, b),
            Format::Plain => format!("{};{};{}", r, g, b),
            Format::RGB => format!("rgb({}, {}, {})", r, g, b),
        }
    }
}

// Tests

#[test]
fn test_literal() {
    assert_eq!(literal("foo%bar").unwrap().0, "%bar");
}

#[test]
fn test_pad() {
    assert_eq!(pad("0000").unwrap().1.len, 0);
    assert_eq!(pad("123").unwrap().1.len, 23);
    assert_eq!(pad("001").unwrap().1.len, 1);
    assert!(pad("1").to_full_result().is_err());
    assert!(pad("").to_full_result().is_err());
    assert!(pad("x").to_full_result().is_err());
}

#[test]
fn test_expansion() {
    match expansion("%{r}").unwrap().1 {
        FormatPart::Expansion { channel: Channel::R, .. } => (),
        _ => panic!()
    }

    match expansion("%{04b}").unwrap().1 {
        FormatPart::Expansion { channel: Channel::B, pad: Some(Pad { char: '0', len: 4 }), .. } => (),
        _ => panic!()
    }

    match expansion("%%").unwrap().1 {
        FormatPart::Literal(ref s) if s == "%" => (),
        _ => panic!()
    }
}

#[test]
fn test_format_color() {
    let string: FormatString = "rgb(%{r}, %{g}, %{b})".parse().unwrap();
    assert_eq!(string.format((0, 255, 0)), "rgb(0, 255, 0)");

    let string: Result<FormatString, _> = "".parse();
    assert!(string.is_ok());

    let should_err = vec!["%{}", "%}", "%{gg}", "%%%{-a}", "%a{}", "%foo"];
    for case in should_err {
        assert!(case.parse::<FormatString>().is_err());
    }
}


