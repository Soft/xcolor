use std;
use xcb::Connection;
use xcb::xproto;
use failure::{Error, err_msg};

#[derive(Clone, Copy, PartialEq)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

impl RGB {
    pub fn new(r: u8, g: u8, b: u8) -> RGB {
        RGB { r, g, b }
    }

    pub fn is_compactable(&self) -> bool {
        fn compact(n: u8) -> bool {
            (n >> 4) == (n & 0xf)
        }
        compact(self.r) && compact(self.g) && compact(self.b)
    }
}

impl From<RGB> for u32 {
    fn from(color: RGB) -> u32 {
        (color.r as u32)  << 16 | (color.g as u32) << 8 | (color.b as u32)
    }
}

#[test]
fn test_compaction() {
    assert!(RGB::new(0xff, 0xff, 0xff).is_compactable());
    assert!(RGB::new(0xee, 0xee, 0xee).is_compactable());
    assert!(RGB::new(0x00, 0x00, 0x00).is_compactable());
    assert!(!RGB::new(0xf7, 0xf7, 0xf7).is_compactable());
    assert!(!RGB::new(0xff, 0xf7, 0xff).is_compactable());
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
    Ok(RGB::new(r, g, b))
}

