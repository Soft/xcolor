use failure::{Error, err_msg};
use xcb::base as xbase;
use xcb::base::Connection;
use xcb::xproto;
use xcb::xproto::Screen;

const PREVIEW_WIDTH: u16 = 32;
const PREVIEW_HEIGHT: u16 = 32;
const PREVIEW_OFFSET_X: u16 = 16;
const PREVIEW_OFFSET_Y: u16 = 16;

pub struct Preview<'a> {
    conn: &'a Connection,
    window: xproto::Window,
}

impl<'a> Preview<'a> {
    pub fn create(conn: &'a Connection,
                  screen: &Screen)
                  -> Result<Preview<'a>, Error> {
        let root = screen.root();
        let net_wm_window_type = xproto::intern_atom(conn, true, "_NET_WM_WINDOW_TYPE")
            .get_reply()?.atom();
        let net_wm_window_type_tooltip = xproto::intern_atom(conn, true, "_NET_WM_WINDOW_TYPE_TOOLTIP")
            .get_reply()?.atom();

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

        xproto::change_property(conn,
                                xproto::PROP_MODE_REPLACE as u8,
                                window,
                                net_wm_window_type,
                                xproto::ATOM_ATOM,
                                32,
                                &[net_wm_window_type_tooltip])
            .request_check()?;

        xproto::map_window(conn, window)
            .request_check()?;

        Ok(Preview { conn, window })
    }

    pub fn draw(&self, event: &xproto::ExposeEvent) -> Result<(), Error> {
        Ok(())
    }

    pub fn position(&self, event: &xproto::MotionNotifyEvent) -> Result<(), Error> {
        let values: &[(u16, u32)] = &[
            (xproto::CONFIG_WINDOW_X as u16, event.root_x() as u32 + PREVIEW_OFFSET_X as u32),
            (xproto::CONFIG_WINDOW_Y as u16, event.root_y() as u32 + PREVIEW_OFFSET_Y as u32)
        ];
        xproto::configure_window(self.conn, self.window, values);
        self.conn.flush();
        Ok(())
    }
}

