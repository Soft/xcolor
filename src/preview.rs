use failure::{Error, err_msg};
use xcb::base as xbase;
use xcb::base::Connection;
use xcb::xproto;
use xcb::xproto::Screen;
use xcb::shape as xshape;

use x11;
use x11::RGB;

// TODO:
// - Set window class
// - Intern cache
// - HiDPI

const PREVIEW_WIDTH: u16 = 32;
const PREVIEW_HEIGHT: u16 = 32;
const PREVIEW_OFFSET_X: u16 = 10;
const PREVIEW_OFFSET_Y: u16 = 10;
const WINDOW_NAME: &str = "xcolor";

pub struct Preview<'a> {
    conn: &'a Connection,
    window: xproto::Window,
    gc: xproto::Gc
}

impl<'a> Preview<'a> {
    pub fn create(conn: &'a Connection,
                  screen: &Screen,
                  use_shaped: bool)
                  -> Result<Preview<'a>, Error> {
        let root = screen.root();
        let net_wm_window_type = xproto::intern_atom(conn, true, "_NET_WM_WINDOW_TYPE")
            .get_reply()?.atom();
        let net_wm_window_type_tooltip = xproto::intern_atom(conn, true, "_NET_WM_WINDOW_TYPE_TOOLTIP")
            .get_reply()?.atom();
        let net_wm_name = xproto::intern_atom(conn, true, "_NET_WM_NAME")
            .get_reply()?.atom();
        let utf8_string = xproto::intern_atom(conn, false, "UTF8_STRING")
            .get_reply()?.atom();

        let values = [ (xproto::GC_FOREGROUND, screen.white_pixel()) ];
        let gc = conn.generate_id();
        xproto::create_gc(conn, gc, root, &values);

        let window = conn.generate_id();

        let values = [
            (xproto::CW_EVENT_MASK, xproto::EVENT_MASK_EXPOSURE),
            (xproto::CW_BACK_PIXEL, screen.white_pixel()),
            (xproto::CW_OVERRIDE_REDIRECT, 1)
        ];

        xproto::create_window(conn,
                              xbase::COPY_FROM_PARENT as u8, // Depth
                              window, // Window
                              root, // Parent
                              0, 0, PREVIEW_WIDTH, PREVIEW_HEIGHT, // Size
                              0, // Border
                              xproto::WINDOW_CLASS_INPUT_OUTPUT as u16, // Class
                              xbase::COPY_FROM_PARENT, // Visual
                              &values)
            .request_check()?;

        // Window properties
        xproto::change_property(conn,
                                xproto::PROP_MODE_REPLACE as u8,
                                window,
                                net_wm_window_type,
                                xproto::ATOM_ATOM,
                                32,
                                &[net_wm_window_type_tooltip])
            .request_check()?;
        
        // Set window name
        xproto::change_property(conn,
                                xproto::PROP_MODE_REPLACE as u8,
                                window,
                                net_wm_name,
                                utf8_string,
                                8,
                                WINDOW_NAME.as_bytes())
            .request_check()?;

        xproto::change_property(conn,
                                xproto::PROP_MODE_REPLACE as u8,
                                window,
                                xproto::ATOM_WM_NAME,
                                xproto::ATOM_STRING,
                                8,
                                WINDOW_NAME.as_bytes())
            .request_check()?;


        // Setup shape mask
        let shape_ext = conn.get_extension_data(xshape::id());
        if use_shaped && shape_ext.map_or(false, |ext| ext.present()) {
          let mask = conn.generate_id();
          xproto::create_pixmap(conn, 1, mask, window, PREVIEW_WIDTH, PREVIEW_HEIGHT);

          let values = [ (xproto::GC_FOREGROUND, 0) ];
          let mask_gc = conn.generate_id();
          xproto::create_gc(conn, mask_gc, mask, &values);

          let rect = xproto::Rectangle::new(0, 0, PREVIEW_WIDTH, PREVIEW_HEIGHT);
          xproto::poly_fill_rectangle(conn, mask, mask_gc, &[rect]);

          let values = [ (xproto::GC_FOREGROUND, 1) ];
          xproto::change_gc(conn, mask_gc, &values);

          let arc = xproto::Arc::new(0, 0, PREVIEW_WIDTH, PREVIEW_HEIGHT, 0, 360 << 6);
          xproto::poly_fill_arc(conn, mask, mask_gc, &[arc]);

          xshape::mask(conn, xshape::SO_SET as u8, xshape::SK_BOUNDING as u8, window, 0, 0, mask);
        }

        Ok(Preview { conn, window, gc })
    }

    pub fn map(&self) -> Result<(), Error> {
        xproto::map_window(self.conn, self.window)
            .request_check()?;
        Ok(())
    }

    pub fn reposition(&self, (x, y): (i16, i16)) -> Result<(), Error> {
        // These casts seem bad
        let values: &[(u16, u32)] = &[
            (xproto::CONFIG_WINDOW_X as u16, x as u32 + PREVIEW_OFFSET_X as u32),
            (xproto::CONFIG_WINDOW_Y as u16, y as u32 + PREVIEW_OFFSET_Y as u32)
        ];
        xproto::configure_window(self.conn, self.window, values);
        self.conn.flush();
        Ok(())
    }

    pub fn redraw(&self, (r, g, b): RGB) -> Result<(), Error> {
        let color: u32 = (r as u32)  << 16 | (g as u32) << 8 | (b as u32);
        let values: &[(u32, u32)] = &[
            (xproto::GC_FOREGROUND, color)
        ];
        xproto::change_gc(self.conn, self.gc, values);
        let rect = xproto::Rectangle::new(0, 0, PREVIEW_WIDTH, PREVIEW_HEIGHT);
        xproto::poly_fill_rectangle(self.conn, self.window, self.gc, &[rect]);
        self.conn.flush();
        Ok(())
    }
}

