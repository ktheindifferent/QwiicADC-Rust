extern crate qwiic_adc_rs;

use qwiic_adc_rs::*;
use std::thread;
use std::time::Duration;

fn main() {
    let config = QwiicADCConfig::default();
    let mut qwiic_relay = QwiicADC::new(config, "/dev/i2c-1", 0x48).expect("Could not init device");

    let cfg = qwiic_relay.read_register(0x01).unwrap();



    let get_single_ended = qwiic_relay.get_single_ended(0).unwrap();

    let get_differential = qwiic_relay.get_differential(None).unwrap();


    let get_analog_data = qwiic_relay.get_analog_data(0).unwrap();
    let get_analog_data1 = qwiic_relay.get_analog_data(1).unwrap();
    let get_analog_data2 = qwiic_relay.get_analog_data(2).unwrap();
    let get_analog_data3 = qwiic_relay.get_analog_data(3).unwrap();
    let get_analog_data4 = qwiic_relay.get_analog_data(4).unwrap();


    println!("cfg: {}", cfg);
    println!("get_single_ended: {}", get_single_ended);
    println!("get_differential: {}", get_differential);


    println!("get_analog_data: {}", get_analog_data);
    println!("get_analog_data1: {}", get_analog_data1);
    println!("get_analog_data2: {}", get_analog_data2);
    println!("get_analog_data3: {}", get_analog_data3);
    println!("get_analog_data4: {}", get_analog_data4);
}