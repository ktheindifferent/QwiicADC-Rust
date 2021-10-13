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



pub struct QwiicADCConfig {
    model: String
}

impl QwiicADCConfig {
    pub fn new(model: String) -> QwiicADCConfig {
        QwiicADCConfig {
            model: model,
        }
    }

    pub fn default() -> QwiicADCConfig {
        QwiicADCConfig::new("ADS1015".to_string())
    }
}

// QwiicADC
pub struct QwiicADC {
    dev: LinuxI2CDevice,
    config: QwiicADCConfig,
}

type RelayDeviceStatus = Result<bool, LinuxI2CError>;
type ADCResult = Result<(), LinuxI2CError>;
type VersionResult = Result<u8, LinuxI2CError>;

impl QwiicADC {
    pub fn new(config: QwiicADCConfig, bus: &str, i2c_addr: u16) -> Result<QwiicADC, LinuxI2CError> {
        let dev = LinuxI2CDevice::new(bus, i2c_addr)?;
        Ok(QwiicADC {
               dev,
               config,
           })
    }

    pub fn init(&mut self) -> ADCResult {





  
        // Wait for the QwiicADC to set up
        thread::sleep(Duration::from_millis(200));

        Ok(())
    }




    pub fn write_byte(&mut self, command: u8) -> ADCResult {
        self.dev.smbus_write_byte(command)?;
        thread::sleep(Duration::new(0, 10_000));
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {

        let config = QwiicADCConfig::default();
        let mut qwiic_relay = QwiicADC::new(config, "/dev/i2c-1", 0x08).expect("Could not init device");
 
    }
}


