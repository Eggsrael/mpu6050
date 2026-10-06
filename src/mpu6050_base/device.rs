use core::f32;


// Board
pub const EXTERNAL_CLOCK_FREQ: f32 = 32.768;

// accelerometer
pub const ACCEL_SENSITIVTY: (f32, f32, f32, f32) = (16_384.0, 8_192.0, 4_096.0, 2_048.0);

//gyro
pub const GYRO_SENSITIVITY: (f32, f32, f32, f32) = (131.0, 65.5, 32.8, 16.4);
pub const REFERNCE_GRAVITY: f32 = 9.80665;

// temp
pub const TEMP_SENSITIVITY: f32 = 340.0;
pub const TEMP_OFFSET: f32 = 36.53; // c 35

//dlfp
pub const EXT_SYNC_SET: u8 = 3;
pub const DLPF_CFG_BIT_SIZE: u8 = 3;


// temp = lsb - offset /sensitivity +
// temp = (lsb + 521 / 340) + 35


// Digital Low Pass Filter Config
pub enum DlpfConfig {
    Dlpf256 = 0,
    Dlpf188 = 1,
    Dlpf98 = 2,
    Dlpf42 = 3,
    Dlpf21 = 4,
    Dlpf10 = 5,
    Dlpf5 = 6,
}

// Gyroscope Range
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum GyroRange {
    Degree250 = 0,
    Degree500,
    Degree1000,
    Degree2000,
}

impl TryFrom<u8> for GyroRange {
    type Error = ();

    fn try_from(range: u8) -> Result<Self, Self::Error> {
        match range {
            0 => Ok(GyroRange::Degree250),
            1 => Ok(GyroRange::Degree500),
            2 => Ok(GyroRange::Degree1000),
            3 => Ok(GyroRange::Degree2000),
            _ => Err(()),
        }
    }
}

// Accelerometer Range
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AccelRange {
    G2 = 0,
    G4,
    G8,
    G16,
}

impl TryFrom<u8> for AccelRange {
    type Error = ();

    fn try_from(range: u8) -> Result<Self, Self::Error> {
        match range {
            0 => Ok(AccelRange::G2),
            1 => Ok(AccelRange::G4),
            2 => Ok(AccelRange::G8),
            3 => Ok(AccelRange::G16),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TemperatureRange {
    C = 0,
    F,
    K,
}

impl TryFrom<u8> for TemperatureRange {
    type Error = ();

    fn try_from(temp_setting: u8) -> Result<Self, Self::Error> {
        match temp_setting {
            0 => Ok(TemperatureRange::C),
            1 => Ok(TemperatureRange::F),
            2 => Ok(TemperatureRange::K),
            _ => Err(()),
        }
    }
}

impl DlpfConfig {
    pub fn from_raw(raw: u8) -> Option<Self> {
        match raw {
            0 => Some(DlpfConfig::Dlpf256),
            1 => Some(DlpfConfig::Dlpf188),
            2 => Some(DlpfConfig::Dlpf98),
            3 => Some(DlpfConfig::Dlpf42),
            4 => Some(DlpfConfig::Dlpf21),
            5 => Some(DlpfConfig::Dlpf10),
            6 => Some(DlpfConfig::Dlpf5),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SlaveAddress{
    slave_address0: u8,
    slave_address1: u8,
    slave_address2: u8,
    slave_address3: u8,
}

impl SlaveAddress {
    pub fn default() -> Self {
        Self::new(0x0, 0x0, 0x0, 0x0)
    }

    pub fn new(slave_address0: u8, slave_address1: u8, slave_address2: u8, slave_address3: u8) -> Self {
        Self {
            slave_address0,
            slave_address1,
            slave_address2,
            slave_address3,
        }
    }
}
