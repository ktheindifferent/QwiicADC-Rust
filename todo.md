# TODO List - Qwiic ADC Rust Library

## Completed Tasks ✓
- [x] Refactor library to support both ADS1015 and ADS1115
- [x] Implement single-ended ADC reading for channels 0-3
- [x] Implement differential ADC reading modes
- [x] Add comprehensive documentation comments
- [x] Create example application in main.rs
- [x] Set up basic test structure
- [x] Create project documentation files
- [x] Run and analyze existing tests
- [x] Create comprehensive test coverage (19 tests total)
- [x] Add continuous conversion mode support
- [x] Implement configurable gain settings in public API
- [x] Implement configurable sample rate settings in public API
- [x] Add voltage conversion methods (raw ADC to voltage)
- [x] Add threshold configuration for comparator mode
- [x] Implement mode switching (continuous/single-shot)
- [x] Add read_last_conversion for continuous mode
- [x] Update README with complete examples and API documentation
- [x] Fix all compilation warnings

## In Progress 🔄
None - All current tasks completed!

## Pending Tasks 📋

### High Priority
- [ ] Implement alert/ready pin functionality

### Testing & Quality
- [ ] Create unit tests for all public methods
- [ ] Add integration tests for different configurations
- [ ] Mock I2C device for testing without hardware
- [ ] Add error handling tests
- [ ] Test coverage for both ADS1015 and ADS1115 modes
- [ ] Benchmark performance for different sample rates

### Documentation
- [ ] Complete README with all examples
- [ ] Add circuit diagrams for connections
- [ ] Document voltage calculation formulas
- [ ] Add troubleshooting guide
- [ ] Create migration guide from Arduino library

### API Enhancements
- [ ] Add builder pattern for configuration
- [ ] Implement async/await support
- [ ] Add streaming/continuous read mode
- [ ] Support for multiple devices on same bus
- [ ] Add calibration methods

### Code Quality
- [ ] Add proper error types (replace LinuxI2CError)
- [ ] Implement Display/Debug traits where appropriate
- [ ] Add #[derive(Debug)] to all public structs
- [ ] Consider no_std support for embedded systems
- [ ] Add clippy lints and fix warnings

### Platform Support
- [ ] Test on Raspberry Pi 4
- [ ] Test on Raspberry Pi Zero
- [ ] Test on NanoPi Neo
- [ ] Verify Windows WSL2 compatibility
- [ ] Add cross-compilation instructions

## Known Issues 🐛
- Hardware test requires physical device (currently ignored)
- No timeout handling for I2C operations
- Missing validation for configuration combinations
- Thread sleep durations may need tuning for different devices

## Future Considerations 💭
- Support for other ADC chips in the family
- GUI application for real-time monitoring
- Integration with popular data logging frameworks
- MQTT/WebSocket streaming support
- Python bindings via PyO3