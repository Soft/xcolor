extern crate xcb;
extern crate failure;
extern crate clap;

use failure::{Error, err_msg};
use xcb::base::{Connection};
use xcb::xproto;
use xcb::base as xbase;

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
    Ok(result)
}

fn run() -> Result<(), Error> {
    let (conn, screen) = Connection::connect(None)?;
    let screen = conn.get_setup().roots().nth(screen as usize)
        .ok_or_else(|| err_msg("Could not find screen"))?;
    let root = screen.root();

    if let Some((x, y)) = wait_for_location(&conn, root)? {
        println!("{}, {}", x, y);
    }

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {}", err);
        std::process::exit(1);
    }
}
