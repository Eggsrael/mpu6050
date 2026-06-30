use core::f32;

// Board
pub const ADDRESS_AD0_LOW: u8 = 0x68;
pub const ADDRESS_AD0_HIGH: u8 = 0x69;
pub const DEFAULT_ADDRRESS: u8 = ADDRESS_AD0_LOW;

pub const EXTERNAL_CLOCK_FREQ: f32 = 32.768;

// accelerometer
pub const ACCEL_SENSITIVTY: (f32, f32, f32, f32) = (16_384.0, 8_192.0, 4_096.0, 2_048.0);

//gyro
pub const GYRO_SENSITIVITY: (f32, f32, f32, f32) = (131.0, 65.5, 32.8, 16.4);
pub const REFERNCE_GRAVITY: f32 = 9.80665;

// temp
pub const TEMP_SENSITIVITY: f32 = 340.0;
pub const TEMP_OFFSET: f32 = -521.0; // c 35

// temp = lsb - offset /sensitivity +
// temp = (lsb + 521 / 340) + 35

// Gyroscope Range
#[derive(Debug)]
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
#[derive(Debug)]
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

pub enum TempatureRange {
    C = 0,
    F,
    K,
}

impl From<u8> for TempatureRange {
    fn from(temp_setting: u8) -> Self {
        match temp_setting {
            0 => TempatureRange::C,
            1 => TempatureRange::F,
            2 => TempatureRange::K,
            _ => TempatureRange::C,
        }
    }
}
