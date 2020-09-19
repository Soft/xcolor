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
            Arg::with_name("no_preview")
                .short("n")
                .long("no-preview")
                .help("Disable preview popup"),
        )
}
