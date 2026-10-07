//! WMI and `LibreHardwareMonitor` / `OpenHardwareMonitor` real-time sensor integration.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// Snapshot of real hardware sensors retrieved via WMI.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WmiHardwareSensors {
    /// Indicates if a live Ring-0 sensor provider is currently active.
    pub is_available: bool,
    /// Provider name: "`LibreHardwareMonitor`", "`OpenHardwareMonitor`", "ACPI `ThermalZone`", or "Estimado".
    pub provider_name: String,
    /// Real CPU Temperature in °C.
    pub cpu_temp_c: Option<f32>,
    /// Real CPU Cooler Fan speed in RPM.
    pub cpu_fan_rpm: Option<u32>,
    /// Additional chassis/case fans in RPM (name, rpm).
    pub case_fans_rpm: Vec<(String, u32)>,
    /// Real CPU Core Voltage (Vcore).
    pub vcore: Option<f32>,
    /// Real +12V Rail voltage.
    pub voltage_12v: Option<f32>,
    /// Real +5V Rail voltage.
    pub voltage_5v: Option<f32>,
    /// Real +3.3V Rail voltage.
    pub voltage_3v3: Option<f32>,
    /// Real CPU Package Power in Watts.
    pub cpu_power_w: Option<f32>,
    /// Real GPU Board Power in Watts.
    pub gpu_power_w: Option<f32>,
    /// Real GPU Core Temperature in °C.
    pub gpu_temp_c: Option<f32>,
    /// Real GPU Fan Speed in RPM.
    pub gpu_fan_rpm: Option<u32>,
}

#[cfg(target_os = "windows")]
#[derive(Deserialize, Debug)]
#[serde(rename = "Sensor")]
#[serde(rename_all = "PascalCase")]
struct LhmSensor {
    name: String,
    sensor_type: String,
    value: f32,
    identifier: String,
}

#[cfg(target_os = "windows")]
#[derive(Deserialize, Debug)]
#[serde(rename = "MSAcpi_ThermalZoneTemperature")]
#[serde(rename_all = "PascalCase")]
struct AcpiThermalZone {
    current_temperature: u32,
}

/// Thread-safe manager for querying WMI sensor providers in the background.
pub struct WmiSensorEngine {
    data: Arc<RwLock<WmiHardwareSensors>>,
    stop_signal: Arc<AtomicBool>,
}

impl Default for WmiSensorEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl WmiSensorEngine {
    /// Starts background polling for WMI hardware sensors.
    #[must_use]
    pub fn new() -> Self {
        let data = Arc::new(RwLock::new(WmiHardwareSensors::default()));
        let stop_signal = Arc::new(AtomicBool::new(false));

        #[cfg(target_os = "windows")]
        {
            let data_clone = Arc::clone(&data);
            let stop_clone = Arc::clone(&stop_signal);

            std::thread::Builder::new()
                .name("wmi-sensor-poller".to_string())
                .spawn(move || {
                    while !stop_clone.load(Ordering::Relaxed) {
                        let sample = query_wmi_hardware_sensors();
                        if let Ok(mut lock) = data_clone.write() {
                            *lock = sample;
                        }
                        std::thread::sleep(Duration::from_millis(1000));
                    }
                })
                .ok();
        }

        Self { data, stop_signal }
    }

    /// Retrieves the most recent WMI sensor snapshot without blocking.
    #[must_use]
    pub fn get_latest(&self) -> WmiHardwareSensors {
        self.data.read().as_deref().cloned().unwrap_or_default()
    }
}

impl Drop for WmiSensorEngine {
    fn drop(&mut self) {
        self.stop_signal.store(true, Ordering::Relaxed);
    }
}

#[cfg(target_os = "windows")]
fn query_wmi_hardware_sensors() -> WmiHardwareSensors {
    use wmi::WMIConnection;

    // 1. Try LibreHardwareMonitor WMI namespace first (primary modern standard)
    if let Ok(con) = WMIConnection::with_namespace_path(r"ROOT\LibreHardwareMonitor") {
        if let Ok(sensors) = con.raw_query::<LhmSensor>("SELECT Name, SensorType, Value, Identifier FROM Sensor") {
            if !sensors.is_empty() {
                return parse_lhm_sensors(sensors, "LibreHardwareMonitor");
            }
        }
    }

    // 2. Try OpenHardwareMonitor WMI namespace (legacy standard)
    if let Ok(con) = WMIConnection::with_namespace_path(r"ROOT\OpenHardwareMonitor") {
        if let Ok(sensors) = con.raw_query::<LhmSensor>("SELECT Name, SensorType, Value, Identifier FROM Sensor") {
            if !sensors.is_empty() {
                return parse_lhm_sensors(sensors, "OpenHardwareMonitor");
            }
        }
    }

    // 3. Try standard Windows ACPI ThermalZone in root\WMI
    if let Ok(con) = WMIConnection::with_namespace_path(r"ROOT\WMI") {
        if let Ok(zones) = con.raw_query::<AcpiThermalZone>("SELECT CurrentTemperature FROM MSAcpi_ThermalZoneTemperature") {
            for zone in zones {
                if zone.current_temperature > 2732 {
                    let c = (zone.current_temperature as f32 - 2732.0) / 10.0;
                    if (15.0..=115.0).contains(&c) {
                        return WmiHardwareSensors {
                            is_available: true,
                            provider_name: "ACPI ThermalZone".to_string(),
                            cpu_temp_c: Some(c),
                            ..Default::default()
                        };
                    }
                }
            }
        }
    }

    WmiHardwareSensors::default()
}

#[cfg(target_os = "windows")]
fn parse_lhm_sensors(sensors: Vec<LhmSensor>, provider_name: &str) -> WmiHardwareSensors {
    let mut result = WmiHardwareSensors {
        is_available: true,
        provider_name: provider_name.to_string(),
        ..Default::default()
    };

    let mut cpu_temp_candidates: Vec<(f32, u8)> = Vec::new();

    for s in sensors {
        let name_lower = s.name.to_lowercase();
        let id_lower = s.identifier.to_lowercase();

        match s.sensor_type.as_str() {
            "Temperature" => {
                if id_lower.contains("cpu") {
                    // Score temperature importance: Package > Total > Core Max > Cores
                    let priority = if name_lower.contains("package") {
                        4
                    } else if name_lower.contains("tctl") || name_lower.contains("tdie") {
                        3
                    } else if name_lower.contains("core max") || name_lower.contains("total") {
                        2
                    } else {
                        1
                    };
                    if (15.0..=120.0).contains(&s.value) {
                        cpu_temp_candidates.push((s.value, priority));
                    }
                } else if id_lower.contains("gpu") && (name_lower.contains("core") || result.gpu_temp_c.is_none()) {
                    result.gpu_temp_c = Some(s.value);
                }
            }
            "Fan" => {
                if (id_lower.contains("cpu") || name_lower.contains("cpu")) && result.cpu_fan_rpm.is_none() {
                    result.cpu_fan_rpm = Some(s.value as u32);
                } else if id_lower.contains("gpu") {
                    result.gpu_fan_rpm = Some(s.value as u32);
                } else if result.cpu_fan_rpm.is_none() && (name_lower.contains("fan #1") || name_lower.contains("fan 1")) {
                    result.cpu_fan_rpm = Some(s.value as u32);
                } else {
                    result.case_fans_rpm.push((s.name, s.value as u32));
                }
            }
            "Voltage" => {
                if name_lower.contains("+12v") || name_lower.contains("12v") {
                    if (10.5..=13.5).contains(&s.value) {
                        result.voltage_12v = Some(s.value);
                    }
                } else if name_lower.contains("+5v") || name_lower.contains("5v") {
                    if (4.2..=5.8).contains(&s.value) {
                        result.voltage_5v = Some(s.value);
                    }
                } else if name_lower.contains("+3.3v") || name_lower.contains("3.3v") || name_lower.contains("3v3") {
                    if (2.8..=3.8).contains(&s.value) {
                        result.voltage_3v3 = Some(s.value);
                    }
                } else if (name_lower.contains("vcore") || name_lower.contains("cpu vdd")) && (0.5..=1.7).contains(&s.value) {
                    result.vcore = Some(s.value);
                }
            }
            "Power" => {
                if id_lower.contains("cpu") && (name_lower.contains("package") || name_lower.contains("total") || result.cpu_power_w.is_none()) {
                    result.cpu_power_w = Some(s.value);
                } else if id_lower.contains("gpu") && (name_lower.contains("package") || name_lower.contains("board") || result.gpu_power_w.is_none()) {
                    result.gpu_power_w = Some(s.value);
                }
            }
            _ => {}
        }
    }

    // Pick the highest priority CPU temperature reading
    if let Some(&(best_temp, _)) = cpu_temp_candidates.iter().max_by_key(|&&(_, p)| p) {
        result.cpu_temp_c = Some(best_temp);
    }

    result
}

#[cfg(not(target_os = "windows"))]
pub struct WmiSensorEngine;

#[cfg(not(target_os = "windows"))]
impl Default for WmiSensorEngine {
    fn default() -> Self {
        Self
    }
}

#[cfg(not(target_os = "windows"))]
impl WmiSensorEngine {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
    #[must_use]
    pub fn get_latest(&self) -> WmiHardwareSensors {
        WmiHardwareSensors::default()
    }
}
