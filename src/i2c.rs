use stm32f4::stm32f429 as pac;

// I2C1 on PB8 (SCL) and PB9 (SDA) for MPU6050
pub fn init_i2c1() {
    let dp = unsafe { pac::Peripherals::steal() };
    
    // Enable clocks
    dp.RCC.ahb1enr.modify(|_, w| w.gpioben().enabled());
    dp.RCC.apb1enr.modify(|_, w| w.i2c1en().enabled());
    
    // Configure GPIO pins PB8 and PB9 for I2C1
    dp.GPIOB.moder.modify(|_, w| w
        .moder8().alternate()
        .moder9().alternate()
    );
    // Set alternate function AF4 for I2C1
    dp.GPIOB.afrh.write(|w| unsafe {
        w.bits(
            (dp.GPIOB.afrh.read().bits() & !(0xF << (8-8)*4) & !(0xF << (9-8)*4)) |
            (4 << (8-8)*4) | (4 << (9-8)*4)
        )
    });
    dp.GPIOB.otyper.modify(|_, w| w
        .ot8().open_drain()
        .ot9().open_drain()
    );
    dp.GPIOB.pupdr.modify(|_, w| w
        .pupdr8().pull_up()
        .pupdr9().pull_up()
    );
    dp.GPIOB.ospeedr.modify(|_, w| w
        .ospeedr8().very_high_speed()
        .ospeedr9().very_high_speed()
    );

    // Reset I2C1
    dp.RCC.apb1rstr.modify(|_, w| w.i2c1rst().set_bit());
    dp.RCC.apb1rstr.modify(|_, w| w.i2c1rst().clear_bit());

    // Configure I2C1 for 100kHz
    // APB1 clock is 42MHz, for 100kHz I2C: CCR = 42MHz / (2 * 100kHz) = 210
    dp.I2C1.cr2.modify(|_, w| unsafe { w.freq().bits(42) }); // APB1 freq in MHz
    dp.I2C1.ccr.modify(|_, w| unsafe { w.ccr().bits(210) }); // For 100kHz
    dp.I2C1.trise.modify(|_, w| w.trise().bits(43)); // 1000ns / 23.8ns + 1
    
    // Enable I2C1
    dp.I2C1.cr1.modify(|_, w| w.pe().enabled());
}

pub fn i2c1_write_reg(device_addr: u8, reg_addr: u8, data: u8) -> Result<(), ()> {
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
    
    // Send data
    i2c.dr.write(|w| w.dr().bits(data));
    
    // Wait for byte transfer finished
    while !i2c.sr1.read().tx_e().bit_is_set() {}
    
    // Generate stop condition
    i2c.cr1.modify(|_, w| w.stop().set_bit());
    
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