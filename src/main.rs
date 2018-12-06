mod format;
mod preview;
mod selection;
mod location;
mod cli;
mod color;
mod atoms;

use failure::{Error, err_msg};
use xcb::base::Connection;
use clap::ArgMatches;
use nix::unistd::ForkResult;

use crate::format::{Format, FormatString, FormatColor};
use crate::selection::{Selection, into_daemon, set_selection};
use crate::cli::get_cli;
use crate::location::wait_for_location;
use crate::color::window_color_at_point;
use crate::preview::Preview;

fn run(args: &ArgMatches) -> Result<(), Error> {
    fn error(message: &str) -> ! {
        clap::Error::with_description(message, clap::ErrorKind::InvalidValue)
            .exit()
    }

    let custom_format;
    let simple_format;
    let formatter: &FormatColor = if let Some(custom) = args.value_of("custom") {
        custom_format = custom.parse::<FormatString>()
            .unwrap_or_else(|_| error("Invalid format string"));
        &custom_format

    } else {
        simple_format = args.value_of("format")
            .unwrap_or("hex")
            .parse::<Format>()
            .unwrap_or_else(|e| error(&format!("{}", e)));
        &simple_format
    };

    let selection = args.values_of("selection")
        .and_then(|mut v| v.next().map_or(Some(Selection::Primary),
                                          |v| v.parse::<Selection>().ok()));
    let use_selection = selection.is_some();
    let background = std::env::var("XCOLOR_FOREGROUND").is_err();

    let use_preview = !args.is_present("no_preview");
    let use_shape = std::env::var("XCOLOR_DISABLE_SHAPE").is_err();

    let mut in_parent = true;

    let (conn, screen) = Connection::connect(None)?;

    {
        let screen = conn.get_setup().roots().nth(screen as usize)
            .ok_or_else(|| err_msg("Could not find screen"))?;
        let root = screen.root();

        let point = if use_preview {
            let mut preview = Preview::create(&conn, &screen, use_shape)?;
            wait_for_location(&conn, &screen, |event| preview.handle_event(event))?
        } else {
            wait_for_location(&conn, &screen, |_| Ok(true))?
        };

        if let Some(point) = point {
            let color = window_color_at_point(&conn, root, point)?;
            let output = formatter.format(color);

            if use_selection {
                if background {
                    in_parent = match into_daemon()? {
                        ForkResult::Parent { .. } => true,
                        ForkResult::Child => false
                    }
                }

                if !(background && in_parent) {
                    set_selection(&conn, root, &selection.unwrap(), &output)?;
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
    if let Err(err) = run(&args) {
        eprintln!("error: {}", err);
        std::process::exit(1);
    }
}
