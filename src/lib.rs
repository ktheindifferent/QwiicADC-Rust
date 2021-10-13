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

#[derive(Copy, Clone)]
pub enum Addresses {
    Gnd = 0x48,
    Vdd = 0x19,
    Sda = 0x4A,
    Scl = 0x4B
}

#[derive(Copy, Clone)]
pub enum OS {
    No = 0x48,
    Single = 0x19,
    Ready = 0x4A,
    NotReady = 0x4B
}

#[derive(Copy, Clone)]
pub enum Pointers {
    Convert = 0x00,
    Config = 0x01,
    LowThresh = 0x02,
    HighThresh = 0x03
}

#[derive(Copy, Clone)]
pub enum Modes {
    Continuous =0x0000,
    Single = 0x0100,
}

#[derive(Copy, Clone)]
pub enum Mux {
    Single0 = 0x4000,
    Single1 = 0x5000,
    Single2 = 0x6000,
    Single3 = 0x7000,
    DiffP0N1 = 0x0000,
    DiffP0N3 = 0x1000,
    DiffP1N3 = 0x2000,
    DiffP2N3 = 0x3000,
}

#[derive(Copy, Clone)]
pub enum SampleRates {
    S_128HZ = 0x0000,
    S_250HZ = 0x0020,
    S_490HZ = 0x0040,
    S_920HZ = 0x0060,
    S_1600HZ = 0x0080,
    S_2400HZ = 0x00A0,
    S_3300HZ = 0x00C0
}


#[derive(Copy, Clone)]
pub enum PGA {
    Mask = 0x0E00,
    TwoThirds = 0x0000, // +/- 6.144v
    One = 0x0200,       // +/- 4.096v
    Two = 0x0400,       // +/- 2.048v
    Four = 0x0600,      // +/- 1.024v
    Eight = 0x0800,     // +/- 0.512v
    Sixteen = 0x0A00    // +/- 0.256v
}

#[derive(Copy, Clone)]
pub enum Cmode {
    Trad = 0x0000,      // Traditional comparator with hysteresis (default)
    Window = 0x0010,    // Window comparator
}

#[derive(Copy, Clone)]
pub enum Cpol {
    ActvLow = 0x0000,   // ALERT/RDY pin is low when active (default)
    ActvHigh = 0x0008,  // ALERT/RDY pin is high when active
}

#[derive(Copy, Clone)]
pub enum Clat {
    NonLat = 0x0000,
    Latch = 0x0004,
}

#[derive(Copy, Clone)]
pub enum Cque {
    OneConv = 0x0000,
    TwoConv = 0x0001,
    FourConv = 0x0002,
    None = 0x0003
}

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
type ReadResult = Result<u8, LinuxI2CError>;

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

    pub fn get_single_ended(&mut self, channel: u8) -> ReadResult {
        if (channel > 3) {
            return Ok(0);
        }

        let mut config = (OS::Ready as u8) | (Modes::Continuous as u8) | (SampleRates::S_1600HZ as u8);
        config = config | PGA::Two as u8;

        if channel == 0 {
            config = config | Mux::Single0 as u8;
        }

        if channel == 1 {
            config = config | Mux::Single1 as u8;
        }

        if channel == 2 {
            config = config | Mux::Single2 as u8;
        }

        if channel == 3 {
            config = config | Mux::Single3 as u8;
        }

        self.write_register((Pointers::Convert as u8), config.into());

        // delay(ADS1015_DELAY);
        thread::sleep(Duration::new(0, 10_000));


        let read = self.read_register(Pointers::Convert as u8);
        match read {
            Ok(r) => {
                return Ok(r >> 4);
            },
            Err(e) => {
                return Ok(0);
            }
        }

      

    }



    pub fn read_register(&mut self, location: u8) -> ReadResult {
        // self.dev.smbus_write_byte(Pointers::Convert as u8)?; // Do we need this?
        let byte = self.dev.smbus_read_byte_data(location)?;
        Ok(byte)
    }
    



    pub fn write_register(&mut self, register: u8, val: usize) -> ADCResult {
        self.dev.smbus_write_byte(register)?;
        self.dev.smbus_write_byte((val >> 8) as u8)?;
        self.dev.smbus_write_byte((val as u8) & 0xFF)?;
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
        let mut qwiic_relay = QwiicADC::new(config, "/dev/i2c-1", 0x48).expect("Could not init device");
 
        let cfg = qwiic_relay.read_register(0x01).unwrap();



        let get_single_ended = qwiic_relay.get_single_ended(1);


        println!("cfg: {}", cfg);
        println!("get_single_ended: {}", get_single_ended);



    }
}


