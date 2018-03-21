extern crate xcb;
extern crate failure;
extern crate clap;
#[macro_use]
extern crate nom;

mod format;
mod x11;

use failure::{Error, err_msg};
use xcb::base::Connection;
use clap::{App, Arg, ArgMatches};

use format::{Format, FormatString, FormatColor};

fn get_args() -> ArgMatches<'static> {
    App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(Arg::with_name("format")
             .short("f")
             .long("format")
             .takes_value(true)
             .value_name("NAME")
             .help("Output format (defaults to hex)")
             .possible_values(&["hex", "HEX", "plain", "rgb"])
             .conflicts_with("custom"))
        .arg(Arg::with_name("custom")
             .short("c")
             .long("custom")
             .takes_value(true)
             .value_name("FORMAT")
             .help("Custom output format")
             .conflicts_with("format"))
        .get_matches()
}

fn run<'a>(args: ArgMatches<'a>) -> Result<(), Error> {
    fn error(message: &str) -> ! {
        clap::Error::with_description(message, clap::ErrorKind::InvalidValue)
            .exit()
    }
    let formatter = if let Some(custom) = args.value_of("custom") {
        Box::new(custom.parse::<FormatString>()
                 .unwrap_or_else(|_| error("invalid format string")))
            as Box<FormatColor>

    } else {
        Box::new(args.value_of("format")
                 .unwrap_or("hex")
                 .parse::<Format>()
                 .unwrap_or_else(|e| error(&format!("{}", e))))
            as Box<FormatColor>
    };
    let (conn, screen) = Connection::connect(None)?;
    let screen = conn.get_setup().roots().nth(screen as usize)
        .ok_or_else(|| err_msg("Could not find screen"))?;
    let root = screen.root();

    if let Some(point) = x11::wait_for_location(&conn, root)? {
        let color = x11::window_color_at_point(&conn, root, point)?;
        println!("{}", formatter.format(color));
    }
    Ok(())
}

fn main() {
    let args = get_args();
    if let Err(err) = run(args) {
        eprintln!("error: {}", err);
        std::process::exit(1);
    }
}
