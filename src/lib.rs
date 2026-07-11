#![no_std]

mod device;

use crate::device::device::{
    ACCEL_SENSITIVTY, DEFAULT_ADDRRESS, GYRO_SENSITIVITY, TEMP_OFFSET, TEMP_SENSITIVITY,
};

use crate::device::registers::{self, *};
pub use device::device::{AccelRange, GyroRange, TempatureRange};
use embedded_hal::delay::{self, DelayNs};
use embedded_hal::i2c::{I2c, SevenBitAddress};

use core::f32::consts::PI;

#[derive(Debug)]
pub enum MpuError<E> {
    I2cError(E),
    InvalidChipID(u8),
    InvalidGyroRange(GyroRange),
    InvalidTempatureRange(TempatureRange),
}

impl<E> From<E> for MpuError<E> {
    fn from(error: E) -> Self {
        MpuError::I2cError(error)
    }
}
#[derive(Debug)]
pub struct Mpu6050Config {
    global_delay_time: u32,
    acel_range: AccelRange,
    gyro_range: GyroRange,
    temp_measurment: TempatureRange,
    calibration_length_sec: u32,
    calib_offsets: (f32, f32, f32),
}

impl Default for Mpu6050Config {
    fn default() -> Self {
        Mpu6050Config {
            global_delay_time: 200,
            acel_range: AccelRange::G2,
            gyro_range: GyroRange::Degree250,
            temp_measurment: TempatureRange::C,
            calibration_length_sec: 2,
            calib_offsets: (0.0, 0.0, 0.0),
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
    pub fn new(i2c: I2C) -> Result<Mpu6050<I2C>, MpuError<E>> {
        Ok(Mpu6050 {
            i2c,
            address: DEFAULT_ADDRRESS,
            config: Mpu6050Config::default(),
        })
    }

    fn wake(&mut self) -> Result<(), MpuError<E>> {
        self.i2c.write(self.address, &[RA_PWR_MGMT_2, 0x00])?;
        Ok(())
    }

    #[allow(dead_code)]
    fn sleep(&mut self) -> Result<(), MpuError<E>> {
        self.i2c.write(self.address, &[RA_PWR_MGMT_2, 0x01])?;
        Ok(())
    }

    /*
    pub fn calibrate_gyro(delay: &mut impl DelayNs) -> Result<(), <MpuError<E>> {
        unimplemented!("help");
        let mut i = 0;
        loop {

        }
        Ok(())
    }
        */

    pub fn init(&mut self, delay: &mut impl DelayNs) -> Result<(), MpuError<E>> {
        self.wake()?;
        delay.delay_ns(self.config.global_delay_time);
        self.set_accel_range(self.config.acel_range)?;
        delay.delay_ns(self.config.global_delay_time);
        self.set_gyro_range(self.config.gyro_range)?;
        Ok(())
    }

    pub fn swap_device(
        &mut self,
        new_device: u8,
        delay: &mut impl DelayNs,
    ) -> Result<(), MpuError<E>> {
        unimplemented!("Master/Slave not implemented as of this time");
    }

    pub fn set_temp_measure(&mut self, temp_measure: TempatureRange) -> Result<(), MpuError<E>> {
        self.config.temp_measurment = temp_measure;
        Ok(())
    }

    pub fn set_accel_range(&mut self, range: AccelRange) -> Result<(), MpuError<E>> {
        self.config.acel_range = range;
        let range_bits = (range as u8) << 3;
        self.i2c
            .write(self.address, &[RA_ACCEL_CONFIG, range_bits])?;
        Ok(())
    }

    pub fn set_gyro_range(&mut self, range: GyroRange) -> Result<(), MpuError<E>> {
        self.config.gyro_range = range;
        let gyro_range_bits = (range as u8) << 3;
        self.i2c
            .write(self.address, &[RA_GYRO_CONFIG, gyro_range_bits])?;
        Ok(())
    }

    pub fn get_tempature(&mut self) -> Result<f32, MpuError<E>> {
        let mut buf = [0u8; 2];
        self.i2c
            .write_read(self.address, &[RA_TEMP_OUT_H], &mut buf)?;

        let raw_temp = i16::from_be_bytes(buf);
        let celcius = ((raw_temp as f32) / TEMP_SENSITIVITY) + TEMP_OFFSET; //(((raw  / 340) + 36.53)
        match self.config.temp_measurment {
            TempatureRange::C => return Ok(celcius),
            TempatureRange::F => return Ok((celcius * 9.0 / 5.0) + 35.0),
            TempatureRange::K => return Ok((celcius + 273.15).abs()),
        }
    }

    pub fn set_delay_length(&mut self, delay: u32) {
        self.config.global_delay_time = delay;
    }

    // returns a tuple of (x, y, z) values
    pub fn get_gyro(&mut self, delay: &mut impl DelayNs) -> Result<(f32, f32, f32), MpuError<E>> {
        delay.delay_ms(self.config.global_delay_time);

        let scale = match self.config.gyro_range {
            GyroRange::Degree250 => GYRO_SENSITIVITY.0,
            GyroRange::Degree500 => GYRO_SENSITIVITY.1,
            GyroRange::Degree1000 => GYRO_SENSITIVITY.2,
            GyroRange::Degree2000 => GYRO_SENSITIVITY.3,
        };

        let (x, y, z) = self.get_raw_gyro_all()?;
        let x = x as f32 / scale;
        let y = y as f32 / scale;
        let z = z as f32 / scale;
        Ok((x, y, z))
    }

    pub fn get_accel(&mut self, delay: &mut impl DelayNs) -> Result<(f32, f32, f32), MpuError<E>> {
        delay.delay_ms(self.config.global_delay_time);
        let scale = match self.config.acel_range {
            AccelRange::G2 => ACCEL_SENSITIVTY.0,
            AccelRange::G4 => ACCEL_SENSITIVTY.1,
            AccelRange::G8 => ACCEL_SENSITIVTY.2,
            AccelRange::G16 => ACCEL_SENSITIVTY.3,
        };

        let (x, y, z) = self.get_raw_accel_all()?;
        let x = x as f32 / scale;
        let y = y as f32 / scale;
        let z = z as f32 / scale;
        Ok((x, y, z))
    }

    fn get_raw_accel_all(&mut self) -> Result<(i16, i16, i16), MpuError<E>> {
        let mut buf = [0u8; 6];
        self.i2c
            .write_read(self.address, &[RA_ACCEL_XOUT_H], &mut buf)?;
        let x = i16::from_be_bytes([buf[0], buf[1]]);
        let y = i16::from_be_bytes([buf[2], buf[3]]);
        let z = i16::from_be_bytes([buf[4], buf[5]]);
        Ok((x, y, z))
    }

    fn get_raw_gyro_all(&mut self) -> Result<(i16, i16, i16), MpuError<E>> {
        let mut buf = [0u8; 6];
        self.i2c
            .write_read(self.address, &[RA_GYRO_XOUT_H], &mut buf)?;
        let x = i16::from_be_bytes([buf[0], buf[1]]);
        let y = i16::from_be_bytes([buf[2], buf[3]]);
        let z = i16::from_be_bytes([buf[4], buf[5]]);

        Ok((x, y, z))
    }
}
