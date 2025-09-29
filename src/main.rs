#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;

mod sdram;
mod clock;
mod lcd;
mod lcd_spi;
mod draw;
mod i2c;
mod mpu6050;


#[entry]
fn main() -> ! {
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
    lcd::init_ltdc();
    // Initialize display panel over SPI
    lcd_spi::init();

    // Initialize I2C and MPU6050
    i2c::init_i2c1();
    
    // Small delay for MPU6050 to stabilize
    clock::delay_ms(100);
    
    let _mpu_init_result = mpu6050::init();
    
    // Keep Layer 2 fully opaque
    lcd::set_layer2_alpha(0xFF);
    
    // Tilt-responsive square control
    let mut square_x: i32 = ((lcd::LCD_WIDTH - lcd::LAYER2_W) / 2) as i32; // Center X
    let mut square_y: i32 = ((lcd::LCD_HEIGHT - lcd::LAYER2_H) / 2) as i32; // Center Y
    
    // Movement parameters  
    const TILT_SENSITIVITY: i32 = 100; // Higher = more sensitive
    const MAX_SPEED: i32 = 8; // Maximum pixels per frame
    
    loop {
        // Read MPU6050 data
        if let Ok(data) = mpu6050::read_data() {
            // Convert tilt to screen movement
            // Accelerometer range is ±2g = ±32768 LSB
            // Map tilt to screen velocity with sensitivity and speed limiting
            
            // X-axis: Roll (tilt left/right) -> horizontal movement
            let vel_x = ((data.accel_y as i32) / TILT_SENSITIVITY).max(-MAX_SPEED).min(MAX_SPEED);
            
            // Y-axis: Pitch (tilt forward/backward) -> vertical movement  
            let vel_y = ((data.accel_x as i32) / TILT_SENSITIVITY).max(-MAX_SPEED).min(MAX_SPEED);
            
            // Update square position
            square_x += vel_x;
            square_y += vel_y;
            
            // Clamp to screen boundaries (allow edge sliding)
            let max_x = (lcd::LCD_WIDTH - lcd::LAYER2_W) as i32;
            let max_y = (lcd::LCD_HEIGHT - lcd::LAYER2_H) as i32;
            
            square_x = square_x.max(0).min(max_x);
            square_y = square_y.max(0).min(max_y);
        }
        
        // Update square position on screen
        lcd::set_layer2_position(square_x as u32, square_y as u32);
        
        // Minimal delay for maximum responsiveness
        clock::delay_ms(1);
    }
}
