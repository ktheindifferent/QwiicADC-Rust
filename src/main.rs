extern crate qwiic_adc_rs;

use qwiic_adc_rs::*;

fn main() {
    let config = QwiicADCConfig::default();
    let mut adc = QwiicADC::new(config, "/dev/i2c-1", 0x48).expect("Could not init ADC device");

    // Initialize the ADC
    adc.init().expect("Failed to initialize ADC");

    // Check if device is connected
    if !adc.is_connected() {
        println!("ADC device not found at address 0x48");
        return;
    }
    println!("ADC device connected successfully");

    // Read configuration register
    let cfg = adc.read_register(0x01).unwrap_or(0);
    println!("Configuration register: 0x{cfg:02X}");

    // Read single-ended channels (0-3)
    println!("\nSingle-ended readings:");
    for channel in 0..4 {
        match adc.get_single_ended(channel) {
            Ok(value) => println!("  Channel {channel}: {value}"),
            Err(e) => println!("  Channel {channel}: Error - {e:?}"),
        }
    }

    // Read differential mode
    println!("\nDifferential reading:");
    match adc.get_differential(None) {
        Ok(value) => println!("  P0-N1: {value}"),
        Err(e) => println!("  P0-N1: Error - {e:?}"),
    }

    // Read analog data using the convenience function
    println!("\nAnalog data readings:");
    for channel in 0..4 {
        match adc.get_analog_data(channel) {
            Ok(value) => println!("  Channel {channel}: {value}"),
            Err(e) => println!("  Channel {channel}: Error - {e:?}"),
        }
    }
}