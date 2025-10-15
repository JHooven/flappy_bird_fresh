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

// Image rotation options (can be combined with flips)
#[derive(Copy, Clone)]
pub enum ImageRotation {
    None,               // No rotation
    Clockwise90,        // 90° clockwise rotation
    CounterClockwise90, // 90° counter-clockwise rotation
    Rotate180,          // 180° rotation
}

// Image flip options (can be combined with rotation)
#[derive(Copy, Clone)]
pub struct ImageFlip {
    pub horizontal: bool, // Flip horizontally (mirror left-right)
    pub vertical: bool,   // Flip vertically (mirror top-bottom)
}

impl ImageFlip {
    pub const NONE: ImageFlip = ImageFlip {
        horizontal: false,
        vertical: false,
    };
    pub const HORIZONTAL: ImageFlip = ImageFlip {
        horizontal: true,
        vertical: false,
    };
    pub const VERTICAL: ImageFlip = ImageFlip {
        horizontal: false,
        vertical: true,
    };
    pub const BOTH: ImageFlip = ImageFlip {
        horizontal: true,
        vertical: true,
    };
}

// Image transformation combines rotation and flip
#[derive(Copy, Clone)]
pub struct ImageTransform {
    pub rotation: ImageRotation,
    pub flip: ImageFlip,
}

impl ImageTransform {
    pub const NONE: ImageTransform = ImageTransform {
        rotation: ImageRotation::None,
        flip: ImageFlip::NONE,
    };
    pub const FLIP_H: ImageTransform = ImageTransform {
        rotation: ImageRotation::None,
        flip: ImageFlip::HORIZONTAL,
    };
    pub const FLIP_V: ImageTransform = ImageTransform {
        rotation: ImageRotation::None,
        flip: ImageFlip::VERTICAL,
    };
    pub const CW90_FLIP_H: ImageTransform = ImageTransform {
        rotation: ImageRotation::Clockwise90,
        flip: ImageFlip::HORIZONTAL,
    };
}

// Legacy constants for backward compatibility
pub const ROTATION_NONE: u8 = 0;
pub const ROTATION_CLOCKWISE_90: u8 = 1;
pub const ROTATION_COUNTER_CLOCKWISE_90: u8 = 2;
pub const ROTATION_180: u8 = 3;
pub const ROTATION_FLIP_HORIZONTAL: u8 = 4;
pub const ROTATION_FLIP_VERTICAL: u8 = 5;
pub const ROTATION_FLIP_BOTH: u8 = 6;
pub const ROTATION_CLOCKWISE_90_FLIP_H: u8 = 7;

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

// Font definitions using actual font data
pub static FONT_7X10: crate::assets::fonts::Font = crate::assets::fonts::Font7x10;
pub static FONT_11X18: crate::assets::fonts::Font = crate::assets::fonts::Font11x18;
pub static FONT_16X26: crate::assets::fonts::Font = crate::assets::fonts::Font16x26;

pub struct Display {
    lcd_driver: LcdDriver,
    orientation: DisplayOrientation,
}

// FontDef removed - now using crate::assets::fonts::Font instead

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
    /*pub fn draw_image(&self, x: Coord, w: u32, y: Coord, h: u32, image_data: &[u16]) {
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
            self.draw_image_to_framebuffer(
                x as u32,
                y as u32,
                w as u32,
                h as u32,
                image_data,
                ImageTransform::NONE,
            );
        }
    */
    // Draw image with transform support (rotation + flip)
    pub fn draw_image_transformed(
        &self,
        x: Coord,
        w: u32,
        y: Coord,
        h: u32,
        image_data: &[u16],
        transform: ImageTransform,
    ) {
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

        // Write directly to LTDC Layer 2 framebuffer with transform
        self.draw_image_to_framebuffer(
            x as u32, y as u32, w as u32, h as u32, image_data, transform,
        );
    }

    // Convenience function with separate rotation and flip parameters
    pub fn draw_image_rotated_flipped(
        &self,
        x: Coord,
        w: u32,
        y: Coord,
        h: u32,
        image_data: &[u16],
        rotation: ImageRotation,
        flip_h: bool,
        flip_v: bool,
    ) {
        let transform = ImageTransform {
            rotation,
            flip: ImageFlip {
                horizontal: flip_h,
                vertical: flip_v,
            },
        };
        self.draw_image_transformed(x, w, y, h, image_data, transform);
    }

    // Backward compatibility function
    pub fn draw_image_rotated(
        &self,
        x: Coord,
        w: u32,
        y: Coord,
        h: u32,
        image_data: &[u16],
        rotation: ImageRotation,
    ) {
        let transform = ImageTransform {
            rotation,
            flip: ImageFlip::NONE,
        };
        self.draw_image_transformed(x, w, y, h, image_data, transform);
    }

    // Helper function to draw image to LTDC Layer 1 framebuffer with rotation and flip support
    fn draw_image_to_framebuffer(
        &self,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        image_data: &[u16],
        transform: ImageTransform,
    ) {
        use crate::config::{GAME_HEIGHT, GAME_WIDTH};
        use crate::lcd::{LAYER1_BASE, LCD_WIDTH};
        use core::slice;

        // Get Layer 1 framebuffer as ARGB8888 buffer
        let framebuffer = unsafe {
            slice::from_raw_parts_mut(
                LAYER1_BASE as *mut u32,
                (LCD_WIDTH * crate::lcd::LCD_HEIGHT) as usize,
            )
        };

        for row in 0..h {
            for col in 0..w {
                // Game coordinates (landscape)
                let game_x = x + col;
                let game_y = y + row;

                // Bounds check in game coordinates
                if game_x >= GAME_WIDTH || game_y >= GAME_HEIGHT {
                    continue;
                }

                // Transform from game coordinates (320x240 landscape) to LCD coordinates (240x320 portrait)
                // 90° counter-clockwise rotation: game_x,game_y → lcd_x=game_y, lcd_y=(GAME_WIDTH-1-game_x)
                let lcd_x = game_y;
                let lcd_y = GAME_WIDTH - 1 - game_x;

                // Bounds check in LCD coordinates
                if lcd_x >= LCD_WIDTH || lcd_y >= crate::lcd::LCD_HEIGHT {
                    continue;
                }

                // Apply transformation: first rotation, then flip
                let (mut img_row, mut img_col) = match transform.rotation {
                    ImageRotation::None => (row, col),
                    ImageRotation::Clockwise90 => (w - 1 - col, row),
                    ImageRotation::CounterClockwise90 => (col, h - 1 - row),
                    ImageRotation::Rotate180 => (h - 1 - row, w - 1 - col),
                };

                // Apply flips
                if transform.flip.horizontal {
                    img_col = w - 1 - img_col;
                }
                if transform.flip.vertical {
                    img_row = h - 1 - img_row;
                }
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

                // Write to framebuffer using LCD coordinates
                let fb_index = (lcd_y * LCD_WIDTH + lcd_x) as usize;
                framebuffer[fb_index] = argb8888;
            }
        }

        // Memory barrier to ensure writes complete
        cortex_m::asm::dsb();
    }

    // Fill screen with color (ported from gc9a01a_fill_screen)
    pub fn set_background_color(&self, bg_color: u16) {
        self.fill_rect(0, GAME_WIDTH as u16, 0, GAME_HEIGHT as u16, bg_color);
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
        use crate::config::{GAME_HEIGHT, GAME_WIDTH};

        let mut x: u16 = x.try_into().expect("X co-ordinate is out of range");
        let mut y: u16 = y.try_into().expect("y co-ordinate is out of range");

        if let Ok(rust_str) = c_str.to_str() {
            for ch in rust_str.chars() {
                // Handle line wrapping using game coordinates
                if x + FONT_16X26.width as u16 >= GAME_WIDTH as u16 {
                    x = 0;
                    y += FONT_16X26.height as u16;
                    if y + FONT_16X26.height as u16 >= GAME_HEIGHT as u16 {
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
    fn write_char(
        &self,
        x: u16,
        y: u16,
        ch: u8,
        font: crate::assets::fonts::Font,
        color: u16,
        bgcolor: u16,
    ) {
        use crate::config::{GAME_HEIGHT, GAME_WIDTH};
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
            // Get font data for this character row
            let char_index = (ch - 32) as usize; // ASCII printable characters start at 32
            let row_index = char_index * font.height as usize + i as usize;

            let b = if ch >= 32 && ch <= 126 && row_index < font.data.len() {
                font.data[row_index]
            } else {
                // Fallback: draw a solid block for any character issues
                0xFFFF // All bits set = solid block
            };

            for j in 0..font.width {
                // Game coordinates (landscape)
                let game_x = x as u32 + j as u32;
                let game_y = y as u32 + i as u32;

                // Bounds check in game coordinates
                if game_x >= GAME_WIDTH || game_y >= GAME_HEIGHT {
                    continue;
                }

                // Transform from game coordinates to LCD coordinates (90° CCW rotation)
                let lcd_x = game_y;
                let lcd_y = GAME_WIDTH - 1 - game_x;

                // Bounds check in LCD coordinates
                if lcd_x >= LCD_WIDTH || lcd_y >= crate::lcd::LCD_HEIGHT {
                    continue;
                }

                // Check if this pixel should be drawn (bit test from LSB, left to right)
                let bit_position = j; // For 16-bit font: bit 0 = leftmost, bit 15 = rightmost
                let pixel_color = if (b & (1 << bit_position)) != 0 {
                    color
                } else {
                    bgcolor
                };

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

                // Write to framebuffer using LCD coordinates
                let fb_index = (lcd_y * LCD_WIDTH + lcd_x) as usize;
                framebuffer[fb_index] = argb8888;
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
        use crate::config::{GAME_HEIGHT, GAME_WIDTH};
        use crate::lcd::{LAYER1_BASE, LCD_WIDTH};
        use core::slice;

        // Bounds checking using game coordinates
        if x >= GAME_WIDTH as u16 || y >= GAME_HEIGHT as u16 {
            return;
        }

        let w = if (x + w - 1) >= GAME_WIDTH as u16 {
            GAME_WIDTH as u16 - x
        } else {
            w
        };

        let h = if (y + h - 1) >= GAME_HEIGHT as u16 {
            GAME_HEIGHT as u16 - y
        } else {
            h
        };

        // Get Layer 1 framebuffer as ARGB8888 buffer
        let framebuffer = unsafe {
            slice::from_raw_parts_mut(
                LAYER1_BASE as *mut u32,
                (LCD_WIDTH * crate::lcd::LCD_HEIGHT) as usize,
            )
        };

        // Convert RGB565 to ARGB8888
        let r = ((color >> 11) & 0x1F) as u32;
        let g = ((color >> 5) & 0x3F) as u32;
        let b = (color & 0x1F) as u32;

        // Scale to 8-bit values
        let r8 = (r * 255 / 31) as u32;
        let g8 = (g * 255 / 63) as u32;
        let b8 = (b * 255 / 31) as u32;

        // Create ARGB8888 pixel (fully opaque)
        let argb8888 = 0xFF000000 | (r8 << 16) | (g8 << 8) | b8;

        // Fill rectangle using coordinate transformation
        for row in 0..h {
            for col in 0..w {
                // Game coordinates
                let game_x = x as u32 + col as u32;
                let game_y = y as u32 + row as u32;

                // Bounds check in game coordinates
                if game_x >= GAME_WIDTH || game_y >= GAME_HEIGHT {
                    continue;
                }

                // Transform from game coordinates to LCD coordinates (90° CCW rotation)
                let lcd_x = game_y;
                let lcd_y = GAME_WIDTH - 1 - game_x;

                // Bounds check in LCD coordinates
                if lcd_x >= LCD_WIDTH || lcd_y >= crate::lcd::LCD_HEIGHT {
                    continue;
                }

                // Write to framebuffer using LCD coordinates
                let fb_index = (lcd_y * LCD_WIDTH + lcd_x) as usize;
                framebuffer[fb_index] = argb8888;
            }
        }

        // Memory barrier to ensure writes complete
        cortex_m::asm::dsb();
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
/*pub fn draw_image(x: Coord, w: u32, y: Coord, h: u32, image_data: *const u16) {
    let image_data = unsafe { core::slice::from_raw_parts(image_data, (w * h) as usize) };
    let display = get_display();
    display.draw_image(x, w, y, h, image_data);
}
*/
#[no_mangle]
pub fn draw_image_rotated(
    x: Coord,
    w: u32,
    y: Coord,
    h: u32,
    image_data: *const u16,
    rotation: u8,
) {
    let image_data = unsafe { core::slice::from_raw_parts(image_data, (w * h) as usize) };
    let transform = match rotation {
        0 => ImageTransform::NONE, // No rotation, no flip
        1 => ImageTransform {
            rotation: ImageRotation::Clockwise90,
            flip: ImageFlip::NONE,
        },
        2 => ImageTransform {
            rotation: ImageRotation::CounterClockwise90,
            flip: ImageFlip::NONE,
        },
        3 => ImageTransform {
            rotation: ImageRotation::Rotate180,
            flip: ImageFlip::NONE,
        },
        4 => ImageTransform::FLIP_H, // Horizontal flip only
        5 => ImageTransform::FLIP_V, // Vertical flip only
        6 => ImageTransform {
            rotation: ImageRotation::None,
            flip: ImageFlip::BOTH,
        },
        7 => ImageTransform::CW90_FLIP_H, // 90° CW + horizontal flip
        _ => ImageTransform::NONE,
    };
    let display = get_display();
    display.draw_image_transformed(x, w, y, h, image_data, transform);
}

// New C-style function with separate rotation and flip parameters
#[no_mangle]
pub fn draw_image_rotated_flipped(
    x: Coord,
    w: u32,
    y: Coord,
    h: u32,
    image_data: *const u16,
    rotation: u8,
    flip_h: bool,
    flip_v: bool,
) {
    let image_data = unsafe { core::slice::from_raw_parts(image_data, (w * h) as usize) };
    let img_rotation = match rotation {
        0 => ImageRotation::None,
        1 => ImageRotation::Clockwise90,
        2 => ImageRotation::CounterClockwise90,
        3 => ImageRotation::Rotate180,
        _ => ImageRotation::None,
    };
    let display = get_display();
    display.draw_image_rotated_flipped(x, w, y, h, image_data, img_rotation, flip_h, flip_v);
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
/*pub fn draw_image_rust(x: Coord, w: u32, y: Coord, h: u32, image_data: &[u16]) {
    let display = get_display();
    display.draw_image(x, w, y, h, image_data);
}
*/

pub fn draw_image_rotated_rust(
    x: Coord,
    w: u32,
    y: Coord,
    h: u32,
    image_data: &[u16],
    rotation: ImageRotation,
) {
    let display = get_display();
    display.draw_image_rotated(x, w, y, h, image_data, rotation);
}

// New Rust wrapper with separate rotation and flip
pub fn draw_image_rotated_flipped_rust(
    x: Coord,
    w: u32,
    y: Coord,
    h: u32,
    image_data: &[u16],
    rotation: ImageRotation,
    flip_h: bool,
    flip_v: bool,
) {
    let display = get_display();
    display.draw_image_rotated_flipped(x, w, y, h, image_data, rotation, flip_h, flip_v);
}

// New Rust wrapper with ImageTransform
pub fn draw_image_transformed_rust(
    x: Coord,
    w: u32,
    y: Coord,
    h: u32,
    image_data: &[u16],
    transform: ImageTransform,
) {
    let display = get_display();
    display.draw_image_transformed(x, w, y, h, image_data, transform);
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
