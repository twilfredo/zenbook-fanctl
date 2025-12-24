# Zenbook Fan Control (zenbook-fanctl-rs)

A lightweight Rust utility for controlling CPU fan speed on ASUS Zenbook laptops based on CPU package temperature (acpitz thermal zone).

## Overview

This daemon monitors the CPU package temperature and automatically adjusts the CPU fan control mode:
- **Above threshold (≥88°C)**: Disables PWM control, forcing the fan to run at maximum speed
- **Below threshold (<88°C)**: Reverts to automatic PWM control for normal fan operation

The utility polls the temperature every 2000ms (2 seconds) by default and adjusts fan behavior accordingly.

## Tested Hardware

- **ASUS Zenbook UX3405MA.311**

This utility should work on other ASUS Zenbook models that expose fan controls via the `asus-nb-wmi` kernel module, but has only been tested on the model listed above.

## Requirements

### System Requirements
- Linux kernel with `asus-nb-wmi` module loaded
- ACPI thermal zone support (`acpitz`)
- Root/sudo privileges (required for writing to sysfs fan control files)

## Installation

## Build Steps

### 1. Clone or navigate to the repository

```bash
cd /path/to/zenbook-fanctl-rs
```

### 2. Build the project

For development/testing:
```bash
cargo build
```

For optimized release build:
```bash
cargo build --release
```

The compiled binary will be located at:
- Debug build: `target/debug/zenbook-fanctl-rs`
- Release build: `target/release/zenbook-fanctl-rs`

### 3. Run the utility

The utility requires root privileges to write to sysfs fan control files:

```bash
sudo ./target/release/zenbook-fanctl-rs
```

### Command-Line Options

The utility supports the following command-line arguments:

```bash
sudo ./target/release/zenbook-fanctl-rs [OPTIONS]
```

**Options:**
- `-p, --polling-ms <POLLING_MS>`: Package temperature polling period in milliseconds (default: 2000)
- `-t, --temp-max <TEMP_MAX>`: Max package temperature threshold in Celsius before toggling PWM off (default: 88)
- `-h, --help`: Print help information
- `-V, --version`: Print version information

**Examples:**

```bash
# Use default settings (88°C threshold, 2000ms polling)
sudo ./target/release/zenbook-fanctl-rs

# Custom configuration
sudo ./target/release/zenbook-fanctl-rs --temp-max 85 --polling-ms 1000
```

## Configuration

You can customize the utility's behavior using command-line arguments (see [Command-Line Options](#command-line-options) above):

- Temperature threshold: Use `--temp-max` flag (default: 88°C)
- Polling interval: Use `--polling-ms` flag (default: 2000ms)

## Running as a System Service

To run the utility automatically on boot, create a systemd service:

### 1. Install the binary

```bash
sudo cp target/release/zenbook-fanctl-rs /usr/local/bin/
```

### 2. Create a systemd service file

Create `/etc/systemd/system/zenbook-fanctl.service`:

```ini
[Unit]
Description=Zenbook Fan Control Service
After=multi-user.target

[Service]
Type=simple
ExecStart=/usr/local/bin/zenbook-fanctl-rs
# Optional: Customize with command-line arguments
# ExecStart=/usr/local/bin/zenbook-fanctl-rs --temp-max 85 --polling-ms 1000
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

### 3. Enable and start the service

```bash
sudo systemctl daemon-reload
sudo systemctl enable zenbook-fanctl.service
sudo systemctl start zenbook-fanctl.service
```

### 4. Check service status

```bash
sudo systemctl status zenbook-fanctl.service
```

View logs:
```bash
sudo journalctl -u zenbook-fanctl.service -f
```

## How It Works

1. **Thermal Zone Detection**: Scans `/sys/class/thermal/` for the `acpitz` thermal zone
2. **Fan Control Detection**: Locates the CPU fan PWM control at `/sys/devices/platform/asus-nb-wmi/hwmon/*/pwm1_enable`
3. **Temperature Monitoring**: Reads temperature from the thermal zone at the configured polling interval
4. **Fan Control**:
   - Writes `0` to `pwm1_enable` to disable PWM (full speed) when temperature ≥ threshold
   - Writes `2` to `pwm1_enable` to enable automatic PWM control when temperature < threshold

## Troubleshooting

### "CPU fan control not found"
- Ensure the `asus-nb-wmi` kernel module is loaded: `lsmod | grep asus`
- Check if fan controls exist: `ls /sys/devices/platform/asus-nb-wmi/hwmon/`

### "Failed to find acpitz thermal zone"
- Verify thermal zones exist: `ls /sys/class/thermal/`
- Check for acpitz: `cat /sys/class/thermal/thermal_zone*/type`

### Permission denied errors
- Ensure you're running with sudo/root privileges
- Check file permissions on `/sys/devices/platform/asus-nb-wmi/hwmon/*/pwm1_enable`

## License

This project is provided as-is for personal use. Use at your own risk.

## Contributing

Contributions, bug reports, and feature requests are welcome. Please test thoroughly on your hardware before submitting changes.

## Disclaimer

This utility directly controls hardware fan speeds. While it has been tested on the ASUS Zenbook UX3405MA.311, use it at your own risk. Monitor your system temperatures when first using this utility to ensure proper operation.
