#![allow(dead_code)]

use stm32f4::stm32f429 as pac;

// Simple delay function for I2C timing
fn delay_us(us: u32) {
    // Rough delay based on 168MHz system clock
    let cycles = 168 * us;
    for _ in 0..cycles {
        cortex_m::asm::nop();
    }
}

const I2C_TIMEOUT: u32 = 100_000; // Timeout counter

// Reset I2C1 peripheral (useful for recovery from stuck state)
pub fn reset_i2c1() {
    let dp = unsafe { pac::Peripherals::steal() };

    // Disable I2C1
    dp.I2C1.cr1.modify(|_, w| w.pe().disabled());
    delay_us(100);

    // Reset I2C1
    dp.RCC.apb1rstr.modify(|_, w| w.i2c1rst().set_bit());
    delay_us(10);
    dp.RCC.apb1rstr.modify(|_, w| w.i2c1rst().clear_bit());
    delay_us(100);

    // Reconfigure I2C1
    init_i2c1_registers();
}

// Configure I2C1 registers (extracted for reuse)
fn init_i2c1_registers() {
    let dp = unsafe { pac::Peripherals::steal() };

    // Configure I2C1 for 100kHz
    // APB1 clock is 42MHz, for 100kHz I2C: CCR = 42MHz / (2 * 100kHz) = 210
    dp.I2C1.cr2.modify(|_, w| unsafe { w.freq().bits(42) }); // APB1 freq in MHz
    dp.I2C1.ccr.modify(|_, w| unsafe { w.ccr().bits(210) }); // For 100kHz
    dp.I2C1.trise.modify(|_, w| w.trise().bits(43)); // 1000ns / 23.8ns + 1

    // Enable I2C1
    dp.I2C1.cr1.modify(|_, w| w.pe().enabled());
}

// I2C1 on PB8 (SCL) and PB9 (SDA) for MPU6050
pub fn init_i2c1() {
    let dp = unsafe { pac::Peripherals::steal() };

    // Enable clocks
    dp.RCC.ahb1enr.modify(|_, w| w.gpioben().enabled());
    dp.RCC.apb1enr.modify(|_, w| w.i2c1en().enabled());

    // Configure GPIO pins PB8 (SCL) and PB9 (SDA) for I2C1
    dp.GPIOB
        .moder
        .modify(|_, w| w.moder8().alternate().moder9().alternate());

    // Set alternate function AF4 for I2C1 (more robust method)
    dp.GPIOB.afrh.modify(|_, w| {
        w.afrh8()
            .bits(4) // AF4 for PB8 (SCL)
            .afrh9()
            .bits(4) // AF4 for PB9 (SDA)
    });

    // Configure as open drain (required for I2C)
    dp.GPIOB
        .otyper
        .modify(|_, w| w.ot8().open_drain().ot9().open_drain());

    // Enable pull-ups (important for I2C bus)
    dp.GPIOB
        .pupdr
        .modify(|_, w| w.pupdr8().pull_up().pupdr9().pull_up());

    // Set speed to medium (high speed can cause issues with I2C)
    dp.GPIOB
        .ospeedr
        .modify(|_, w| w.ospeedr8().medium_speed().ospeedr9().medium_speed());

    // Reset I2C1
    dp.RCC.apb1rstr.modify(|_, w| w.i2c1rst().set_bit());
    delay_us(10);
    dp.RCC.apb1rstr.modify(|_, w| w.i2c1rst().clear_bit());
    delay_us(100);

    // Configure I2C1 registers
    init_i2c1_registers();
}

pub fn i2c1_write_reg(device_addr: u8, reg_addr: u8, data: u8) -> Result<(), ()> {
    let dp = unsafe { pac::Peripherals::steal() };
    let i2c = &dp.I2C1;

    // Wait until bus is free with timeout
    let mut timeout = I2C_TIMEOUT;
    while i2c.sr2.read().busy().bit_is_set() {
        timeout -= 1;
        if timeout == 0 {
            return Err(());
        }
    }

    // Generate start condition
    i2c.cr1.modify(|_, w| w.start().set_bit());
    delay_us(10); // Small delay after start

    // Wait for start condition with timeout
    timeout = I2C_TIMEOUT;
    while !i2c.sr1.read().sb().bit_is_set() {
        timeout -= 1;
        if timeout == 0 {
            return Err(());
        }
    }

    // Send device address (write)
    i2c.dr.write(|w| w.dr().bits(device_addr << 1));

    // Wait for address sent with timeout
    timeout = I2C_TIMEOUT;
    while !i2c.sr1.read().addr().bit_is_set() {
        timeout -= 1;
        if timeout == 0 {
            // Generate stop condition on error
            i2c.cr1.modify(|_, w| w.stop().set_bit());
            return Err(());
        }
    }
    let _ = i2c.sr2.read(); // Clear ADDR flag

    // Send register address
    i2c.dr.write(|w| w.dr().bits(reg_addr));

    // Wait for byte transfer finished with timeout
    timeout = I2C_TIMEOUT;
    while !i2c.sr1.read().tx_e().bit_is_set() {
        timeout -= 1;
        if timeout == 0 {
            i2c.cr1.modify(|_, w| w.stop().set_bit());
            return Err(());
        }
    }

    // Send data
    i2c.dr.write(|w| w.dr().bits(data));

    // Wait for byte transfer finished with timeout
    timeout = I2C_TIMEOUT;
    while !i2c.sr1.read().tx_e().bit_is_set() {
        timeout -= 1;
        if timeout == 0 {
            i2c.cr1.modify(|_, w| w.stop().set_bit());
            return Err(());
        }
    }

    // Generate stop condition
    i2c.cr1.modify(|_, w| w.stop().set_bit());
    delay_us(10); // Small delay after stop

    Ok(())
}

pub fn i2c1_read_reg(device_addr: u8, reg_addr: u8) -> Result<u8, ()> {
    let dp = unsafe { pac::Peripherals::steal() };
    let i2c = &dp.I2C1;

    // Wait until bus is free
    while i2c.sr2.read().busy().bit_is_set() {}

    // Generate start condition
    i2c.cr1.modify(|_, w| w.start().set_bit());

    // Wait for start condition
    while !i2c.sr1.read().sb().bit_is_set() {}

    // Send device address (write)
    i2c.dr.write(|w| w.dr().bits(device_addr << 1));

    // Wait for address sent
    while !i2c.sr1.read().addr().bit_is_set() {}
    let _ = i2c.sr2.read(); // Clear ADDR flag

    // Send register address
    i2c.dr.write(|w| w.dr().bits(reg_addr));

    // Wait for byte transfer finished
    while !i2c.sr1.read().tx_e().bit_is_set() {}

    // Generate repeated start
    i2c.cr1.modify(|_, w| w.start().set_bit());

    // Wait for start condition
    while !i2c.sr1.read().sb().bit_is_set() {}

    // Send device address (read)
    i2c.dr.write(|w| w.dr().bits((device_addr << 1) | 1));

    // Wait for address sent
    while !i2c.sr1.read().addr().bit_is_set() {}

    // Disable ACK
    i2c.cr1.modify(|_, w| w.ack().clear_bit());

    let _ = i2c.sr2.read(); // Clear ADDR flag

    // Generate stop condition
    i2c.cr1.modify(|_, w| w.stop().set_bit());

    // Wait for receive register not empty
    while !i2c.sr1.read().rx_ne().bit_is_set() {}

    let data = i2c.dr.read().dr().bits();

    // Re-enable ACK for future transfers
    i2c.cr1.modify(|_, w| w.ack().set_bit());

    Ok(data)
}

pub fn i2c1_read_bytes(device_addr: u8, reg_addr: u8, buffer: &mut [u8]) -> Result<(), ()> {
    let dp = unsafe { pac::Peripherals::steal() };
    let i2c = &dp.I2C1;

    if buffer.is_empty() {
        return Ok(());
    }

    // Wait until bus is free
    while i2c.sr2.read().busy().bit_is_set() {}

    // Generate start condition
    i2c.cr1.modify(|_, w| w.start().set_bit());

    // Wait for start condition
    while !i2c.sr1.read().sb().bit_is_set() {}

    // Send device address (write)
    i2c.dr.write(|w| w.dr().bits(device_addr << 1));

    // Wait for address sent
    while !i2c.sr1.read().addr().bit_is_set() {}
    let _ = i2c.sr2.read(); // Clear ADDR flag

    // Send register address
    i2c.dr.write(|w| w.dr().bits(reg_addr));

    // Wait for byte transfer finished
    while !i2c.sr1.read().tx_e().bit_is_set() {}

    // Generate repeated start
    i2c.cr1.modify(|_, w| w.start().set_bit());

    // Wait for start condition
    while !i2c.sr1.read().sb().bit_is_set() {}

    // Send device address (read)
    i2c.dr.write(|w| w.dr().bits((device_addr << 1) | 1));

    // Wait for address sent
    while !i2c.sr1.read().addr().bit_is_set() {}

    if buffer.len() == 1 {
        // Single byte read
        i2c.cr1.modify(|_, w| w.ack().clear_bit());
        let _ = i2c.sr2.read(); // Clear ADDR flag
        i2c.cr1.modify(|_, w| w.stop().set_bit());

        while !i2c.sr1.read().rx_ne().bit_is_set() {}
        buffer[0] = i2c.dr.read().dr().bits();
    } else {
        // Multi-byte read
        let _ = i2c.sr2.read(); // Clear ADDR flag

        for i in 0..buffer.len() {
            if i == buffer.len() - 1 {
                // Last byte
                i2c.cr1.modify(|_, w| w.ack().clear_bit());
                i2c.cr1.modify(|_, w| w.stop().set_bit());
            }

            while !i2c.sr1.read().rx_ne().bit_is_set() {}
            buffer[i] = i2c.dr.read().dr().bits();
        }
    }

    // Re-enable ACK for future transfers
    i2c.cr1.modify(|_, w| w.ack().set_bit());

    Ok(())
}
