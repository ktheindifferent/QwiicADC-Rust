extern crate qwiic_adc_rs;

use qwiic_adc_rs::*;
use std::thread;
use std::time::Duration;

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

    // Set gain to 2x (±2.048V range)
    adc.set_gain(PGA::Two).expect("Failed to set gain");
    println!("\nGain set to 2x (±2.048V range)");

    // Set sample rate to 1600 Hz
    adc.set_sample_rate(SampleRates::S1600Hz).expect("Failed to set sample rate");
    println!("Sample rate set to 1600 Hz");

    // Read single-ended channels (0-3) with voltage conversion
    println!("\nSingle-ended readings:");
    for channel in 0..4 {
        match adc.get_single_ended(channel) {
            Ok(value) => {
                match adc.raw_to_voltage(value, PGA::Two) {
                    Ok(voltage) => println!("  Channel {channel}: {value} (raw) = {voltage:.2} mV"),
                    Err(e) => println!("  Channel {channel}: Voltage conversion error - {e:?}"),
                }
            },
            Err(e) => println!("  Channel {channel}: Error - {e:?}"),
        }
    }

    // Read differential mode with multiple configurations
    println!("\nDifferential readings:");
    let diff_modes = vec![
        (Some(Mux::DiffP0N1 as u16), "P0-N1"),
        (Some(Mux::DiffP0N3 as u16), "P0-N3"),
        (Some(Mux::DiffP1N3 as u16), "P1-N3"),
        (Some(Mux::DiffP2N3 as u16), "P2-N3"),
    ];
    
    for (mode, name) in diff_modes {
        match adc.get_differential(mode) {
            Ok(value) => {
                match adc.raw_to_voltage(value, PGA::Two) {
                    Ok(voltage) => println!("  {name}: {value} (raw) = {voltage:.2} mV"),
                    Err(e) => println!("  {name}: Voltage conversion error - {e:?}"),
                }
            },
            Err(e) => println!("  {name}: Error - {e:?}"),
        }
    }

    // Set thresholds for comparator (demonstrating validation)
    println!("\nSetting comparator thresholds:");
    // Valid thresholds for ADS1015: -2048 to 2047
    adc.set_low_threshold(1000).expect("Failed to set low threshold");
    adc.set_high_threshold(2000).expect("Failed to set high threshold");
    println!("  Low threshold: 1000 (valid for ADS1015: -2048 to 2047)");
    println!("  High threshold: 2000 (valid for ADS1015: -2048 to 2047)");
    
    // Demonstrate invalid threshold handling
    println!("\nDemonstrating threshold validation:");
    match adc.set_low_threshold(-3000) {
        Ok(_) => println!("  Threshold -3000 accepted (unexpected)"),
        Err(e) => println!("  Threshold -3000 rejected: {}", e),
    }

    // Demonstrate continuous mode
    println!("\nContinuous mode on channel 0 (5 readings):");
    adc.start_continuous(0).expect("Failed to start continuous mode");
    
    for i in 0..5 {
        thread::sleep(Duration::from_millis(10));
        match adc.read_last_conversion() {
            Ok(value) => {
                match adc.raw_to_voltage(value, PGA::Two) {
                    Ok(voltage) => println!("  Reading {}: {value} (raw) = {voltage:.2} mV", i + 1),
                    Err(e) => println!("  Reading {}: Voltage conversion error - {e:?}", i + 1),
                }
            },
            Err(e) => println!("  Reading {}: Error - {e:?}", i + 1),
        }
    }
    
    adc.stop_continuous().expect("Failed to stop continuous mode");
    println!("Continuous mode stopped");

    // Test different gain settings
    println!("\nTesting different gain settings on channel 0:");
    let gains = vec![
        (PGA::TwoThirds, "±6.144V"),
        (PGA::One, "±4.096V"),
        (PGA::Two, "±2.048V"),
        (PGA::Four, "±1.024V"),
    ];
    
    for (gain, range) in gains {
        adc.set_gain(gain).expect("Failed to set gain");
        match adc.get_single_ended(0) {
            Ok(value) => {
                match adc.raw_to_voltage(value, gain) {
                    Ok(voltage) => println!("  Gain {range}: {value} (raw) = {voltage:.2} mV"),
                    Err(e) => println!("  Gain {range}: Voltage conversion error - {e:?}"),
                }
            },
            Err(e) => println!("  Gain {range}: Error - {e:?}"),
        }
    }
}