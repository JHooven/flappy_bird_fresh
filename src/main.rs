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

    display::register_driver(&lcd_driver);

    display::init(); // Initialize display module

    let input = DummyInputDevice::new();
    let mut game_instance = Game::init(input).expect("Failed to initialize game");

    // Game loop
    loop {
        game_instance.update();
    }
}

fn init() -> lcd::LcdDriver {
    // Configure system clocks to 168MHz from HSE to match C demo
    clock::setup_system_clocks_168mhz();
    // SysTick and base clocks
    let cp = cortex_m::Peripherals::take().unwrap();
    let _syst = clock::setup(cp.SYST);

    // SDRAM
    sdram::init();

    // Preload frame buffers
    draw::layer1_checkerboard();
    draw::layer2_sprite();

    // LTDC pixel clock via PLLSAI and enable LTDC clock
    clock::setup_pllsai_for_ltdc();
    // Configure LTDC layers first to provide sync
    let lcd_driver = lcd::LcdDriver::new();
    // Initialize display panel over SPI
    lcd_spi::init();

    // Initialize I2C and MPU6050
    i2c::init_i2c1();

    // Small delay for MPU6050 to stabilize
    clock::delay_ms(100);

    let _mpu_init_result = mpu6050::init();

    // Keep Layer 2 fully opaque
    lcd_driver.set_layer2_alpha(0xFF);

    lcd_driver
}
