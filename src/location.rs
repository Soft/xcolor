use std;

use failure::{Error, err_msg};
use xcb::base as xbase;
use xcb::base::Connection;
use xcb::xproto;
use xcb::xproto::Screen;

// Left mouse button
const SELECTION_BUTTON: xproto::Button = 1;

pub fn wait_for_location<F>(conn: &Connection, screen: &Screen, mut handler: F)
                            -> Result<Option<(i16, i16)>, Error>
    where F: FnMut(&xbase::GenericEvent) -> Result<bool, Error> {
    const XC_CROSSHAIR: u16 = 34;

    let root = screen.root();
    let cursor_font = conn.generate_id();
    let cursor = conn.generate_id();

    xproto::open_font_checked(conn, cursor_font, "cursor").request_check()?;
    xproto::create_glyph_cursor_checked(conn,
                                        cursor,
                                        cursor_font,
                                        cursor_font,
                                        XC_CROSSHAIR, XC_CROSSHAIR + 1,
                                        0, 0, 0,
                                        std::u16::MAX, std::u16::MAX, std::u16::MAX)
        .request_check()?;

    let grab_mask = xproto::EVENT_MASK_BUTTON_PRESS as u16
        | xproto::EVENT_MASK_POINTER_MOTION as u16;
    let reply = xproto::grab_pointer(conn,
                                     false,
                                     root,
                                     grab_mask,
                                     xproto::GRAB_MODE_ASYNC as u8,
                                     xproto::GRAB_MODE_ASYNC as u8,
                                     xbase::NONE,
                                     cursor,
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
                    if event.detail() == SELECTION_BUTTON {
                        break Some((event.root_x(), event.root_y()));
                    }
                },
                _ => if !handler(&event)? {
                    break None
                }
            }
        } else {
            break None;
        }
    };
    xproto::ungrab_pointer(conn, xbase::CURRENT_TIME);
    conn.flush();
    Ok(result)
}

