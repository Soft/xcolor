mod atoms;
mod cli;
mod color;
mod draw;
mod format;
mod location;
mod pixel;
mod selection;
mod util;

use anyhow::{anyhow, Result};
use atty::Stream;
use clap::{value_t, ArgMatches, ErrorKind};
use nix::unistd::ForkResult;
use xcb::base::Connection;

use crate::cli::get_cli;
use crate::format::{Format, FormatColor, FormatString};
use crate::location::wait_for_location;
use crate::selection::{into_daemon, set_selection, Selection};

const DEFAULT_PREVIEW_SIZE: u32 = 256 - 1;
const DEFAULT_SCALE: u32 = 8;

fn run(args: &ArgMatches) -> Result<()> {
    fn error(message: &str) -> ! {
        clap::Error::with_description(message, clap::ErrorKind::InvalidValue).exit()
    }

    let custom_format;
    let simple_format;
    let formatter: &dyn FormatColor = if let Some(custom) = args.value_of("custom") {
        custom_format = custom
            .parse::<FormatString>()
            .unwrap_or_else(|_| error("Invalid format string"));
        &custom_format
    } else {
        simple_format = args
            .value_of("format")
            .unwrap_or("hex")
            .parse::<Format>()
            .unwrap_or_else(|e| error(&format!("{}", e)));
        &simple_format
    };

    let scale = value_t!(args.value_of("scale"), u32).unwrap_or_else(|e| match e.kind {
        ErrorKind::ArgumentNotFound => DEFAULT_SCALE,
        _ => error(&format!("{}", e)),
    });
    let preview_size =
        value_t!(args.value_of("preview_size"), u32).unwrap_or_else(|e| match e.kind {
            ErrorKind::ArgumentNotFound => DEFAULT_PREVIEW_SIZE,
            _ => error(&format!("{}", e)),
        });

    let selection = args.values_of("selection").and_then(|mut v| {
        v.next()
            .map_or(Some(Selection::Clipboard), |v| v.parse::<Selection>().ok())
    });
    let use_selection = selection.is_some();
    let background = std::env::var("XCOLOR_FOREGROUND").is_err();

    let mut in_parent = true;

    let (conn, screen) = Connection::connect_with_xlib_display()?;

    {
        let screen = conn
            .get_setup()
            .roots()
            .nth(screen as usize)
            .ok_or_else(|| anyhow!("Could not find screen"))?;
        let root = screen.root();

        if let Some(color) = wait_for_location(&conn, &screen, preview_size, scale)? {
            let output = formatter.format(color);

            if use_selection {
                if background {
                    in_parent = match into_daemon()? {
                        ForkResult::Parent { .. } => true,
                        ForkResult::Child => false,
                    }
                }

                if !(background && in_parent) {
                    set_selection(&conn, root, &selection.unwrap(), &output)?;
                }
            } else {
                let trailing_newline = atty::is(Stream::Stdout).then_some("\n").unwrap_or("");
                print!("{}{}", output, trailing_newline);
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
