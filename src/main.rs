use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::{fs, time::Duration};

const POLL_PERIOD_SECS: u64 = 2;
/// When this threshold is surpassed, the CPU fan will be at
/// max RPM
const CPU_TEMP_THRESHOLD: u64 = 90;

#[derive(PartialEq, Debug, Clone)]
enum CpuFanState {
    PwmOff,
    PwmOn,
}

impl CpuFanState {
    pub fn from_str(val: &str) -> Option<Self> {
        match val.trim() {
            "0" => Some(Self::PwmOff),
            "2" => Some(Self::PwmOn),
            _ => None,
        }
    }
}

fn get_acpitz_thermal_zone() -> io::Result<PathBuf> {
    let path = Path::new("/sys/class/thermal");
    if !path.is_dir() {
        return Err(io::Error::new(ErrorKind::NotFound, "No thermal Zones"));
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        let path_str = path.to_str().expect("Invalid string");
        if path_str.contains("thermal_zone") {
            let zone = format!("{}{}", path_str, "/type");
            let zone_type = fs::read_to_string(zone)?;
            if zone_type.contains("acpitz") {
                return Ok(path);
            }
        }
    }
    Err(io::Error::new(ErrorKind::NotFound, "No thermal Zones"))
}

fn get_thermal_zone_temp(path: &PathBuf) -> io::Result<u64> {
    let path_str = path.to_str().expect("Invalid string");
    let zone_temp_path = format!("{}{}", path_str, "/temp");
    let zone_temp_str = fs::read_to_string(zone_temp_path)?;
    let temp: u64 = zone_temp_str
        .trim()
        .parse::<u64>()
        .expect("Invalid temperature value");
    Ok(temp / 1000)
}

fn get_cpu_fan_pwm_path() -> io::Result<PathBuf> {
    let path = Path::new("/sys/devices/platform/asus-nb-wmi/hwmon/");
    if !path.is_dir() {
        return Err(io::Error::new(ErrorKind::NotFound, "hwmon not found"));
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        let path_str = path.to_str().expect("Invalid string");

        let fan_label_path = format!("{}{}", path_str, "/fan1_label");
        let label = fs::read_to_string(&fan_label_path)?;
        if label.contains("cpu_fan") {
            let pwm_enable_path = PathBuf::from(format!("{}{}", path_str, "/pwm1_enable"));
            if !pwm_enable_path.exists() {
                continue;
            }
            return Ok(pwm_enable_path);
        }
    }
    Err(io::Error::new(ErrorKind::NotFound, "hwmon not found"))
}

fn get_cpu_fan_pwm_state(path: &PathBuf) -> io::Result<CpuFanState> {
    let pwm_path = path.to_str().expect("Invalid String");
    let pwm_state = fs::read_to_string(&pwm_path)?;

    if let Some(state) = CpuFanState::from_str(&pwm_state) {
        return Ok(state);
    }
    Err(io::Error::new(
        ErrorKind::NotFound,
        "CPU Fan PWM state not found",
    ))
}

fn cpu_fan_set_pwm_off(path: &PathBuf) -> io::Result<()> {
    fs::write(&path, "0")?;
    Ok(())
}

fn cpu_fan_set_auto(path: &PathBuf) -> io::Result<()> {
    fs::write(&path, "2")?;
    Ok(())
}

fn main() {
    println!("Starting Zenbook Fanctl Service");

    let pwm_path = get_cpu_fan_pwm_path().unwrap_or_else(|_| {
        println!("CPU fan control not found");
        std::process::exit(1);
    });

    let acpitz_path = get_acpitz_thermal_zone().unwrap_or_else(|_| {
        println!("Failed to find acpitz thermal zone");
        std::process::exit(1);
    });

    println!("acpitz thermal zone: {acpitz_path:?}");

    let mut current_state = get_cpu_fan_pwm_state(&pwm_path).expect("Failed to get cpu fan state");
    let mut prev_state = current_state.clone();
    loop {
        let Ok(temp) = get_thermal_zone_temp(&acpitz_path) else {
            std::thread::sleep(Duration::from_secs(POLL_PERIOD_SECS));
            continue;
        };

        if temp >= CPU_TEMP_THRESHOLD {
            current_state = CpuFanState::PwmOff;
        } else {
            current_state = CpuFanState::PwmOn;
        }

        if current_state != prev_state {
            match current_state {
                CpuFanState::PwmOn => {
                    if let Err(e) = cpu_fan_set_auto(&pwm_path) {
                        println!("Failed to set cpu fan control to auto: {e:?}");
                    }
                    println!("Toggling PWM on");
                }
                CpuFanState::PwmOff => {
                    if let Err(e) = cpu_fan_set_pwm_off(&pwm_path) {
                        println!("Failed to set cpu fan pwm off: {e:?}");
                    }
                    println!("Toggling PWM off (Max RPM) : Package: {temp}°C");
                }
            }
        }
        prev_state = current_state;
        std::thread::sleep(Duration::from_secs(POLL_PERIOD_SECS));
    }
}
