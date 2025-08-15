# Qwiic ADC Rust Library Codebase Overview

## Project Description

This is a Rust library for controlling SparkFun Qwiic ADC devices (ADS1015/ADS1115) using I2C communication on Linux systems. The library is primarily targeted at ARM devices like Raspberry Pi and NanoPi Neo, but works on any Linux distribution with I2C bus access.

## Key Features

- **Multi-channel ADC Support**: 4 single-ended or 2 differential input channels
- **Configurable Gain Settings**: 6 programmable gain levels (±6.144V to ±0.256V)
- **Variable Sample Rates**: 128Hz to 3300Hz sampling rates
- **Dual Operation Modes**: Single-shot and continuous conversion modes
- **Voltage Conversion**: Automatic conversion from raw ADC values to millivolts
- **Comparator Support**: Configurable high/low thresholds for comparator mode
- **Device Support**: Both ADS1015 (12-bit) and ADS1115 (16-bit) ADC chips

## Technology Stack

- **Language**: Rust (Edition 2021)
- **Primary Dependency**: `i2cdev` (0.4.4) - Linux I2C device interface
- **Helper Library**: `enum_primitive` (0.1.1) - Enum conversion utilities
- **License**: Dual MIT OR Apache-2.0
- **Package Version**: 0.1.11

## Project Structure

```
/root/repo/
├── Cargo.toml          # Rust package manifest
├── LICENSE             # License file
├── README.md           # User documentation and examples
├── Claude.md           # This file - codebase documentation
├── overview.md         # Project overview documentation
├── project_description.md  # Project description
├── todo.md             # Task tracking
└── src/
    ├── lib.rs          # Main library implementation (~540 lines)
    └── main.rs         # Example/demo application (108 lines)
```

## Core Components

### Main Structures

1. **QwiicADC** - Main ADC controller struct
   - Manages I2C device communication
   - Handles configuration and data reading

2. **QwiicADCConfig** - Configuration container
   - Stores ADC model type (ADS1015/ADS1115)
   - Used during initialization

### Key Enums

- **Addresses**: I2C addresses (0x48-0x4B based on ADDR pin)
- **PGA**: Programmable gain amplifier settings
- **SampleRates**: Data rate configurations (128Hz-3300Hz)
- **Mux**: Input multiplexer configurations
- **Modes**: Operating modes (Single/Continuous)
- **Pointers**: Register addresses
- **Comparator enums**: Cmode, Cpol, Clat, Cque

### Primary API Methods

#### Initialization & Connection
- `new()` - Create ADC instance with config, bus path, and I2C address
- `init()` - Initialize the device
- `is_connected()` - Verify device connectivity

#### Data Reading
- `get_single_ended(channel)` - Read single-ended channel (0-3)
- `get_differential(cfg_mux_diff)` - Read differential input
- `get_analog_data(channel)` - Convenience wrapper for single-ended
- `read_last_conversion()` - Read last conversion result

#### Configuration
- `set_gain()/get_gain()` - Configure/read gain settings
- `set_sample_rate()/get_sample_rate()` - Configure/read sample rate
- `set_mode()` - Set operating mode

#### Continuous Mode
- `start_continuous(channel)` - Begin continuous conversion
- `stop_continuous()` - Stop continuous mode

#### Comparator
- `set_low_threshold()` - Set lower threshold
- `set_high_threshold()` - Set upper threshold
- `get_low_threshold()/get_high_threshold()` - Read thresholds

#### Utilities
- `raw_to_voltage(raw_value, gain)` - Convert raw ADC to millivolts
- `read_register()/write_register()` - Low-level register access

## Example Usage Pattern

```rust
// 1. Create configuration
let config = QwiicADCConfig::default();

// 2. Initialize ADC
let mut adc = QwiicADC::new(config, "/dev/i2c-1", 0x48)?;
adc.init()?;

// 3. Configure settings
adc.set_gain(PGA::Two)?;
adc.set_sample_rate(SampleRates::S1600Hz)?;

// 4. Read data
let raw = adc.get_single_ended(0)?;
let voltage = adc.raw_to_voltage(raw, PGA::Two);
```

## Testing & Examples

The `main.rs` file provides a comprehensive demonstration of the library's capabilities including:
- Single-ended readings from all 4 channels
- Differential readings with multiple configurations
- Continuous mode operation
- Different gain settings testing
- Comparator threshold configuration

## Build & Run

```bash
# Build library
cargo build

# Run example
cargo run

# Build for release
cargo build --release
```

## Dependencies

- Linux system with I2C support
- I2C device at `/dev/i2c-1` (configurable)
- ADS1015 or ADS1115 ADC connected via I2C

## Documentation

- [Crates.io Package](https://crates.io/crates/qwiic-adc-rs)
- [API Documentation](https://docs.rs/qwiic-adc-rs)
- [GitHub Repository](https://github.com/PixelCoda/QwiicADC-Rust)

## Recent Updates

The library has been refactored to include:
- Full ADC control API with comprehensive gain, sample rate, and mode configurations
- Support for both ADS1015 and ADS1115 chips
- Improved error handling using Result types
- Comprehensive example demonstrating all features
- Clean separation between library (lib.rs) and examples (main.rs)