use crate::mpu6050_base::device::{
    ACCEL_SENSITIVITY, DlpfConfig, GYRO_SENSITIVITY, TEMP_OFFSET, TEMP_SENSITIVITY,
};

pub use crate::mpu6050_base::device::{AccelRange, GyroRange, TemperatureRange};
use crate::mpu6050_base::registers::*;
use embedded_hal::delay::DelayNs;
use embedded_hal::i2c::{I2c, SevenBitAddress};

#[derive(Debug)]
pub enum MpuError<E> {
    I2cError(E),
    InvalidChipId(u8),
    InvalidAddress(u8),
    InvalidCalibrationCycles,
}

impl<E> From<E> for MpuError<E> {
    fn from(error: E) -> Self {
        MpuError::I2cError(error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mpu6050Config {
    dlpf: DlpfConfig,
    accel_range: AccelRange,
    gyro_range: GyroRange,
    temperature_unit: TemperatureRange,
    sample_rate_divider: u8,
    calibration_cycles: u64,
    gyro_bias: (f32, f32, f32),
    accel_bias: (f32, f32, f32),
}

impl Mpu6050Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_dlpf(mut self, dlpf: DlpfConfig) -> Self {
        self.dlpf = dlpf;
        self
    }

    pub fn with_accel_range(mut self, accel_range: AccelRange) -> Self {
        self.accel_range = accel_range;
        self
    }

    pub fn with_gyro_range(mut self, gyro_range: GyroRange) -> Self {
        self.gyro_range = gyro_range;
        self
    }

    pub fn with_temperature_unit(mut self, temperature_unit: TemperatureRange) -> Self {
        self.temperature_unit = temperature_unit;
        self
    }

    pub fn with_sample_rate_divider(mut self, sample_rate_divider: u8) -> Self {
        self.sample_rate_divider = sample_rate_divider;
        self
    }

    pub fn with_calibration_cycles(mut self, calibration_cycles: u64) -> Self {
        self.calibration_cycles = calibration_cycles;
        self
    }

    pub fn with_gyro_bias(mut self, gyro_bias: (f32, f32, f32)) -> Self {
        self.gyro_bias = gyro_bias;
        self
    }

    pub fn with_accel_bias(mut self, accel_bias: (f32, f32, f32)) -> Self {
        self.accel_bias = accel_bias;
        self
    }
}

impl Default for Mpu6050Config {
    fn default() -> Self {
        Mpu6050Config {
            dlpf: DlpfConfig::Dlpf256,
            accel_range: AccelRange::G2,
            gyro_range: GyroRange::Degree250,
            temperature_unit: TemperatureRange::C,
            sample_rate_divider: 0,
            calibration_cycles: 100,
            gyro_bias: (0.0, 0.0, 0.0),
            accel_bias: (0.0, 0.0, 0.0),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct RawReading {
    pub accel: [i16; 3],
    pub temperature: i16,
    pub gyro: [i16; 3],
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Reading {
    pub accel: [f32; 3],
    pub temperature: f32,
    pub gyro: [f32; 3],
}

#[derive(Debug)]
pub struct Mpu6050<I2C> {
    i2c: I2C,
    address: u8,
    config: Mpu6050Config,
}

impl<I2C, E> Mpu6050<I2C>
where
    I2C: I2c<SevenBitAddress, Error = E>,
{
    pub fn new(i2c: I2C) -> Mpu6050<I2C> {
        Mpu6050 {
            i2c,
            address: DEFAULT_ADDRESS,
            config: Mpu6050Config::default(),
        }
    }

    pub fn new_with_config(i2c: I2C, config: Mpu6050Config) -> Mpu6050<I2C> {
        Mpu6050 {
            i2c,
            address: DEFAULT_ADDRESS,
            config,
        }
    }

    pub fn new_with_address(i2c: I2C, address: u8) -> Result<Mpu6050<I2C>, MpuError<E>> {
        let mut device = Self::new(i2c);
        device.set_address(address)?;
        Ok(device)
    }

    pub fn address(&self) -> u8 {
        self.address
    }

    pub fn set_address(&mut self, address: u8) -> Result<(), MpuError<E>> {
        if address != ADDRESS_AD0_LOW && address != ADDRESS_AD0_HIGH {
            return Err(MpuError::InvalidAddress(address));
        }

        self.address = address;
        Ok(())
    }

    pub fn config(&self) -> &Mpu6050Config {
        &self.config
    }

    pub fn set_calibration_cycles(&mut self, cycles: u64) {
        self.config.calibration_cycles = cycles;
    }

    pub fn set_gyro_bias(&mut self, bias: (f32, f32, f32)) {
        self.config.gyro_bias = bias;
    }

    pub fn set_accel_bias(&mut self, bias: (f32, f32, f32)) {
        self.config.accel_bias = bias;
    }

    pub fn gyro_bias(&self) -> (f32, f32, f32) {
        self.config.gyro_bias
    }

    pub fn accel_bias(&self) -> (f32, f32, f32) {
        self.config.accel_bias
    }

    pub fn wake(&mut self) -> Result<(), MpuError<E>> {
        self.update_register(
            RA_PWR_MGMT_1,
            PWR_MGMT_1_SLEEP | PWR_MGMT_1_CLKSEL_MASK,
            PWR_MGMT_1_CLKSEL_PLL_X_GYRO,
        )
    }

    pub fn sleep(&mut self) -> Result<(), MpuError<E>> {
        self.set_sleep(true)
    }

    pub fn set_sleep(&mut self, sleep: bool) -> Result<(), MpuError<E>> {
        self.update_register(
            RA_PWR_MGMT_1,
            PWR_MGMT_1_SLEEP,
            if sleep { PWR_MGMT_1_SLEEP } else { 0 },
        )
    }

    pub fn reset_device(&mut self, delay: &mut impl DelayNs) -> Result<(), MpuError<E>> {
        self.write_register(RA_PWR_MGMT_1, PWR_MGMT_1_DEVICE_RESET)?;
        delay.delay_ms(100);
        self.init(delay)
    }

    pub fn verify_device(&mut self) -> Result<(), MpuError<E>> {
        let iam = self.get_who_am_i()?;
        if iam == self.address {
            Ok(())
        } else {
            Err(MpuError::InvalidChipID(iam))
        }
    }

    pub fn init(&mut self, delay: &mut impl DelayNs) -> Result<(), MpuError<E>> {
        self.verify_device()?;
        self.wake()?;
        delay.delay_ms(100);

        self.set_sample_rate_divider(self.config.sample_rate_divider)?;
        self.set_dlpf(self.config.dlpf)?;
        self.set_accel_range(self.config.accel_range)?;
        self.set_gyro_range(self.config.gyro_range)?;

        Ok(())
    }

    pub fn calibrate_gyro(&mut self, delay: &mut impl DelayNs) -> Result<(), MpuError<E>> {
        let cycles = self.config.calibration_cycles;
        if cycles == 0 {
            return Err(MpuError::InvalidCalibrationCycles);
        }

        let mut total = (0i64, 0i64, 0i64);

        for _ in 0..cycles {
            let (x, y, z) = self.get_raw_gyro_all()?;
            total.0 += x as i64;
            total.1 += y as i64;
            total.2 += z as i64;
            delay.delay_ms(2);
        }

        let scale = gyro_scale(self.config.gyro_range);
        let cycles = cycles as f32;
        self.config.gyro_bias = (
            total.0 as f32 / cycles / scale,
            total.1 as f32 / cycles / scale,
            total.2 as f32 / cycles / scale,
        );

        Ok(())
    }

    pub fn calibrate_device(&mut self, delay: &mut impl DelayNs) -> Result<(), MpuError<E>> {
        self.calibrate_gyro(delay)
    }

    pub fn set_sample_rate_divider(&mut self, divider: u8) -> Result<(), MpuError<E>> {
        self.write_register(RA_SMPLRT_DIV, divider)?;
        self.config.sample_rate_divider = divider;
        Ok(())
    }

    pub fn set_dlpf(&mut self, dlpf: DlpfConfig) -> Result<(), MpuError<E>> {
        self.update_register(RA_CONFIG, CONFIG_DLPF_CFG_MASK, dlpf as u8)?;
        self.config.dlpf = dlpf;
        Ok(())
    }

    pub fn set_temperature_unit(&mut self, unit: TemperatureRange) {
        self.config.temperature_unit = unit;
    }

    pub fn set_accel_range(&mut self, range: AccelRange) -> Result<(), MpuError<E>> {
        self.update_register(
            RA_ACCEL_CONFIG,
            ACCEL_CONFIG_AFS_SEL_MASK,
            (range as u8) << 3,
        )?;
        self.config.accel_range = range;
        Ok(())
    }

    pub fn set_gyro_range(&mut self, range: GyroRange) -> Result<(), MpuError<E>> {
        self.update_register(RA_GYRO_CONFIG, GYRO_CONFIG_FS_SEL_MASK, (range as u8) << 3)?;
        self.config.gyro_range = range;
        Ok(())
    }

    pub fn set_fifo_enabled(&mut self, enabled: bool) -> Result<(), MpuError<E>> {
        self.update_register(
            RA_USER_CTRL,
            USER_CTRL_FIFO_EN,
            if enabled { USER_CTRL_FIFO_EN } else { 0 },
        )
    }

    pub fn set_fifo_sources(&mut self, sources: u8) -> Result<(), MpuError<E>> {
        self.write_register(RA_FIFO_EN, sources)
    }

    pub fn get_fifo_count(&mut self) -> Result<u16, MpuError<E>> {
        let bytes = self.read_registers::<2>(RA_FIFO_COUNTH)?;
        Ok(u16::from_be_bytes(bytes))
    }

    pub fn get_data(&mut self) -> Result<Reading, MpuError<E>> {
        let raw = self.get_raw_data()?;
        let accel_scale = accel_scale(self.config.accel_range);
        let gyro_scale = gyro_scale(self.config.gyro_range);

        Ok(Reading {
            accel: [
                raw.accel[0] as f32 / accel_scale - self.config.accel_bias.0,
                raw.accel[1] as f32 / accel_scale - self.config.accel_bias.1,
                raw.accel[2] as f32 / accel_scale - self.config.accel_bias.2,
            ],
            temperature: self
                .convert_temperature(raw.temperature as f32 / TEMP_SENSITIVITY + TEMP_OFFSET),
            gyro: [
                raw.gyro[0] as f32 / gyro_scale - self.config.gyro_bias.0,
                raw.gyro[1] as f32 / gyro_scale - self.config.gyro_bias.1,
                raw.gyro[2] as f32 / gyro_scale - self.config.gyro_bias.2,
            ],
        })
    }

    pub fn get_temperature(&mut self) -> Result<f32, MpuError<E>> {
        let buf = self.read_registers::<2>(RA_TEMP_OUT_H)?;
        let raw_temperature = i16::from_be_bytes(buf);
        let celsius = raw_temperature as f32 / TEMP_SENSITIVITY + TEMP_OFFSET;

        Ok(self.convert_temperature(celsius))
    }

    pub fn get_gyro(&mut self) -> Result<(f32, f32, f32), MpuError<E>> {
        let scale = gyro_scale(self.config.gyro_range);
        let (x, y, z) = self.get_raw_gyro_all()?;

        Ok((
            x as f32 / scale - self.config.gyro_bias.0,
            y as f32 / scale - self.config.gyro_bias.1,
            z as f32 / scale - self.config.gyro_bias.2,
        ))
    }

    pub fn get_accel(&mut self) -> Result<(f32, f32, f32), MpuError<E>> {
        let scale = accel_scale(self.config.accel_range);
        let (x, y, z) = self.get_raw_accel_all()?;

        Ok((
            x as f32 / scale - self.config.accel_bias.0,
            y as f32 / scale - self.config.accel_bias.1,
            z as f32 / scale - self.config.accel_bias.2,
        ))
    }

    pub fn get_raw_data(&mut self) -> Result<RawReading, MpuError<E>> {
        let buf = self.read_registers::<14>(RA_ACCEL_XOUT_H)?;

        Ok(RawReading {
            accel: [
                i16::from_be_bytes([buf[0], buf[1]]),
                i16::from_be_bytes([buf[2], buf[3]]),
                i16::from_be_bytes([buf[4], buf[5]]),
            ],
            temperature: i16::from_be_bytes([buf[6], buf[7]]),
            gyro: [
                i16::from_be_bytes([buf[8], buf[9]]),
                i16::from_be_bytes([buf[10], buf[11]]),
                i16::from_be_bytes([buf[12], buf[13]]),
            ],
        })
    }

    pub fn get_raw_accel_all(&mut self) -> Result<(i16, i16, i16), MpuError<E>> {
        let buf = self.read_registers::<6>(RA_ACCEL_XOUT_H)?;
        let x = i16::from_be_bytes([buf[0], buf[1]]);
        let y = i16::from_be_bytes([buf[2], buf[3]]);
        let z = i16::from_be_bytes([buf[4], buf[5]]);

        Ok((x, y, z))
    }

    pub fn get_raw_gyro_all(&mut self) -> Result<(i16, i16, i16), MpuError<E>> {
        let buf = self.read_registers::<6>(RA_GYRO_XOUT_H)?;
        let x = i16::from_be_bytes([buf[0], buf[1]]);
        let y = i16::from_be_bytes([buf[2], buf[3]]);
        let z = i16::from_be_bytes([buf[4], buf[5]]);

        Ok((x, y, z))
    }

    pub fn get_who_am_i(&mut self) -> Result<u8, MpuError<E>> {
        self.read_register(RA_WHO_AM_I)
    }

    pub fn read_register(&mut self, reg: u8) -> Result<u8, MpuError<E>> {
        Ok(self.read_registers::<1>(reg)?[0])
    }

    pub fn read_registers<const N: usize>(&mut self, reg: u8) -> Result<[u8; N], MpuError<E>> {
        let mut buf = [0u8; N];
        self.i2c.write_read(self.address, &[reg], &mut buf)?;

        Ok(buf)
    }

    pub fn write_register(&mut self, reg: u8, data: u8) -> Result<(), MpuError<E>> {
        self.i2c.write(self.address, &[reg, data])?;
        Ok(())
    }

    pub fn update_register(&mut self, reg: u8, mask: u8, value: u8) -> Result<(), MpuError<E>> {
        let current = self.read_register(reg)?;
        let updated = (current & !mask) | (value & mask);
        self.write_register(reg, updated)
    }

    fn convert_temperature(&self, celsius: f32) -> f32 {
        match self.config.temperature_unit {
            TemperatureRange::C => celsius,
            TemperatureRange::F => (celsius * 9.0 / 5.0) + 32.0,
            TemperatureRange::K => celsius + 273.15,
        }
    }
}

fn accel_scale(range: AccelRange) -> f32 {
    match range {
        AccelRange::G2 => ACCEL_SENSITIVITY.0,
        AccelRange::G4 => ACCEL_SENSITIVITY.1,
        AccelRange::G8 => ACCEL_SENSITIVITY.2,
        AccelRange::G16 => ACCEL_SENSITIVITY.3,
    }
}

fn gyro_scale(range: GyroRange) -> f32 {
    match range {
        GyroRange::Degree250 => GYRO_SENSITIVITY.0,
        GyroRange::Degree500 => GYRO_SENSITIVITY.1,
        GyroRange::Degree1000 => GYRO_SENSITIVITY.2,
        GyroRange::Degree2000 => GYRO_SENSITIVITY.3,
    }
}


