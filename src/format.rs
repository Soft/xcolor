use std::{iter, fmt};
use std::str::FromStr;
use failure::{Error, err_msg};
use nom::*;

use crate::color::RGB;

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
    fn extract(&self, color: RGB) -> u8 {
        match self {
            Channel::R => color.r,
            Channel::G => color.g,
            Channel::B => color.b
        }
    }
}

impl NumberFormat {
    fn format<T>(&self, value: T) -> String
        where T: fmt::LowerHex + fmt::UpperHex + fmt::Octal + fmt::Binary + fmt::Display {
        match self {
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
            FormatPart::Literal(s) => s.clone(),
            FormatPart::Expansion { channel, format, pad } => {
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

#[derive(PartialEq)]
pub enum HexCompaction {
    Compact,
    Full
}

pub enum Format {
    LowercaseHex(HexCompaction),
    UppercaseHex(HexCompaction),
    Plain,
    RGB
}

impl FromStr for Format {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "hex" => Ok(Format::LowercaseHex(HexCompaction::Full)),
            "HEX" => Ok(Format::UppercaseHex(HexCompaction::Full)),
            "hex!" => Ok(Format::LowercaseHex(HexCompaction::Compact)),
            "HEX!" => Ok(Format::UppercaseHex(HexCompaction::Compact)),
            "plain" => Ok(Format::Plain),
            "rgb" => Ok(Format::RGB),
            _ => Err(err_msg("Invalid format"))
        }
    }
}

impl FormatColor for Format {
    fn format(&self, color: RGB) -> String {
        match self {
           Format::LowercaseHex(comp) => {
               if *comp == HexCompaction::Compact && color.is_compactable() {
                   format!("#{:x}{:x}{:x}", color.r & 0xf, color.g & 0xf, color.b & 0xf)
               } else {
                   format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b)
               }
            },
           Format::UppercaseHex(comp) => {
               if *comp == HexCompaction::Compact && color.is_compactable() {
                   format!("#{:X}{:X}{:X}", color.r & 0xf, color.g & 0xf, color.b & 0xf)
               } else {
                   format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b)
               }
            },
            Format::Plain => format!("{};{};{}", color.r, color.g, color.b),
            Format::RGB => format!("rgb({}, {}, {})", color.r, color.g, color.b),
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
    let string: Result<FormatString, _> = "".parse();
    assert!(string.is_ok());

    let should_err = vec!["%{}", "%}", "%{gg}", "%%%{-a}", "%a{}", "%foo"];
    for case in should_err {
        assert!(case.parse::<FormatString>().is_err());
    }
}

#[test]
fn test_examples_from_readme() {
    let fmt: FormatString = "#%{02hr}%{02hg}%{02hb}".parse().unwrap();
    assert_eq!(fmt.format(RGB::new(255, 0, 255)), "#ff00ff");

    let fmt: FormatString = "#%{02Hr}%{02Hg}%{02Hb}".parse().unwrap();
    assert_eq!(fmt.format(RGB::new(0, 255, 0)), "#00FF00");

    let fmt: FormatString = "rgb(%{r}, %{g}, %{b})".parse().unwrap();
    assert_eq!(fmt.format(RGB::new(255, 255, 255)), "rgb(255, 255, 255)");

    let fmt: FormatString = "%{r};%{g};%{b}".parse().unwrap();
    assert_eq!(fmt.format(RGB::new(0, 0, 0)), "0;0;0");

    let fmt: FormatString = "%{r}, %{g}, %{b}".parse().unwrap();
    assert_eq!(fmt.format(RGB::new(0, 0, 0)), "0, 0, 0");

    let fmt: FormatString = "Green: %{-4g}".parse().unwrap();
    assert_eq!(fmt.format(RGB::new(0, 7, 0)), "Green: ---7");

    let fmt: FormatString = "%{016Br}".parse().unwrap();
    assert_eq!(fmt.format(RGB::new(3, 0, 0)), "0000000000000011");
}


