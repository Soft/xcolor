# xcolor 🌈

[![Build Status](https://api.travis-ci.org/Soft/xcolor.svg?branch=master)](https://travis-ci.org/Soft/xcolor)
[![Latest Version](https://img.shields.io/crates/v/xcolor.svg)](https://crates.io/crates/xcolor)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/xcolor)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Lightweight color picker for X11. Use mouse cursor to select colors visible
anywhere on the screen to view their RGB representation.

### Installation

Currently, the easiest way to install xcolor is to use
[cargo](https://doc.rust-lang.org/stable/cargo/):

``` shell
$ cargo install xcolor
```

### Usage

Simply invoke the `xcolor` command to select a color. The selected color will be
printed to the standard output. By default, color values will be printed in
lowercase hexadecimal format. The output format can be changed using the `-f
FORMAT` switch. The possible format values are listed bellow:

| Format Specifier | Description                       | Example             |
| ---------------- | --------------------------------- | ------------------- |
| `hex`            | Lowercase hexadecimal (default)   | #ff00ff             |
| `HEX`            | Uppercase hexadecimal             | #00FF00             |
| `rgb`            | Decimal RGB                       | rgb(255, 255, 255)  |
| `plain`          | Decimal with semicolon separators | 0;0;0               |
