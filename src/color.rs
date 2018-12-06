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

pub const BLACK: RGB = RGB { r: 0, g: 0, b: 0 };
pub const WHITE: RGB = RGB { r: 0xff, g: 0xff, b: 0xff };

impl RGB {
    pub const fn new(r: u8, g: u8, b: u8) -> RGB {
        RGB { r, g, b }
    }

    pub fn is_compactable(self) -> bool {
        fn compact(n: u8) -> bool {
            (n >> 4) == (n & 0xf)
        }
        compact(self.r) && compact(self.g) && compact(self.b)
    }

    pub fn is_dark(self) -> bool {
        self.distance(BLACK) < self.distance(WHITE)
    }

    pub fn distance(self, other: RGB) -> f32 {
        ((f32::from(other.r) - f32::from(self.r)).powi(2) +
         (f32::from(other.g) - f32::from(self.g)).powi(2) +
         (f32::from(other.b) - f32::from(self.b)).powi(2))
            .sqrt()
    }

    pub fn interpolate(self, other: RGB, amount: f32) -> RGB {
        fn lerp(a: u8, b: u8, x: f32) -> u8 {
            ((1.0 - x) * f32::from(a) + x * f32::from(b)).ceil() as u8
        }
        RGB {
            r: lerp(self.r, other.r, amount),
            g: lerp(self.g, other.g, amount),
            b: lerp(self.b, other.b, amount)
        }
    }

    pub fn lighten(self, amount: f32) -> RGB {
        self.interpolate(WHITE, amount)
    }

    pub fn darken(self, amount: f32) -> RGB {
        self.interpolate(BLACK, amount)
    }
}

impl From<RGB> for u32 {
    fn from(color: RGB) -> u32 {
        u32::from(color.r) << 16 | u32::from(color.g) << 8 | u32::from(color.b)
    }
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

#[test]
fn test_compaction() {
    assert!(RGB::new(0xff, 0xff, 0xff).is_compactable());
    assert!(RGB::new(0xee, 0xee, 0xee).is_compactable());
    assert!(RGB::new(0x00, 0x00, 0x00).is_compactable());
    assert!(!RGB::new(0xf7, 0xf7, 0xf7).is_compactable());
    assert!(!RGB::new(0xff, 0xf7, 0xff).is_compactable());
}
