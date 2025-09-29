//! Basic clock + SysTick setup matching libopencm3 example assumptions
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::SYST;
use stm32f4::stm32f429 as pac;

// Configure system clock to 168MHz from 8MHz HSE, matching libopencm3's rcc_clock_setup_pll
pub fn setup_system_clocks_168mhz() {
    let dp = unsafe { pac::Peripherals::steal() };
    let rcc = dp.RCC;
    let pwr = dp.PWR;
    let flash = dp.FLASH;

    // Enable HSE
    rcc.cr.modify(|_, w| w.hseon().on());
    while rcc.cr.read().hserdy().is_not_ready() {}

    // Enable PWR and set VOS scale 1 (for 168MHz)
    rcc.apb1enr.modify(|_, w| w.pwren().enabled());
    // VOS bits in PWR_CR [15:14]: 11 = Scale 1 mode
    pwr.cr.modify(|_, w| unsafe { w.vos().bits(0b11) });

    // Configure Flash latency and caches
    // 5 wait states for 168MHz at 3.3V, enable I/D cache and prefetch
    flash.acr.modify(|_, w| w
        .latency().ws5() // 5 wait states
        .icen().set_bit()
        .dcen().set_bit()
        .prften().set_bit()
    );

    // Set prescalers: AHB=1, APB1=4, APB2=2
    rcc.cfgr.modify(|_, w| w
        .hpre().div1()
        .ppre1().div4()
        .ppre2().div2()
    );

    // Configure PLL: PLLSRC=HSE, PLLM=8, PLLN=336, PLLP=2, PLLQ=7
    // PLLCFGR fields: PLLSRC (bit22), PLLM[5:0], PLLN[14:6], PLLP[17:16] (00=>/2), PLLQ[27:24]
    let pllm: u32 = 8;
    let plln: u32 = 336;
    let pllp_bits: u32 = 0b00; // /2
    let pllq: u32 = 7;
    let pllcfgr = (1 << 22) // PLLSRC=HSE
        | (pllm & 0x3F)
        | ((plln & 0x1FF) << 6)
        | (pllp_bits << 16)
        | ((pllq & 0x0F) << 24);
    rcc.pllcfgr.write(|w| unsafe { w.bits(pllcfgr) });

    // Enable PLL and wait ready
    rcc.cr.modify(|_, w| w.pllon().on());
    while rcc.cr.read().pllrdy().is_not_ready() {}

    // Switch SYSCLK to PLL
    rcc.cfgr.modify(|_, w| w.sw().pll());
    while !rcc.cfgr.read().sws().is_pll() {}

    // Optionally disable HSI to save power
    rcc.cr.modify(|_, w| w.hsion().off());
}

pub fn setup(mut syst: SYST) -> SYST {
    // We assume device already starts with HSE 8MHz and system at 168MHz
    // like libopencm3's rcc_clock_setup_pll does. For Rust PAC, a full
    // PLL setup is verbose; for now we just enable SysTick at 1kHz.
    syst.set_clock_source(SystClkSource::Core);
    // 168_000_000 / 168_000 = 1000 Hz
    syst.set_reload(168_000 - 1);
    syst.clear_current();
    syst.enable_counter();
    // Don't enable SysTick interrupt until a handler is provided.
    // We only use busy-wait delays for now.
    syst
}

// Configure PLLSAI to provide ~6MHz pixel clock and enable LTDC clock
pub fn setup_pllsai_for_ltdc() {
    let dp = unsafe { pac::Peripherals::steal() };
    let rcc = dp.RCC;
    // Set PLLSAIN=192, PLLSAIR=4, keep Q as reset value, DIVR=8
    // RCC_PLLSAICFGR: bits [14:6]=PLLSAIN, [30:28]=PLLSAIR
    let saicfgr = rcc.pllsaicfgr.read().bits();
    let saiq = (saicfgr >> 24) & 0xF; // preserve Q
    let sain: u32 = 192;
    let sair: u32 = 4;
    let new_saicfgr = (sain << 6) | (saiq << 24) | (sair << 28);
    rcc.pllsaicfgr.write(|w| unsafe { w.bits(new_saicfgr) });

    // DIVR in DCKCFGR bits [17:16]: 00=2,01=4,10=8,11=16
    let mut dckcfgr = rcc.dckcfgr.read().bits();
    dckcfgr &= !(0b11 << 16);
    #[cfg(feature = "pclk-div-2")]
    { dckcfgr |= 0b00 << 16; }
    #[cfg(feature = "pclk-div-4")]
    { dckcfgr |= 0b01 << 16; }
    #[cfg(all(not(feature = "pclk-div-2"), not(feature = "pclk-div-4"), not(feature = "pclk-div-16")))]
    { dckcfgr |= 0b10 << 16; } // default DIVR_8
    #[cfg(feature = "pclk-div-16")]
    { dckcfgr |= 0b11 << 16; }
    rcc.dckcfgr.write(|w| unsafe { w.bits(dckcfgr) });

    // Enable PLLSAI and wait ready
    rcc.cr.modify(|_, w| w.pllsaion().on());
    while rcc.cr.read().pllsairdy().is_not_ready() {}
    // Enable LTDC clock on APB2
    rcc.apb2enr.modify(|_, w| w.ltdcen().enabled());

    // Debug-only sanity checks
    debug_assert!(rcc.cr.read().pllsairdy().is_ready());
    debug_assert!(rcc.apb2enr.read().ltdcen().is_enabled());
}

// Crude busy-wait millisecond delay assuming SysTick at 1kHz
pub fn delay_ms(ms: u32) {
    // Fallback: busy loop scaled for ~168MHz (very rough)
    let cycles = 168_000 * ms;
    let mut n = cycles;
    while n != 0 { cortex_m::asm::nop(); n -= 1; }
}
