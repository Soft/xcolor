extern crate xcb;
extern crate failure;
extern crate clap;
extern crate nix;
#[macro_use]
extern crate nom;

mod format;
mod x11;
mod selection;

use failure::{Error, err_msg};
use xcb::base::Connection;
use clap::{App, Arg, ArgMatches};
use nix::unistd::ForkResult;

use format::{Format, FormatString, FormatColor};
use selection::{Selection, daemonize, set_selection};

fn get_cli() -> App<'static, 'static> {
    let mut app = App::new(env!("CARGO_PKG_NAME"))
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
        .arg(Arg::with_name("selection")
             .short("s")
             .long("selection")
             .takes_value(true)
             .value_name("SELECTION")
             .min_values(0)
             .max_values(1)
             .possible_values(&["primary", "secondary"])
             .help("Output to selection (defaults to primary)"));

    if cfg!(debug_assertions) {
        app = app.arg(Arg::with_name("foreground")
                .short("F")
                .long("foreground")
                .requires("selection")
                .help("Stay in the foreground"));
    }

    app
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
    let selection = args.values_of("selection")
        .and_then(|mut v| v.next().map_or(Some(Selection::Primary),
                                          |v| v.parse::<Selection>().ok()));
    let use_selection = selection.is_some();
    let background = !args.is_present("foreground");
    let mut in_parent = true;

    let (conn, screen) = Connection::connect(None)?;

    {
        let screen = conn.get_setup().roots().nth(screen as usize)
            .ok_or_else(|| err_msg("Could not find screen"))?;
        let root = screen.root();

        if let Some(point) = x11::wait_for_location(&conn, root)? {
            let color = x11::window_color_at_point(&conn, root, point)?;
            let output = formatter.format(color);
            if use_selection {
                if background {
                    in_parent = match daemonize()? {
                        ForkResult::Parent { .. } => true,
                        ForkResult::Child => false
                    }
                }

                if !(background && in_parent) {
                    set_selection(&conn, root, selection.unwrap(), &output)?;
                }
            } else {
                println!("{}", output);
            }
        }
    }

    if background && in_parent {
        std::mem::forget(conn);
    }

    Ok(())
}

fn main() {
    let args = get_cli().get_matches();
    if let Err(err) = run(args) {
        eprintln!("error: {}", err);
        std::process::exit(1);
    }
}
