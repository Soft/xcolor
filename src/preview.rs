use failure::Error;
use xcb::base as xbase;
use xcb::base::Connection;
use xcb::xproto;
use xcb::xproto::Screen;
use xcb::shape as xshape;

use atoms;
use color::RGB;

// TODO:
// - Set window class
// - HiDPI

const PREVIEW_WIDTH: u16 = 32;
const PREVIEW_HEIGHT: u16 = 32;
const PREVIEW_OFFSET_X: u16 = 10;
const PREVIEW_OFFSET_Y: u16 = 10;
const WINDOW_NAME: &str = "xcolor";

pub struct Preview<'a> {
    conn: &'a Connection,
    window: xproto::Window,
    background_gc: xproto::Gc,
    border_gc: xproto::Gc,
    shaped: bool
}

impl<'a> Preview<'a> {
    pub fn create(conn: &'a Connection,
                  screen: &Screen,
                  use_shaped: bool)
                  -> Result<Preview<'a>, Error> {
        let root = screen.root();

        // Intern atoms
        let net_wm_window_type = atoms::get(conn, "_NET_WM_WINDOW_TYPE")?;
        let net_wm_window_type_tooltip = atoms::get(conn, "_NET_WM_WINDOW_TYPE_TOOLTIP")?;
        let net_wm_name = atoms::get(conn, "_NET_WM_NAME")?;
        let utf8_string = atoms::get(conn, "UTF8_STRING")?;
        let net_wm_state = atoms::get(conn, "_NET_WM_STATE")?;
        let net_wm_state_above = atoms::get(conn, "_NET_WM_STATE_ABOVE")?;
        let net_wm_state_sticky = atoms::get(conn, "_NET_WM_STATE_STICKY")?;
        let net_wm_state_skip_taskbar = atoms::get(conn, "_NET_WM_STATE_SKIP_TASKBAR")?;
        let net_wm_state_skip_pager = atoms::get(conn, "_NET_WM_STATE_SKIP_PAGER")?;

        // Create GCs
        let values = [ (xproto::GC_FOREGROUND, screen.white_pixel()) ];
        let background_gc = conn.generate_id();
        xproto::create_gc(conn, background_gc, root, &values);

        let values = [ (xproto::GC_FOREGROUND, screen.black_pixel()) ];
        let border_gc = conn.generate_id();
        xproto::create_gc(conn, border_gc, root, &values);

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

        let wm_state = [net_wm_state_above,
                        net_wm_state_sticky,
                        net_wm_state_skip_taskbar,
                        net_wm_state_skip_pager];
        xproto::change_property(conn,
                                xproto::PROP_MODE_REPLACE as u8,
                                window,
                                net_wm_state,
                                xproto::ATOM_ATOM,
                                32,
                                &wm_state)
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


        let mut shaped = false;
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

            shaped = true;
        }

        Ok(Preview { conn, window, background_gc, border_gc, shaped })
    }

    pub fn map(&self) -> Result<(), Error> {
        xproto::map_window(self.conn, self.window)
            .request_check()?;
        Ok(())
    }

    pub fn unmap(&self) -> Result<(), Error> {
        xproto::unmap_window(self.conn, self.window)
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

    pub fn redraw(&self, color: RGB) -> Result<(), Error> {
        let color: u32 = color.into();
        let values: &[(u32, u32)] = &[ (xproto::GC_FOREGROUND, color) ];
        xproto::change_gc(self.conn, self.background_gc, values);
        let rect = xproto::Rectangle::new(0, 0, PREVIEW_WIDTH, PREVIEW_HEIGHT);
        xproto::poly_fill_rectangle(self.conn, self.window, self.background_gc, &[rect]);

        let values: &[(u32, u32)] = &[ (xproto::GC_FOREGROUND, !color) ];
        xproto::change_gc(self.conn, self.border_gc, values);
        if self.shaped {
          let arc = xproto::Arc::new(1, 1, PREVIEW_WIDTH-2, PREVIEW_HEIGHT-2, 0, 360 << 6);
          xproto::poly_arc(self.conn, self.window, self.border_gc, &[arc]);
        } else {
          let rect = xproto::Rectangle::new(0, 0, PREVIEW_WIDTH-1, PREVIEW_HEIGHT-1);
          xproto::poly_rectangle(self.conn, self.window, self.border_gc, &[rect]);
        }

        self.conn.flush();

        Ok(())
    }
}

