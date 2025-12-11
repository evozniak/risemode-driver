# RiseMode Driver

An experimental Rust driver for the **Rise Mode Aura Ice Black** water cooler with temperature display.

## Overview

This driver reads CPU temperature from your system and sends it to the Rise Mode Aura Ice Black water cooler's integrated display. The driver communicates with the display via USB HID protocol.

## Features

- Real-time CPU temperature monitoring
- Automatic device detection and connection
- Continuous temperature updates (1 second interval)
- Automatic reconnection on device disconnect

## Compatibility

### ✅ Linux
Fully supported. Reads CPU temperature from:
- `/sys/class/hwmon/` (coretemp, k10temp, zenpower)
- `/sys/class/thermal/` (thermal zones)

### ❌ Windows
**Currently NOT supported**. Temperature reading requires Linux-specific sysfs paths. The HID communication would work, but temperature reading needs to be implemented using Windows APIs or the `sysinfo` crate.

### ❓ macOS
Untested. Would require macOS-specific temperature reading implementation.

## Requirements

- Rust 1.70 or later
- Rise Mode Aura Ice Black water cooler connected via USB
- Linux operating system (for temperature reading)
- Appropriate permissions to access HID devices (see below)

## Installation

### 1. Clone the repository

```bash
git clone https://github.com/evozniak/risemode-driver.git
cd risemode-driver
```

### 2. Build the project

```bash
cargo build --release
```

The compiled binary will be located at `target/release/risemode-driver`.

## Usage

### Running the driver

```bash
cargo run --release
```

Or run the compiled binary directly:

```bash
./target/release/risemode-driver
```

### Linux Permissions

On Linux, you may need to run with sudo or configure udev rules to access HID devices:

```bash
sudo ./target/release/risemode-driver
```

#### Setting up udev rules (recommended)

Create a udev rule file to allow non-root access:

```bash
sudo nano /etc/udev/rules.d/99-risemode.rules
```

Add the following line (replace `YOUR_USERNAME` with your actual username):

```
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="aa88", ATTRS{idProduct}=="8666", MODE="0666", GROUP="YOUR_USERNAME"
```

Reload udev rules:

```bash
sudo udevadm control --reload-rules
sudo udevadm trigger
```

## Device Information

- **Vendor ID**: `0xaa88` (43656)
- **Product ID**: `0x8666` (34406)
- **Communication**: USB HID
- **Update Rate**: 1 Hz (every second)

## Output

The driver displays real-time information:

```
SendTemp (Rust version) - Starting...
Reading CPU temperature and sending to water cooler display
Searching for HID devices (VID: 0xaa88, PID: 0x8666)...
Found device: ...
Successfully opened HID device
Connected to 1 device(s)
Starting temperature monitoring...
CPU: 45.3°C (sending bytes: 2d 03)
CPU: 46.1°C (sending bytes: 2e 01)
...
```

## Development

### Building for development

```bash
cargo build
```

### Running with debug output

```bash
RUST_LOG=debug cargo run
```

## Technical Details

The driver sends temperature data in a 24-byte buffer format:
- Byte 0: Integer part of temperature (°C)
- Byte 1: First decimal digit (0-9)
- Bytes 2-23: Padding (zeros)

## Troubleshooting

### Device not found
- Ensure the Rise Mode Aura Ice Black is properly connected via USB
- Check if the device appears in `lsusb` output
- Verify vendor/product IDs match: `lsusb | grep aa88:8666`

### Permission denied
- Run with sudo or set up udev rules (see Usage section)

### No temperature readings
- Verify CPU temperature sensors are available: `ls /sys/class/hwmon/*/temp*_input`
- Check thermal zones: `ls /sys/class/thermal/thermal_zone*/temp`

## Contributing

This is an experimental driver. Contributions are welcome! Please feel free to submit issues or pull requests.

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Disclaimer

This is an unofficial, experimental driver. Use at your own risk. The author is not responsible for any hardware damage or data loss.
