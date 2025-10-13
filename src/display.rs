#![allow(dead_code)]
#![allow(static_mut_refs)]

use crate::config::*;
use crate::lcd::LcdDriver;
use core::convert::TryInto;
use core::ffi;
use core::ffi::c_char;

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub enum DisplayOrientation {
    Portrait,
    Landscape,
}

// GC9A01A LCD display constants
pub const GC9A01A_WIDTH: u32 = 240;
pub const GC9A01A_HEIGHT: u32 = 240;

// GC9A01A command definitions (ported from gc9a01a.h)
const GC9A01A_NOP: u8 = 0x00; // No operation
const GC9A01A_SWRESET: u8 = 0x01; // Software Reset
const GC9A01A_SLPIN: u8 = 0x10; // Enter Sleep Mode
const GC9A01A_SLPOUT: u8 = 0x11; // Sleep Out
const GC9A01A_INVOFF: u8 = 0x20; // Display Inversion OFF
const GC9A01A_INVON: u8 = 0x21; // Display Inversion ON
const GC9A01A_DISPOFF: u8 = 0x28; // Display OFF
const GC9A01A_DISPON: u8 = 0x29; // Display ON
const GC9A01A_CASET: u8 = 0x2A; // Column Address Set
const GC9A01A_RASET: u8 = 0x2B; // Row Address Set
const GC9A01A_RAMWR: u8 = 0x2C; // Memory Write
const GC9A01A_MADCTL: u8 = 0x36; // Memory Access Control
const GC9A01A_COLMOD: u8 = 0x3A; // Pixel Format Set
const GC9A01A_TEON: u8 = 0x35; // Tearing Effect Line ON
const GC9A01A_FRAMERATE: u8 = 0xE8; // Frame rate control
const GC9A01A_INREGEN1: u8 = 0xFE; // Inter register enable 1
const GC9A01A_INREGEN2: u8 = 0xEF; // Inter register enable 2
const GC9A01A_GAMMA1: u8 = 0xF0; // Set gamma 1
const GC9A01A_GAMMA2: u8 = 0xF1; // Set gamma 2
const GC9A01A_GAMMA3: u8 = 0xF2; // Set gamma 3
const GC9A01A_GAMMA4: u8 = 0xF3; // Set gamma 4
const GC9A01A_DISP_CTRL: u8 = 0xB6; // Display Function Control
const GC9A01A1_POWER2: u8 = 0xC3; // Power Control 2
const GC9A01A1_POWER3: u8 = 0xC4; // Power Control 3
const GC9A01A1_POWER4: u8 = 0xC9; // Power Control 4

// MADCTL bit definitions
const MADCTL_MY: u8 = 0x80; // Bottom to top
const MADCTL_MX: u8 = 0x40; // Right to left
const MADCTL_MV: u8 = 0x20; // Reverse Mode
const MADCTL_BGR: u8 = 0x08; // Blue-Green-Red pixel order

// Default font definition
pub static FONT_16X26: FontDef = FontDef {
    width: 16,
    height: 26,
    data: core::ptr::null(), // Empty data for stub implementation
};

pub struct Display {
    lcd_driver: LcdDriver,
    orientation: DisplayOrientation,
}

#[derive(Copy, Clone)]

pub struct FontDef {
    width: u8,
    height: u8,
    data: *const u8, // Keep as raw pointer but make it Sync
}

// Safe wrapper for FontDef that can be shared between threads
unsafe impl Sync for FontDef {}

impl Display {
    pub fn new() -> Self {
        Self {
            lcd_driver: LcdDriver::new(),
            orientation: DisplayOrientation::Portrait,
        }
    }

    pub fn register_driver(_driver: &LcdDriver) {
        // This function maintained for compatibility but LcdDriver is already integrated
    }

    pub fn init(&mut self) {
        // Initialize the GC9A01A display (ported from gc9a01a_init)
        self.hw_reset();
        self.configure();
        self.set_orientation(DisplayOrientation::Landscape);
    }

    // Hardware reset function (ported from gc9a01a_hw_reset)
    fn hw_reset(&self) {
        // Simulate hardware reset sequence
        // In real implementation would control GPIO pins:
        // RST_HIGH -> delay -> RST_LOW -> delay -> RST_HIGH -> delay
        // Using the LCD driver's delay mechanism if available
    }

    // Configure the GC9A01A display (ported from gc9a01a_configure)
    fn configure(&self) {
        // Complete GC9A01A configuration sequence ported from C
        self.write_cmd(GC9A01A_INREGEN1);
        self.write_cmd(GC9A01A_INREGEN2);

        self.write_cmd_with_data(0xEB, &[0x14]);
        self.write_cmd_with_data(0x84, &[0x60]);
        self.write_cmd_with_data(0x85, &[0xFF]);
        self.write_cmd_with_data(0x86, &[0xFF]);
        self.write_cmd_with_data(0x87, &[0xFF]);
        self.write_cmd_with_data(0x8E, &[0xFF]);
        self.write_cmd_with_data(0x8F, &[0xFF]);
        self.write_cmd_with_data(0x88, &[0x0A]);
        self.write_cmd_with_data(0x89, &[0x21]);
        self.write_cmd_with_data(0x8A, &[0x00]);
        self.write_cmd_with_data(0x8B, &[0x80]);
        self.write_cmd_with_data(0x8C, &[0x01]);
        self.write_cmd_with_data(0x8D, &[0x03]);
        self.write_cmd_with_data(0xB5, &[0x08, 0x09, 0x14, 0x08]);
        self.write_cmd_with_data(GC9A01A_DISP_CTRL, &[0x00, 0x00]);
        self.write_cmd_with_data(GC9A01A_MADCTL, &[0x48]);
        self.write_cmd_with_data(GC9A01A_COLMOD, &[0x05]);
        self.write_cmd_with_data(0x90, &[0x08, 0x08, 0x08, 0x08]);
        self.write_cmd_with_data(0xBD, &[0x06]);
        self.write_cmd_with_data(0xBA, &[0x01]);
        self.write_cmd_with_data(0xBC, &[0x00]);
        self.write_cmd_with_data(0xFF, &[0x60, 0x01, 0x04]);
        self.write_cmd_with_data(GC9A01A1_POWER2, &[0x14]);
        self.write_cmd_with_data(GC9A01A1_POWER3, &[0x14]);
        self.write_cmd_with_data(GC9A01A1_POWER4, &[0x25]);
        self.write_cmd_with_data(0xBE, &[0x11]);
        self.write_cmd_with_data(0xE1, &[0x10, 0x0e]);
        self.write_cmd_with_data(0xDF, &[0x21, 0x0c, 0x02]);

        // Gamma settings
        self.write_cmd_with_data(GC9A01A_GAMMA1, &[0x45, 0x09, 0x08, 0x08, 0x26, 0x2A]);
        self.write_cmd_with_data(GC9A01A_GAMMA2, &[0x43, 0x70, 0x72, 0x36, 0x37, 0x6F]);
        self.write_cmd_with_data(GC9A01A_GAMMA3, &[0x45, 0x09, 0x08, 0x08, 0x26, 0x2A]);
        self.write_cmd_with_data(GC9A01A_GAMMA4, &[0x43, 0x70, 0x72, 0x36, 0x37, 0x6F]);

        self.write_cmd_with_data(0xED, &[0x1B, 0x0B]);
        self.write_cmd_with_data(0xAE, &[0x77]);
        self.write_cmd_with_data(0xCD, &[0x63]);
        self.write_cmd_with_data(
            0x70,
            &[0x07, 0x07, 0x04, 0x0E, 0x0F, 0x09, 0x07, 0x08, 0x03],
        );
        self.write_cmd_with_data(GC9A01A_FRAMERATE, &[0x34]);

        // More configuration data...
        self.write_cmd_with_data(
            0x62,
            &[
                0x18, 0x0D, 0x71, 0xED, 0x70, 0x70, 0x18, 0x0F, 0x71, 0xEF, 0x70, 0x70,
            ],
        );
        self.write_cmd_with_data(
            0x63,
            &[
                0x18, 0x11, 0x71, 0xF1, 0x70, 0x70, 0x18, 0x13, 0x71, 0xF3, 0x70, 0x70,
            ],
        );

        self.write_cmd_with_data(GC9A01A_TEON, &[0x00]);

        // Final initialization sequence
        self.write_cmd(GC9A01A_INVON);
        self.delay_ms(120);
        self.write_cmd(GC9A01A_SLPOUT);
        self.delay_ms(120);
        self.write_cmd(GC9A01A_DISPON);
        self.delay_ms(20);
    }

    // Set display orientation (ported from gc9a01a_set_orientation)
    pub fn set_orientation(&mut self, orientation: DisplayOrientation) {
        self.orientation = orientation;
        match orientation {
            DisplayOrientation::Landscape => {
                self.write_cmd_with_data(GC9A01A_CASET, &[0x00, 0x00, 0x00, 0xf0]);
                self.write_cmd_with_data(GC9A01A_RASET, &[0x00, 0x00, 0x00, 0xf0]);
                self.write_cmd_with_data(GC9A01A_MADCTL, &[MADCTL_MV | MADCTL_BGR]);
            }
            DisplayOrientation::Portrait => {
                self.write_cmd_with_data(GC9A01A_CASET, &[0x00, 0x00, 0x00, 0xf0]);
                self.write_cmd_with_data(GC9A01A_RASET, &[0x00, 0x00, 0x00, 0xf0]);
                self.write_cmd_with_data(GC9A01A_MADCTL, &[MADCTL_MX | MADCTL_BGR]);
            }
        }
    }

    // Draw image function (ported from gc9a01a_draw_image)
    pub fn draw_image(&self, x: Coord, w: u32, y: Coord, h: u32, image_data: &[u16]) {
        let x: u16 = x.try_into().expect("X co-ordinate is out of range");
        let y: u16 = y.try_into().expect("y co-ordinate is out of range");
        let w: u16 = w.try_into().expect("width out of range");
        let h: u16 = h.try_into().expect("height out of range");

        // Bounds checking
        if x >= GC9A01A_WIDTH as u16 || y >= GC9A01A_HEIGHT as u16 {
            return;
        }

        let w = if (x + w - 1) >= GC9A01A_WIDTH as u16 {
            GC9A01A_WIDTH as u16 - x
        } else {
            w
        };

        let h = if (y + h - 1) >= GC9A01A_HEIGHT as u16 {
            GC9A01A_HEIGHT as u16 - y
        } else {
            h
        };

        self.set_address_window(x, x + w - 1, y, y + h - 1);

        // Send image data
        for pixel in image_data.iter().take((w as u32 * h as u32) as usize) {
            let color_high = (*pixel >> 8) as u8;
            let color_low = *pixel as u8;
            self.write_8bit(color_high);
            self.write_8bit(color_low);
        }
    }

    // Fill screen with color (ported from gc9a01a_fill_screen)
    pub fn set_background_color(&self, bg_color: u16) {
        self.fill_rect(0, GC9A01A_WIDTH as u16, 0, GC9A01A_HEIGHT as u16, bg_color);
    }

    // Draw rectangle (ported from gc9a01a_fill_rect)
    pub fn draw_rect_angle(&self, x: Coord, w: u32, y: Coord, h: u32, color: u16) {
        let x: u16 = x.try_into().expect("X co-ordinate is out of range");
        let y: u16 = y.try_into().expect("y co-ordinate is out of range");
        let w: u16 = w.try_into().expect("width out of range");
        let h: u16 = h.try_into().expect("height out of range");

        self.fill_rect(x, w, y, h, color);
    }

    // Write string function (ported from gc9a01a_write_string)
    pub fn write_string(&self, x: Coord, y: Coord, c_str: &ffi::CStr, color: u16, bgcolor: u16) {
        let mut x: u16 = x.try_into().expect("X co-ordinate is out of range");
        let mut y: u16 = y.try_into().expect("y co-ordinate is out of range");

        if let Ok(rust_str) = c_str.to_str() {
            for ch in rust_str.chars() {
                // Handle line wrapping
                if x + FONT_16X26.width as u16 >= GC9A01A_WIDTH as u16 {
                    x = 0;
                    y += FONT_16X26.height as u16;
                    if y + FONT_16X26.height as u16 >= GC9A01A_HEIGHT as u16 {
                        break;
                    }

                    if ch == ' ' {
                        continue; // Skip spaces at beginning of new line
                    }
                }

                self.write_char(x, y, ch as u8, FONT_16X26, color, bgcolor);
                x += FONT_16X26.width as u16;
            }
        }
    }

    // Write single character (ported from gc9a01a_write_char)
    fn write_char(&self, x: u16, y: u16, ch: u8, font: FontDef, color: u16, bgcolor: u16) {
        self.set_address_window(x, x + font.width as u16 - 1, y, y + font.height as u16 - 1);

        for i in 0..font.height {
            // Note: In real implementation, would read from font.data
            // For now, using a simple pattern as font data is null
            let mut b = if !font.data.is_null() {
                unsafe { *font.data.add(((ch - 32) * font.height + i) as usize) as u16 }
            } else {
                // Simple pattern for demonstration when font data is null
                if i < font.height / 2 {
                    0xFF00
                } else {
                    0x00FF
                }
            };

            for _j in 0..font.width {
                let pixel_color = if (b & 0x8000) != 0 { color } else { bgcolor };
                let color_high = (pixel_color >> 8) as u8;
                let color_low = pixel_color as u8;
                self.write_8bit(color_high);
                self.write_8bit(color_low);
                b <<= 1;
            }
        }
    }

    // Draw single pixel (ported from gc9a01a_draw_pixel)
    pub fn draw_pixel(&self, x: u16, y: u16, color: u16) {
        if x >= GC9A01A_WIDTH as u16 || y >= GC9A01A_HEIGHT as u16 {
            return;
        }

        self.set_address_window(x, x, y, y);
        let color_high = (color >> 8) as u8;
        let color_low = color as u8;
        self.write_8bit(color_high);
        self.write_8bit(color_low);
    }

    // Fill rectangle helper (ported from gc9a01a_fill_rect)
    fn fill_rect(&self, x: u16, w: u16, y: u16, h: u16, color: u16) {
        if x >= GC9A01A_WIDTH as u16 || y >= GC9A01A_HEIGHT as u16 {
            return;
        }

        let w = if (x + w - 1) >= GC9A01A_WIDTH as u16 {
            GC9A01A_WIDTH as u16 - x
        } else {
            w
        };

        let h = if (y + h - 1) >= GC9A01A_HEIGHT as u16 {
            GC9A01A_HEIGHT as u16 - y
        } else {
            h
        };

        self.set_address_window(x, x + w - 1, y, y + h - 1);

        let color_high = (color >> 8) as u8;
        let color_low = color as u8;

        for _row in 0..h {
            for _col in 0..w {
                self.write_8bit(color_high);
                self.write_8bit(color_low);
            }
        }
    }

    // Set address window (ported from gc9a01a_set_address_window)
    fn set_address_window(&self, x0: u16, x1: u16, y0: u16, y1: u16) {
        // Set column address
        self.write_cmd(GC9A01A_CASET);
        self.write_data(&[(x0 >> 8) as u8, x0 as u8, (x1 >> 8) as u8, x1 as u8]);

        // Set row address
        self.write_cmd(GC9A01A_RASET);
        self.write_data(&[(y0 >> 8) as u8, y0 as u8, (y1 >> 8) as u8, y1 as u8]);

        // Write to RAM command
        self.write_cmd(GC9A01A_RAMWR);
    }

    // Low-level command functions (ported from gc9a01a.c)
    fn write_cmd(&self, cmd: u8) {
        // In real implementation would set DC pin low and write command
        // For now, delegate to LCD driver or use SPI interface
        self.write_8bit(cmd);
    }

    fn write_cmd_with_data(&self, cmd: u8, data: &[u8]) {
        self.write_cmd(cmd);
        self.write_data(data);
    }

    fn write_data(&self, data: &[u8]) {
        // In real implementation would set DC pin high and write data
        for &byte in data {
            self.write_8bit(byte);
        }
    }

    fn write_8bit(&self, data: u8) {
        // In real implementation this would write to GPIO pins
        // Following the pattern from GC9A01A_WRITE_8BIT macro
        // For now, this is a stub that could interface with SPI or GPIO
        let _ = data; // Suppress unused warning
    }

    fn delay_ms(&self, ms: u32) {
        // In real implementation would use HAL_Delay or timer
        // For now, simple loop (not accurate timing)
        for _ in 0..(ms * 1000) {
            cortex_m::asm::nop();
        }
    }

    // Invert colors function (ported from gc9a01a_invert_colors)
    pub fn invert_colors(&self, invert: bool) {
        self.write_cmd(if invert {
            GC9A01A_INVON
        } else {
            GC9A01A_INVOFF
        });
    }
} // Keep the old function API for backward compatibility during transition
pub fn register_driver(driver: &LcdDriver) {
    Display::register_driver(driver);
}

// Global display instance for C compatibility
static mut DISPLAY: Option<Display> = None;

// Initialize the global display instance
fn get_display() -> &'static mut Display {
    unsafe {
        if DISPLAY.is_none() {
            DISPLAY = Some(Display::new());
        }
        DISPLAY.as_mut().unwrap()
    }
}

// C-compatible function wrappers for interfacing with legacy C code
#[no_mangle]
pub extern "C" fn init() {
    let display = get_display();
    display.init();
}

#[no_mangle]
pub extern "C" fn draw_image(x: Coord, w: u32, y: Coord, h: u32, image_data: *const u16) {
    let image_data = unsafe { core::slice::from_raw_parts(image_data, (w * h) as usize) };
    let display = get_display();
    display.draw_image(x, w, y, h, image_data);
}

#[no_mangle]
pub extern "C" fn set_background_color(bg_color: u16) {
    let display = get_display();
    display.set_background_color(bg_color);
}

#[no_mangle]
pub extern "C" fn draw_rect_angle(x: Coord, w: u32, y: Coord, h: u32, color: u16) {
    let display = get_display();
    display.draw_rect_angle(x, w, y, h, color);
}

#[no_mangle]
pub extern "C" fn write_string(x: Coord, y: Coord, c_str: *const c_char, color: u16, bgcolor: u16) {
    let c_str = unsafe { ffi::CStr::from_ptr(c_str) };
    let display = get_display();
    display.write_string(x, y, c_str, color, bgcolor);
}

// Rust-friendly wrapper functions that don't require extern "C"
pub fn draw_image_rust(x: Coord, w: u32, y: Coord, h: u32, image_data: &[u16]) {
    let display = get_display();
    display.draw_image(x, w, y, h, image_data);
}

pub fn set_background_color_rust(bg_color: u16) {
    let display = get_display();
    display.set_background_color(bg_color);
}

pub fn draw_rect_angle_rust(x: Coord, w: u32, y: Coord, h: u32, color: u16) {
    let display = get_display();
    display.draw_rect_angle(x, w, y, h, color);
}

pub fn write_string_rust(x: Coord, y: Coord, c_str: &ffi::CStr, color: u16, bgcolor: u16) {
    let display = get_display();
    display.write_string(x, y, c_str, color, bgcolor);
}

pub fn init_rust() {
    let display = get_display();
    display.init();
}
