use anyhow::{anyhow, Result};
use std;
use xcb::xproto;
use xcb::Connection;

#[derive(Clone, Copy, PartialEq)]
pub struct ARGB {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl ARGB {
    pub const TRANSPARENT: ARGB = ARGB {
        a: 0,
        r: 0,
        g: 0,
        b: 0,
    };
    pub const BLACK: ARGB = ARGB {
        a: 0xff,
        r: 0,
        g: 0,
        b: 0,
    };
    pub const WHITE: ARGB = ARGB {
        a: 0xff,
        r: 0xff,
        g: 0xff,
        b: 0xff,
    };

    pub const fn new(a: u8, r: u8, g: u8, b: u8) -> ARGB {
        ARGB { a, r, g, b }
    }

    pub fn is_compactable(self) -> bool {
        fn compact(n: u8) -> bool {
            (n >> 4) == (n & 0xf)
        }
        compact(self.r) && compact(self.g) && compact(self.b)
    }

    pub fn is_dark(self) -> bool {
        self.distance(Self::BLACK) < self.distance(Self::WHITE)
    }

    pub fn distance(self, other: ARGB) -> f32 {
        ((f32::from(other.r) - f32::from(self.r)).powi(2)
            + (f32::from(other.g) - f32::from(self.g)).powi(2)
            + (f32::from(other.b) - f32::from(self.b)).powi(2))
        .sqrt()
    }

    pub fn interpolate(self, other: ARGB, amount: f32) -> ARGB {
        fn lerp(a: u8, b: u8, x: f32) -> u8 {
            ((1.0 - x) * f32::from(a) + x * f32::from(b)).ceil() as u8
        }
        ARGB {
            a: self.a,
            r: lerp(self.r, other.r, amount),
            g: lerp(self.g, other.g, amount),
            b: lerp(self.b, other.b, amount),
        }
    }

    pub fn lighten(self, amount: f32) -> ARGB {
        self.interpolate(Self::WHITE, amount)
    }

    pub fn darken(self, amount: f32) -> ARGB {
        self.interpolate(Self::BLACK, amount)
    }
}

impl From<ARGB> for u32 {
    fn from(color: ARGB) -> u32 {
        u32::from(color.a) << 24
            | u32::from(color.r) << 16
            | u32::from(color.g) << 8
            | u32::from(color.b)
    }
}

pub fn window_rect(
    conn: &Connection,
    window: xproto::Window,
    (x, y, width, height): (i16, i16, u16, u16),
) -> Result<Vec<ARGB>> {
    let reply = xproto::get_image(
        conn,
        xproto::IMAGE_FORMAT_Z_PIXMAP as u8,
        window,
        x,
        y,
        width,
        height,
        std::u32::MAX,
    )
    .get_reply()?;

    if reply.depth() != 24 {
        // TODO: Figure out what to do with these
        return Err(anyhow!("Unsupported color depth"));
    }

    let data = reply.data();
    let mut pixels = Vec::with_capacity(data.len());
    for chunk in data.chunks(4) {
        pixels.push(ARGB::new(0xff, chunk[2], chunk[1], chunk[0]));
    }

    Ok(pixels)
}

#[test]
fn test_compaction() {
    assert!(ARGB::new(0xff, 0xff, 0xff, 0xff).is_compactable());
    assert!(ARGB::new(0xff, 0xee, 0xee, 0xee).is_compactable());
    assert!(ARGB::new(0xff, 0x00, 0x00, 0x00).is_compactable());
    assert!(!ARGB::new(0xff, 0xf7, 0xf7, 0xf7).is_compactable());
    assert!(!ARGB::new(0xff, 0xff, 0xf7, 0xff).is_compactable());
}
