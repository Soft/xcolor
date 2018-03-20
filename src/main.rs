extern crate xcb;
extern crate failure;
#[macro_use]
extern crate structopt;

mod format;
mod x11;

use failure::{Error, err_msg};
use xcb::base::Connection;
use structopt::StructOpt;

use format::Format;

#[derive(StructOpt)]
struct Args {
    #[structopt(short="f", long="format", help="output format", default_value="hex")]
    format: Format
}

fn run(args: Args) -> Result<(), Error> {
    let (conn, screen) = Connection::connect(None)?;
    let screen = conn.get_setup().roots().nth(screen as usize)
        .ok_or_else(|| err_msg("Could not find screen"))?;
    let root = screen.root();

    if let Some(point) = x11::wait_for_location(&conn, root)? {
        let color = x11::window_color_at_point(&conn, root, point)?;
        println!("{}", args.format.format_color(color));
    }
    Ok(())
}

fn main() {
    let args = Args::from_args();
    if let Err(err) = run(args) {
        eprintln!("error: {}", err);
        std::process::exit(1);
    }
}
