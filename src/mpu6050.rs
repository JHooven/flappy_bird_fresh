#![allow(dead_code)]

use crate::i2c;
use crate::input_device::AccelData;

const MPU6050_ADDR: u8 = 0x68;

// MPU6050 Register addresses
const WHO_AM_I: u8 = 0x75;
const PWR_MGMT_1: u8 = 0x6B;
const GYRO_CONFIG: u8 = 0x1B;
const ACCEL_CONFIG: u8 = 0x1C;
const ACCEL_XOUT_H: u8 = 0x3B;

pub struct Mpu6050Data {
    pub accel_x: i32,
    pub accel_y: i32,
    pub accel_z: i32,
    pub temp: i32,
    pub gyro_x: i32,
    pub gyro_y: i32,
    pub gyro_z: i32,
}

pub fn init() -> Result<(), ()> {
    // Check WHO_AM_I register
    match i2c::i2c1_read_reg(MPU6050_ADDR, WHO_AM_I) {
        Ok(id) if id == 0x68 => {}
        _ => return Err(()),
    }

    // Wake up the MPU6050 (exit sleep mode)
    i2c::i2c1_write_reg(MPU6050_ADDR, PWR_MGMT_1, 0x00)?;

    // Set gyroscope range to ±250°/s
    i2c::i2c1_write_reg(MPU6050_ADDR, GYRO_CONFIG, 0x00)?;

    // Set accelerometer range to ±2g
    i2c::i2c1_write_reg(MPU6050_ADDR, ACCEL_CONFIG, 0x00)?;

    Ok(())
}

pub fn read_data() -> Result<Mpu6050Data, ()> {
    let mut buffer = [0u8; 14];

    // Read all data registers at once (ACCEL_XOUT_H to GYRO_ZOUT_L)
    i2c::i2c1_read_bytes(MPU6050_ADDR, ACCEL_XOUT_H, &mut buffer)?;

    // Convert bytes to i32 values (big-endian)
    let accel_x = ((buffer[0] as i32) << 8) | (buffer[1] as i32);
    let accel_y = ((buffer[2] as i32) << 8) | (buffer[3] as i32);
    let accel_z = ((buffer[4] as i32) << 8) | (buffer[5] as i32);
    let temp = ((buffer[6] as i32) << 8) | (buffer[7] as i32);
    let gyro_x = ((buffer[8] as i32) << 8) | (buffer[9] as i32);
    let gyro_y = ((buffer[10] as i32) << 8) | (buffer[11] as i32);
    let gyro_z = ((buffer[12] as i32) << 8) | (buffer[13] as i32);

    Ok(Mpu6050Data {
        accel_x,
        accel_y,
        accel_z,
        temp,
        gyro_x,
        gyro_y,
        gyro_z,
    })
}
pub fn read_accel_data() -> Result<AccelData, ()> {
    let mut buffer = [0u8; 6];

    // Read accelerometer registers (ACCEL_XOUT_H to ACCEL_ZOUT_L)
    i2c::i2c1_read_bytes(MPU6050_ADDR, ACCEL_XOUT_H, &mut buffer)?;

    // Convert bytes to i32 values (big-endian)
    let accel_x = ((buffer[0] as i32) << 8) | (buffer[1] as i32);
    let accel_y = ((buffer[2] as i32) << 8) | (buffer[3] as i32);
    let accel_z = ((buffer[4] as i32) << 8) | (buffer[5] as i32);

    Ok(AccelData {
        accel_x,
        accel_y,
        accel_z,
    })
}
