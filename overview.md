# Project Overview - Qwiic ADC Rust Library

## Architecture

```
qwiic-adc-rs/
├── src/
│   ├── lib.rs          # Main library implementation
│   └── main.rs         # Example/demo application
├── Cargo.toml          # Project metadata and dependencies
├── LICENSE             # Apache 2.0 license
└── README.md           # User documentation
```

## Core Design

### Library Structure (`lib.rs`)

The library follows a clean, modular design:

1. **Device Abstraction Layer**
   - `QwiicADC`: Main controller struct encapsulating I2C device
   - `QwiicADCConfig`: Configuration management for different ADC models

2. **Hardware Constants**
   - Register addresses (Pointers enum)
   - I2C addresses based on ADDR pin configuration
   - Operating modes and multiplexer settings
   - Gain and sample rate configurations

3. **Public API**
   - `new()`: Device initialization
   - `init()`: Hardware setup
   - `is_connected()`: Connection verification
   - `get_single_ended()`: Read single channel (0-3)
   - `get_differential()`: Read differential pairs
   - `get_analog_data()`: Convenience wrapper

4. **Low-Level I/O**
   - Register read/write operations
   - 16-bit data handling for ADC results
   - I2C SMBus protocol implementation

### Key Features

- **Multi-Model Support**: ADS1015 (12-bit) and ADS1115 (16-bit)
- **Flexible Input Configuration**: 4 single-ended or 2 differential channels
- **Programmable Gain**: 6 gain settings from ±0.256V to ±6.144V
- **Variable Sample Rates**: 128 Hz to 3300 Hz
- **Linux Integration**: Direct I2C bus access via /dev/i2c-*

### Usage Pattern

```rust
1. Create configuration
2. Initialize device with I2C bus and address
3. Verify connection
4. Read ADC values (single-ended or differential)
5. Process results (12-bit or 16-bit based on model)
```

### Testing Strategy

- Unit tests for configuration and enum values
- Integration tests for hardware interaction (requires physical device)
- Example application demonstrating all features

## Target Platforms

- Primary: ARM Linux (Raspberry Pi, NanoPi Neo)
- Secondary: Any Linux system with I2C support
- Hardware: SparkFun Qwiic ADC boards with ADS1015/ADS1115 chips