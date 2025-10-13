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
    flash.acr.modify(|_, w| {
        w.latency()
            .ws5() // 5 wait states
            .icen()
            .set_bit()
            .dcen()
            .set_bit()
            .prften()
            .set_bit()
    });

    // Configure PLL: HSE = 8MHz, VCO = 336MHz, SYSCLK = 168MHz
    // PLLM = 8, PLLN = 336, PLLP = 2, PLLQ = 7
    rcc.pllcfgr.modify(|_, w| unsafe {
        w.pllsrc()
            .hse()
            .pllm()
            .bits(8)
            .plln()
            .bits(336)
            .pllp()
            .div2()
            .pllq()
            .bits(7)
    });

    // Enable PLL
    rcc.cr.modify(|_, w| w.pllon().on());
    while rcc.cr.read().pllrdy().is_not_ready() {}

    // Configure prescalers: AHB = SYSCLK, APB1 = SYSCLK/4, APB2 = SYSCLK/2
    rcc.cfgr.modify(
        |_, w| {
            w.hpre()
                .div1() // AHB prescaler = 1
                .ppre1()
                .div4() // APB1 prescaler = 4 (42MHz max)
                .ppre2()
                .div2()
        }, // APB2 prescaler = 2 (84MHz max)
    );

    // Switch to PLL
    rcc.cfgr.modify(|_, w| w.sw().pll());
    while !rcc.cfgr.read().sws().is_pll() {}
}

// Setup SysTick for basic timing - returns the configured SYST peripheral
pub fn setup(mut syst: SYST) -> SYST {
    // Configure SysTick to tick every millisecond at 168MHz
    // SysTick reload = (168MHz / 1000Hz) - 1 = 167999
    syst.set_reload(167_999);
    syst.clear_current();
    syst.set_clock_source(SystClkSource::Core);
    syst.enable_counter();
    // Note: We're not enabling interrupts for this simple implementation
    syst
}

// Configure PLLSAI for LTDC pixel clock
pub fn setup_pllsai_for_ltdc() {
    let dp = unsafe { pac::Peripherals::steal() };
    let rcc = dp.RCC;

    // Configure PLLSAI for LCD timing
    // We only use busy-wait delays for now.

    // PLLSAI configuration for LTDC
    // This is a simplified setup - adjust based on your exact timing requirements
    rcc.pllsaicfgr.modify(|_, w| unsafe {
        w.pllsain()
            .bits(192) // VCO frequency
            .pllsair()
            .bits(4) // Division factor for LCD clock
    });

    // Enable PLLSAI
    rcc.cr.modify(|_, w| w.pllsaion().on());
    while rcc.cr.read().pllsairdy().is_not_ready() {}

    // Enable LTDC clock
    rcc.apb2enr.modify(|_, w| w.ltdcen().enabled());

    // Verify clocks are ready
    debug_assert!(rcc.cr.read().pllsairdy().is_ready());
    debug_assert!(rcc.apb2enr.read().ltdcen().is_enabled());
}

// Crude busy-wait millisecond delay assuming SysTick at 1kHz
pub fn delay_ms(ms: u32) {
    // Fallback: busy loop scaled for ~168MHz (very rough)
    //let cycles = 168_000 * ms;
    let mut n = ms;
    while n != 0 {
        cortex_m::asm::nop();
        n -= 1;
    }
}
