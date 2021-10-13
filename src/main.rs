extern crate qwiic_adc_rs;

use qwiic_adc_rs::*;
use std::thread;
use std::time::Duration;

fn main() {
    let config = QwiicADCConfig::default();
    let mut qwiic_relay = QwiicADC::new(config, "/dev/i2c-1", 0x48).expect("Could not init device");

    let cfg = qwiic_relay.read_register(0x01).unwrap();

    println!("cfg: {}", cfg);

}