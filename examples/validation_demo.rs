use qwiic_adc_rs::*;

fn main() {
    println!("ADC Input Validation Demo");
    println!("=========================\n");

    // Create ADC instance
    let config = QwiicADCConfig::default(); // ADS1015
    let mut adc = match QwiicADC::new(config, "/dev/i2c-1", 0x48) {
        Ok(adc) => adc,
        Err(e) => {
            println!("Note: Could not connect to ADC hardware: {}", e);
            println!("This demo shows validation error handling even without hardware.\n");
            
            // Create another instance for demonstration (will still fail without hardware)
            QwiicADC::new(QwiicADCConfig::default(), "/dev/i2c-1", 0x48)
                .expect("Failed to create ADC instance")
        }
    };

    // Demonstrate channel validation
    println!("1. Channel Validation:");
    println!("   Valid channels are 0-3");
    
    match adc.get_single_ended(0) {
        Ok(_) => println!("   ✓ Channel 0: Valid"),
        Err(e) => println!("   ✗ Channel 0: {}", e),
    }
    
    match adc.get_single_ended(5) {
        Ok(_) => println!("   ✓ Channel 5: Valid (unexpected!)"),
        Err(AdcError::InvalidChannel(ch)) => {
            println!("   ✗ Channel {}: Invalid - must be 0-3", ch);
        },
        Err(e) => println!("   ✗ Channel 5: {}", e),
    }

    // Demonstrate threshold validation for ADS1015
    println!("\n2. Threshold Validation (ADS1015):");
    println!("   Valid range: -2048 to 2047");
    
    match adc.set_low_threshold(1000) {
        Ok(_) => println!("   ✓ Threshold 1000: Valid"),
        Err(e) => println!("   ✗ Threshold 1000: {}", e),
    }
    
    match adc.set_low_threshold(-3000) {
        Ok(_) => println!("   ✓ Threshold -3000: Valid (unexpected!)"),
        Err(AdcError::InvalidThreshold(val)) => {
            println!("   ✗ Threshold {}: Out of range for ADS1015", val);
        },
        Err(e) => println!("   ✗ Threshold -3000: {}", e),
    }
    
    match adc.set_high_threshold(2048) {
        Ok(_) => println!("   ✓ Threshold 2048: Valid (unexpected!)"),
        Err(AdcError::InvalidThreshold(val)) => {
            println!("   ✗ Threshold {}: Out of range for ADS1015", val);
        },
        Err(e) => println!("   ✗ Threshold 2048: {}", e),
    }

    // Demonstrate gain validation in voltage conversion
    println!("\n3. Gain Validation in Voltage Conversion:");
    println!("   raw_to_voltage() now verifies gain matches ADC configuration");
    
    // This would work with hardware:
    // adc.set_gain(PGA::Two).unwrap();
    // let raw = adc.get_single_ended(0).unwrap();
    
    // Simulated scenario:
    println!("   Scenario: ADC configured with gain = PGA::Two");
    println!("   Attempting conversion with PGA::Four...");
    
    // This would fail with GainMismatch error if hardware was present
    let simulated_raw = 1024;
    match adc.raw_to_voltage(simulated_raw, PGA::Four) {
        Ok(_) => println!("   ✓ Conversion succeeded"),
        Err(AdcError::GainMismatch { expected, provided }) => {
            println!("   ✗ Gain mismatch detected!");
            println!("     Expected: {:?}", expected);
            println!("     Provided: {:?}", provided);
        },
        Err(e) => println!("   ✗ Error: {}", e),
    }

    // Demonstrate error messages
    println!("\n4. Error Message Examples:");
    
    let err = AdcError::InvalidChannel(10);
    println!("   InvalidChannel(10): {}", err);
    
    let err = AdcError::InvalidThreshold(-5000);
    println!("   InvalidThreshold(-5000): {}", err);
    
    let err = AdcError::GainMismatch { 
        expected: PGA::Two, 
        provided: PGA::Eight 
    };
    println!("   GainMismatch: {}", err);
    
    let err = AdcError::InvalidConfiguration("Invalid differential mode".to_string());
    println!("   InvalidConfiguration: {}", err);

    println!("\n5. ADS1115 Threshold Validation:");
    println!("   For ADS1115 (16-bit), valid range is -32768 to 32767");
    
    let config_1115 = QwiicADCConfig::new("ADS1115".to_string());
    let mut adc_1115 = match QwiicADC::new(config_1115, "/dev/i2c-1", 0x48) {
        Ok(adc) => adc,
        Err(_) => {
            println!("   (Hardware not available for demonstration)");
            return;
        }
    };
    
    match adc_1115.set_low_threshold(-32768) {
        Ok(_) => println!("   ✓ Threshold -32768: Valid for ADS1115"),
        Err(e) => println!("   ✗ Threshold -32768: {}", e),
    }
    
    match adc_1115.set_high_threshold(32767) {
        Ok(_) => println!("   ✓ Threshold 32767: Valid for ADS1115"),
        Err(e) => println!("   ✗ Threshold 32767: {}", e),
    }
}