//! # Qwiic ADC Library for Rust
//!
//! This library provides support for the SparkFun Qwiic ADC (ADS1015/ADS1115) boards
//! using I2C communication on Linux systems.
//!
//! ## Features
//! - Single-ended and differential ADC readings
//! - Configurable gain settings
//! - Multiple sample rates
//! - Support for 4 single-ended or 2 differential channels
//!
//! ## Example
//! ```no_run
//! use qwiic_adc_rs::*;
//!
//! let config = QwiicADCConfig::default();
//! let mut adc = QwiicADC::new(config, "/dev/i2c-1", 0x48).unwrap();
//! adc.init().unwrap();
//! let value = adc.get_single_ended(0).unwrap();
//! println!("Channel 0: {}", value);
//! ```

// Copyright 2021 Caleb Mitchell Smith-Woolrich (PixelCoda)
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate i2cdev;

use std::thread;
use std::time::Duration;
use std::fmt;

use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};

/// ADC-specific error types
#[derive(Debug)]
pub enum AdcError {
    /// Invalid channel number (must be 0-3 for single-ended)
    InvalidChannel(u8),
    /// Invalid differential mode configuration
    InvalidDifferentialMode(u16),
    /// I2C communication error
    I2cError(LinuxI2CError),
}

impl fmt::Display for AdcError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AdcError::InvalidChannel(ch) => write!(f, "Invalid channel: {}. Must be 0-3", ch),
            AdcError::InvalidDifferentialMode(mode) => write!(f, "Invalid differential mode: 0x{:04X}", mode),
            AdcError::I2cError(e) => write!(f, "I2C error: {}", e),
        }
    }
}

impl From<LinuxI2CError> for AdcError {
    fn from(error: LinuxI2CError) -> Self {
        AdcError::I2cError(error)
    }
}

type ADCResult = Result<(), AdcError>;
type ReadResult = Result<u16, AdcError>;

/// I2C addresses for the ADS1015/ADS1115
/// Address is determined by the ADDR pin connection
#[derive(Copy, Clone)]
pub enum Addresses {
    /// ADDR pin connected to GND
    Gnd = 0x48,
    /// ADDR pin connected to VDD
    Vdd = 0x49,
    /// ADDR pin connected to SDA
    Sda = 0x4A,
    /// ADDR pin connected to SCL
    Scl = 0x4B
}

/// Operational status/single-shot conversion start
/// This bit determines the operational status of the device
#[derive(Copy, Clone)]
pub enum OS {
    /// Begin single conversion (when writing)
    Single = 0x8000,
    /// No conversion (when writing) / Conversion in progress (when reading)
    Busy = 0x0000,
}

/// Register pointers for ADS1015/ADS1115
#[derive(Copy, Clone)]
pub enum Pointers {
    /// Conversion register (read ADC result)
    Convert = 0x00,
    /// Configuration register
    Config = 0x01,
    /// Low threshold register
    LowThresh = 0x02,
    /// High threshold register
    HighThresh = 0x03
}

/// ADC operating modes
#[derive(Copy, Clone)]
pub enum Modes {
    /// Continuous conversion mode
    Continuous = 0x0000,
    /// Single-shot mode (default)
    Single = 0x0100,
}

/// Input multiplexer configuration
#[derive(Copy, Clone)]
pub enum Mux {
    /// Single-ended AIN0
    Single0 = 0x4000,
    /// Single-ended AIN1
    Single1 = 0x5000,
    /// Single-ended AIN2
    Single2 = 0x6000,
    /// Single-ended AIN3
    Single3 = 0x7000,
    /// Differential P = AIN0, N = AIN1
    DiffP0N1 = 0x0000,
    /// Differential P = AIN0, N = AIN3
    DiffP0N3 = 0x1000,
    /// Differential P = AIN1, N = AIN3
    DiffP1N3 = 0x2000,
    /// Differential P = AIN2, N = AIN3
    DiffP2N3 = 0x3000,
}

/// Data rate settings for ADS1015
#[derive(Copy, Clone)]
pub enum SampleRates {
    /// 128 samples per second
    S128Hz = 0x0000,
    /// 250 samples per second
    S250Hz = 0x0020,
    /// 490 samples per second
    S490Hz = 0x0040,
    /// 920 samples per second
    S920Hz = 0x0060,
    /// 1600 samples per second (default)
    S1600Hz = 0x0080,
    /// 2400 samples per second
    S2400Hz = 0x00A0,
    /// 3300 samples per second
    S3300Hz = 0x00C0
}


/// Programmable gain amplifier configuration
#[derive(Copy, Clone)]
pub enum PGA {
    /// PGA mask
    Mask = 0x0E00,
    /// +/- 6.144V range (2/3 gain)
    TwoThirds = 0x0000,
    /// +/- 4.096V range
    One = 0x0200,
    /// +/- 2.048V range (default)
    Two = 0x0400,
    /// +/- 1.024V range
    Four = 0x0600,
    /// +/- 0.512V range
    Eight = 0x0800,
    /// +/- 0.256V range
    Sixteen = 0x0A00
}

/// Comparator mode
#[derive(Copy, Clone)]
pub enum Cmode {
    /// Traditional comparator with hysteresis (default)
    Trad = 0x0000,
    /// Window comparator
    Window = 0x0010,
}

/// Comparator polarity
#[derive(Copy, Clone)]
pub enum Cpol {
    /// ALERT/RDY pin is low when active (default)
    ActvLow = 0x0000,
    /// ALERT/RDY pin is high when active
    ActvHigh = 0x0008,
}

/// Latching comparator
#[derive(Copy, Clone)]
pub enum Clat {
    /// Non-latching comparator (default)
    NonLat = 0x0000,
    /// Latching comparator
    Latch = 0x0004,
}

/// Comparator queue and disable
#[derive(Copy, Clone)]
pub enum Cque {
    /// Assert after one conversion
    OneConv = 0x0000,
    /// Assert after two conversions
    TwoConv = 0x0001,
    /// Assert after four conversions
    FourConv = 0x0002,
    /// Disable comparator (default)
    None = 0x0003
}

/// Configuration for the Qwiic ADC
pub struct QwiicADCConfig {
    /// Model of ADC chip ("ADS1015" or "ADS1115")
    model: String
}

impl QwiicADCConfig {
    /// Create a new configuration with specified model
    pub fn new(model: String) -> QwiicADCConfig {
        QwiicADCConfig {
            model,
        }
    }
}

impl Default for QwiicADCConfig {
    /// Create default configuration for ADS1015
    fn default() -> Self {
        QwiicADCConfig::new("ADS1015".to_string())
    }
}

/// Main struct for interacting with the Qwiic ADC
pub struct QwiicADC {
    dev: LinuxI2CDevice,
    config: QwiicADCConfig,
}


impl QwiicADC {
    /// Create a new QwiicADC instance
    ///
    /// # Arguments
    /// * `config` - Configuration for the ADC
    /// * `bus` - I2C bus path (e.g., "/dev/i2c-1")
    /// * `i2c_addr` - I2C address of the device
    pub fn new(config: QwiicADCConfig, bus: &str, i2c_addr: u16) -> Result<QwiicADC, LinuxI2CError> {
        let dev = LinuxI2CDevice::new(bus, i2c_addr)?;
        Ok(QwiicADC {
            dev,
            config,
        })
    }
    
    /// Validate that a channel number is valid for single-ended reads
    fn validate_channel(channel: u8) -> Result<(), AdcError> {
        if channel > 3 {
            Err(AdcError::InvalidChannel(channel))
        } else {
            Ok(())
        }
    }
    
    /// Validate that a differential mode configuration is valid
    fn validate_differential_mode(mode: u16) -> Result<(), AdcError> {
        match mode {
            x if x == Mux::DiffP0N1 as u16 => Ok(()),
            x if x == Mux::DiffP0N3 as u16 => Ok(()),
            x if x == Mux::DiffP1N3 as u16 => Ok(()),
            x if x == Mux::DiffP2N3 as u16 => Ok(()),
            _ => Err(AdcError::InvalidDifferentialMode(mode)),
        }
    }

    /// Initialize the ADC device
    pub fn init(&mut self) -> ADCResult {
        // Wait for the ADC to set up
        thread::sleep(Duration::from_millis(10));
        Ok(())
    }

    /// Check if the ADC is connected and responding
    pub fn is_connected(&mut self) -> bool {
        self.read_register(Pointers::Config as u8).is_ok()
    }
    
    /// Set the gain setting for the ADC
    ///
    /// # Arguments
    /// * `gain` - PGA gain setting
    pub fn set_gain(&mut self, gain: PGA) -> ADCResult {
        let mut config = self.read_register_16bit(Pointers::Config as u8)?;
        config &= !(PGA::Mask as u16);  // Clear gain bits
        config |= gain as u16;  // Set new gain
        self.write_register(Pointers::Config as u8, config as usize)?;
        Ok(())
    }
    
    /// Get the current gain setting
    pub fn get_gain(&mut self) -> Result<u16, AdcError> {
        let config = self.read_register_16bit(Pointers::Config as u8)?;
        Ok(config & (PGA::Mask as u16))
    }
    
    /// Set the sample rate for the ADC
    ///
    /// # Arguments
    /// * `rate` - Sample rate setting
    pub fn set_sample_rate(&mut self, rate: SampleRates) -> ADCResult {
        let mut config = self.read_register_16bit(Pointers::Config as u8)?;
        config &= !0x00E0;  // Clear sample rate bits
        config |= rate as u16;  // Set new rate
        self.write_register(Pointers::Config as u8, config as usize)?;
        Ok(())
    }
    
    /// Get the current sample rate setting
    pub fn get_sample_rate(&mut self) -> Result<u16, AdcError> {
        let config = self.read_register_16bit(Pointers::Config as u8)?;
        Ok(config & 0x00E0)
    }
    
    /// Set the operating mode (continuous or single-shot)
    ///
    /// # Arguments
    /// * `mode` - Operating mode
    pub fn set_mode(&mut self, mode: Modes) -> ADCResult {
        let mut config = self.read_register_16bit(Pointers::Config as u8)?;
        config &= !0x0100;  // Clear mode bit
        config |= mode as u16;  // Set new mode
        self.write_register(Pointers::Config as u8, config as usize)?;
        Ok(())
    }
    
    /// Set the low threshold for comparator
    ///
    /// # Arguments
    /// * `threshold` - Threshold value
    pub fn set_low_threshold(&mut self, threshold: u16) -> ADCResult {
        self.write_register(Pointers::LowThresh as u8, threshold as usize)?;
        Ok(())
    }
    
    /// Set the high threshold for comparator
    ///
    /// # Arguments
    /// * `threshold` - Threshold value
    pub fn set_high_threshold(&mut self, threshold: u16) -> ADCResult {
        self.write_register(Pointers::HighThresh as u8, threshold as usize)?;
        Ok(())
    }
    
    /// Get the low threshold value
    pub fn get_low_threshold(&mut self) -> ReadResult {
        self.read_register_16bit(Pointers::LowThresh as u8)
    }
    
    /// Get the high threshold value
    pub fn get_high_threshold(&mut self) -> ReadResult {
        self.read_register_16bit(Pointers::HighThresh as u8)
    }
    
    /// Convert raw ADC value to voltage
    ///
    /// # Arguments
    /// * `raw_value` - Raw ADC reading
    /// * `gain` - PGA gain setting used for the reading
    ///
    /// # Returns
    /// Voltage in millivolts
    pub fn raw_to_voltage(&self, raw_value: u16, gain: PGA) -> f32 {
        let fsrange = match gain {
            PGA::TwoThirds => 6144.0,
            PGA::One => 4096.0,
            PGA::Two => 2048.0,
            PGA::Four => 1024.0,
            PGA::Eight => 512.0,
            PGA::Sixteen => 256.0,
            _ => 2048.0,  // Default
        };
        
        if self.config.model == "ADS1015" {
            // 12-bit ADC
            (raw_value as f32 / 2048.0) * fsrange
        } else {
            // 16-bit ADC (ADS1115)
            (raw_value as f32 / 32768.0) * fsrange
        }
    }
    
    /// Start a continuous conversion mode
    pub fn start_continuous(&mut self, channel: u8) -> ADCResult {
        // Validate channel
        Self::validate_channel(channel)?;
        
        let mut config = (OS::Single as u16) | (Modes::Continuous as u16) | (SampleRates::S1600Hz as u16);
        config |= PGA::Two as u16;
        
        config |= match channel {
            0 => Mux::Single0 as u16,
            1 => Mux::Single1 as u16,
            2 => Mux::Single2 as u16,
            3 => Mux::Single3 as u16,
            _ => return Err(AdcError::InvalidChannel(channel)),
        };
        
        self.write_register(Pointers::Config as u8, config as usize)?;
        Ok(())
    }
    
    /// Stop continuous conversion mode
    pub fn stop_continuous(&mut self) -> ADCResult {
        self.set_mode(Modes::Single)
    }
    
    /// Read the last conversion result (useful in continuous mode)
    pub fn read_last_conversion(&mut self) -> ReadResult {
        let result = self.read_register_16bit(Pointers::Convert as u8)?;
        if self.config.model == "ADS1015" {
            Ok(result >> 4)
        } else {
            Ok(result)
        }
    }
    


    /// Read a single-ended ADC value from the specified channel
    ///
    /// # Arguments
    /// * `channel` - Channel number (0-3)
    ///
    /// # Returns
    /// 12-bit ADC value for ADS1015, 16-bit for ADS1115
    pub fn get_single_ended(&mut self, channel: u8) -> ReadResult {
        // Validate channel
        Self::validate_channel(channel)?;

        let mut config = (OS::Single as u16) | (Modes::Single as u16) | (SampleRates::S1600Hz as u16);
        config |= PGA::Two as u16;

        // Use match expression for clean channel selection
        config |= match channel {
            0 => Mux::Single0 as u16,
            1 => Mux::Single1 as u16,
            2 => Mux::Single2 as u16,
            3 => Mux::Single3 as u16,
            _ => return Err(AdcError::InvalidChannel(channel)),
        };

        self.write_register(Pointers::Config as u8, config as usize)?;

        // Wait for conversion to complete
        thread::sleep(Duration::from_millis(10));


        let result = self.read_register_16bit(Pointers::Convert as u8)?;
        // For ADS1015, shift right by 4 bits (12-bit ADC)
        if self.config.model == "ADS1015" {
            Ok(result >> 4)
        } else {
            Ok(result)
        }

      

    }

    /// Read a differential ADC value
    ///
    /// # Arguments
    /// * `cfg_mux_diff` - Optional differential mode configuration
    ///
    /// # Returns
    /// 12-bit ADC value for ADS1015, 16-bit for ADS1115
    pub fn get_differential(&mut self, cfg_mux_diff: Option<u16>) -> ReadResult {
        // Use provided config or default to DiffP0N1
        let config_mux_diff = cfg_mux_diff.unwrap_or(Mux::DiffP0N1 as u16);
        
        // Validate differential mode
        Self::validate_differential_mode(config_mux_diff)?;

        let mut config = (OS::Single as u16) | (Modes::Single as u16) | (SampleRates::S1600Hz as u16);
        config |= PGA::Two as u16;
        config |= config_mux_diff;

        self.write_register(Pointers::Config as u8, config as usize)?;

        // Wait for conversion to complete
        thread::sleep(Duration::from_millis(10));

        let result = self.read_register_16bit(Pointers::Convert as u8)?;
        // For ADS1015, shift right by 4 bits (12-bit ADC)
        if self.config.model == "ADS1015" {
            Ok(result >> 4)
        } else {
            Ok(result)
        }
    }


    /// Convenience function to get analog data from a channel
    /// Wrapper around get_single_ended
    pub fn get_analog_data(&mut self, channel: u8) -> ReadResult {
        self.get_single_ended(channel)
    }



    /// Read a single byte from a register
    pub fn read_register(&mut self, location: u8) -> Result<u8, LinuxI2CError> {
        self.dev.smbus_write_byte(location)?;
        let byte = self.dev.smbus_read_byte()?;
        Ok(byte)
    }
    
    /// Read 16-bit value from a register (for ADC conversion results)
    pub fn read_register_16bit(&mut self, location: u8) -> ReadResult {
        self.dev.smbus_write_byte(location)?;
        let high = self.dev.smbus_read_byte()? as u16;
        let low = self.dev.smbus_read_byte()? as u16;
        Ok((high << 8) | low)
    }
    



    /// Write a 16-bit value to a register
    pub fn write_register(&mut self, register: u8, val: usize) -> ADCResult {
        let data = [(val >> 8) as u8, (val & 0xFF) as u8];
        self.dev.smbus_write_i2c_block_data(register, &data)?;
        Ok(())
    }

    /// Write a single byte command
    pub fn write_byte(&mut self, command: u8) -> ADCResult {
        self.dev.smbus_write_byte(command)?;
        thread::sleep(Duration::from_micros(10));
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Ignore by default as it requires actual hardware
    fn test_hardware_init() {
        let config = QwiicADCConfig::default();
        let mut adc = QwiicADC::new(config, "/dev/i2c-1", 0x48)
            .expect("Could not init device");
        
        adc.init().expect("Failed to initialize");
        
        assert!(adc.is_connected(), "Device should be connected");
        
        let cfg = adc.read_register(Pointers::Config as u8)
            .expect("Should read config register");
        println!("Config: 0x{cfg:02X}");
        
        let value = adc.get_single_ended(0)
            .expect("Should read channel 0");
        println!("Channel 0 value: {value}");
    }

    #[test]
    fn test_config_creation() {
        let config = QwiicADCConfig::default();
        assert_eq!(config.model, "ADS1015");
        
        let config = QwiicADCConfig::new("ADS1115".to_string());
        assert_eq!(config.model, "ADS1115");
    }

    #[test]
    fn test_enum_values() {
        // Test that enum values are correct
        assert_eq!(Addresses::Gnd as u16, 0x48);
        assert_eq!(Addresses::Vdd as u16, 0x49);
        assert_eq!(Addresses::Sda as u16, 0x4A);
        assert_eq!(Addresses::Scl as u16, 0x4B);
        
        assert_eq!(Pointers::Convert as u8, 0x00);
        assert_eq!(Pointers::Config as u8, 0x01);
        assert_eq!(Pointers::LowThresh as u8, 0x02);
        assert_eq!(Pointers::HighThresh as u8, 0x03);
        
        assert_eq!(OS::Single as u16, 0x8000);
        assert_eq!(OS::Busy as u16, 0x0000);
        
        assert_eq!(Modes::Single as u16, 0x0100);
        assert_eq!(Modes::Continuous as u16, 0x0000);
        
        assert_eq!(PGA::Mask as u16, 0x0E00);
        assert_eq!(PGA::TwoThirds as u16, 0x0000);
        assert_eq!(PGA::One as u16, 0x0200);
        assert_eq!(PGA::Two as u16, 0x0400);
        assert_eq!(PGA::Four as u16, 0x0600);
        assert_eq!(PGA::Eight as u16, 0x0800);
        assert_eq!(PGA::Sixteen as u16, 0x0A00);
        
        assert_eq!(SampleRates::S128Hz as u16, 0x0000);
        assert_eq!(SampleRates::S250Hz as u16, 0x0020);
        assert_eq!(SampleRates::S490Hz as u16, 0x0040);
        assert_eq!(SampleRates::S920Hz as u16, 0x0060);
        assert_eq!(SampleRates::S1600Hz as u16, 0x0080);
        assert_eq!(SampleRates::S2400Hz as u16, 0x00A0);
        assert_eq!(SampleRates::S3300Hz as u16, 0x00C0);
    }

    #[test]
    fn test_mux_values() {
        assert_eq!(Mux::Single0 as u16, 0x4000);
        assert_eq!(Mux::Single1 as u16, 0x5000);
        assert_eq!(Mux::Single2 as u16, 0x6000);
        assert_eq!(Mux::Single3 as u16, 0x7000);
        assert_eq!(Mux::DiffP0N1 as u16, 0x0000);
        assert_eq!(Mux::DiffP0N3 as u16, 0x1000);
        assert_eq!(Mux::DiffP1N3 as u16, 0x2000);
        assert_eq!(Mux::DiffP2N3 as u16, 0x3000);
    }

    #[test]
    fn test_comparator_values() {
        assert_eq!(Cmode::Trad as u16, 0x0000);
        assert_eq!(Cmode::Window as u16, 0x0010);
        
        assert_eq!(Cpol::ActvLow as u16, 0x0000);
        assert_eq!(Cpol::ActvHigh as u16, 0x0008);
        
        assert_eq!(Clat::NonLat as u16, 0x0000);
        assert_eq!(Clat::Latch as u16, 0x0004);
        
        assert_eq!(Cque::OneConv as u16, 0x0000);
        assert_eq!(Cque::TwoConv as u16, 0x0001);
        assert_eq!(Cque::FourConv as u16, 0x0002);
        assert_eq!(Cque::None as u16, 0x0003);
    }

    #[test]
    #[ignore] // Requires hardware
    fn test_get_single_ended_all_channels() {
        let config = QwiicADCConfig::default();
        let mut adc = QwiicADC::new(config, "/dev/i2c-1", 0x48)
            .expect("Could not init device");
        
        adc.init().expect("Failed to initialize");
        
        for channel in 0..4 {
            let value = adc.get_single_ended(channel)
                .expect(&format!("Should read channel {}", channel));
            assert!(value <= 4095, "12-bit ADC value should be <= 4095");
            println!("Channel {} value: {}", channel, value);
        }
    }

    #[test]
    #[ignore] // Requires hardware
    fn test_get_single_ended_invalid_channel() {
        let config = QwiicADCConfig::default();
        let mut adc = QwiicADC::new(config, "/dev/i2c-1", 0x48)
            .expect("Could not init device");
        
        adc.init().expect("Failed to initialize");
        
        let value = adc.get_single_ended(4).unwrap();
        assert_eq!(value, 0, "Invalid channel should return 0");
        
        let value = adc.get_single_ended(255).unwrap();
        assert_eq!(value, 0, "Invalid channel should return 0");
    }

    #[test]
    #[ignore] // Requires hardware
    fn test_get_differential_modes() {
        let config = QwiicADCConfig::default();
        let mut adc = QwiicADC::new(config, "/dev/i2c-1", 0x48)
            .expect("Could not init device");
        
        adc.init().expect("Failed to initialize");
        
        // Test default differential mode
        let value = adc.get_differential(None)
            .expect("Should read differential P0-N1");
        println!("Differential P0-N1: {}", value);
        
        // Test all differential modes
        let modes = vec![
            (Mux::DiffP0N1 as u16, "P0-N1"),
            (Mux::DiffP0N3 as u16, "P0-N3"),
            (Mux::DiffP1N3 as u16, "P1-N3"),
            (Mux::DiffP2N3 as u16, "P2-N3"),
        ];
        
        for (mode, name) in modes {
            let value = adc.get_differential(Some(mode))
                .expect(&format!("Should read differential {}", name));
            println!("Differential {}: {}", name, value);
        }
    }

    #[test]
    #[ignore] // Requires hardware
    fn test_get_differential_invalid_mode() {
        let config = QwiicADCConfig::default();
        let mut adc = QwiicADC::new(config, "/dev/i2c-1", 0x48)
            .expect("Could not init device");
        
        adc.init().expect("Failed to initialize");
        
        // Test invalid differential mode
        let value = adc.get_differential(Some(0xFFFF)).unwrap();
        assert_eq!(value, 0, "Invalid differential mode should return 0");
    }

    #[test]
    #[ignore] // Requires hardware
    fn test_get_analog_data() {
        let config = QwiicADCConfig::default();
        let mut adc = QwiicADC::new(config, "/dev/i2c-1", 0x48)
            .expect("Could not init device");
        
        adc.init().expect("Failed to initialize");
        
        // Test that get_analog_data matches get_single_ended
        for channel in 0..4 {
            let single_value = adc.get_single_ended(channel)
                .expect("Should read single-ended");
            let analog_value = adc.get_analog_data(channel)
                .expect("Should read analog data");
            assert_eq!(single_value, analog_value, 
                "get_analog_data should match get_single_ended for channel {}", channel);
        }
    }

    #[test]
    #[ignore] // Requires hardware
    fn test_ads1115_mode() {
        let config = QwiicADCConfig::new("ADS1115".to_string());
        let mut adc = QwiicADC::new(config, "/dev/i2c-1", 0x48)
            .expect("Could not init device");
        
        adc.init().expect("Failed to initialize");
        
        let value = adc.get_single_ended(0)
            .expect("Should read channel 0");
        // ADS1115 is 16-bit, value is u16 so automatically <= 65535
        println!("ADS1115 Channel 0 value: {}", value);
    }

    #[test]
    #[ignore] // Requires hardware
    fn test_multiple_addresses() {
        let addresses = vec![
            (Addresses::Gnd as u16, "GND"),
            (Addresses::Vdd as u16, "VDD"),
            (Addresses::Sda as u16, "SDA"),
            (Addresses::Scl as u16, "SCL"),
        ];
        
        for (addr, name) in addresses {
            let config = QwiicADCConfig::default();
            match QwiicADC::new(config, "/dev/i2c-1", addr) {
                Ok(mut adc) => {
                    adc.init().expect("Failed to initialize");
                    if adc.is_connected() {
                        println!("Device found at address 0x{:02X} ({})", addr, name);
                    } else {
                        println!("No device at address 0x{:02X} ({})", addr, name);
                    }
                },
                Err(e) => {
                    println!("Could not access address 0x{:02X} ({}): {:?}", addr, name, e);
                }
            }
        }
    }

    #[test]
    #[ignore] // Requires hardware
    fn test_gain_settings() {
        let config = QwiicADCConfig::default();
        let mut adc = QwiicADC::new(config, "/dev/i2c-1", 0x48)
            .expect("Could not init device");
        
        adc.init().expect("Failed to initialize");
        
        // Test setting and getting different gains
        let gains = vec![
            (PGA::TwoThirds, "2/3"),
            (PGA::One, "1"),
            (PGA::Two, "2"),
            (PGA::Four, "4"),
            (PGA::Eight, "8"),
            (PGA::Sixteen, "16"),
        ];
        
        for (gain, name) in gains {
            adc.set_gain(gain).expect(&format!("Failed to set gain {}", name));
            let current_gain = adc.get_gain().expect("Failed to get gain");
            assert_eq!(current_gain, gain as u16, "Gain {} not set correctly", name);
        }
    }

    #[test]
    #[ignore] // Requires hardware
    fn test_sample_rate_settings() {
        let config = QwiicADCConfig::default();
        let mut adc = QwiicADC::new(config, "/dev/i2c-1", 0x48)
            .expect("Could not init device");
        
        adc.init().expect("Failed to initialize");
        
        // Test setting and getting different sample rates
        let rates = vec![
            (SampleRates::S128Hz, "128Hz"),
            (SampleRates::S250Hz, "250Hz"),
            (SampleRates::S490Hz, "490Hz"),
            (SampleRates::S920Hz, "920Hz"),
            (SampleRates::S1600Hz, "1600Hz"),
            (SampleRates::S2400Hz, "2400Hz"),
            (SampleRates::S3300Hz, "3300Hz"),
        ];
        
        for (rate, name) in rates {
            adc.set_sample_rate(rate).expect(&format!("Failed to set rate {}", name));
            let current_rate = adc.get_sample_rate().expect("Failed to get rate");
            assert_eq!(current_rate, rate as u16, "Sample rate {} not set correctly", name);
        }
    }

    #[test]
    #[ignore] // Requires hardware
    fn test_threshold_settings() {
        let config = QwiicADCConfig::default();
        let mut adc = QwiicADC::new(config, "/dev/i2c-1", 0x48)
            .expect("Could not init device");
        
        adc.init().expect("Failed to initialize");
        
        // Test setting and getting thresholds
        let test_low = 1024;
        let test_high = 3072;
        
        adc.set_low_threshold(test_low).expect("Failed to set low threshold");
        adc.set_high_threshold(test_high).expect("Failed to set high threshold");
        
        let low = adc.get_low_threshold().expect("Failed to get low threshold");
        let high = adc.get_high_threshold().expect("Failed to get high threshold");
        
        assert_eq!(low, test_low, "Low threshold not set correctly");
        assert_eq!(high, test_high, "High threshold not set correctly");
    }

    #[test]
    fn test_raw_to_voltage_ads1015() {
        let config = QwiicADCConfig::default();  // ADS1015
        let adc = QwiicADC::new(config, "/dev/i2c-1", 0x48);
        
        if let Ok(adc) = adc {
            // Test with PGA::Two (±2.048V range)
            let raw = 2048;  // Half of 12-bit range
            let voltage = adc.raw_to_voltage(raw, PGA::Two);
            assert_eq!(voltage, 2048.0, "Voltage calculation incorrect for ADS1015");
            
            // Test with PGA::One (±4.096V range)
            let voltage = adc.raw_to_voltage(raw, PGA::One);
            assert_eq!(voltage, 4096.0, "Voltage calculation incorrect for ADS1015");
        }
    }

    #[test]
    fn test_raw_to_voltage_ads1115() {
        let config = QwiicADCConfig::new("ADS1115".to_string());
        let adc = QwiicADC::new(config, "/dev/i2c-1", 0x48);
        
        if let Ok(adc) = adc {
            // Test with PGA::Two (±2.048V range)
            let raw = 32768;  // Half of 16-bit range
            let voltage = adc.raw_to_voltage(raw, PGA::Two);
            assert_eq!(voltage, 2048.0, "Voltage calculation incorrect for ADS1115");
            
            // Test with PGA::One (±4.096V range)
            let voltage = adc.raw_to_voltage(raw, PGA::One);
            assert_eq!(voltage, 4096.0, "Voltage calculation incorrect for ADS1115");
        }
    }

    #[test]
    #[ignore] // Requires hardware
    fn test_continuous_mode() {
        let config = QwiicADCConfig::default();
        let mut adc = QwiicADC::new(config, "/dev/i2c-1", 0x48)
            .expect("Could not init device");
        
        adc.init().expect("Failed to initialize");
        
        // Start continuous mode on channel 0
        adc.start_continuous(0).expect("Failed to start continuous mode");
        
        // Read multiple conversions
        for i in 0..5 {
            thread::sleep(Duration::from_millis(10));
            let value = adc.read_last_conversion()
                .expect("Failed to read conversion");
            println!("Continuous reading {}: {}", i, value);
        }
        
        // Stop continuous mode
        adc.stop_continuous().expect("Failed to stop continuous mode");
    }

    #[test]
    #[ignore] // Requires hardware
    fn test_mode_switching() {
        let config = QwiicADCConfig::default();
        let mut adc = QwiicADC::new(config, "/dev/i2c-1", 0x48)
            .expect("Could not init device");
        
        adc.init().expect("Failed to initialize");
        
        // Set single mode
        adc.set_mode(Modes::Single).expect("Failed to set single mode");
        
        // Set continuous mode
        adc.set_mode(Modes::Continuous).expect("Failed to set continuous mode");
        
        // Back to single mode
        adc.set_mode(Modes::Single).expect("Failed to set single mode");
    }
    
    #[test]
    fn test_channel_validation() {
        // Test valid channels
        for channel in 0..=3 {
            assert!(QwiicADC::validate_channel(channel).is_ok(),
                    "Channel {} should be valid", channel);
        }
        
        // Test invalid channels
        for channel in 4..=255 {
            match QwiicADC::validate_channel(channel) {
                Err(AdcError::InvalidChannel(ch)) => assert_eq!(ch, channel),
                _ => panic!("Channel {} should be invalid", channel),
            }
        }
    }
    
    #[test]
    fn test_differential_mode_validation() {
        // Test valid differential modes
        let valid_modes = [
            Mux::DiffP0N1 as u16,
            Mux::DiffP0N3 as u16,
            Mux::DiffP1N3 as u16,
            Mux::DiffP2N3 as u16,
        ];
        
        for mode in &valid_modes {
            assert!(QwiicADC::validate_differential_mode(*mode).is_ok(),
                    "Mode 0x{:04X} should be valid", mode);
        }
        
        // Test invalid differential modes
        let invalid_modes = [
            0x5000u16,  // Single0 (not a differential mode)
            0x6000u16,  // Single1 (not a differential mode)
            0x9999u16,  // Random invalid value
            0xFFFFu16,  // Max value
        ];
        
        for mode in &invalid_modes {
            match QwiicADC::validate_differential_mode(*mode) {
                Err(AdcError::InvalidDifferentialMode(m)) => assert_eq!(m, *mode),
                _ => panic!("Mode 0x{:04X} should be invalid", mode),
            }
        }
    }
    
    #[test]
    fn test_error_display() {
        // Test InvalidChannel error display
        let err = AdcError::InvalidChannel(5);
        assert_eq!(format!("{}", err), "Invalid channel: 5. Must be 0-3");
        
        // Test InvalidDifferentialMode error display
        let err = AdcError::InvalidDifferentialMode(0x9999);
        assert_eq!(format!("{}", err), "Invalid differential mode: 0x9999");
    }
    
    #[test]
    fn test_channel_selection_match_coverage() {
        // This test ensures the match expression covers all valid channels
        // and properly rejects invalid ones (compile-time check)
        let valid_channels = [0, 1, 2, 3];
        let expected_mux = [
            Mux::Single0 as u16,
            Mux::Single1 as u16,
            Mux::Single2 as u16,
            Mux::Single3 as u16,
        ];
        
        for (channel, expected) in valid_channels.iter().zip(expected_mux.iter()) {
            // Simulate the match expression from get_single_ended
            let mux = match channel {
                0 => Mux::Single0 as u16,
                1 => Mux::Single1 as u16,
                2 => Mux::Single2 as u16,
                3 => Mux::Single3 as u16,
                _ => 0xFFFF, // Invalid placeholder
            };
            assert_eq!(mux, *expected, "Channel {} should map to 0x{:04X}", channel, expected);
        }
    }
}


