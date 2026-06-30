#![no_std]

mod device;

use core::default;

use crate::device::device::{
    ACCEL_SENSITIVTY, AccelRange, DEFAULT_ADDRRESS, GYRO_SENSITIVITY, TEMP_OFFSET, TEMP_SENSITIVITY,
};
use crate::device::registers::*;
use embedded_hal::delay::DelayNs;
use embedded_hal::i2c::{I2c, SevenBitAddress};

pub enum MpuError<E> {
    I2cError(E),
    InvalidChipID(u8),
}

impl<E> From<E> for MpuError<E> {
    fn from(error: E) -> Self {
        MpuError::I2cError(error)
    }
}
#[derive(Debug)]
pub struct Mpu6050Config {
    delay_time: u32,
    acel_sens: f32,
    gyro_sens: f32,
}

impl Default for Mpu6050Config {
    fn default() -> Self {
        Mpu6050Config {
            delay_time: 100,
            acel_sens: ACCEL_SENSITIVTY.0,
            gyro_sens: GYRO_SENSITIVITY.0,
        }
    }
}

#[derive(Debug)]
pub struct Mpu6050<I2C> {
    i2c: I2C,
    address: u8,
    config: Mpu6050Config,
}

impl<I2C: I2c<SevenBitAddress>, E> Mpu6050<I2C>
where
    I2C: I2c<Error = E>,
{
    pub fn new(i2c: I2C) -> Mpu6050<I2C> {
        Mpu6050 {
            i2c,
            address: DEFAULT_ADDRRESS,
            config: Mpu6050Config::default(),
        }
    }

    fn wake<D: DelayNs>(&mut self, delay: &mut D) -> Result<(), MpuError<E>> {
        self.i2c.write(self.address, &[RA_PWR_MGM_1, 0x00])?;
        delay.delay_ms(self.config.delay_time);
        Ok(())
    }

    pub fn set_delay_length(&mut self, delay: u32) {
        self.config.delay_time = delay;
    }

    fn sleep<D: DelayNs>(&mut self, delay: &mut D) -> Result<(), MpuError<E>> {
        self.i2c.write(self.address, &[RA_PWR_MGM_1, 0x01])?;
        delay.delay_ms(self.config.delay_time);
        Ok(())
    }

    pub fn read_tempature(&mut self) -> Result<f32, MpuError<E>> {
        let mut buf = [0u8; 2];
        self.i2c
            .write_read(self.address, &[RA_TEMP_OUT_H], &mut buf)?;

        let raw_temp = i16::from_be_bytes(buf);
        let celcius = ((raw_temp as f32 + TEMP_OFFSET) / TEMP_SENSITIVITY) + 35.0;

        Ok(celcius)
    }

    pub fn set_accel_range<D: DelayNs>(
        &mut self,
        range: AccelRange,
        delay: &mut D,
    ) -> Result<(), MpuError<E>> {
        self.wake(delay)?;
        self.i2c
            .write_read(self.address, &[RA_ACCEL_CONFIG], &mut [range as u8])?;
        self.sleep(delay)?;
        Ok(())
    }
}
