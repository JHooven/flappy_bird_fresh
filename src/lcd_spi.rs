use stm32f4::stm32f429 as pac;

const ILI_PWR_CTL_1: u8 = 0xc0;
const ILI_PWR_CTL_2: u8 = 0xc1;
const ILI_VCOM_CTL_1: u8 = 0xc5;
const ILI_VCOM_CTL_2: u8 = 0xc7;
const ILI_MEM_ACC_CTL: u8 = 0x36;
const ILI_RGB_IFC_CTL: u8 = 0xb0;
const ILI_IFC_CTL: u8 = 0xf6;
const ILI_GAMMA_SET: u8 = 0x26;
const ILI_POS_GAMMA: u8 = 0xe0;
const ILI_NEG_GAMMA: u8 = 0xe1;
const ILI_SLEEP_OUT: u8 = 0x11;
const ILI_DISP_ON: u8 = 0x29;

// Pins: PC2=CS, PD13=D/CX, PF7=SCK(AF5), PF9=MOSI(AF5)

fn spi5() -> pac::SPI5 { unsafe { pac::Peripherals::steal().SPI5 } }

fn select() { let gpioc = unsafe { &*pac::GPIOC::ptr() }; gpioc.bsrr.write(|w| w.br2().set_bit()); }
fn deselect() { let gpioc = unsafe { &*pac::GPIOC::ptr() }; gpioc.bsrr.write(|w| w.bs2().set_bit()); }
fn set_data() { let gpiod = unsafe { &*pac::GPIOD::ptr() }; gpiod.bsrr.write(|w| w.bs13().set_bit()); }
fn set_cmd() { let gpiod = unsafe { &*pac::GPIOD::ptr() }; gpiod.bsrr.write(|w| w.br13().set_bit()); }

fn spi_send_byte(b: u8) {
    let spi = spi5();
    // Wait TXE
    while spi.sr.read().txe().bit_is_clear() {}
    spi.dr.write(|w| w.dr().bits(b.into()));
    // Wait BSY clear
    while spi.sr.read().bsy().bit_is_set() {}
    // Read and clear RXNE if set
    let _ = spi.dr.read().dr().bits();
}

fn lcd_command(cmd: u8, delay_ms: u16, data: &[u8]) {
    select();
    set_cmd();
    spi_send_byte(cmd);
    if !data.is_empty() {
        set_data();
        for &b in data { spi_send_byte(b); }
    }
    deselect();
    // crude delay
    if delay_ms != 0 { crate::clock::delay_ms(delay_ms as u32); }
}

pub fn init() {
    // Clocks for GPIOC, GPIOD, GPIOF, SPI5
    let dp = unsafe { pac::Peripherals::steal() };
    let rcc = dp.RCC;
    rcc.ahb1enr.modify(|_, w| w.gpiocen().enabled().gpioden().enabled().gpiofen().enabled());
    rcc.apb2enr.modify(|_, w| w.spi5en().enabled());

    // GPIO modes
    let gpioc = unsafe { &*pac::GPIOC::ptr() };
    let gpiod = unsafe { &*pac::GPIOD::ptr() };
    let gpiof = unsafe { &*pac::GPIOF::ptr() };

    // PC2 output
    gpioc.moder.modify(|r, w| unsafe { w.bits((r.bits() & !(0b11 << (2*2))) | (0b01 << (2*2))) });
    gpioc.ospeedr.modify(|r, w| unsafe { w.bits((r.bits() & !(0b11 << (2*2))) | (0b10 << (2*2))) });
    // PD13 output
    gpiod.moder.modify(|r, w| unsafe { w.bits((r.bits() & !(0b11 << (13*2))) | (0b01 << (13*2))) });
    gpiod.ospeedr.modify(|r, w| unsafe { w.bits((r.bits() & !(0b11 << (13*2))) | (0b10 << (13*2))) });
    // PF7, PF9 AF5
    for &pin in &[7u32, 9u32] {
    gpiof.moder.modify(|r, w| unsafe { w.bits((r.bits() & !(0b11 << (pin*2))) | (0b10 << (pin*2))) });
    gpiof.ospeedr.modify(|r, w| unsafe { w.bits((r.bits() & !(0b11 << (pin*2))) | (0b10 << (pin*2))) });
        if pin < 8 {
            let idx = pin;
            gpiof.afrl.modify(|r, w| unsafe {
                let mut v = r.bits(); v &= !(0xF << (idx*4)); v |= 5 << (idx*4); w.bits(v)
            });
        } else {
            let idx = pin - 8;
            gpiof.afrh.modify(|r, w| unsafe {
                let mut v = r.bits(); v &= !(0xF << (idx*4)); v |= 5 << (idx*4); w.bits(v)
            });
        }
    }

    // SPI5 config: master, baud=fpclk/4, 8-bit, software NSS (SSM=1, SSI=1), full-duplex (BIDIMODE=0)
    let spi = spi5();
    // Ensure NSS is high (software-managed) and configure before enabling SPE
    spi.cr1.modify(|_, w| w.spe().clear_bit());
    spi.cr1.modify(|_, w| w
        .mstr().set_bit()
        .br().div4()
        .ssm().set_bit()
        .ssi().set_bit()
        .cpol().clear_bit()
        .cpha().clear_bit()
        .bidimode().clear_bit() // 2-line unidirectional
    );
    // Idle lines and CS high before enabling
    deselect();
    set_data();
    // Now enable SPI
    spi.cr1.modify(|_, w| w.spe().set_bit());

    // Initialization sequence (exactly as the C demo)
    lcd_command(ILI_PWR_CTL_1, 0, &[0x10]);
    lcd_command(ILI_PWR_CTL_2, 0, &[0x10]);
    lcd_command(ILI_VCOM_CTL_1, 0, &[0x45, 0x15]);
    lcd_command(ILI_VCOM_CTL_2, 0, &[0x90]);
    // Portrait orientation: MADCTL = BGR (0x08)
    lcd_command(ILI_MEM_ACC_CTL, 0, &[0x08]);
    // RGB interface control and interface control
    lcd_command(ILI_RGB_IFC_CTL, 0, &[0xC0]);
    lcd_command(ILI_IFC_CTL, 0, &[0x01, 0x00, 0x06]);
    lcd_command(ILI_GAMMA_SET, 0, &[0x01]);
    let pos_gamma: [u8; 15] = [0x0F,0x29,0x24,0x0C,0x0E,0x09,0x4E,0x78,0x3C,0x09,0x13,0x05,0x17,0x11,0x00];
    let neg_gamma: [u8; 15] = [0x00,0x16,0x1B,0x04,0x11,0x07,0x31,0x33,0x42,0x05,0x0C,0x0A,0x28,0x2F,0x0F];
    lcd_command(ILI_POS_GAMMA, 0, &pos_gamma);
    lcd_command(ILI_NEG_GAMMA, 0, &neg_gamma);
    lcd_command(ILI_SLEEP_OUT, 5, &[]);
    lcd_command(ILI_DISP_ON, 0, &[]);
}
