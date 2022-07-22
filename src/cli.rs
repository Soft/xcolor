use clap::{App, AppSettings, Arg};

pub fn get_cli() -> App<'static, 'static> {
    App::new(env!("CARGO_PKG_NAME"))
        .setting(AppSettings::ColoredHelp)
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("format")
                .short("f")
                .long("format")
                .takes_value(true)
                .value_name("NAME")
                .help("Output format (defaults to hex)")
                .possible_values(&["hex", "HEX", "hex!", "HEX!", "plain", "rgb"])
                .conflicts_with("custom"),
        )
        .arg(
            Arg::with_name("custom")
                .short("c")
                .long("custom")
                .takes_value(true)
                .value_name("FORMAT")
                .help("Custom output format")
                .conflicts_with("format"),
        )
        .arg(
            Arg::with_name("selection")
                .short("s")
                .long("selection")
                .takes_value(true)
                .value_name("SELECTION")
                .min_values(0)
                .max_values(1)
                .possible_values(&["primary", "secondary", "clipboard"])
                .help("Output to selection (defaults to clipboard)"),
        )
        .arg(
            Arg::with_name("scale")
                .short("S")
                .long("scale")
                .takes_value(true)
                .value_name("SCALE")
                .help("Scale of magnification (defaults to 8)"),
        )
        .arg(
            Arg::with_name("preview_size")
                .short("P")
                .long("preview-size")
                .takes_value(true)
                .value_name("PREVIEW_SIZE")
                .help("Size of preview, must be odd (defaults to 255)"),
        )
        .arg(
            Arg::with_name("position")
                .short("p")
                .long("position")
                .required(false)
                .takes_value(false)
                // .value_name("POSITION")
                .help("Should xcolor also print out the position of the pointer"),
        )
}
