#![allow(unused)]
// Registers
// Hidden
// [ 7 {AUX_VDDIO, BLANK} | 6:1 XG_OFFS_TC]
pub const RA_AUX_VDDIO: u8 = 0x00;
pub const RA_YH_OFFS_TC: u8 = 0x01;
pub const RA_ZH_OFFS_TC: u8 = 0x02;

//[7:0]
pub const RA_X_FINE_GAIN: u8 = 0x03;
pub const RA_Y_FINE_GAIN: u8 = 0x04;
pub const RA_Z_FINE_GAIN: u8 = 0x05;

// [15:0]
pub const RA_XA_OFFS_H: u8 = 0x06;
pub const RA_XA_OFFS_L_TC: u8 = 0x07;
pub const RA_YA_OFFS_H: u8 = 0x08;
pub const RA_YA_OFFS_L_TC: u8 = 0x09;
pub const RA_ZA_OFFS_H: u8 = 0x0a;
pub const RA_ZA_OFFS_L_TC: u8 = 0x0b;

// Test registers
// [7-5 {X/Y/Z}_TEST[4-2] | G_TEST[4-0] ]
pub const RA_SELF_TEST_X: u8 = 0x0D;
pub const RA_SELF_TEST_Y: u8 = 0x0E;
pub const RA_SELF_TEST_Z: u8 = 0x0F;
// []
pub const RA_SELF_TEST_A: u8 = 0x10;

pub const RA_SMPLRT_DIV: u8 = 0x19;
pub const RA_CONFIG: u8 = 0x1A;
pub const RA_GYRO_CONFIG: u8 = 0x1B;
pub const RA_ACCEL_CONFIG: u8 = 0x1C;
pub const RA_MOT_THR: u8 = 0x1F;
pub const RA_FIFO_EN: u8 = 0x23;
pub const RA_I2C_MST_CTRL: u8 = 0x24;

pub const RA_I2C_SLV0_ADDER: u8 = 0x25;
pub const RA_I2C_SLV0_REG: u8 = 0x26;
pub const RA_I2C_SLV0_CTRL: u8 = 0x27;

pub const RA_I2C_SLV1_ADDER: u8 = 0x28;
pub const RA_I2C_SLV1_REG: u8 = 0x29;
pub const RA_I2C_SLV1_CTRL: u8 = 0x2A;

pub const RA_I2C_SLV2_ADDER: u8 = 0x2B;
pub const RA_I2C_SLV2_REG: u8 = 0x2C;
pub const RA_I2C_SLV2_CTRL: u8 = 0x2D;

pub const RA_I2C_SLV3_ADDER: u8 = 0x2E;
pub const RA_I2C_SLV3_REG: u8 = 0x2F;
pub const RA_I2C_SLV3_CTRL: u8 = 0x30;

pub const RA_I2C_SLV4_ADDER: u8 = 0x31;
pub const RA_I2C_SLV4_REG: u8 = 0x32;
pub const RA_I2C_SLV4_DO: u8 = 0x33;
pub const RA_I2C_SLV4_CTRL: u8 = 0x34;
pub const RA_I2C_SLV4_DI: u8 = 0x35;

pub const RA_I2C_MST_STATUS: u8 = 0x36;
pub const RA_INT_PIN_CFG: u8 = 0x37;
pub const RA_INT_ENABLE: u8 = 0x38;
pub const RA_INIT_STATUS: u8 = 0x3A;

pub const RA_ACCEL_XOUT_H: u8 = 0x3B;
pub const RA_ACCEL_XOUT_L: u8 = 0x3C;
pub const RA_ACCEL_YOUT_H: u8 = 0x3D;
pub const RA_ACCEL_YOUT_L: u8 = 0x3E;
pub const RA_ACCEL_ZOUT_H: u8 = 0x3F;
pub const RA_ACCEL_ZOUT_L: u8 = 0x40;

pub const RA_TEMP_OUT_L: u8 = 0x41;
pub const RA_TEMP_OUT_H: u8 = 0x42;

pub const RA_GYRO_XOUT_H: u8 = 0x43;
pub const RA_GYRO_XOUT_L: u8 = 0x44;
pub const RA_GYRO_YOUT_H: u8 = 0x45;
pub const RA_GYRO_YOUT_L: u8 = 0x46;
pub const RA_GYRO_ZOUT_H: u8 = 0x47;
pub const RA_GYRO_ZOUT_L: u8 = 0x48;

// external sensors storage [7:0]
pub const RA_EXT_SENS_DATA_00: u8 = 0x49;
pub const RA_EXT_SENS_DATA_01: u8 = 0x4A;
pub const RA_EXT_SENS_DATA_02: u8 = 0x4B;
pub const RA_EXT_SENS_DATA_03: u8 = 0x4C;
pub const RA_EXT_SENS_DATA_04: u8 = 0x4D;
pub const RA_EXT_SENS_DATA_05: u8 = 0x4E;
pub const RA_EXT_SENS_DATA_06: u8 = 0x4F;
pub const RA_EXT_SENS_DATA_07: u8 = 0x50;
pub const RA_EXT_SENS_DATA_08: u8 = 0x51;
pub const RA_EXT_SENS_DATA_09: u8 = 0x52;
pub const RA_EXT_SENS_DATA_10: u8 = 0x53;
pub const RA_EXT_SENS_DATA_11: u8 = 0x54;
pub const RA_EXT_SENS_DATA_12: u8 = 0x55;
pub const RA_EXT_SENS_DATA_13: u8 = 0x56;
pub const RA_EXT_SENS_DATA_14: u8 = 0x57;
pub const RA_EXT_SENS_DATA_15: u8 = 0x58;
pub const RA_EXT_SENS_DATA_16: u8 = 0x59;
pub const RA_EXT_SENS_DATA_17: u8 = 0x5A;
pub const RA_EXT_SENS_DATA_18: u8 = 0x5B;
pub const RA_EXT_SENS_DATA_19: u8 = 0x5C;
pub const RA_EXT_SENS_DATA_20: u8 = 0x5D;
pub const RA_EXT_SENS_DATA_21: u8 = 0x5E;
pub const RA_EXT_SENS_DATA_22: u8 = 0x5F;
pub const RA_EXT_SENS_DATA_23: u8 = 0x60;



pub const RA_I2C_SLV0_DO: u8 = 0x63;
pub const RA_I2C_SLV1_DO: u8 = 0x64;
pub const RA_I2C_SLV2_DO: u8 = 0x65;
pub const RA_I2C_SLV3_DO: u8 = 0x66;
pub const RA_SIGNAL_PATH_RESET: u8 = 0x67;
pub const RA_MOT_DETECT_CTRL: u8 = 0x68;
pub const RA_USER_CTRL: u8 = 0x69;
pub const RA_PWR_MGMT_1: u8 = 0x6A;
pub const RA_PWR_MGMT_2: u8 = 0x6B;
pub const RA_FIFO_COUNTH: u8 = 0x72;
pub const RA_FIFO_COUNTL: u8 = 0x73;
pub const RA_FIFO_R_W: u8 = 0x74;
pub const RA_WHO_AM_I: u8 = 0x75;
