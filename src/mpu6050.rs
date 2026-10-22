use crate::mpu6050_base::device::{
    ACCEL_SENSITIVTY, DlpfConfig, GYRO_SENSITIVITY, TEMP_OFFSET, TEMP_SENSITIVITY,
};

use crate::mpu6050_base::registers::DEFAULT_ADDRRESS;

pub use crate::mpu6050_base::device::{AccelRange, GyroRange, TemperatureRange};
use crate::mpu6050_base::registers::*;
use embedded_hal::i2c::{I2c, SevenBitAddress};

#[derive(Debug)]
pub enum MpuError<E> {
    NoData,
    I2cError(E),
    InvalidChipID(u8),
    InvalidCalibrationCycles,
    InvalidDlpfValue(u8),
    InvalidGyroRange(u8),
    InvalidTempatureRange(u8),
    InvalidAccelRange(u8),
}

impl<E> From<E> for MpuError<E> {
    fn from(error: E) -> Self {
        MpuError::I2cError(error)
    }
}
#[derive(Debug)]
pub struct Mpu6050Config {
    dlpf: DlpfConfig,
    acel_range: AccelRange,
    gyro_range: GyroRange,
    temp_measurment: TemperatureRange,
    calibration_cycles: u64,
    gyro_bias: (f32, f32, f32),
    accel_bias: (f32, f32, f32),
}

impl Default for Mpu6050Config {
    fn default() -> Self {
        Mpu6050Config {
            dlpf: DlpfConfig::Dlpf256,
            acel_range: AccelRange::G2,
            gyro_range: GyroRange::Degree250,
            temp_measurment: TemperatureRange::C,
            calibration_cycles: 100,
            gyro_bias: (0.0, 0.0, 0.0),
            accel_bias: (0.0, 0.0, 0.0),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct RawReading {
    pub accel: [i16; 3],
    pub tempature: i16,
    pub gyro: [i16; 3],
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Reading {
    pub accel: [f32; 3],
    pub tempature: f32,
    pub gyro: [f32; 3],
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
        self.i2c.write(self.address, &[RA_PWR_MGMT_1, 0x01])?;
        Ok(())
    }

    #[allow(dead_code)]
    fn sleep(&mut self) -> Result<(), MpuError<E>> {
        self.i2c.write(self.address, &[RA_PWR_MGMT_1, 0x41])?;
        Ok(())
    }

    pub fn reset_device(&mut self) -> Result<(), MpuError<E>>{
        self.i2c.write(self.address, &[RA_PWR_MGMT_1, 0x80])?;
        Ok(())
    }

    pub fn verify_device(&mut self) -> Result<(), MpuError<E>> {
        let iam = self.get_who_am_i()?;
        if iam == DEFAULT_ADDRRESS {
            Ok(())
        } else {
            Err(MpuError::InvalidChipID(iam))
        }
    }

    // not a real calibration.
    // just an average over a time and
    pub fn calibrate_device(&mut self) -> Result<(), MpuError<E>> {
        self.i2c.write(self.address, &[RA_CONFIG, 0x01])?;
        self.i2c.write(self.address, &[RA_SMPLRT_DIV, 0x00])?;

        let mut total_accel = (0.0, 0.0, 0.0);
        let mut total_gyro = (0.0, 0.0, 0.0);
        let mut count = 0;

        while count < self.config.calibration_cycles {
            let accel = self.get_accel()?;
            let gyro = self.get_gyro()?;

            total_accel.0 += accel.0;
            total_accel.1 += accel.1;
            total_accel.2 += accel.2;

            total_gyro.0 += gyro.0;
            total_gyro.1 += gyro.1;
            total_gyro.2 += gyro.2;

            count += 1;
        }

        self.config.accel_bias = (
            total_accel.0 / count as f32,
            total_accel.1 / count as f32,
            total_accel.2 / count as f32,
        );
        self.config.gyro_bias = (
            total_gyro.0 / count as f32,
            total_gyro.1 / count as f32,
            total_gyro.2 / count as f32,
        );
        Ok(())
    }

    pub fn init(&mut self) -> Result<(), MpuError<E>> {
        // verify device ID
        self.verify_device()?;
        // wake device
        self.wake()?;
        self.set_dlpf(self.config.dlpf as u8)?;
        self.set_accel_range(self.config.acel_range as u8)?;
        self.set_gyro_range(self.config.gyro_range as u8)?;

        Ok(())
    }

    pub fn set_dlpf(&mut self, dlpf: u8) -> Result<(), MpuError<E>> {
        self.i2c.write(self.address, &[RA_CONFIG, dlpf & 0x7])?;
        self.config.dlpf =
            DlpfConfig::try_from(dlpf).map_err(|_| MpuError::InvalidDlpfValue(dlpf))?;
        Ok(())
    }

    pub fn set_temp_measure(&mut self, temp_measure: u8) -> Result<(), MpuError<E>> {
        self.config.temp_measurment = TemperatureRange::try_from(temp_measure)
            .map_err(|_| MpuError::InvalidTempatureRange(temp_measure))?;
        Ok(())
    }

    pub fn set_accel_range(&mut self, range: u8) -> Result<(), MpuError<E>> {
        self.i2c
            .write(self.address, &[RA_ACCEL_CONFIG, range as u8])?;

        self.config.acel_range =
            AccelRange::try_from(range).map_err(|_| MpuError::InvalidAccelRange(range))?;
        Ok(())
    }

    pub fn set_gyro_range(&mut self, range: u8) -> Result<(), MpuError<E>> {
        self.i2c
            .write(self.address, &[RA_GYRO_CONFIG, range << 3])?;

        self.config.gyro_range =
            GyroRange::try_from(range).map_err(|_| MpuError::InvalidGyroRange(range))?;
        Ok(())
    }

    pub fn get_data(&mut self) -> Result<Reading, MpuError<E>> {
        let Ok(raw) = self.get_raw_data() else {
            return Err(MpuError::NoData);
        };

        let celcius = raw.tempature as f32 / TEMP_SENSITIVITY + TEMP_OFFSET;
        let accel_scale = match self.config.acel_range {
            AccelRange::G2 => ACCEL_SENSITIVTY.0,
            AccelRange::G4 => ACCEL_SENSITIVTY.1,
            AccelRange::G8 => ACCEL_SENSITIVTY.2,
            AccelRange::G16 => ACCEL_SENSITIVTY.3,
        };
        let gyro_scale = match self.config.gyro_range {
            GyroRange::Degree250 => GYRO_SENSITIVITY.0,
            GyroRange::Degree500 => GYRO_SENSITIVITY.1,
            GyroRange::Degree1000 => GYRO_SENSITIVITY.2,
            GyroRange::Degree2000 => GYRO_SENSITIVITY.3,
        };

        // Convert the raw readings to readable readings
        Ok(Reading {
            accel: raw.accel.map(|val| val as f32 / accel_scale),
            tempature: match self.config.temp_measurment {
                TemperatureRange::C => celcius,
                TemperatureRange::F => (celcius * 9.0 / 5.0) + 32.0,
                TemperatureRange::K => celcius + 273.15,
            },
            gyro: raw.gyro.map(|val| val as f32 / gyro_scale),
        })
    }

    pub fn get_temperature(&mut self) -> Result<f32, MpuError<E>> {
        let mut buf = [0u8; 2];
        self.i2c
            .write_read(self.address, &[RA_TEMP_OUT_H], &mut buf)?;

        let raw_temp = i16::from_be_bytes(buf);
        let celcius = (raw_temp as f32 / TEMP_SENSITIVITY) + TEMP_OFFSET;

        Ok(match self.config.temp_measurment {
            TemperatureRange::C => celcius,
            TemperatureRange::F => (celcius * 9.0 / 5.0) + 32.0,
            TemperatureRange::K => celcius + 273.15,
        })
    }

    // returns a tuple of (x, y, z) values
    pub fn get_gyro(&mut self) -> Result<(f32, f32, f32), MpuError<E>> {
        let scale = match self.config.gyro_range {
            GyroRange::Degree250 => GYRO_SENSITIVITY.0,
            GyroRange::Degree500 => GYRO_SENSITIVITY.1,
            GyroRange::Degree1000 => GYRO_SENSITIVITY.2,
            GyroRange::Degree2000 => GYRO_SENSITIVITY.3,
        };

        let (x, y, z) = self.get_raw_gyro_all()?;
        let x = x as f32 / scale - self.config.gyro_bias.0;
        let y = y as f32 / scale - self.config.gyro_bias.1;
        let z = z as f32 / scale - self.config.gyro_bias.2;
        Ok((x, y, z))
    }

    // returns a tuple of (x, y, z) values
    pub fn get_accel(&mut self) -> Result<(f32, f32, f32), MpuError<E>> {
        let scale = match self.config.acel_range {
            AccelRange::G2 => ACCEL_SENSITIVTY.0,
            AccelRange::G4 => ACCEL_SENSITIVTY.1,
            AccelRange::G8 => ACCEL_SENSITIVTY.2,
            AccelRange::G16 => ACCEL_SENSITIVTY.3,
        };

        let (x, y, z) = self.get_raw_accel_all()?;
        let x = x as f32 / scale - self.config.accel_bias.0;
        let y = y as f32 / scale - self.config.accel_bias.1;
        let z = z as f32 / scale - self.config.accel_bias.2;
        Ok((x, y, z))
    }

    pub fn get_raw_data(&mut self) -> Result<RawReading, MpuError<E>> {
        let mut buf = [0u8; 14];
        self.i2c
            .write_read(self.address, &[RA_ACCEL_XOUT_H], &mut buf)?;
        Ok(RawReading {
            accel: [
                i16::from_be_bytes([buf[0], buf[1]]),
                i16::from_be_bytes([buf[2], buf[3]]),
                i16::from_be_bytes([buf[4], buf[5]]),
            ],
            tempature: i16::from_be_bytes([buf[6], buf[7]]),
            gyro: [
                i16::from_be_bytes([buf[8], buf[9]]),
                i16::from_be_bytes([buf[10], buf[11]]),
                i16::from_be_bytes([buf[12], buf[13]]),
            ],
        })
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

    fn get_who_am_i(&mut self) -> Result<u8, MpuError<E>> {
        let mut buf = [0u8; 1];
        self.i2c
            .write_read(self.address, &[RA_WHO_AM_I], &mut buf)?;

        Ok(u8::from_be_bytes([buf[0]]))
    }
}

// --------- TESTS ----------
