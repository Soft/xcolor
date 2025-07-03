use std::num::ParseIntError;
use std::str::FromStr;
use std::{fmt, iter};

use nom::branch::alt;
use nom::bytes::complete::{tag, take_till1};
use nom::character::complete::{anychar, digit1};
use nom::combinator::{all_consuming, complete, map, map_res, opt, value};
use nom::error::{FromExternalError, ParseError};
use nom::multi::many0;
use nom::sequence::{preceded, terminated, tuple};
use nom::IResult;

use anyhow::{anyhow, Error, Result};

use crate::color::ARGB;

pub struct FormatString(Vec<FormatPart>);

#[derive(Clone, Copy)]
enum Channel {
    R,
    G,
    B,
}

struct Pad {
    char: char,
    len: u16,
}

#[derive(Clone, Copy)]
enum NumberFormat {
    LowercaseHex,
    UppercaseHex,
    Decimal,
    Octal,
    Binary,
}

enum FormatPart {
    Literal(String),
    Expansion {
        channel: Channel,
        format: NumberFormat,
        pad: Option<Pad>,
    },
}

fn literal<'a, E>(input: &'a str) -> IResult<&'a str, FormatPart, E>
where
    E: ParseError<&'a str>,
{
    map(take_till1(|c| c == '%'), |s: &str| {
        FormatPart::Literal(s.to_owned())
    })(input)
}

fn channel<'a, E>(input: &'a str) -> IResult<&'a str, Channel, E>
where
    E: ParseError<&'a str>,
{
    alt((
        value(Channel::R, tag("r")),
        value(Channel::G, tag("g")),
        value(Channel::B, tag("b")),
    ))(input)
}

fn format<'a, E>(input: &'a str) -> IResult<&'a str, NumberFormat, E>
where
    E: ParseError<&'a str>,
{
    alt((
        value(NumberFormat::LowercaseHex, tag("h")),
        value(NumberFormat::UppercaseHex, tag("H")),
        value(NumberFormat::Octal, tag("o")),
        value(NumberFormat::Binary, tag("B")),
        value(NumberFormat::Decimal, tag("d")),
    ))(input)
}

fn pad<'a, E>(input: &'a str) -> IResult<&'a str, Pad, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    let digit = map_res(digit1, |s: &str| s.parse::<u16>());
    map(tuple((anychar, digit)), |(char, len)| Pad { char, len })(input)
}

fn expansion<'a, E>(input: &'a str) -> IResult<&'a str, FormatPart, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    let escape = map(tag("%%"), |_| FormatPart::Literal("%".to_owned()));
    let inner = complete(map(
        tuple((opt(pad), opt(format), channel)),
        |(pad, format, channel)| FormatPart::Expansion {
            channel,
            pad,
            format: format.unwrap_or(NumberFormat::Decimal),
        },
    ));
    let expansion = preceded(tag("%{"), terminated(inner, tag("}")));
    alt((escape, expansion))(input)
}

fn parse_format_string<'a, E>(input: &'a str) -> IResult<&'a str, FormatString, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    map(
        all_consuming(many0(alt((literal, expansion)))),
        FormatString,
    )(input)
}

impl FromStr for FormatString {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_format_string::<()>(s)
            .map(|(_, result)| result)
            .map_err(|_| anyhow!("Invalid format string"))
    }
}

pub trait FormatColor {
    fn format(&self, color: ARGB) -> String;
}

impl Channel {
    fn extract(&self, color: ARGB) -> u8 {
        match self {
            Channel::R => color.r,
            Channel::G => color.g,
            Channel::B => color.b,
        }
    }
}

impl NumberFormat {
    fn format<T>(&self, value: T) -> String
    where
        T: fmt::LowerHex + fmt::UpperHex + fmt::Octal + fmt::Binary + fmt::Display,
    {
        match self {
            NumberFormat::LowercaseHex => format!("{:x}", value),
            NumberFormat::UppercaseHex => format!("{:X}", value),
            NumberFormat::Octal => format!("{:o}", value),
            NumberFormat::Binary => format!("{:b}", value),
            NumberFormat::Decimal => format!("{}", value),
        }
    }
}

impl FormatColor for FormatPart {
    fn format(&self, color: ARGB) -> String {
        match self {
            FormatPart::Literal(s) => s.clone(),
            FormatPart::Expansion {
                channel,
                format,
                pad,
            } => {
                let value = channel.extract(color);
                let base = format.format(value);
                if let Some(Pad { char, len }) = *pad {
                    let base_len = base.chars().count();
                    if let Some(pad_len) = (len as usize).checked_sub(base_len) {
                        let mut padded: String = iter::repeat(char).take(pad_len).collect();
                        padded.push_str(&base);
                        return padded;
                    }
                }
                base
            }
        }
    }
}

impl FormatColor for FormatString {
    fn format(&self, color: ARGB) -> String {
        self.0.iter().map(|part| part.format(color)).collect()
    }
}

// Formatting Shortcuts

#[derive(PartialEq)]
pub enum HexCompaction {
    Compact,
    Full,
}

pub enum Format {
    LowercaseHex(HexCompaction),
    UppercaseHex(HexCompaction),
    Plain,
    RGB,
}

impl FromStr for Format {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "hex" => Ok(Format::LowercaseHex(HexCompaction::Full)),
            "HEX" => Ok(Format::UppercaseHex(HexCompaction::Full)),
            "hex!" => Ok(Format::LowercaseHex(HexCompaction::Compact)),
            "HEX!" => Ok(Format::UppercaseHex(HexCompaction::Compact)),
            "plain" => Ok(Format::Plain),
            "rgb" => Ok(Format::RGB),
            _ => Err(anyhow!("Invalid format")),
        }
    }
}

impl FormatColor for Format {
    fn format(&self, color: ARGB) -> String {
        match self {
            Format::LowercaseHex(comp) => {
                if *comp == HexCompaction::Compact && color.is_compactable() {
                    format!("#{:x}{:x}{:x}", color.r & 0xf, color.g & 0xf, color.b & 0xf)
                } else {
                    format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b)
                }
            }
            Format::UppercaseHex(comp) => {
                if *comp == HexCompaction::Compact && color.is_compactable() {
                    format!("#{:X}{:X}{:X}", color.r & 0xf, color.g & 0xf, color.b & 0xf)
                } else {
                    format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b)
                }
            }
            Format::Plain => format!("{};{};{}", color.r, color.g, color.b),
            Format::RGB => format!("rgb({}, {}, {})", color.r, color.g, color.b),
        }
    }
}

// Tests

#[test]
fn test_literal() {
    assert_eq!(literal::<()>("foo%bar").unwrap().0, "%bar");
}

#[test]
fn test_pad() {
    assert_eq!(pad::<()>("#0").unwrap().1.char, '#');
    assert_eq!(pad::<()>("-10").unwrap().1.char, '-');
    assert_eq!(pad::<()>("-10").unwrap().1.len, 10);
    assert_eq!(pad::<()>("0000").unwrap().1.len, 0);
    assert_eq!(pad::<()>("123").unwrap().1.len, 23);
    assert_eq!(pad::<()>("001").unwrap().1.len, 1);
    assert_eq!(pad::<()>("065535").unwrap().1.len, 65535);
    assert!(pad::<()>("065536").is_err());
    assert!(pad::<()>("1").is_err());
    assert!(pad::<()>("").is_err());
    assert!(pad::<()>("x").is_err());
}

#[test]
fn test_expansion() {
    match expansion::<()>("%{r}").unwrap().1 {
        FormatPart::Expansion {
            channel: Channel::R,
            ..
        } => (),
        _ => panic!(),
    }

    match expansion::<()>("%{04b}").unwrap().1 {
        FormatPart::Expansion {
            channel: Channel::B,
            pad: Some(Pad { char: '0', len: 4 }),
            ..
        } => (),
        _ => panic!(),
    }

    match expansion::<()>("%%").unwrap().1 {
        FormatPart::Literal(ref s) if s == "%" => (),
        _ => panic!(),
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
    assert_eq!(fmt.format(ARGB::new(0xff, 255, 0, 255)), "#ff00ff");

    let fmt: FormatString = "#%{02Hr}%{02Hg}%{02Hb}".parse().unwrap();
    assert_eq!(fmt.format(ARGB::new(0xff, 0, 255, 0)), "#00FF00");

    let fmt: FormatString = "rgb(%{r}, %{g}, %{b})".parse().unwrap();
    assert_eq!(
        fmt.format(ARGB::new(0xff, 255, 255, 255)),
        "rgb(255, 255, 255)"
    );

    let fmt: FormatString = "%{r};%{g};%{b}".parse().unwrap();
    assert_eq!(fmt.format(ARGB::new(0xff, 0, 0, 0)), "0;0;0");

    let fmt: FormatString = "%{r}, %{g}, %{b}".parse().unwrap();
    assert_eq!(fmt.format(ARGB::new(0xff, 0, 0, 0)), "0, 0, 0");

    let fmt: FormatString = "Green: %{-4g}".parse().unwrap();
    assert_eq!(fmt.format(ARGB::new(0xff, 0, 7, 0)), "Green: ---7");

    let fmt: FormatString = "%{016Br}".parse().unwrap();
    assert_eq!(fmt.format(ARGB::new(0xff, 3, 0, 0)), "0000000000000011");
}
