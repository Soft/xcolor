use std;

use failure::{Error, err_msg};
use xcb::base as xbase;
use xcb::base::Connection;
use xcb::xproto;
use xcb::xproto::Screen;

use preview::Preview;

pub type RGB = (u8, u8, u8);

pub fn wait_for_location(conn: &Connection, screen: &Screen)
                         -> Result<Option<(i16, i16)>, Error> {
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

    let preview = Preview::create(conn, screen)?;

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
                xproto::EXPOSE => {
                    let event: &xproto::ExposeEvent = unsafe {
                        xbase::cast_event(&event)
                    };
                    preview.draw(event)?;
                },
                xproto::MOTION_NOTIFY => {
                    let event: &xproto::MotionNotifyEvent = unsafe {
                        xbase::cast_event(&event)
                    };
                    preview.position(event)?;
                },
                _ => break None
            }
        } else {
            break None;
        }
    };
    xproto::ungrab_pointer(&conn, xbase::CURRENT_TIME);
    conn.flush();
    Ok(result)
}

pub fn window_color_at_point(conn: &Connection, window: xproto::Window, (x, y): (i16, i16))
                         -> Result<RGB, Error> {
    let reply = xproto::get_image(conn,
                                  xproto::IMAGE_FORMAT_Z_PIXMAP as u8,
                                  window,
                                  x, y, 1, 1,
                                  std::u32::MAX)
        .get_reply()?;
    if reply.depth() != 24 {
        // TODO: Figure out what to do with these
        return Err(err_msg("Unsupported color depth"));
    }
    let data = reply.data();
    let r = data[2];
    let g = data[1];
    let b = data[0];
    Ok((r, g, b))
}

