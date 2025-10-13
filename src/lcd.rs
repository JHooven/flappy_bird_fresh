#![allow(dead_code)]

use stm32f4::stm32f429 as pac;

pub struct LcdDriver {
    ltdc: pac::LTDC,
}

pub const LCD_WIDTH: u32 = 240;
pub const LCD_HEIGHT: u32 = 320;

// Exact timings from libopencm3 lcd-dma example
const HSYNC: u32 = 10;
const HBP: u32 = 20;
const HFP: u32 = 10;
const VSYNC: u32 = 2;
const VBP: u32 = 2;
const VFP: u32 = 4;

// Framebuffer addresses in SDRAM (must match sdram::SDRAM_BASE)
pub const LAYER1_BASE: u32 = super::sdram::SDRAM_BASE; // Layer1 full screen
#[cfg(feature = "l1-16bpp")]
pub const LAYER1_BPP: u32 = 2; // RGB565
#[cfg(not(feature = "l1-16bpp"))]
pub const LAYER1_BPP: u32 = 4; // ARGB8888
pub const LAYER1_SIZE: u32 = LCD_WIDTH * LCD_HEIGHT * LAYER1_BPP;
pub const LAYER2_BASE: u32 = LAYER1_BASE + LAYER1_SIZE;
// Single knob to adjust Layer 2 square size
pub const LAYER2_SIDE: u32 = 64;
pub const LAYER2_W: u32 = LAYER2_SIDE;
pub const LAYER2_H: u32 = LAYER2_SIDE;
pub const LAYER2_BPP: u32 = 4; // ARGB8888 for richer colors
pub const LAYER2_SIZE: u32 = LAYER2_W * LAYER2_H * LAYER2_BPP;
// Second framebuffer for Layer1 to enable double-buffering and avoid mid-scan writes
pub const LAYER1_BASE_B: u32 = LAYER2_BASE + LAYER2_SIZE;
// Track which L1 buffer is currently presented
static mut L1_FRONT: u32 = LAYER1_BASE;

impl LcdDriver {
    pub fn new() -> Self {
        let dp = unsafe { pac::Peripherals::steal() };
        let ltdc = dp.LTDC;
        // Ensure GPIOs are configured for LTDC signals
        Self::setup_ltdc_gpio(); // Configure sync and porch timings
        ltdc.sscr.write(|w| {
            w.hsw()
                .bits((HSYNC - 1) as u16)
                .vsh()
                .bits((VSYNC - 1) as u16)
        });
        ltdc.bpcr.write(|w| {
            w.ahbp()
                .bits((HSYNC + HBP - 1) as u16)
                .avbp()
                .bits((VSYNC + VBP - 1) as u16)
        });
        ltdc.awcr.write(|w| {
            w.aaw()
                .bits((HSYNC + HBP + LCD_WIDTH - 1) as u16)
                .aah()
                .bits((VSYNC + VBP + LCD_HEIGHT - 1) as u16)
        });
        ltdc.twcr.write(|w| {
            w.totalw()
                .bits((HSYNC + HBP + LCD_WIDTH + HFP - 1) as u16)
                .totalh()
                .bits((VSYNC + VBP + LCD_HEIGHT + VFP - 1) as u16)
        });

        // Polarity: configure pixel clock sampling edge; and optionally HS/VS/DE via feature flags
        // Match C example: pixel clock on rising edge (active high)
        #[cfg(feature = "pcpol-falling")]
        {
            ltdc.gcr.modify(|_, w| w.pcpol().clear_bit());
        }
        #[cfg(not(feature = "pcpol-falling"))]
        {
            ltdc.gcr.modify(|_, w| w.pcpol().set_bit());
        }

        // Optional polarity overrides via features (if none set, keep reset defaults; C example leaves HS/VS/DE defaults)
        #[cfg(feature = "hsync-high")]
        {
            ltdc.gcr.modify(|_, w| w.hspol().set_bit());
        }
        #[cfg(feature = "hsync-low")]
        {
            ltdc.gcr.modify(|_, w| w.hspol().clear_bit());
        }
        #[cfg(feature = "vsync-high")]
        {
            ltdc.gcr.modify(|_, w| w.vspol().set_bit());
        }
        #[cfg(feature = "vsync-low")]
        {
            ltdc.gcr.modify(|_, w| w.vspol().clear_bit());
        }
        #[cfg(feature = "de-high")]
        {
            ltdc.gcr.modify(|_, w| w.depol().set_bit());
        }
        #[cfg(feature = "de-low")]
        {
            ltdc.gcr.modify(|_, w| w.depol().clear_bit());
        }

        // Background color black
        // BCCR: background color components (all zero = black)
        ltdc.bccr
            .write(|w| w.bcblue().bits(0).bcgreen().bits(0).bcred().bits(0));

        // Do not enable LTDC interrupts until a Rust handler is provided

        // Layer 1 config (ARGB8888, full screen)
        {
            let h_start = HSYNC + HBP + 0;
            let h_stop = HSYNC + HBP + LCD_WIDTH - 1;
            ltdc.layer1.whpcr.write(|w| {
                w.whstpos()
                    .bits(h_start as u16)
                    .whsppos()
                    .bits(h_stop as u16)
            });
            let v_start = VSYNC + VBP + 0;
            let v_stop = VSYNC + VBP + LCD_HEIGHT - 1;
            ltdc.layer1.wvpcr.write(|w| {
                w.wvstpos()
                    .bits(v_start as u16)
                    .wvsppos()
                    .bits(v_stop as u16)
            });

            // Pixel format selectable: RGB565=2 (when l1-16bpp), else ARGB8888=0
            #[cfg(feature = "l1-16bpp")]
            ltdc.layer1.pfcr.write(|w| w.pf().bits(2));
            #[cfg(not(feature = "l1-16bpp"))]
            ltdc.layer1.pfcr.write(|w| w.pf().bits(0));
            // Framebuffer address
            ltdc.layer1.cfbar.write(|w| w.cfbadd().bits(LAYER1_BASE));
            // CFBLR: CFBP = pitch in bytes, CFBLL = pitch in bytes + 3
            let pitch_bytes = LCD_WIDTH * LAYER1_BPP;
            let pitch = pitch_bytes as u16;
            let line_len = (pitch_bytes + 3) as u16;
            ltdc.layer1
                .cfblr
                .write(|w| w.cfbp().bits(pitch).cfbll().bits(line_len));
            // Number of lines
            ltdc.layer1
                .cfblnr
                .write(|w| w.cfblnbr().bits(LCD_HEIGHT as u16));
            // Alpha and blending
            ltdc.layer1.cacr.write(|w| w.consta().bits(0xFF));
            ltdc.layer1.bfcr.write(|w| unsafe {
                w.bf1()
                    .bits(6) // BF1: pixel alpha x const alpha
                    .bf2()
                    .bits(7) // BF2: pixel alpha x const alpha
            });
            // Enable layer
            ltdc.layer1.cr.modify(|_, w| w.len().set_bit());
        }

        // Layer 2 config (ARGB8888, 64x64)
        {
            let h_start = HSYNC + HBP + 0;
            let h_stop = HSYNC + HBP + LAYER2_W - 1;
            ltdc.layer2.whpcr.write(|w| {
                w.whstpos()
                    .bits(h_start as u16)
                    .whsppos()
                    .bits(h_stop as u16)
            });
            let v_start = VSYNC + VBP + 0;
            let v_stop = VSYNC + VBP + LAYER2_H - 1;
            ltdc.layer2.wvpcr.write(|w| {
                w.wvstpos()
                    .bits(v_start as u16)
                    .wvsppos()
                    .bits(v_stop as u16)
            });

            // Pixel format: ARGB8888 = 0 (per LTDC PFCR encoding)
            ltdc.layer2.pfcr.write(|w| w.pf().bits(0));
            // Framebuffer address
            ltdc.layer2.cfbar.write(|w| w.cfbadd().bits(LAYER2_BASE));
            // CFBLR for layer2: CFBP = pitch in bytes, CFBLL = pitch in bytes + 3
            let pitch_bytes = LAYER2_W * LAYER2_BPP;
            let pitch = pitch_bytes as u16;
            let line_len = (pitch_bytes + 3) as u16;
            ltdc.layer2
                .cfblr
                .write(|w| w.cfbp().bits(pitch).cfbll().bits(line_len));
            // Number of lines
            ltdc.layer2
                .cfblnr
                .write(|w| w.cfblnbr().bits(LAYER2_H as u16));
            // Alpha and blending
            #[cfg(feature = "l2-opaque")]
            {
                // Use constant alpha only: BF1=CA, BF2=1-CA
                ltdc.layer2.cacr.write(|w| w.consta().bits(0xFF));
                ltdc.layer2.bfcr.write(|w| unsafe {
                    w.bf1()
                        .bits(4) // CA
                        .bf2()
                        .bits(5) // 1-CA
                });
            }
            #[cfg(not(feature = "l2-opaque"))]
            {
                // Default: pixel alpha x const alpha
                ltdc.layer2.cacr.write(|w| w.consta().bits(0xFF));
                ltdc.layer2
                    .bfcr
                    .write(|w| unsafe { w.bf1().bits(6).bf2().bits(7) });
            }
            // Enable layer
            ltdc.layer2.cr.modify(|_, w| w.len().set_bit());
        }

        // Reload shadow regs (vertical blank reload) before enabling
        ltdc.srcr.modify(|_, w| w.vbr().set_bit());
        // Enable LTDC
        ltdc.gcr.modify(|_, w| w.ltdcen().set_bit());
        debug_assert!(ltdc.gcr.read().ltdcen().bit_is_set());

        Self { ltdc }
    }

    pub fn set_layer2_position(&self, x: u32, y: u32) {
        use core::cmp::min;
        let ltdc = &self.ltdc;
        // Constrain to screen bounds
        let x = min(x, LCD_WIDTH.saturating_sub(LAYER2_W));
        let y = min(y, LCD_HEIGHT.saturating_sub(LAYER2_H));
        let h_start = HSYNC + HBP + x;
        let h_stop = h_start + LAYER2_W - 1;
        let v_start = VSYNC + VBP + y;
        let v_stop = v_start + LAYER2_H - 1;
        ltdc.layer2.whpcr.write(|w| {
            w.whstpos()
                .bits(h_start as u16)
                .whsppos()
                .bits(h_stop as u16)
        });
        ltdc.layer2.wvpcr.write(|w| {
            w.wvstpos()
                .bits(v_start as u16)
                .wvsppos()
                .bits(v_stop as u16)
        });
        // Apply position update
        #[cfg(feature = "l2-immediate")]
        {
            ltdc.srcr.modify(|_, w| w.imr().set_bit());
        }
        #[cfg(not(feature = "l2-immediate"))]
        {
            ltdc.srcr.modify(|_, w| w.vbr().set_bit());
        }
    }

    pub fn set_layer2_alpha(&self, alpha: u8) {
        let ltdc = &self.ltdc;
        ltdc.layer2.cacr.write(|w| w.consta().bits(alpha));
        // Apply at next VBlank
        ltdc.srcr.modify(|_, w| w.vbr().set_bit());
    }

    // Return the current front and back addresses for Layer1
    pub fn layer1_back_addr() -> u32 {
        unsafe {
            if L1_FRONT == LAYER1_BASE {
                LAYER1_BASE_B
            } else {
                LAYER1_BASE
            }
        }
    }

    // Swap Layer1 front/back by updating CFBAR to the back buffer and latching on VBlank
    #[cfg(feature = "overlay")]
    pub fn swap_layer1_buffers(&self) {
        let ltdc = &self.ltdc;
        let new_front = Self::layer1_back_addr();
        ltdc.layer1.cfbar.write(|w| w.cfbadd().bits(new_front));
        // Latch address change at next VBlank
        ltdc.srcr.modify(|_, w| w.vbr().set_bit());
        // Update tracker
        unsafe {
            L1_FRONT = new_front;
        }
    }

    fn setup_ltdc_gpio() {
        // Enable GPIO clocks: A,B,C,D,F,G
        let dp = unsafe { pac::Peripherals::steal() };
        let rcc = dp.RCC;
        rcc.ahb1enr.modify(|_, w| {
            w.gpioaen()
                .enabled()
                .gpioben()
                .enabled()
                .gpiocen()
                .enabled()
                .gpioden()
                .enabled()
                .gpiofen()
                .enabled()
                .gpiogen()
                .enabled()
        });

        // Helper to set AF and mode/speed on any GPIO port by using a unified
        // register layout (all GPIO ports share the same register map)
        fn set_af(gpio_base: *const pac::gpioa::RegisterBlock, pin: u32, af: u32) {
            let gpio = unsafe { &*gpio_base };
            // Alternate function mode
            gpio.moder.modify(|r, w| unsafe {
                w.bits((r.bits() & !(0b11 << (pin * 2))) | (0b10 << (pin * 2)))
            });
            // Fast (50MHz) speed to match C example
            gpio.ospeedr.modify(|r, w| unsafe {
                w.bits((r.bits() & !(0b11 << (pin * 2))) | (0b10 << (pin * 2)))
            });
            // Push-pull
            gpio.otyper
                .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << pin)) });
            // No pull-up/down
            gpio.pupdr
                .modify(|r, w| unsafe { w.bits(r.bits() & !(0b11 << (pin * 2))) });
            // AF low/high
            if pin < 8 {
                let idx = pin;
                gpio.afrl.modify(|r, w| unsafe {
                    let mut v = r.bits();
                    v &= !(0xF << (idx * 4));
                    v |= af << (idx * 4);
                    w.bits(v)
                });
            } else {
                let idx = pin - 8;
                gpio.afrh.modify(|r, w| unsafe {
                    let mut v = r.bits();
                    v &= !(0xF << (idx * 4));
                    v |= af << (idx * 4);
                    w.bits(v)
                });
            }
        }

        // Unify all ports to the same register layout via cast
        let pa = pac::GPIOA::ptr() as *const pac::gpioa::RegisterBlock;
        let pb = pac::GPIOB::ptr() as *const pac::gpioa::RegisterBlock;
        let pc = pac::GPIOC::ptr() as *const pac::gpioa::RegisterBlock;
        let pd = pac::GPIOD::ptr() as *const pac::gpioa::RegisterBlock;
        let pf = pac::GPIOF::ptr() as *const pac::gpioa::RegisterBlock;
        let pg = pac::GPIOG::ptr() as *const pac::gpioa::RegisterBlock;

        // Map pins per C example comments (AF values vary: AF14, AF9, AF11)
        // R2=PC10(AF14), R3=PB0(AF9), R4=PA11(AF14), R5=PA12(AF14), R6=PB1(AF9), R7=PG6(AF14)
        set_af(pc, 10, 14);
        set_af(pb, 0, 9);
        set_af(pa, 11, 14);
        set_af(pa, 12, 14);
        set_af(pb, 1, 9);
        set_af(pg, 6, 14);
        // G2=PA6(AF14), G3=PG10(AF9), G4=PB10(AF14), G5=PB11(14), G6=PC7(AF14), G7=PD3(AF14)
        set_af(pa, 6, 14);
        set_af(pg, 10, 9);
        set_af(pb, 10, 14);
        set_af(pb, 11, 14);
        set_af(pc, 7, 14);
        set_af(pd, 3, 14);
        // B2=PD6(AF14), B3=PG11(AF14), B4=PG12(AF9), B5=PA3(AF14), B6=PB8(AF14), B7=PB9(AF14)
        set_af(pd, 6, 14);
        set_af(pg, 11, 14);
        set_af(pg, 12, 9);
        set_af(pa, 3, 14);
        set_af(pb, 8, 14);
        set_af(pb, 9, 14);
        // Control: ENABLE=PF10(AF14), DOTCLK=PG7(AF14), HSYNC=PC6(AF14), VSYNC=PA4(AF14)
        set_af(pf, 10, 14);
        set_af(pg, 7, 14);
        set_af(pc, 6, 14);
        set_af(pa, 4, 14);
    }

    // --- Debug helpers ---
    #[allow(dead_code)]
    pub fn ltdc_status(&self) -> (u32, u32) {
        // Returns (ISR, IER) raw bits to inspect underrun/transfer error flags from gdb
        let ltdc = &self.ltdc;
        (ltdc.isr.read().bits(), ltdc.ier.read().bits())
    }
}
