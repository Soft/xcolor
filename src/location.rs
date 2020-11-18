use anyhow::{anyhow, Result};
use x11::xcursor::{XcursorImageCreate, XcursorImageDestroy, XcursorImageLoadCursor};
use xcb::base as xbase;
use xcb::base::Connection;
use xcb::xproto;

use crate::color::{self, ARGB};
use crate::draw::draw_magnifying_glass;
use crate::pixel::{PixelArray, PixelArrayMut};
use crate::util::EnsureOdd;

// Left mouse button
const SELECTION_BUTTON: xproto::Button = 1;
const GRAB_MASK: u16 = (xproto::EVENT_MASK_BUTTON_PRESS | xproto::EVENT_MASK_POINTER_MOTION) as u16;

// Exclusively grabs the pointer so we get all its events
fn grab_pointer(conn: &Connection, root: u32, cursor: u32) -> Result<()> {
    let reply = xproto::grab_pointer(
        conn,
        false,
        root,
        GRAB_MASK,
        xproto::GRAB_MODE_ASYNC as u8,
        xproto::GRAB_MODE_ASYNC as u8,
        xbase::NONE,
        cursor,
        xbase::CURRENT_TIME,
    )
    .get_reply()?;

    if reply.status() != xproto::GRAB_STATUS_SUCCESS as u8 {
        return Err(anyhow!("Could not grab pointer"));
    }

    Ok(())
}

// Updates the cursor for an _already grabbed pointer_
fn update_cursor(conn: &Connection, cursor: u32) -> Result<()> {
    xproto::change_active_pointer_grab_checked(conn, cursor, xbase::CURRENT_TIME, GRAB_MASK)
        .request_check()?;

    Ok(())
}

// Creates a new `XcursorImage`, draws the picker into it and loads it, returning the id for a `Cursor`
fn create_new_cursor(
    conn: &Connection,
    screenshot_pixels: &PixelArray<ARGB>,
    preview_width: u32,
) -> Result<u32> {
    Ok(unsafe {
        let mut cursor_image = XcursorImageCreate(preview_width as i32, preview_width as i32);

        // set the "hot spot" - this is where the pointer actually is inside the image
        (*cursor_image).xhot = preview_width / 2;
        (*cursor_image).yhot = preview_width / 2;

        // get pixel data as a mutable Rust slice
        let mut cursor_pixels =
            PixelArrayMut::from_raw_parts((*cursor_image).pixels, preview_width as usize);

        // find out how large our pixels should be in the picker - this must be an odd number (so
        // there's a center pixel) and it must be slightly higher than the ratio between the
        // cursor and the screenshot (to account for integer division so no out of bounds accesses
        // occur when upscaling the image in `draw_magnifying_glass`)
        let mut pixel_size = cursor_pixels.width() / screenshot_pixels.width();
        if pixel_size % 2 == 0 {
            pixel_size += 1;
        } else {
            pixel_size += 2;
        }

        // draw our custom image
        draw_magnifying_glass(&mut cursor_pixels, screenshot_pixels, pixel_size);

        // convert our XcursorImage into a cursor
        let cursor_id = XcursorImageLoadCursor(conn.get_raw_dpy(), cursor_image) as u32;

        // free the XcursorImage
        XcursorImageDestroy(cursor_image);

        cursor_id
    } as u32)
}

// NOTE: this works for multi-monitor configurations since it seems that X fills in the blank
// space with empty pixels when calling XGetImage with a rect that crosses the boundaries of two differently
// sized or misaligned screens
fn get_window_rect_around_pointer(
    conn: &Connection,
    screen: &xproto::Screen,
    (pointer_x, pointer_y): (i16, i16),
    preview_width: u32,
    scale: u32,
) -> Result<(u16, Vec<ARGB>)> {
    let root = screen.root();
    let root_width = screen.width_in_pixels() as isize;
    let root_height = screen.height_in_pixels() as isize;

    let size = ((preview_width / scale) as isize).ensure_odd();

    // the top left coordinates of the rect: make sure they don't go offscreen
    let mut x = (pointer_x as isize) - (size / 2);
    let mut y = (pointer_y as isize) - (size / 2);
    let x_offset = if x < 0 { -x } else { 0 };
    let y_offset = if y < 0 { -y } else { 0 };
    x += x_offset;
    y += y_offset;

    // the size of the rect: make sure they don't extend past the screen
    let size_x = if x + size > (root_width) {
        (root_width) - x
    } else {
        size - x_offset
    };
    let size_y = if y + size > (root_height) {
        (root_height) - y
    } else {
        size - y_offset
    };

    // grab a screenshot of the rect
    let rect = (x as i16, y as i16, size_x as u16, size_y as u16);
    let screenshot_rect = color::window_rect(conn, root, rect)?;

    // the entire portion of the screenshot is on screen
    if size_x == size && size_y == size {
        return Ok((size as u16, screenshot_rect));
    }

    // NOTE: XCB APIs fail when requesting a region outside the screen, so clamp the rect to the screen and
    // fill the clamped pixels with empty data
    let mut pixels = vec![ARGB::TRANSPARENT; (size * size) as usize];
    for x in 0..size_x {
        for y in 0..size_y {
            let screenshot_idx = (y * size_x) + x;
            let pixels_idx = (y + y_offset) * size + (x + x_offset);

            pixels[pixels_idx as usize] = screenshot_rect[screenshot_idx as usize];
        }
    }

    Ok((size as u16, pixels))
}

pub fn wait_for_location(
    conn: &Connection,
    screen: &xproto::Screen,
    preview_width: u32,
    scale: u32,
) -> Result<Option<ARGB>> {
    let root = screen.root();
    let preview_width = preview_width.ensure_odd();

    let pointer = xproto::query_pointer(conn, root).get_reply()?;
    let pointer_pos = (pointer.root_x(), pointer.root_y());
    let (width, initial_rect) =
        get_window_rect_around_pointer(conn, screen, pointer_pos, preview_width, scale)?;

    let screenshot_pixels = PixelArray::new(&initial_rect[..], width.into());
    let cursor_image = create_new_cursor(conn, &screenshot_pixels, preview_width)?;
    grab_pointer(conn, root, cursor_image)?;

    let result = loop {
        let event = conn.wait_for_event();
        if let Some(event) = event {
            match event.response_type() {
                xproto::BUTTON_PRESS => {
                    let event: &xproto::ButtonPressEvent = unsafe { xbase::cast_event(&event) };
                    if event.detail() == SELECTION_BUTTON {
                        let pixels =
                            color::window_rect(conn, root, (event.root_x(), event.root_y(), 1, 1))?;

                        break Some(pixels[0]);
                    }
                }
                xproto::MOTION_NOTIFY => {
                    let event: &xproto::MotionNotifyEvent = unsafe { xbase::cast_event(&event) };
                    let pointer_pos = (event.root_x(), event.root_y());
                    let (width, pixels) = get_window_rect_around_pointer(
                        conn,
                        screen,
                        pointer_pos,
                        preview_width,
                        scale,
                    )?;

                    let screenshot_pixels = PixelArray::new(&pixels[..], width.into());
                    let cursor_image = create_new_cursor(conn, &screenshot_pixels, preview_width)?;
                    update_cursor(conn, cursor_image)?;
                }
                _ => {}
            }
        } else {
            break None;
        }
    };

    xproto::ungrab_pointer(conn, xbase::CURRENT_TIME);
    conn.flush();

    Ok(result)
}
