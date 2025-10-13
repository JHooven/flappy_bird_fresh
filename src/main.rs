#![no_std]
#![no_main]
#![allow(dead_code)]

use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4 as _;

mod assets;
mod clock;
mod color;
mod config;
mod display;
mod draw;
mod game;
mod i2c;
mod lcd;
mod lcd_spi;
mod mpu6050;
mod obstacle;
mod player;
mod sdram;

// Import the types we need
use config::Coord;
use game::{Game, InputDevice};

// Dummy input device for now
struct DummyInputDevice;

impl DummyInputDevice {
    fn new() -> Self {
        Self
    }
}

impl InputDevice for DummyInputDevice {
    type Error = ();

    fn init(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn is_tap(&mut self, _y_min: Coord, _y_max: Coord) -> Result<(Coord, bool), Self::Error> {
        // For now, never report a tap
        Ok((0, false))
    }
}

#[entry]
fn main() -> ! {
    let lcd_driver = init();

    // Re-enable display module - LTDC timing changes may have broken pure LTDC mode
    display::register_driver(&lcd_driver);
    display::init(); // Initialize display module

    let input = DummyInputDevice::new();
    let _game_instance = Game::init(input).expect("Failed to initialize game");

    // Minimal test loop - just show checkerboard without game updates
    loop {
        // _game_instance.update();  // Disable game updates for testing
        clock::delay_ms(1000); // Very slow for debugging
    }
}

fn init() -> lcd::LcdDriver {
    // Configure system clocks to 168MHz from HSE to match C demo
    //clock::setup_system_clocks_168mhz();
    // SysTick and base clocks
    let cp = cortex_m::Peripherals::take().unwrap();
    let _syst = clock::setup(cp.SYST);

    // Setup clocks first before initializing LTDC
    clock::setup_system_clocks_168mhz();
    clock::setup_pllsai_for_ltdc();

    // Initialize SDRAM for framebuffers
    sdram::init();

    // Setup LTDC and framebuffers
    draw::layer1_checkerboard();
    // draw::layer2_sprite();  // Disable layer2 to reduce conflicts

    // Create LCD driver (this will configure LTDC)
    let lcd_driver = lcd::LcdDriver::new();

    // Initialize SPI display
    lcd_spi::init(); // Initialize I2C and MPU6050
    i2c::init_i2c1();

    // Small delay for I2C to stabilize
    clock::delay_ms(50);

    // Try to initialize MPU6050, with I2C reset on failure
    let mut mpu_init_attempts = 3;
    loop {
        let mpu_init_result = mpu6050::init();
        if mpu_init_result.is_ok() {
            break;
        }

        mpu_init_attempts -= 1;
        if mpu_init_attempts == 0 {
            // If all attempts fail, continue anyway (MPU6050 is not critical for display)
            break;
        }

        // Reset I2C and try again
        i2c::reset_i2c1();
        clock::delay_ms(100);
    }

    // Keep Layer 2 fully opaque
    lcd_driver.set_layer2_alpha(0xFF);

    lcd_driver
}
