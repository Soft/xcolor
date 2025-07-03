use crate::color::ARGB;
use crate::pixel::PixelSquare;

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
    cursor: &mut PixelSquare<&mut [u32]>,
    screenshot: &PixelSquare<&[ARGB]>,
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

// Simple function to draw color text on the cursor
pub fn draw_color_text(cursor_pixels: &mut PixelSquare<&mut [u32]>, color: ARGB) {
    let width = cursor_pixels.width();
    let height = cursor_pixels.height();

    // Convert color to hex string
    let hex_string = format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b);

    // Calculate text dimensions
    let text_height = 24; // Height for larger text
    let text_width = hex_string.len() * 12; // 12 pixels per character for larger font
    let text_start_y = height.saturating_sub(text_height);
    let text_bg_color = 0xFF000000u32; // Black background
    let text_color = 0xFFFFFFFFu32; // White text

    // Calculate center position for text (horizontal centering)
    let text_start_x = (width.saturating_sub(text_width)) / 2;

    // Add some padding around the text
    let padding = 4;
    let bg_start_x = text_start_x.saturating_sub(padding);
    let bg_end_x = (text_start_x + text_width + padding).min(width);

    // Draw background rectangle only for the text area (with padding)
    for y in text_start_y..height {
        for x in bg_start_x..bg_end_x {
            cursor_pixels[(x, y)] = text_bg_color;
        }
    }

    // Draw larger text at the bottom center
    draw_large_text(
        cursor_pixels,
        &hex_string,
        text_start_x,
        text_start_y + 4,
        text_color,
    );
}

// Larger text rendering function (12x16 font)
fn draw_large_text(
    cursor_pixels: &mut PixelSquare<&mut [u32]>,
    text: &str,
    start_x: usize,
    start_y: usize,
    color: u32,
) {
    let width = cursor_pixels.width();
    let height = cursor_pixels.height();

    for (i, ch) in text.chars().enumerate() {
        let x_offset = start_x + i * 12; // 12 pixels per character
        if x_offset + 12 > width {
            break;
        }

        // Get large bitmap for character
        let bitmap = get_large_char_bitmap(ch);

        for (row, &line) in bitmap.iter().enumerate() {
            let y = start_y + row;
            if y >= height {
                break;
            }

            for col in 0..12 {
                let x = x_offset + col;
                if x >= width {
                    break;
                }

                if (line >> (11 - col)) & 1 == 1 {
                    cursor_pixels[(x, y)] = color;
                }
            }
        }
    }
}

// Larger 12x16 bitmap font for hex characters
fn get_large_char_bitmap(ch: char) -> &'static [u16; 16] {
    match ch {
        '0' => &[
            0x0000, 0x03C0, 0x0660, 0x0C30, 0x0C30, 0x0CF0, 0x0DB0, 0x0DB0, 0x0F30, 0x0E30, 0x0C30,
            0x0C30, 0x0660, 0x03C0, 0x0000, 0x0000,
        ],
        '1' => &[
            0x0000, 0x0180, 0x0380, 0x0780, 0x0180, 0x0180, 0x0180, 0x0180, 0x0180, 0x0180, 0x0180,
            0x0180, 0x0180, 0x07E0, 0x0000, 0x0000,
        ],
        '2' => &[
            0x0000, 0x03C0, 0x0660, 0x0C30, 0x0030, 0x0030, 0x0060, 0x00C0, 0x0180, 0x0300, 0x0600,
            0x0C00, 0x0C00, 0x0FF0, 0x0000, 0x0000,
        ],
        '3' => &[
            0x0000, 0x03C0, 0x0660, 0x0C30, 0x0030, 0x0030, 0x01C0, 0x01C0, 0x0030, 0x0030, 0x0030,
            0x0C30, 0x0660, 0x03C0, 0x0000, 0x0000,
        ],
        '4' => &[
            0x0000, 0x0060, 0x00E0, 0x01E0, 0x0360, 0x0660, 0x0C60, 0x0C60, 0x0FF0, 0x0FF0, 0x0060,
            0x0060, 0x0060, 0x0060, 0x0000, 0x0000,
        ],
        '5' => &[
            0x0000, 0x0FF0, 0x0C00, 0x0C00, 0x0C00, 0x0FC0, 0x0FE0, 0x0030, 0x0030, 0x0030, 0x0030,
            0x0C30, 0x0660, 0x03C0, 0x0000, 0x0000,
        ],
        '6' => &[
            0x0000, 0x01C0, 0x0360, 0x0600, 0x0C00, 0x0C00, 0x0FC0, 0x0FE0, 0x0C30, 0x0C30, 0x0C30,
            0x0C30, 0x0660, 0x03C0, 0x0000, 0x0000,
        ],
        '7' => &[
            0x0000, 0x0FF0, 0x0FF0, 0x0030, 0x0060, 0x0060, 0x00C0, 0x00C0, 0x0180, 0x0180, 0x0300,
            0x0300, 0x0300, 0x0300, 0x0000, 0x0000,
        ],
        '8' => &[
            0x0000, 0x03C0, 0x0660, 0x0C30, 0x0C30, 0x0660, 0x03C0, 0x03C0, 0x0660, 0x0C30, 0x0C30,
            0x0C30, 0x0660, 0x03C0, 0x0000, 0x0000,
        ],
        '9' => &[
            0x0000, 0x03C0, 0x0660, 0x0C30, 0x0C30, 0x0C30, 0x0C30, 0x07F0, 0x03F0, 0x0030, 0x0030,
            0x0060, 0x06C0, 0x0380, 0x0000, 0x0000,
        ],
        'A' => &[
            0x0000, 0x0180, 0x03C0, 0x03C0, 0x0660, 0x0660, 0x0C30, 0x0C30, 0x0FF0, 0x0FF0, 0x0C30,
            0x0C30, 0x0C30, 0x0C30, 0x0000, 0x0000,
        ],
        'B' => &[
            0x0000, 0x0FC0, 0x0C60, 0x0C30, 0x0C30, 0x0C60, 0x0FC0, 0x0FC0, 0x0C60, 0x0C30, 0x0C30,
            0x0C30, 0x0C60, 0x0FC0, 0x0000, 0x0000,
        ],
        'C' => &[
            0x0000, 0x03C0, 0x0660, 0x0C30, 0x0C00, 0x0C00, 0x0C00, 0x0C00, 0x0C00, 0x0C00, 0x0C00,
            0x0C30, 0x0660, 0x03C0, 0x0000, 0x0000,
        ],
        'D' => &[
            0x0000, 0x0FC0, 0x0C60, 0x0C30, 0x0C30, 0x0C30, 0x0C30, 0x0C30, 0x0C30, 0x0C30, 0x0C30,
            0x0C30, 0x0C60, 0x0FC0, 0x0000, 0x0000,
        ],
        'E' => &[
            0x0000, 0x0FF0, 0x0C00, 0x0C00, 0x0C00, 0x0C00, 0x0FE0, 0x0FE0, 0x0C00, 0x0C00, 0x0C00,
            0x0C00, 0x0C00, 0x0FF0, 0x0000, 0x0000,
        ],
        'F' => &[
            0x0000, 0x0FF0, 0x0C00, 0x0C00, 0x0C00, 0x0C00, 0x0FE0, 0x0FE0, 0x0C00, 0x0C00, 0x0C00,
            0x0C00, 0x0C00, 0x0C00, 0x0000, 0x0000,
        ],
        '#' => &[
            0x0000, 0x0660, 0x0660, 0x0660, 0x0FF0, 0x0FF0, 0x0660, 0x0660, 0x0FF0, 0x0FF0, 0x0660,
            0x0660, 0x0660, 0x0660, 0x0000, 0x0000,
        ],
        _ => &[
            0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
            0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
        ],
    }
}
