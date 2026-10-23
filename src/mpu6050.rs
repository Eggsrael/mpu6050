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
        if iam == WHO_AM_I_VALUE {
            Ok(())
        } else {
            Err(MpuError::InvalidChipId(iam))
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

        let mut total = (0i32, 0i32, 0i32);

        for _ in 0..cycles {
            let (x, y, z) = self.get_raw_gyro_all()?;
            total.0 += x as i32;
            total.1 += y as i32;
            total.2 += z as i32;
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

// ---------------------------------- TESTS -------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal::i2c::{Error as I2cError, ErrorKind, ErrorType, Operation};
    use std::vec::Vec;

    #[derive(Debug, PartialEq, Eq)]
    struct MockError;

    impl I2cError for MockError {
        fn kind(&self) -> ErrorKind {
            ErrorKind::Other
        }
    }

    struct MockI2c {
        registers: [u8; 256],
        writes: Vec<(u8, u8)>,
    }

    impl MockI2c {
        fn new() -> Self {
            let mut registers = [0u8; 256];
            registers[RA_WHO_AM_I as usize] = WHO_AM_I_VALUE;
            registers[RA_PWR_MGMT_1 as usize] = PWR_MGMT_1_SLEEP;

            Self {
                registers,
                writes: Vec::new(),
            }
        }
    }

    impl ErrorType for MockI2c {
        type Error = MockError;
    }

    impl I2c<SevenBitAddress> for MockI2c {
        fn transaction(
            &mut self,
            _address: SevenBitAddress,
            _operations: &mut [Operation<'_>],
        ) -> Result<(), Self::Error> {
            Err(MockError)
        }

        fn write(&mut self, _address: SevenBitAddress, bytes: &[u8]) -> Result<(), Self::Error> {
            let [reg, data] = bytes else {
                return Err(MockError);
            };

            self.registers[*reg as usize] = *data;
            self.writes.push((*reg, *data));
            Ok(())
        }

        fn write_read(
            &mut self,
            _address: SevenBitAddress,
            bytes: &[u8],
            buffer: &mut [u8],
        ) -> Result<(), Self::Error> {
            let [reg] = bytes else {
                return Err(MockError);
            };

            let start = *reg as usize;
            let end = start + buffer.len();
            if end > self.registers.len() {
                return Err(MockError);
            }

            buffer.copy_from_slice(&self.registers[start..end]);
            Ok(())
        }
    }

    struct MockDelay;

    impl DelayNs for MockDelay {
        fn delay_ns(&mut self, _ns: u32) {}
    }

    fn write_i16(registers: &mut [u8; 256], reg: u8, value: i16) {
        let bytes = value.to_be_bytes();
        registers[reg as usize] = bytes[0];
        registers[reg as usize + 1] = bytes[1];
    }

    fn write_sample_data(sensor: &mut Mpu6050<MockI2c>) {
        write_i16(&mut sensor.i2c.registers, RA_ACCEL_XOUT_H, 16_384);
        write_i16(&mut sensor.i2c.registers, RA_ACCEL_YOUT_H, -16_384);
        write_i16(&mut sensor.i2c.registers, RA_ACCEL_ZOUT_H, 8_192);
        write_i16(&mut sensor.i2c.registers, RA_TEMP_OUT_H, 340);
        write_i16(&mut sensor.i2c.registers, RA_GYRO_XOUT_H, 131);
        write_i16(&mut sensor.i2c.registers, RA_GYRO_YOUT_H, -655);
        write_i16(&mut sensor.i2c.registers, RA_GYRO_ZOUT_H, 0);
    }

    #[test]
    fn verify_device() {
        let mut sensor = Mpu6050::new(MockI2c::new());
        sensor.verify_device().unwrap();

        sensor.set_address(ADDRESS_AD0_HIGH).unwrap();
        assert_eq!(sensor.address(), ADDRESS_AD0_HIGH);
        sensor.verify_device().unwrap();
    }

    #[test]
    fn rejects_invalid_i2c_address() {
        let mut sensor = Mpu6050::new(MockI2c::new());
        assert!(matches!(
            sensor.set_address(0x70),
            Err(MpuError::InvalidAddress(0x70))
        ));
    }

    #[test]
    fn init_configures_expected_registers() {
        let mut sensor = Mpu6050::new(MockI2c::new());
        let mut delay = MockDelay;

        sensor.init(&mut delay).unwrap();

        assert_eq!(
            sensor.i2c.registers[RA_PWR_MGMT_1 as usize],
            PWR_MGMT_1_CLKSEL_PLL_X_GYRO
        );
        assert_eq!(sensor.i2c.registers[RA_SMPLRT_DIV as usize], 0);
        assert_eq!(sensor.i2c.registers[RA_CONFIG as usize], 0);
        assert_eq!(sensor.i2c.registers[RA_ACCEL_CONFIG as usize], 0);
        assert_eq!(sensor.i2c.registers[RA_GYRO_CONFIG as usize], 0);
        assert!(
            sensor
                .i2c
                .writes
                .contains(&(RA_PWR_MGMT_1, PWR_MGMT_1_CLKSEL_PLL_X_GYRO))
        );
    }

    #[test]
    fn setters_preserve_unrelated_bits() {
        let mut sensor = Mpu6050::new(MockI2c::new());

        sensor.i2c.registers[RA_CONFIG as usize] = 0b0010_1000;
        sensor.set_dlpf(DlpfConfig::Dlpf42).unwrap();
        assert_eq!(sensor.i2c.registers[RA_CONFIG as usize], 0b0010_1011);

        sensor.i2c.registers[RA_ACCEL_CONFIG as usize] = 0b1010_0000;
        sensor.set_accel_range(AccelRange::G8).unwrap();
        assert_eq!(sensor.i2c.registers[RA_ACCEL_CONFIG as usize], 0b1011_0000);

        sensor.i2c.registers[RA_GYRO_CONFIG as usize] = 0b1100_0000;
        sensor.set_gyro_range(GyroRange::Degree2000).unwrap();
        assert_eq!(sensor.i2c.registers[RA_GYRO_CONFIG as usize], 0b1101_1000);
    }

    #[test]
    fn reads_raw_and_scaled_data() {
        let mut sensor = Mpu6050::new(MockI2c::new());
        write_sample_data(&mut sensor);

        let raw = sensor.get_raw_data().unwrap();
        assert_eq!(raw.accel, [16_384, -16_384, 8_192]);
        assert_eq!(raw.temperature, 340);
        assert_eq!(raw.gyro, [131, -655, 0]);

        let data = sensor.get_data().unwrap();
        assert_eq!(data.accel, [1.0, -1.0, 0.5]);
        assert!((data.temperature - 37.53).abs() < 0.001);
        assert!((data.gyro[0] - 1.0).abs() < f32::EPSILON);
        assert!((data.gyro[1] + 5.0).abs() < f32::EPSILON);
        assert!((data.gyro[2] - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn get_data_applies_calibration_bias() {
        let mut sensor = Mpu6050::new(MockI2c::new());
        write_sample_data(&mut sensor);
        sensor.set_accel_bias((1.0, -1.0, 0.5));
        sensor.set_gyro_bias((1.0, -5.0, 0.0));

        let data = sensor.get_data().unwrap();
        assert_eq!(data.accel, [0.0, 0.0, 0.0]);
        assert_eq!(data.gyro, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn temperature_units_convert() {
        let mut sensor = Mpu6050::new(MockI2c::new());
        write_i16(&mut sensor.i2c.registers, RA_TEMP_OUT_H, 0);

        assert!((sensor.get_temperature().unwrap() - 36.53).abs() < 0.001);

        sensor.set_temperature_unit(TemperatureRange::F);
        assert!((sensor.get_temperature().unwrap() - 97.754).abs() < 0.001);

        sensor.set_temperature_unit(TemperatureRange::K);
        assert!((sensor.get_temperature().unwrap() - 309.68).abs() < 0.001);
    }

    #[test]
    fn calibration_rejects_zero_cycles() {
        let config = Mpu6050Config::default().with_calibration_cycles(0);
        let mut sensor = Mpu6050::new_with_config(MockI2c::new(), config);
        let mut delay = MockDelay;

        assert!(matches!(
            sensor.calibrate_gyro(&mut delay),
            Err(MpuError::InvalidCalibrationCycles)
        ));
    }

    #[test]
    fn calibration_uses_raw_gyro() {
        let config = Mpu6050Config::default().with_calibration_cycles(2);
        let mut sensor = Mpu6050::new_with_config(MockI2c::new(), config);
        let mut delay = MockDelay;

        write_i16(&mut sensor.i2c.registers, RA_GYRO_XOUT_H, 131);
        write_i16(&mut sensor.i2c.registers, RA_GYRO_YOUT_H, 131);
        write_i16(&mut sensor.i2c.registers, RA_GYRO_ZOUT_H, 131);

        sensor.calibrate_gyro(&mut delay).unwrap();
        assert_eq!(sensor.gyro_bias(), (1.0, 1.0, 1.0));

        let gyro = sensor.get_gyro().unwrap();
        assert_eq!(gyro, (0.0, 0.0, 0.0));
    }

    #[test]
    fn raw_register_access() {
        let mut sensor = Mpu6050::new(MockI2c::new());

        sensor.write_register(0x1, 0x11).unwrap();
        assert_eq!(sensor.read_register(0x1).unwrap(), 0x11);

        sensor.update_register(0x1, 0x0F, 0x05).unwrap();
        assert_eq!(sensor.read_register(0x1).unwrap(), 0x15);
    }

    #[test]
    fn reads_fifo_count() {
        let mut sensor = Mpu6050::new(MockI2c::new());

        // 0b_0000_0001_0000_0010 = 258
        sensor.i2c.registers[RA_FIFO_COUNTH as usize] = 0x01;
        sensor.i2c.registers[RA_FIFO_COUNTL as usize] = 0x02;

        assert_eq!(sensor.get_fifo_count().unwrap(), 258);
    }
}
