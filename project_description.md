# Project Description - Qwiic ADC Rust Library

## Summary of Work

This project is a Rust library for controlling SparkFun Qwiic ADC devices (ADS1015/ADS1115) using I2C communication on Linux systems, primarily targeting ARM devices like Raspberry Pi and NanoPi Neo.

### Recent Refactoring (Branch: terragon/bugfix-refactor-docs)

The library underwent a major refactoring to improve code organization and extend support:

1. **Enhanced ADC Support**: Extended from just ADS1015 to also support ADS1115 (16-bit ADC)
2. **Improved API Design**: Reorganized configuration and control structures for better usability
3. **Documentation**: Added comprehensive inline documentation with examples
4. **Code Structure**: Refactored enums and constants for better type safety and clarity

### Key Components

- **QwiicADC**: Main struct for device interaction
- **QwiicADCConfig**: Configuration struct supporting multiple ADC models
- **Comprehensive Enums**: Well-organized enums for addresses, modes, gain settings, sample rates
- **Read Methods**: Support for both single-ended and differential ADC readings
- **Hardware Abstraction**: Clean I2C interface using Linux I2C device drivers

### Current State

Version: 0.1.11
- **Fully implemented** core functionality for single-ended and differential readings
- Support for 4 single-ended or 2 differential channels
- **Implemented** configurable gain and sample rate settings with getters/setters
- **Added** continuous conversion mode with start/stop controls
- **Added** voltage conversion from raw ADC values
- **Added** threshold configuration for comparator mode
- **Expanded** test suite to 19 tests (6 unit tests, 13 hardware tests)
- **Enhanced** example main.rs demonstrating all library features
- **Updated** README with complete API documentation and usage examples

### Work Completed in This Session

1. **Documentation**: Created project_description.md, overview.md, and todo.md
2. **API Enhancements**:
   - Added `set_gain()`, `get_gain()` methods
   - Added `set_sample_rate()`, `get_sample_rate()` methods
   - Added `set_mode()` for mode switching
   - Added `set_low_threshold()`, `set_high_threshold()`, and getters
   - Added `raw_to_voltage()` for voltage conversion
   - Added `start_continuous()`, `stop_continuous()`, `read_last_conversion()`
3. **Testing**: Expanded from 3 to 19 tests, covering all major functionality
4. **Examples**: Enhanced main.rs to demonstrate all features
5. **Documentation**: Updated README from "not implemented" to full feature list

### Dependencies

- `i2cdev` (0.4.4): Linux I2C device interface
- `enum_primitive` (0.1.1): Enum conversion utilities