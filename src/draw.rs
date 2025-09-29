use core::slice;

use crate::lcd::{self, LCD_HEIGHT, LCD_WIDTH, LAYER1_BASE, LAYER2_BASE, LAYER2_H, LAYER2_W};

pub fn layer1_checkerboard() {
    // Fill both front and back buffers so first swap shows identical content
    fill_checkerboard_to(LAYER1_BASE);
    fill_checkerboard_to(lcd::layer1_back_addr());
}

fn fill_checkerboard_to(base: u32) {
    let pixels = (LCD_WIDTH * LCD_HEIGHT) as usize;
    let buf = unsafe { slice::from_raw_parts_mut(base as *mut u32, pixels) };
    let cel_count = (LCD_WIDTH >> 5) + (LCD_HEIGHT >> 5);
    for row in 0..LCD_HEIGHT {
        for col in 0..LCD_WIDTH {
            let i = (row * LCD_WIDTH + col) as usize;
            let cel = (row >> 5) + (col >> 5);
            let mut a: u8 = if (cel & 1) != 0 { 0 } else { 0xFF };
            let mut r: u8 = (row * 0xFF / LCD_HEIGHT) as u8;
            let mut g: u8 = (col * 0xFF / LCD_WIDTH) as u8;
            let mut b: u8 = (0xFF * (cel_count - cel - 1) / cel_count) as u8;
            if (cel & 3) == 0 { b = 0; }
            if row % 32 == 0 || col % 32 == 0 {
                r = if a != 0 { 0xFF } else { 0 }; g = r; b = r; a = 0xFF;
            }
            let pix = ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
            buf[i] = pix;
        }
    }
}

pub fn layer2_sprite() {
    let pixels = (LAYER2_W * LAYER2_H) as usize;
    let buf = unsafe { slice::from_raw_parts_mut(LAYER2_BASE as *mut u32, pixels) };
    // Clear to fully transparent
    for p in buf.iter_mut() { *p = 0; }

    // Draw a full-size rectangle with a color gradient, fully opaque inside
    let rx0 = 0;
    let ry0 = 0;
    let rw = LAYER2_W.max(1);
    let rh = LAYER2_H.max(1);
    for y in 0..rh {
        for x in 0..rw {
            let gx = ((x * 255) / rw) as u8; // 0..255
            let gy = ((y * 255) / rh) as u8; // 0..255
            let r = gx;
            let g = gy;
            let mut b = 255u8.saturating_sub(((gx as u16 + gy as u16) / 2) as u8);
            // 1px border brighter
            if x == 0 || y == 0 || x == rw - 1 || y == rh - 1 { b = 255; }
            let a: u8 = 0xFF; // fully opaque interior
            let idx = ((ry0 + y) * LAYER2_W + (rx0 + x)) as usize;
            buf[idx] = ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
        }
    }
}

// --- Simple drawing helpers on Layer1 (ARGB8888) ---
// Only needed for the optional FPS overlay
#[cfg(feature = "overlay")]
#[inline]
fn fb1_mut_at(base: u32) -> &'static mut [u32] {
    let pixels = (LCD_WIDTH * LCD_HEIGHT) as usize;
    unsafe { slice::from_raw_parts_mut(base as *mut u32, pixels) }
}

#[cfg(feature = "overlay")]
fn draw_rect_buf(buf: &mut [u32], x: u32, y: u32, w: u32, h: u32, color: u32) {
    let x = x.min(LCD_WIDTH);
    let y = y.min(LCD_HEIGHT);
    let w = w.min(LCD_WIDTH.saturating_sub(x));
    let h = h.min(LCD_HEIGHT.saturating_sub(y));
    if w == 0 || h == 0 { return; }
    let stride = LCD_WIDTH as usize;
    for row in 0..h {
        let base = (y + row) as usize * stride + x as usize;
        for col in 0..w {
            buf[base + col as usize] = color;
        }
    }
}

// 7-segment digit at (x,y), scaled by s
#[cfg(feature = "overlay")]
fn draw_digit_buf(buf: &mut [u32], x: u32, y: u32, digit: u8, s: u32, color: u32) {
    let t = s;           // thickness
    let lh = 4 * s;      // vertical segment length
    let lw = 4 * s;      // horizontal segment length
    // top-left of digit area
    // Segments positions
    //  A
    // F B
    //  G
    // E C
    //  D
    let ax = x + s;           let ay = y;
    let bx = x + 5 * s;       let by = y + s;
    let cx = x + 5 * s;       let cy = y + 5 * s;
    let dx = x + s;           let dy = y + 9 * s;
    let ex = x;               let ey = y + 5 * s;
    let fx = x;               let fy = y + s;
    let gx = x + s;           let gy = y + 4 * s;

    // segment mask per digit: A B C D E F G (bit 6..0)
    let mask = match digit {
        0 => 0b1111110,
        1 => 0b0110000,
        2 => 0b1101101,
        3 => 0b1111001,
        4 => 0b0110011,
        5 => 0b1011011,
        6 => 0b1011111,
        7 => 0b1110000,
        8 => 0b1111111,
        9 => 0b1111011,
        _ => 0,
    };
    if (mask & (1 << 6)) != 0 { draw_seg_h_buf(buf, ax, ay, lw, t, color); } // A
    if (mask & (1 << 5)) != 0 { draw_seg_v_buf(buf, bx, by, lh, t, color); } // B
    if (mask & (1 << 4)) != 0 { draw_seg_v_buf(buf, cx, cy, lh, t, color); } // C
    if (mask & (1 << 3)) != 0 { draw_seg_h_buf(buf, dx, dy, lw, t, color); } // D
    if (mask & (1 << 2)) != 0 { draw_seg_v_buf(buf, ex, ey, lh, t, color); } // E
    if (mask & (1 << 1)) != 0 { draw_seg_v_buf(buf, fx, fy, lh, t, color); } // F
    if (mask & (1 << 0)) != 0 { draw_seg_h_buf(buf, gx, gy, lw, t, color); } // G
}

#[cfg(feature = "overlay")]
fn draw_seg_h_buf(buf: &mut [u32], x: u32, y: u32, len: u32, thick: u32, color: u32) {
    draw_rect_buf(buf, x, y, len, thick, color);
}
#[cfg(feature = "overlay")]
fn draw_seg_v_buf(buf: &mut [u32], x: u32, y: u32, len: u32, thick: u32, color: u32) {
    draw_rect_buf(buf, x, y, thick, len, color);
}

#[cfg(feature = "overlay")]
pub fn draw_fps_overlay(fps: u32) {
    // Draw onto the back buffer, then present at VBlank to avoid mid-scan writes
    let back_addr = lcd::layer1_back_addr();
    let buf = fb1_mut_at(back_addr);

    let s = 2; // scale
    let x0 = 4; let y0 = 4;
    let box_w = 6 * s * 3 + s * 4; // 3 digits + spacing
    let box_h = 10 * s + s * 2;
    draw_rect_buf(buf, x0, y0, box_w, box_h, 0xC0000000);

    let v = fps.min(999);
    let d2 = (v / 100) as u8;
    let d1 = ((v / 10) % 10) as u8;
    let d0 = (v % 10) as u8;
    let white = 0xFFFFFFFF;
    let mut x = x0 + s;
    if v >= 100 { draw_digit_buf(buf, x, y0 + s, d2, s, white); }
    x += 6 * s + s;
    if v >= 10 { draw_digit_buf(buf, x, y0 + s, d1, s, white); }
    x += 6 * s + s;
    draw_digit_buf(buf, x, y0 + s, d0, s, white);

    // Present updated back buffer during vertical blanking
    lcd::swap_layer1_buffers();
}
