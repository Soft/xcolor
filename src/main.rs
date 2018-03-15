extern crate xcb;
extern crate failure;
#[macro_use]
extern crate structopt;

use std::str::FromStr;
use failure::{Error, err_msg};
use xcb::base::{Connection};
use xcb::xproto;
use xcb::base as xbase;
use structopt::StructOpt;

type RGB = (u8, u8, u8);


#[derive(StructOpt)]
struct Args {
    #[structopt(short="f", long="format", help="output format", default_value="hex")]
    format: Format
}

enum Format {
    LowercaseHex,
    UppercaseHex,
    RGB
}

impl FromStr for Format {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "hex" => Ok(Format::LowercaseHex),
            "HEX" => Ok(Format::UppercaseHex),
            "rgb" => Ok(Format::RGB),
            _ => Err(err_msg("Invalid format"))
        }
    }
}

impl Format {
    fn format_color(&self, (r, g, b): RGB) -> String {
        match self {
            &Format::LowercaseHex => format!("#{:02x}{:02x}{:02x}", r, g, b),
            &Format::UppercaseHex => format!("#{:02X}{:02X}{:02X}", r, g, b),
            &Format::RGB => format!("rgb({}, {}, {})", r, g, b),
        }
    }
}

fn wait_for_location(conn: &Connection, root: xproto::Window) -> Result<Option<(i16, i16)>, Error> {
    let reply = xproto::grab_pointer(&conn,
                                     false,
                                     root,
                                     xproto::EVENT_MASK_BUTTON_PRESS as u16,
                                     xproto::GRAB_MODE_ASYNC as u8,
                                     xproto::GRAB_MODE_ASYNC as u8,
                                     xbase::NONE,
                                     xbase::NONE,
                                     xbase::CURRENT_TIME)
        .get_reply()?;
    if reply.status() != xproto::GRAB_STATUS_SUCCESS as u8 {
        return Err(err_msg("Could not grab pointer"));
    }

    let result = loop {
        let event = conn.wait_for_event();
        if let Some(event) = event {
            match event.response_type() {
                xproto::BUTTON_PRESS => {
                    let event: &xproto::ButtonPressEvent = unsafe {
                        xbase::cast_event(&event)
                    };
                    break Some((event.root_x(), event.root_y()));
                },
                _ => {
                    break None;
                }
            }
        } else {
            break None;
        }
    };
    xproto::ungrab_pointer(&conn, xbase::CURRENT_TIME);
    conn.flush();
    Ok(result)
}

fn window_color_at_point(conn: &Connection, window: xproto::Window, (x, y): (i16, i16))
                         -> Result<RGB, Error> {
    let geometry = xproto::get_geometry(conn, window).get_reply()?;
    let reply = xproto::get_image(conn,
                                  xproto::IMAGE_FORMAT_Z_PIXMAP as u8,
                                  window,
                                  geometry.x(),
                                  geometry.y(),
                                  geometry.width(),
                                  geometry.height(),
                                  std::u32::MAX)
        .get_reply()?;
    if reply.depth() != 24 {
        // TODO: Figure out what to do with these
        return Err(err_msg("Unsupported depth"));
    }
    let base = (x as usize * 4) + ((y as usize) * (geometry.width() as usize * 4));
    let data = reply.data();
    let b = data[base + 0];
    let g = data[base + 1];
    let r = data[base + 2];
    Ok((r, g, b))
}

fn run(args: Args) -> Result<(), Error> {
    let (conn, screen) = Connection::connect(None)?;
    let screen = conn.get_setup().roots().nth(screen as usize)
        .ok_or_else(|| err_msg("Could not find screen"))?;
    let root = screen.root();

    if let Some(point) = wait_for_location(&conn, root)? {
        let color = window_color_at_point(&conn, root, point)?;
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
