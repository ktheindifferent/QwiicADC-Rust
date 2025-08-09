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

use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};

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

type ADCResult = Result<(), LinuxI2CError>;
type ReadResult = Result<u16, LinuxI2CError>;

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
    


    /// Read a single-ended ADC value from the specified channel
    ///
    /// # Arguments
    /// * `channel` - Channel number (0-3)
    ///
    /// # Returns
    /// 12-bit ADC value for ADS1015, 16-bit for ADS1115
    pub fn get_single_ended(&mut self, channel: u8) -> ReadResult {
        if channel > 3 {
            return Ok(0);
        }

        let mut config = (OS::Single as u16) | (Modes::Single as u16) | (SampleRates::S1600Hz as u16);
        config |= PGA::Two as u16;

        if channel == 0 {
            config |= Mux::Single0 as u16;
        }

        if channel == 1 {
            config |= Mux::Single1 as u16;
        }

        if channel == 2 {
            config |= Mux::Single2 as u16;
        }

        if channel == 3 {
            config |= Mux::Single3 as u16;
        }

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

        let mut config_mux_diff = Mux::DiffP0N1 as u16;
        if cfg_mux_diff.is_some(){
            config_mux_diff = cfg_mux_diff.unwrap();
        }

        if config_mux_diff == Mux::DiffP0N1 as u16 ||
           config_mux_diff == Mux::DiffP0N3 as u16 ||
           config_mux_diff == Mux::DiffP1N3 as u16 ||
           config_mux_diff == Mux::DiffP2N3 as u16 {
            // Do nothing and carry on below    
        } else {
            return Ok(0);
        }


        let mut config = (OS::Single as u16) | (Modes::Single as u16) | (SampleRates::S1600Hz as u16);
        config |= PGA::Two as u16;

        config |= config_mux_diff; // default is ADS1015_CONFIG_MUX_DIFF_P0_N1


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
        
        assert_eq!(OS::Single as u16, 0x8000);
        assert_eq!(Modes::Single as u16, 0x0100);
        
        assert_eq!(PGA::Two as u16, 0x0400);
        assert_eq!(SampleRates::S1600Hz as u16, 0x0080);
    }
}


