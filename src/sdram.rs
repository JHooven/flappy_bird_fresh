// FMC SDRAM initialization for STM32F429I-Discovery (bank 2)
// Ported from libopencm3 lcd-dma example

#![allow(dead_code)]

use cortex_m::asm;
use stm32f4::stm32f429 as pac;

pub const SDRAM_BASE: u32 = 0xD000_0000; // Bank2 base

// Configure one GPIO pin to AF12 FMC: mode=AF, high speed, push-pull, no pull
macro_rules! cfg_pin_af12 {
    ($gpio:expr, $pin:expr) => { {
        let pin: u32 = $pin as u32;
        // MODER: Alternate Function (10)
        let moder = $gpio.moder.read().bits();
        let new = (moder & !(0b11 << (pin*2))) | (0b10 << (pin*2));
        $gpio.moder.write(|w| unsafe { w.bits(new) });
        // OSPEEDR: High speed (10)
        let ospeed = $gpio.ospeedr.read().bits();
        let new = (ospeed & !(0b11 << (pin*2))) | (0b10 << (pin*2));
        $gpio.ospeedr.write(|w| unsafe { w.bits(new) });
        // OTYPER: Push-pull (0)
        let otyper = $gpio.otyper.read().bits() & !(1 << pin);
        $gpio.otyper.write(|w| unsafe { w.bits(otyper) });
        // PUPDR: No pull (00)
        let pupdr = $gpio.pupdr.read().bits() & !(0b11 << (pin*2));
        $gpio.pupdr.write(|w| unsafe { w.bits(pupdr) });
        // AFR: AF12
        if pin < 8 {
            let idx = pin;
            let afr = $gpio.afrl.read().bits();
            let new = (afr & !(0xF << (idx*4))) | (12 << (idx*4));
            $gpio.afrl.write(|w| unsafe { w.bits(new) });
        } else {
            let idx = pin - 8;
            let afr = $gpio.afrh.read().bits();
            let new = (afr & !(0xF << (idx*4))) | (12 << (idx*4));
            $gpio.afrh.write(|w| unsafe { w.bits(new) });
        }
    }};
}

macro_rules! cfg_pins_af12 {
    ($gpio:expr, [$($pin:expr),* $(,)?]) => { {
        $( cfg_pin_af12!($gpio, $pin); )*
    }};
}

pub fn init() {
    // Safety: we do raw peripheral register writes at startup
    let dp = unsafe { pac::Peripherals::steal() };
    let rcc = dp.RCC;

    // Enable GPIO clocks: B,C,D,E,F,G
    rcc.ahb1enr.modify(|_, w| {
        w.gpioben().enabled()
            .gpiocen().enabled()
            .gpioden().enabled()
            .gpioeen().enabled()
            .gpiofen().enabled()
            .gpiogen().enabled()
    });
    // Small delay after enabling clocks
    asm::nop();

    // Access GPIO register blocks directly
    let gpiob = unsafe { &*pac::GPIOB::ptr() };
    let gpioc = unsafe { &*pac::GPIOC::ptr() };
    let gpiod = unsafe { &*pac::GPIOD::ptr() };
    let gpioe = unsafe { &*pac::GPIOE::ptr() };
    let gpiof = unsafe { &*pac::GPIOF::ptr() };
    let gpiog = unsafe { &*pac::GPIOG::ptr() };

    // Configure all SDRAM-related pins to AF12
    cfg_pins_af12!(gpiob, [5,6]);
    cfg_pins_af12!(gpioc, [0]);
    cfg_pins_af12!(gpiod, [0,1,8,9,10,14,15]);
    cfg_pins_af12!(gpioe, [0,1,7,8,9,10,11,12,13,14,15]);
    cfg_pins_af12!(gpiof, [0,1,2,3,4,5,11,12,13,14,15]);
    cfg_pins_af12!(gpiog, [0,1,4,5,8,15]);

    // Enable FMC clock (AHB3)
    rcc.ahb3enr.modify(|_, w| w.fmcen().enabled());
    asm::nop();

    let fmc = dp.FMC;

    // SDCR configuration (matching C settings):
    // RPIPE=1CLK, SDCLK=2xHCLK, CAS=3 cycles, NB=4 banks, MWID=16-bit, NR=12 row, NC=8 col
    // SDCR1 holds shared fields; SDCR2 holds bank2
    unsafe {
        // Constants are device-specific per RM0090
    let rpipe_1clk = 0b01 << 13;     // RPIPE[14:13]
        let sdclk_2hclk = 0b10 << 10;    // SDCLK[12:10]
        let cas_3 = 0b11 << 7;           // CAS[8:7]
        let nb4 = 1 << 6;                // NB bit
        let mwid_16 = 0b01 << 4;         // MWID[5:4]
        let nr_12 = 0b01 << 2;           // NR[3:2]
        let nc_8 = 0b00 << 0;            // NC[1:0] 8 column

    let cr_shared = rpipe_1clk | sdclk_2hclk | cas_3 | mwid_16 | nr_12 | nc_8 | nb4;

        // SDCR1 (shared fields)
        fmc.sdcr1().write(|w| w.bits(cr_shared));
        // SDCR2 (bank specific)
        fmc.sdcr2().write(|w| w.bits(cr_shared));

        // SDTR timing
        // TMRD(3:0)=2, TXSR(7:4)=7, TRAS(11:8)=4, TRC(15:12)=7, TWR(19:16)=2, TRP(23:20)=2, TRCD(27:24)=2
        let tmrd = 2;
        let txsr = 7;
        let tras = 4;
        let trc = 7;
        let twr = 2;
        let trp = 2;
        let trcd = 2;
        let tr_bits = (tmrd & 0xF)
            | ((txsr & 0xF) << 4)
            | ((tras & 0xF) << 8)
            | ((trc & 0xF) << 12)
            | ((twr & 0xF) << 16)
            | ((trp & 0xF) << 20)
            | ((trcd & 0xF) << 24);

        fmc.sdtr1().write(|w| w.bits(tr_bits));
        fmc.sdtr2().write(|w| w.bits(tr_bits));

        // Command: Clock enable to bank 2
        // SDCMR: MODE[2:0]=001 (Clock Configuration Enable), CTB2=1
        fmc.sdcmr.write(|w| w
            .mode().bits(0b001)
            .ctb2().set_bit()
            .nrfs().bits(0)
            .mrd().bits(0)
        );
        // Delay >= 100us
        delay_cycles(16_000); // ~100us at 168MHz (approx)

        // Command: PALL (precharge all)
        fmc.sdcmr.write(|w| w
            .mode().bits(0b010)
            .ctb2().set_bit()
            .nrfs().bits(0)
            .mrd().bits(0)
        );

        // Command: Auto-refresh, 4 cycles
        fmc.sdcmr.write(|w| w
            .mode().bits(0b011)
            .ctb2().set_bit()
            .nrfs().bits(4) // 4 refresh cycles
            .mrd().bits(0)
        );

        // Command: Load Mode Register
        // MRD value: BL=2 (001), BT=0 (seq), CAS=3 (011), OM=00, WB=1 (single) => 0x231
        let mrd: u16 = 0x231;
        fmc.sdcmr.write(|w| w
            .mode().bits(0b100)
            .ctb2().set_bit()
            .nrfs().bits(1)
            .mrd().bits(mrd)
        );

        // Set refresh rate
        // SDRTR[13:1] COUNTER = 683
        fmc.sdrtr.modify(|_, w| w.reie().clear_bit().count().bits(683));
    }
}

fn delay_cycles(mut n: i32) {
    // Crude busy loop
    while n != 0 { asm::nop(); n -= 1; }
}
