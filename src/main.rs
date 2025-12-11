use anyhow::{Context, Result};
use hidapi::HidApi;
use std::fs;
use std::time::Duration;
use std::thread;

// Vendor ID and Product ID for the water cooler display
const VENDOR_ID: u16 = 0xaa88; // 43656 in decimal
const PRODUCT_ID: u16 = 0x8666; // 34406 in decimal
const UPDATE_INTERVAL_MS: u64 = 1000; // Update every 1 second

fn main() -> Result<()> {
    println!("SendTemp (Rust version) - Starting...");
    println!("Reading CPU temperature and sending to water cooler display");
    
    // Initialize HID API
    let api = HidApi::new().context("Failed to initialize HID API")?;
    
    // Keep trying to connect to devices
    loop {
        match run_temperature_sender(&api) {
            Ok(_) => {
                println!("Temperature sender stopped normally");
                break;
            }
            Err(e) => {
                eprintln!("Error: {}. Retrying in 1 second...", e);
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
    
    Ok(())
}

fn run_temperature_sender(api: &HidApi) -> Result<()> {
    // Find and connect to HID devices with matching vendor/product ID
    let mut devices = Vec::new();
    
    println!("Searching for HID devices (VID: 0x{:04x}, PID: 0x{:04x})...", VENDOR_ID, PRODUCT_ID);
    
    for device_info in api.device_list() {
        if device_info.vendor_id() == VENDOR_ID && device_info.product_id() == PRODUCT_ID {
            println!("Found device: {:?}", device_info.path());
            match device_info.open_device(api) {
                Ok(device) => {
                    println!("Successfully opened HID device");
                    devices.push(device);
                }
                Err(e) => {
                    eprintln!("Failed to open device: {}", e);
                }
            }
        }
    }
    
    if devices.is_empty() {
        anyhow::bail!("No matching HID devices found");
    }
    
    println!("Connected to {} device(s)", devices.len());
    println!("Starting temperature monitoring...");
    
    // Continuously read CPU temperature and send to devices
    loop {
        match read_cpu_temperature() {
            Ok(temp) => {
                // Create a 24-byte buffer with temperature in binary format
                // The display expects: [temp_integer, temp_decimal, padding...]
                let mut buffer = [0u8; 24];
                
                let temp_int = temp as u8;  // Integer part of temperature
                let temp_decimal = ((temp - temp_int as f32) * 10.0) as u8;  // First decimal digit
                
                // Format: byte 0 = integer temp, byte 1 = decimal digit (0-9)
                buffer[0] = temp_int;
                buffer[1] = temp_decimal;
                
                println!("CPU: {:.1}°C (sending bytes: {:02x} {:02x})", temp, buffer[0], buffer[1]);
                
                // Send to all connected HID devices
                for device in &devices {
                    if let Err(e) = device.write(&buffer) {
                        eprintln!("Failed to write to HID device: {}", e);
                        return Err(anyhow::anyhow!("HID write failed: {}", e));
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to read temperature: {}", e);
            }
        }
        
        thread::sleep(Duration::from_millis(UPDATE_INTERVAL_MS));
    }
}

fn read_cpu_temperature() -> Result<f32> {
    // Try to read from hwmon (most common on Linux)
    if let Ok(temp) = read_hwmon_temperature() {
        return Ok(temp);
    }
    
    // Try to read from thermal_zone (alternative method)
    if let Ok(temp) = read_thermal_zone_temperature() {
        return Ok(temp);
    }
    
    anyhow::bail!("Could not read CPU temperature from any source")
}

fn read_hwmon_temperature() -> Result<f32> {
    // Search for CPU temperature in /sys/class/hwmon/
    let hwmon_path = "/sys/class/hwmon";
    
    if let Ok(entries) = fs::read_dir(hwmon_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            // Check if this is a CPU temperature sensor
            if let Ok(name) = fs::read_to_string(path.join("name")) {
                let name = name.trim();
                
                // Look for common CPU temperature sensor names
                if name.contains("coretemp") || name.contains("k10temp") || 
                   name.contains("zenpower") || name.contains("cpu") {
                    
                    // Try to read temp1_input (package temperature)
                    if let Ok(temp_str) = fs::read_to_string(path.join("temp1_input")) {
                        if let Ok(temp_millidegrees) = temp_str.trim().parse::<i32>() {
                            return Ok(temp_millidegrees as f32 / 1000.0);
                        }
                    }
                }
            }
        }
    }
    
    anyhow::bail!("No hwmon temperature sensors found")
}

fn read_thermal_zone_temperature() -> Result<f32> {
    // Try reading from thermal zones
    let thermal_path = "/sys/class/thermal";
    
    if let Ok(entries) = fs::read_dir(thermal_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            
            if name.to_string_lossy().starts_with("thermal_zone") {
                // Check if this is a CPU thermal zone
                if let Ok(zone_type) = fs::read_to_string(path.join("type")) {
                    let zone_type = zone_type.trim();
                    
                    if zone_type.contains("cpu") || zone_type.contains("x86_pkg_temp") {
                        if let Ok(temp_str) = fs::read_to_string(path.join("temp")) {
                            if let Ok(temp_millidegrees) = temp_str.trim().parse::<i32>() {
                                return Ok(temp_millidegrees as f32 / 1000.0);
                            }
                        }
                    }
                }
            }
        }
    }
    
    anyhow::bail!("No thermal zone temperature found")
}
