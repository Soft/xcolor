use failure::Error;
use xcb::base as xbase;
use xcb::base::Connection;
use xcb::xproto;
use xcb::xproto::Screen;
use xcb::shape as xshape;

use crate::atoms;
use crate::color;
use crate::color::RGB;

const PREVIEW_WIDTH: u16 = 32;
const PREVIEW_HEIGHT: u16 = 32;
const PREVIEW_OFFSET_X: i16 = 10;
const PREVIEW_OFFSET_Y: i16 = 10;
const WINDOW_NAME: &str = "xcolor";
const WINDOW_CLASS: &str = "xcolor\0XColor\0";
const BORDER_BLEND_AMOUNT: f32 = 0.3;

pub struct Preview<'a> {
    conn: &'a Connection,
    root: xproto::Window,
    window: xproto::Window,
    background_gc: xproto::Gc,
    color: RGB
}

impl<'a> Preview<'a> {
    pub fn create(conn: &'a Connection,
                  screen: &Screen,
                  use_shaped: bool)
                  -> Result<Preview<'a>, Error> {
        let root = screen.root();

        // Intern atoms
        let utf8_string = atoms::get(conn, "UTF8_STRING")?;
        let net_wm_window_type = atoms::get(conn, "_NET_WM_WINDOW_TYPE")?;
        let net_wm_window_type_tooltip = atoms::get(conn, "_NET_WM_WINDOW_TYPE_TOOLTIP")?;
        let net_wm_name = atoms::get(conn, "_NET_WM_NAME")?;
        let net_wm_state = atoms::get(conn, "_NET_WM_STATE")?;
        let net_wm_state_above = atoms::get(conn, "_NET_WM_STATE_ABOVE")?;
        let net_wm_state_sticky = atoms::get(conn, "_NET_WM_STATE_STICKY")?;
        let net_wm_state_skip_taskbar = atoms::get(conn, "_NET_WM_STATE_SKIP_TASKBAR")?;
        let net_wm_state_skip_pager = atoms::get(conn, "_NET_WM_STATE_SKIP_PAGER")?;

        // Check if SHAPE extension is available and if using it is desired
        let shape_ext = conn.get_extension_data(xshape::id());
        let use_shaped = use_shaped && shape_ext.map_or(false, |ext| ext.present());

        // Create GCs
        let values = [ (xproto::GC_FOREGROUND, screen.white_pixel()) ];
        let background_gc = conn.generate_id();
        xproto::create_gc(conn, background_gc, root, &values);

        let window = conn.generate_id();

        let values = [
            (xproto::CW_EVENT_MASK, xproto::EVENT_MASK_EXPOSURE),
            (xproto::CW_BACK_PIXEL, screen.white_pixel()),
            (xproto::CW_BORDER_PIXEL, screen.black_pixel()),
            (xproto::CW_OVERRIDE_REDIRECT, 1)
        ];

        let pointer = xproto::query_pointer(conn, root)
            .get_reply()?;
        let pointer_x = pointer.root_x();
        let pointer_y = pointer.root_y();

        let color = color::window_color_at_point(conn, root, (pointer_x, pointer_y))?;

        let border_width = if use_shaped { 0 } else { 1 };
        let (position_x, position_y) = preview_position((pointer_x, pointer_y));
        xproto::create_window(conn,
                              xbase::COPY_FROM_PARENT as u8, // Depth
                              window, // Window
                              root, // Parent
                              position_x, position_y, // Location
                              PREVIEW_WIDTH, PREVIEW_HEIGHT, // Size
                              border_width, // Border
                              xproto::WINDOW_CLASS_INPUT_OUTPUT as u16, // Class
                              xbase::COPY_FROM_PARENT, // Visual
                              &values);

        // Window properties
        xproto::change_property(conn,
                                xproto::PROP_MODE_REPLACE as u8,
                                window,
                                net_wm_window_type,
                                xproto::ATOM_ATOM,
                                32,
                                &[net_wm_window_type_tooltip]);

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
                                &wm_state);
        
        // Set window name & class
        xproto::change_property(conn,
                                xproto::PROP_MODE_REPLACE as u8,
                                window,
                                net_wm_name,
                                utf8_string,
                                8,
                                WINDOW_NAME.as_bytes());

        xproto::change_property(conn,
                                xproto::PROP_MODE_REPLACE as u8,
                                window,
                                xproto::ATOM_WM_NAME,
                                xproto::ATOM_STRING,
                                8,
                                WINDOW_NAME.as_bytes());

        xproto::change_property(conn,
                                xproto::PROP_MODE_REPLACE as u8,
                                window,
                                xproto::ATOM_WM_CLASS,
                                xproto::ATOM_STRING,
                                8,
                                WINDOW_CLASS.as_bytes());

        // Setup shape mask
        if use_shaped {
            let transparent = [ (xproto::GC_FOREGROUND, 0) ];
            let solid = [ (xproto::GC_FOREGROUND, 1) ];
            let rect = xproto::Rectangle::new(0, 0, PREVIEW_WIDTH, PREVIEW_HEIGHT);

            let mask = conn.generate_id();
            xproto::create_pixmap(conn, 1, mask, window, PREVIEW_WIDTH, PREVIEW_HEIGHT);

            let mask_gc = conn.generate_id();

            // Set content mask
            xproto::create_gc(conn, mask_gc, mask, &transparent);
            xproto::poly_fill_rectangle(conn, mask, mask_gc, &[rect]);

            xproto::change_gc(conn, mask_gc, &solid);
            let arc = xproto::Arc::new(1, 1, PREVIEW_WIDTH-2, PREVIEW_HEIGHT-2, 0, 360 << 6);
            xproto::poly_fill_arc(conn, mask, mask_gc, &[arc]);

            xshape::mask(conn, xshape::SO_SET as u8, xshape::SK_CLIP as u8, window, 0, 0, mask);

            // Set border mask
            xproto::change_gc(conn, mask_gc, &transparent);
            xproto::poly_fill_rectangle(conn, mask, mask_gc, &[rect]);

            xproto::change_gc(conn, mask_gc, &solid);
            let arc = xproto::Arc::new(0, 0, PREVIEW_WIDTH, PREVIEW_HEIGHT, 0, 360 << 6);
            xproto::poly_fill_arc(conn, mask, mask_gc, &[arc]);

            xshape::mask(conn, xshape::SO_SET as u8, xshape::SK_BOUNDING as u8, window, 0, 0, mask);
        }

        xproto::map_window(conn, window);

        Ok(Preview { conn, root, window, background_gc, color })
    }

    pub fn handle_event(&mut self, event: &xbase::GenericEvent) -> Result<bool, Error> {
        match event.response_type() {
            xproto::EXPOSE => self.redraw(),
            xproto::MOTION_NOTIFY => {
                let event: &xproto::MotionNotifyEvent = unsafe {
                    xbase::cast_event(event)
                };
                let pointer_x = event.root_x();
                let pointer_y = event.root_y();
                self.color = color::window_color_at_point(self.conn, self.root, (pointer_x, pointer_y))?;
                self.reposition((pointer_x, pointer_y));
                self.redraw();
            }
            _ => return Ok(false)
        }
        Ok(true)
    }

    pub fn reposition(&self, (x, y): (i16, i16)) {
        let (x, y) = preview_position((x, y));
        let values: &[(u16, u32)] = &[
            (xproto::CONFIG_WINDOW_X as u16, x as u32),
            (xproto::CONFIG_WINDOW_Y as u16, y as u32)
        ];
        xproto::configure_window(self.conn, self.window, values);
        self.conn.flush();
    }

    pub fn redraw(&self) {
        // Content
        let background_color: u32 = self.color.into();
        let values: &[(u32, u32)] = &[ (xproto::GC_FOREGROUND, background_color) ];
        xproto::change_gc(self.conn, self.background_gc, values);
        let rect = xproto::Rectangle::new(0, 0, PREVIEW_WIDTH, PREVIEW_HEIGHT);
        xproto::poly_fill_rectangle(self.conn, self.window, self.background_gc, &[rect]);

        // Border
        let border_color = if self.color.is_dark() {
            self.color.lighten(BORDER_BLEND_AMOUNT)
        } else {
            self.color.darken(BORDER_BLEND_AMOUNT)
        };
        let border_color: u32 = border_color.into();

        let values: &[(u32, u32)] = &[ (xproto::CW_BORDER_PIXEL, border_color) ];
        xproto::change_window_attributes(self.conn, self.window, values);

        self.conn.flush();
    }
}

impl<'a> Drop for Preview<'a> {
    fn drop(&mut self) {
        xproto::unmap_window(self.conn, self.window);
    }
}

#[inline]
fn preview_position((x, y): (i16, i16)) -> (i16, i16) {
    (x + PREVIEW_OFFSET_X, y + PREVIEW_OFFSET_Y)
}


