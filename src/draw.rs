use crate::color::ARGB;
use crate::pixel::{PixelArray, PixelArrayMut};

#[inline]
fn is_inside_circle(x: isize, y: isize, r: isize) -> bool {
    (x - r).pow(2) + (y - r).pow(2) < r.pow(2)
}

#[inline]
fn border_color(color: ARGB) -> u32 {
    if color.is_dark() {
        ARGB::WHITE.into()
    } else {
        ARGB::BLACK.into()
    }
}

pub fn draw_magnifying_glass(
    cursor: &mut PixelArrayMut<u32>,
    screenshot: &PixelArray<ARGB>,
    pixel_size: usize,
) {
    assert!(pixel_size % 2 != 0, "pixel_size must be odd");
    assert!(cursor.width() % 2 != 0, "cursor.width must be odd");
    assert!(screenshot.width() % 2 != 0, "screenshot.width must be odd");

    let transparent: u32 = ARGB::TRANSPARENT.into();

    let pixel_size = pixel_size as isize;
    let cursor_width = cursor.width() as isize;
    let screenshot_width = screenshot.width() as isize;

    let border_width = 1;
    let border_radius = cursor_width / 2;
    let content_radius = border_radius - border_width;

    let cursor_center = cursor_width / 2;
    let cursor_center_pixel = cursor_center - pixel_size / 2;
    let screenshot_center = screenshot_width / 2;
    let offset = screenshot_center * pixel_size - cursor_center_pixel;

    for cx in 0..cursor_width {
        for cy in 0..cursor_width {
            // screenshot coordinates
            let sx = ((cx + offset) / pixel_size) as usize;
            let sy = ((cy + offset) / pixel_size) as usize;
            let screenshot_color = screenshot[(sx, sy)];

            // set cursor pixel
            cursor[(cx as usize, cy as usize)] = if is_inside_circle(cx, cy, content_radius) {
                let is_grid_line =
                    (cx + offset) % pixel_size == 0 || (cy + offset) % pixel_size == 0;

                if is_grid_line {
                    let is_center_x =
                        cx >= cursor_center_pixel && cx <= cursor_center_pixel + pixel_size;
                    let is_center_y =
                        cy >= cursor_center_pixel && cy <= cursor_center_pixel + pixel_size;

                    // center pixel's border color
                    if is_center_x && is_center_y {
                        border_color(screenshot_color)
                    } else {
                        // grid color
                        if screenshot_color.is_dark() {
                            screenshot_color.lighten(0.2).into()
                        } else {
                            screenshot_color.darken(0.2).into()
                        }
                    }
                } else {
                    screenshot_color.into()
                }
            } else if is_inside_circle(cx + border_width, cy + border_width, border_radius) {
                border_color(screenshot_color)
            } else {
                transparent
            };
        }
    }
}
