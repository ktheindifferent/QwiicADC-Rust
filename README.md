# Qwiic ADS1015 I2C library for Rust (WIP)

## Description

This library aims at controlling Qwiic ADC devices using I2C from Linux. Its primary target is ARM devices such as RaspberryPi or FriendlyARM's NanoPi Neo. It should nonetheless work on other Linux distributions with access to an I2C bus.

## How to use library

Add the following line to your cargo.toml:
```
qwiic-adc-rs = "0.1.11"
```

Or for the most recent commit on the master branch use:
```
qwiic-adc-rs = { git = "https://github.com/PixelCoda/QwiicADC-Rust.git", version = "*" }
```

## Example Usage

```rust
use qwiic_adc_rs::*;

fn main() {
    // Create ADC with default configuration (ADS1015)
    let config = QwiicADCConfig::default();
    let mut adc = QwiicADC::new(config, "/dev/i2c-1", 0x48).unwrap();
    
    // Initialize and check connection
    adc.init().unwrap();
    if !adc.is_connected() {
        println!("ADC device not found!");
        return;
    }
    
    // Configure the ADC
    adc.set_gain(PGA::Two).unwrap();  // ±2.048V range
    adc.set_sample_rate(SampleRates::S1600Hz).unwrap();
    
    // Read single-ended channel
    let raw_value = adc.get_single_ended(0).unwrap();
    let voltage = adc.raw_to_voltage(raw_value, PGA::Two);
    println!("Channel 0: {} mV", voltage);
    
    // Read differential input
    let diff_value = adc.get_differential(Some(Mux::DiffP0N1 as u16)).unwrap();
    println!("Differential P0-N1: {}", diff_value);
    
    // Use continuous mode for streaming
    adc.start_continuous(0).unwrap();
    for _ in 0..10 {
        let value = adc.read_last_conversion().unwrap();
        println!("Continuous reading: {}", value);
    }
    adc.stop_continuous().unwrap();
}
```

## Features

- ✅ Single-ended ADC readings (4 channels)
- ✅ Differential ADC readings (4 configurations)
- ✅ Configurable gain settings (6 levels)
- ✅ Configurable sample rates (128Hz to 3300Hz)
- ✅ Continuous and single-shot conversion modes
- ✅ Voltage conversion from raw ADC values
- ✅ Threshold settings for comparator mode
- ✅ Support for both ADS1015 (12-bit) and ADS1115 (16-bit)

## API Documentation

The library provides a comprehensive API for controlling the ADC:

- `new()` - Create a new ADC instance
- `init()` - Initialize the device
- `is_connected()` - Check if device is responding
- `get_single_ended()` - Read single-ended channel (0-3)
- `get_differential()` - Read differential input
- `get_analog_data()` - Convenience wrapper for single-ended read
- `set_gain()` / `get_gain()` - Configure/read gain settings
- `set_sample_rate()` / `get_sample_rate()` - Configure/read sample rate
- `set_mode()` - Set operating mode (continuous/single-shot)
- `start_continuous()` / `stop_continuous()` - Control continuous mode
- `read_last_conversion()` - Read last conversion result
- `set_low_threshold()` / `set_high_threshold()` - Configure comparator
- `raw_to_voltage()` - Convert raw ADC to millivolts

See the [documentation](https://docs.rs/qwiic-adc-rs) for detailed API information.


## References

* https://github.com/sparkfun/SparkFun_ADS1015_Arduino_Library

## License

Released under Apache 2.0.

# Support and follow my work by:

#### Buying my dope NTFs:
 * https://opensea.io/accounts/PixelCoda

#### Checking out my Github:
 * https://github.com/PixelCoda

#### Following my facebook page:
 * https://www.facebook.com/pixelcoda/

#### Subscribing to my Patreon:
 * https://www.patreon.com/calebsmith_pixelcoda

#### Or donating crypto:
 * ADA: addr1qyp299a45tgvveh83tcxlf7ds3yaeh969yt3v882lvxfkkv4e0f46qvr4wzj8ty5c05jyffzq8a9pfwz9dl6m0raac7s4rac48
 * ALGO: VQ5EK4GA3IUTGSPNGV64UANBUVFAIVBXVL5UUCNZSDH544XIMF7BAHEDM4
 * ATOM: cosmos1wm7lummcealk0fxn3x9tm8hg7xsyuz06ul5fw9
 * BTC: bc1qh5p3rff4vxnv23vg0hw8pf3gmz3qgc029cekxz
 * ETH: 0x7A66beaebF7D0d17598d37525e63f524CfD23452
 * ERC20: 0x7A66beaebF7D0d17598d37525e63f524CfD23452
 * XLM: GCJAUMCO2L7PTYMXELQ6GHBTF25MCQKEBNSND2C4QMUPTSVCPEN3LCOG
 * XTZ: tz1SgJppPn56whprsDDGcqR4fxqCr2PXvg1R
