# xcolor 🌈

[![Build Status](https://api.travis-ci.org/Soft/xcolor.svg?branch=master)](https://travis-ci.org/Soft/xcolor)
[![Latest Version](https://img.shields.io/crates/v/xcolor.svg)](https://crates.io/crates/xcolor)
[![GitHub release](https://img.shields.io/github/release/Soft/xcolor.svg)](https://github.com/Soft/xcolor/releases)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/crate/xcolor)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Lightweight color picker for X11. Use mouse cursor to select colors visible
anywhere on the screen to view their RGB representation.

### Installation

There are statically linked release binaries available on the [GitHub releases
page](https://github.com/Soft/xcolor/releases). These binaries should work on
most recent Linux systems without any additional dependencies or configuration.

Alternatively, `xcolor` can be easily installed from the source using
[cargo](https://doc.rust-lang.org/stable/cargo/):

``` shell
$ cargo install xcolor
```

Building and running `xcolor` requires [xcb](https://xcb.freedesktop.org)
libraries to be present. To get the latest development version of `xcolor`, you
can direct cargo to install from the git repository:

``` shell
$ cargo install --git 'https://github.com/Soft/xcolor.git'
```

### Usage

Simply invoke the `xcolor` command to select a color. The selected color will be
printed to the standard output. 

``` text
xcolor 0.3.0
Samuel Laurén <samuel.lauren@iki.fi>
Lightweight color picker for X11

USAGE:
    xcolor [FLAGS] [OPTIONS]

FLAGS:
    -h, --help          Prints help information
    -V, --version       Prints version information

OPTIONS:
    -c, --custom <FORMAT>          Custom output format
    -f, --format <NAME>            Output format (defaults to hex) [possible values: hex, HEX, hex!, HEX!, plain, rgb]
    -s, --selection <SELECTION>    Output to selection (defaults to primary) [possible values: primary, secondary]
```

### Saving to Selection

By default, the selected color is printed to the standard output. By specifying
the `-s` flag, `xcolor` can be instructed to instead save the color to X11's
selection. The selection to use can be specified as an argument. Possible
selection values are `primary` (the default) and `secondary`.

Because of the way selections work in X11, `xcolor` forks into background when
`-s` mode is used. This behavior can be disabled by defining `XCOLOR_FOREGROUND`
environment variable.

### Formatting

By default, the color values will be printed in lowercase hexadecimal format.
The output format can be changed using the `-f FORMAT` switch. The possible
format values are listed bellow:

| Format Specifier | Description                               | Example               | Custom Format Equivalent |
| ---------------- | ---------------------------------         | --------------------- | ------------------------ |
| `hex`            | Lowercase hexadecimal (default)           | `#ff00ff`             | `#%{02hr}%{02hg}%{02hb}` |
| `HEX`            | Uppercase hexadecimal                     | `#00FF00`             | `#%{02Hr}%{02Hg}%{02Hb}` |
| `hex!`           | Compact lowercase hexadecimal<sup>1</sup> | `#fff`                | Not expressible          |
| `HEX!`           | Compact uppercase hexadecimal<sup>1</sup> | `#F0F`                | Not expressible          |
| `rgb`            | Decimal RGB                               | `rgb(255, 255, 255)`  | `rgb(%{r}, %{g}, %{b})`  |
| `plain`          | Decimal with semicolon separators         | `0;0;0`               | `%{r};%{g};%{b}`         |

**1:** The compact form refers to CSS three-letter color codes as specified by [CSS
Color Module Level 3](https://www.w3.org/TR/2018/PR-css-color-3-20180315/#rgb-color).
If the color is not expressible in three-letter form, the regular six-letter
form will be used.

### Custom Formatting

The `-f` switch provides quick access to some commonly used formatting options.
However, if custom output formatting is desired, this can be achieved using the
`-c FORMAT` switch. The `FORMAT` parameter specifies a template for the output
and supports a simple template language.

`FORMAT` templates can contain special expansions that are written inside
`%{...}` blocks. These blocks will be expanded into color values according to
the specifiers defined inside the block. Here are some examples of valid format
strings and what they might translate to:

| Format String            | Example Output     |
| ------------------------ | ------------------ |
| `%{r}, %{g}, %{b}`       | `255, 0, 100`      |
| `Green: %{-4g}`          | `Green: ---7`      |
| `#%{02hr}%{02hg}%{02hb}` | `#00ff00`          |
| `%{016Br}`               | `0000000000000011` |

Expansion blocks in format strings always contain a channel specifier (`r` for
red, `g` for green, and `b` for blue). Additionally, they can contain an
optional number format specifier (`h` for lowercase hexadecimal, `H` for
uppercase hexadecimal, `o` for octal, `B` for binary, and `d` for decimal) and
an optional padding specifier consisting of a character to use for padding and
the length the string should be padded to. We can use these rules to decode the
above example string:

``` text
  %{016Br}
    | |||
    | ||`- Channel (red)
    | |`-- Number format specifier (binary)
    | `--- Padding length (16)
    `----- Character to use for padding (0)
```

In the output, we get the contents of the red color channel formatted in binary
and padded with zeroes to be sixteen characters long.

