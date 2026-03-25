use ab_glyph::{FontArc, PxScale};
use image::{Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_hollow_rect_mut, draw_text_mut, text_size};
use imageproc::rect::Rect;

use crate::core::types::WindowInfo;

// Embedded font
const FONT_BYTES: &[u8] = include_bytes!("../../assets/DejaVuSansMono.ttf");

const COLORS: &[Rgba<u8>] = &[
    Rgba([0, 255, 0, 255]),     // green
    Rgba([255, 100, 0, 255]),   // orange
    Rgba([0, 150, 255, 255]),   // blue
    Rgba([255, 0, 255, 255]),   // magenta
    Rgba([255, 255, 0, 255]),   // yellow
    Rgba([0, 255, 255, 255]),   // cyan
    Rgba([255, 50, 50, 255]),   // red
    Rgba([150, 100, 255, 255]), // purple
];

const LABEL_BG: Rgba<u8> = Rgba([0, 0, 0, 200]);
const LABEL_FG: Rgba<u8> = Rgba([255, 255, 255, 255]);

pub fn annotate_screenshot(image: &mut RgbaImage, windows: &[WindowInfo]) {
    let font = match FontArc::try_from_slice(FONT_BYTES) {
        Ok(f) => f,
        Err(_) => return, // Silently skip annotation if font fails to load
    };
    let scale = PxScale { x: 18.0, y: 18.0 };

    // Iterate back-to-front so topmost window labels are drawn last (not occluded)
    for (i, win) in windows.iter().enumerate().rev() {
        if win.minimized {
            continue; // Don't annotate hidden windows
        }

        let color = COLORS[i % COLORS.len()];
        let x = win.x.max(0);
        let y = win.y.max(0);
        let w = win.width;
        let h = win.height;

        // Draw bounding box (2px thick by drawing at offset 0 and 1)
        if w > 0 && h > 0 {
            draw_hollow_rect_mut(image, Rect::at(x, y).of_size(w, h), color);
            if w > 2 && h > 2 {
                draw_hollow_rect_mut(image, Rect::at(x + 1, y + 1).of_size(w - 2, h - 2), color);
            }
        }

        // Draw label
        let label = format!("@{}", win.ref_id);
        let (tw, th) = text_size(scale, &font, &label);
        let label_x = x;
        let label_y = (y - th as i32 - 4).max(0);

        // Label background
        draw_filled_rect_mut(
            image,
            Rect::at(label_x, label_y).of_size(tw + 6, th + 4),
            LABEL_BG,
        );

        // Label text
        draw_text_mut(
            image,
            LABEL_FG,
            label_x + 3,
            label_y + 2,
            scale,
            &font,
            &label,
        );
    }
}
