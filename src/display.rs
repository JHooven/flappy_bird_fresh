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

// ILI9341 LCD display constants for STM32F429ZI Discovery board
pub const DISPLAY_WIDTH: u32 = 240;
pub const DISPLAY_HEIGHT: u32 = 320;

// ILI9341 command definitions (correct for STM32F429ZI Discovery board)
const ILI9341_NOP: u8 = 0x00; // No operation
const ILI9341_SWRESET: u8 = 0x01; // Software Reset
const ILI9341_RDDID: u8 = 0x04; // Read Display ID
const ILI9341_RDDST: u8 = 0x09; // Read Display Status
const ILI9341_SLPIN: u8 = 0x10; // Enter Sleep Mode
const ILI9341_SLPOUT: u8 = 0x11; // Sleep Out
const ILI9341_PTLON: u8 = 0x12; // Partial Mode ON
const ILI9341_NORON: u8 = 0x13; // Normal Display Mode ON
const ILI9341_INVOFF: u8 = 0x20; // Display Inversion OFF
const ILI9341_INVON: u8 = 0x21; // Display Inversion ON
const ILI9341_GAMMASET: u8 = 0x26; // Gamma Set
const ILI9341_DISPOFF: u8 = 0x28; // Display OFF
const ILI9341_DISPON: u8 = 0x29; // Display ON
const ILI9341_CASET: u8 = 0x2A; // Column Address Set
const ILI9341_PASET: u8 = 0x2B; // Page Address Set
const ILI9341_RAMWR: u8 = 0x2C; // Memory Write
const ILI9341_RAMRD: u8 = 0x2E; // Memory Read
const ILI9341_MADCTL: u8 = 0x36; // Memory Access Control
const ILI9341_PIXFMT: u8 = 0x3A; // Pixel Format Set
const ILI9341_FRMCTR1: u8 = 0xB1; // Frame Rate Control (In Normal Mode/Full Colors)
const ILI9341_FRMCTR2: u8 = 0xB2; // Frame Rate Control (In Idle Mode/8 colors)
const ILI9341_FRMCTR3: u8 = 0xB3; // Frame Rate Control (In Partial Mode/Full Colors)
const ILI9341_INVCTR: u8 = 0xB4; // Display Inversion Control
const ILI9341_DFUNCTR: u8 = 0xB6; // Display Function Control
const ILI9341_PWCTR1: u8 = 0xC0; // Power Control 1
const ILI9341_PWCTR2: u8 = 0xC1; // Power Control 2
const ILI9341_PWCTR3: u8 = 0xC2; // Power Control 3
const ILI9341_PWCTR4: u8 = 0xC3; // Power Control 4
const ILI9341_PWCTR5: u8 = 0xC4; // Power Control 5
const ILI9341_VMCTR1: u8 = 0xC5; // VCOM Control 1
const ILI9341_VMCTR2: u8 = 0xC7; // VCOM Control 2
const ILI9341_GMCTRP1: u8 = 0xE0; // Positive Gamma Correction
const ILI9341_GMCTRN1: u8 = 0xE1; // Negative Gamma Correction

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

    // Configure the ILI9341 display (proper initialization for STM32F429ZI Discovery board)
    fn configure(&self) {
        // ILI9341 initialization sequence for STM32F429ZI Discovery board
        self.write_cmd(ILI9341_SWRESET);
        self.delay_ms(200);

        // Power control A
        self.write_cmd_with_data(0xCB, &[0x39, 0x2C, 0x00, 0x34, 0x02]);

        // Power control B
        self.write_cmd_with_data(0xCF, &[0x00, 0xC1, 0x30]);

        // Driver timing control A
        self.write_cmd_with_data(0xE8, &[0x85, 0x00, 0x78]);

        // Driver timing control B
        self.write_cmd_with_data(0xEA, &[0x00, 0x00]);

        // Power on sequence control
        self.write_cmd_with_data(0xED, &[0x64, 0x03, 0x12, 0x81]);

        // Pump ratio control
        self.write_cmd_with_data(0xF7, &[0x20]);

        // Power control 1
        self.write_cmd_with_data(ILI9341_PWCTR1, &[0x23]);

        // Power control 2
        self.write_cmd_with_data(ILI9341_PWCTR2, &[0x10]);

        // VCOM control 1
        self.write_cmd_with_data(ILI9341_VMCTR1, &[0x3e, 0x28]);

        // VCOM control 2
        self.write_cmd_with_data(ILI9341_VMCTR2, &[0x86]);

        // Memory Access Control
        self.write_cmd_with_data(ILI9341_MADCTL, &[0x48]);

        // Pixel Format
        self.write_cmd_with_data(ILI9341_PIXFMT, &[0x55]);

        // Frame Rate Control
        self.write_cmd_with_data(ILI9341_FRMCTR1, &[0x00, 0x18]);

        // Display Function Control
        self.write_cmd_with_data(ILI9341_DFUNCTR, &[0x08, 0x82, 0x27]);

        // 3Gamma Function Disable
        self.write_cmd_with_data(0xF2, &[0x00]);

        // Gamma curve selected
        self.write_cmd_with_data(ILI9341_GAMMASET, &[0x01]);

        // Positive Gamma Correction
        self.write_cmd_with_data(
            ILI9341_GMCTRP1,
            &[
                0x0F, 0x31, 0x2B, 0x0C, 0x0E, 0x08, 0x4E, 0xF1, 0x37, 0x07, 0x10, 0x03, 0x0E, 0x09,
                0x00,
            ],
        );

        // Negative Gamma Correction
        self.write_cmd_with_data(
            ILI9341_GMCTRN1,
            &[
                0x00, 0x0E, 0x14, 0x03, 0x11, 0x07, 0x31, 0xC1, 0x48, 0x08, 0x0F, 0x0C, 0x31, 0x36,
                0x0F,
            ],
        );

        // Sleep out
        self.write_cmd(ILI9341_SLPOUT);
        self.delay_ms(120);

        // Display on
        self.write_cmd(ILI9341_DISPON);
        self.delay_ms(20);
    }

    // Set display orientation (ported from gc9a01a_set_orientation)
    pub fn set_orientation(&mut self, orientation: DisplayOrientation) {
        self.orientation = orientation;
        match orientation {
            DisplayOrientation::Landscape => {
                self.write_cmd_with_data(ILI9341_CASET, &[0x00, 0x00, 0x01, 0x3F]); // 0-319
                self.write_cmd_with_data(ILI9341_PASET, &[0x00, 0x00, 0x00, 0xEF]); // 0-239
                self.write_cmd_with_data(ILI9341_MADCTL, &[MADCTL_MV | MADCTL_BGR]);
            }
            DisplayOrientation::Portrait => {
                self.write_cmd_with_data(ILI9341_CASET, &[0x00, 0x00, 0x00, 0xEF]); // 0-239
                self.write_cmd_with_data(ILI9341_PASET, &[0x00, 0x00, 0x01, 0x3F]); // 0-319
                self.write_cmd_with_data(ILI9341_MADCTL, &[MADCTL_MX | MADCTL_BGR]);
            }
        }
    }

    // Draw image function (LTDC Layer 1 framebuffer approach for STM32F429ZI Discovery)
    pub fn draw_image(&self, x: Coord, w: u32, y: Coord, h: u32, image_data: &[u16]) {
        let x: u16 = x.try_into().expect("X co-ordinate is out of range");
        let y: u16 = y.try_into().expect("y co-ordinate is out of range");
        let w: u16 = w.try_into().expect("width out of range");
        let h: u16 = h.try_into().expect("height out of range");

        // Bounds checking
        if x >= DISPLAY_WIDTH as u16 || y >= DISPLAY_HEIGHT as u16 {
            return;
        }

        let w = if (x + w - 1) >= DISPLAY_WIDTH as u16 {
            DISPLAY_WIDTH as u16 - x
        } else {
            w
        };

        let h = if (y + h - 1) >= DISPLAY_HEIGHT as u16 {
            DISPLAY_HEIGHT as u16 - y
        } else {
            h
        };

        // Write directly to LTDC Layer 2 framebuffer
        self.draw_image_to_framebuffer(x as u32, y as u32, w as u32, h as u32, image_data);
    }

    // Helper function to draw image to LTDC Layer 1 framebuffer
    fn draw_image_to_framebuffer(&self, x: u32, y: u32, w: u32, h: u32, image_data: &[u16]) {
        use crate::lcd::{LAYER1_BASE, LCD_WIDTH};
        use core::slice;

        // Get Layer 1 framebuffer as ARGB8888 buffer
        let framebuffer = unsafe {
            slice::from_raw_parts_mut(
                LAYER1_BASE as *mut u32,
                (LCD_WIDTH * crate::lcd::LCD_HEIGHT) as usize,
            )
        };

        // Image orientation for STM32F429ZI Discovery board LTDC framebuffer
        // Mode 2 (vertical flip) is correct for proper text and image orientation
        // 0: Normal (row, col) - text appears upside down
        // 1: Horizontal flip (row, w-1-col) - text appears mirrored
        // 2: Vertical flip (h-1-row, col) - CORRECT orientation ✓
        // 3: Both flips (h-1-row, w-1-col) - text appears both upside down and mirrored
        let orientation_mode = 2; // Vertical flip - correct for STM32F429ZI Discovery

        for row in 0..h {
            for col in 0..w {
                let pixel_x = x + col;
                let pixel_y = y + row;

                // Bounds check
                if pixel_x >= LCD_WIDTH || pixel_y >= crate::lcd::LCD_HEIGHT {
                    continue;
                }

                // Calculate image data index based on orientation mode
                let (img_row, img_col) = match orientation_mode {
                    0 => (row, col),                 // Normal
                    1 => (row, w - 1 - col),         // Horizontal flip
                    2 => (h - 1 - row, col),         // Vertical flip
                    3 => (h - 1 - row, w - 1 - col), // Both flips
                    _ => (row, col),                 // Default to normal
                };
                let img_idx = (img_row * w + img_col) as usize;

                if img_idx >= image_data.len() {
                    continue;
                }

                // Convert RGB565 to ARGB8888
                let rgb565 = image_data[img_idx];
                let r = ((rgb565 >> 11) & 0x1F) as u32;
                let g = ((rgb565 >> 5) & 0x3F) as u32;
                let b = (rgb565 & 0x1F) as u32;

                // Scale to 8-bit values
                let r8 = (r * 255 / 31) as u32;
                let g8 = (g * 255 / 63) as u32;
                let b8 = (b * 255 / 31) as u32;

                // Create ARGB8888 pixel (fully opaque)
                let argb8888 = 0xFF000000 | (r8 << 16) | (g8 << 8) | b8;

                // Write to framebuffer
                let fb_index = (pixel_y * LCD_WIDTH + pixel_x) as usize;
                framebuffer[fb_index] = argb8888;
            }
        }

        // Memory barrier to ensure writes complete
        cortex_m::asm::dsb();
    }

    // Fill screen with color (ported from gc9a01a_fill_screen)
    pub fn set_background_color(&self, bg_color: u16) {
        self.fill_rect(0, DISPLAY_WIDTH as u16, 0, DISPLAY_HEIGHT as u16, bg_color);
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
                if x + FONT_16X26.width as u16 >= DISPLAY_WIDTH as u16 {
                    x = 0;
                    y += FONT_16X26.height as u16;
                    if y + FONT_16X26.height as u16 >= DISPLAY_HEIGHT as u16 {
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

    // Write single character (LTDC framebuffer approach for STM32F429ZI Discovery)
    fn write_char(&self, x: u16, y: u16, ch: u8, font: FontDef, color: u16, bgcolor: u16) {
        use crate::lcd::{LAYER1_BASE, LCD_WIDTH};
        use core::slice;

        // Get Layer 1 framebuffer as ARGB8888 buffer
        let framebuffer = unsafe {
            slice::from_raw_parts_mut(
                LAYER1_BASE as *mut u32,
                (LCD_WIDTH * crate::lcd::LCD_HEIGHT) as usize,
            )
        };

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

            for j in 0..font.width {
                let pixel_x = x as u32 + j as u32;
                let pixel_y = y as u32 + i as u32;

                // Bounds check
                if pixel_x >= LCD_WIDTH || pixel_y >= crate::lcd::LCD_HEIGHT {
                    continue;
                }

                let pixel_color = if (b & 0x8000) != 0 { color } else { bgcolor };

                // Convert RGB565 to ARGB8888
                let r = ((pixel_color >> 11) & 0x1F) as u32;
                let g = ((pixel_color >> 5) & 0x3F) as u32;
                let b_val = (pixel_color & 0x1F) as u32;

                // Scale to 8-bit values
                let r8 = (r * 255 / 31) as u32;
                let g8 = (g * 255 / 63) as u32;
                let b8 = (b_val * 255 / 31) as u32;

                // Create ARGB8888 pixel (fully opaque)
                let argb8888 = 0xFF000000 | (r8 << 16) | (g8 << 8) | b8;

                // Write to framebuffer
                let fb_index = (pixel_y * LCD_WIDTH + pixel_x) as usize;
                framebuffer[fb_index] = argb8888;

                b <<= 1;
            }
        }

        // Memory barrier to ensure writes complete
        cortex_m::asm::dsb();
    }

    // Draw single pixel (ported from gc9a01a_draw_pixel)
    pub fn draw_pixel(&self, x: u16, y: u16, color: u16) {
        if x >= DISPLAY_WIDTH as u16 || y >= DISPLAY_HEIGHT as u16 {
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
        if x >= DISPLAY_WIDTH as u16 || y >= DISPLAY_HEIGHT as u16 {
            return;
        }

        let w = if (x + w - 1) >= DISPLAY_WIDTH as u16 {
            DISPLAY_WIDTH as u16 - x
        } else {
            w
        };

        let h = if (y + h - 1) >= DISPLAY_HEIGHT as u16 {
            DISPLAY_HEIGHT as u16 - y
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

    // Set address window (ILI9341 compatible)
    fn set_address_window(&self, x0: u16, x1: u16, y0: u16, y1: u16) {
        // Set column address
        self.write_cmd(ILI9341_CASET);
        self.write_data(&[(x0 >> 8) as u8, x0 as u8, (x1 >> 8) as u8, x1 as u8]);

        // Set row address
        self.write_cmd(ILI9341_PASET);
        self.write_data(&[(y0 >> 8) as u8, y0 as u8, (y1 >> 8) as u8, y1 as u8]);

        // Write to RAM command
        self.write_cmd(ILI9341_RAMWR);
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

    // Invert colors function (ILI9341 compatible)
    pub fn invert_colors(&self, invert: bool) {
        self.write_cmd(if invert {
            ILI9341_INVON
        } else {
            ILI9341_INVOFF
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
