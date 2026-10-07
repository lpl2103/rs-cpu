//! WMI and `LibreHardwareMonitor` / `OpenHardwareMonitor` real-time sensor integration.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

/// Snapshot of real hardware sensors retrieved via WMI or Sidecar.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WmiHardwareSensors {
    /// Indicates if a live Ring-0 sensor provider is currently active.
    pub is_available: bool,
    /// Provider name: "`LibreHardwareMonitor Sidecar`", "`LibreHardwareMonitor`", "`OpenHardwareMonitor`", "ACPI `ThermalZone`", or "Estimado".
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

#[derive(Deserialize, Debug, Default)]
struct JsonSensorNode {
    #[serde(default, rename = "Text")]
    text: String,
    #[serde(default, rename = "Value")]
    value: String,
    #[serde(default, rename = "ImageURL")]
    image_url: String,
    #[serde(default, rename = "Children")]
    children: Vec<Self>,
}

/// Manages the background invisible sidecar process for `LibreHardwareMonitor`.
pub struct SidecarProcess {
    #[cfg(target_os = "windows")]
    child: Option<std::process::Child>,
}

impl SidecarProcess {
    #[cfg(target_os = "windows")]
    fn try_spawn() -> Option<Self> {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;

        let mut candidate_paths = Vec::new();

        if let Ok(current) = std::env::current_exe() {
            if let Some(parent) = current.parent() {
                candidate_paths.push(parent.join("LibreHardwareMonitor.NET.10").join("LibreHardwareMonitor.exe"));
                candidate_paths.push(parent.join("LibreHardwareMonitor").join("LibreHardwareMonitor.exe"));
                if let Some(grandparent) = parent.parent() {
                    candidate_paths.push(grandparent.join("LibreHardwareMonitor.NET.10").join("LibreHardwareMonitor.exe"));
                    candidate_paths.push(grandparent.join("LibreHardwareMonitor").join("LibreHardwareMonitor.exe"));
                }
            }
        }

        candidate_paths.push(std::path::PathBuf::from(r".\LibreHardwareMonitor.NET.10\LibreHardwareMonitor.exe"));
        candidate_paths.push(std::path::PathBuf::from(r"..\LibreHardwareMonitor.NET.10\LibreHardwareMonitor.exe"));
        candidate_paths.push(std::path::PathBuf::from(r".\LibreHardwareMonitor\LibreHardwareMonitor.exe"));

        let mut target_exe = None;
        for path in &candidate_paths {
            if path.exists() {
                target_exe = Some(path.clone());
                break;
            }
        }

        let exe_path = target_exe?;
        let exe_dir = exe_path.parent().unwrap_or_else(|| std::path::Path::new("."));

        let mut cmd = std::process::Command::new(&exe_path);
        cmd.current_dir(exe_dir);
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.stdout(std::process::Stdio::null());
        cmd.stderr(std::process::Stdio::null());
        cmd.stdin(std::process::Stdio::null());

        match cmd.spawn() {
            Ok(child) => {
                tracing::info!("Sidecar LibreHardwareMonitor iniciado de forma invisivel (PID {})", child.id());
                Some(Self { child: Some(child) })
            }
            Err(e) => {
                tracing::warn!("Falha ao iniciar sidecar LibreHardwareMonitor: {}", e);
                None
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn try_spawn() -> Option<Self> {
        None
    }
}

impl Drop for SidecarProcess {
    fn drop(&mut self) {
        #[cfg(target_os = "windows")]
        if let Some(mut child) = self.child.take() {
            tracing::info!("Finalizando processo sidecar LibreHardwareMonitor (PID {})...", child.id());
            let _ = child.kill();
        }
    }
}

/// Thread-safe manager for querying WMI sensor providers and background sidecar.
pub struct WmiSensorEngine {
    data: Arc<RwLock<WmiHardwareSensors>>,
    stop_signal: Arc<AtomicBool>,
    _sidecar: Arc<Mutex<Option<SidecarProcess>>>,
}

impl Default for WmiSensorEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl WmiSensorEngine {
    /// Starts background polling for hardware sensors and manages sidecar daemon.
    #[must_use]
    pub fn new() -> Self {
        let data = Arc::new(RwLock::new(WmiHardwareSensors::default()));
        let stop_signal = Arc::new(AtomicBool::new(false));
        let sidecar = Arc::new(Mutex::new(SidecarProcess::try_spawn()));

        #[cfg(target_os = "windows")]
        {
            let data_clone = Arc::clone(&data);
            let stop_clone = Arc::clone(&stop_signal);

            std::thread::Builder::new()
                .name("wmi-sensor-poller".to_string())
                .spawn(move || {
                    while !stop_clone.load(Ordering::Relaxed) {
                        let sample = query_all_sensor_sources();
                        if let Ok(mut lock) = data_clone.write() {
                            *lock = sample;
                        }
                        std::thread::sleep(Duration::from_millis(1000));
                    }
                })
                .ok();
        }

        Self {
            data,
            stop_signal,
            _sidecar: sidecar,
        }
    }

    /// Retrieves the most recent sensor snapshot without blocking.
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
fn query_all_sensor_sources() -> WmiHardwareSensors {
    // 1. Try local HTTP JSON API from LibreHardwareMonitor sidecar
    if let Some(json_data) = fetch_local_json() {
        if let Ok(root) = serde_json::from_str::<JsonSensorNode>(&json_data) {
            let mut result = WmiHardwareSensors {
                is_available: true,
                provider_name: "LibreHardwareMonitor Sidecar".to_string(),
                ..Default::default()
            };
            parse_json_node(&root, "", &mut result);
            if result.cpu_temp_c.is_some() || result.cpu_fan_rpm.is_some() || result.voltage_12v.is_some() {
                return result;
            }
        }
    }

    // 2. Try WMI provider (LibreHardwareMonitor / OpenHardwareMonitor / ACPI)
    query_wmi_hardware_sensors()
}

#[cfg(target_os = "windows")]
fn fetch_local_json() -> Option<String> {
    use std::io::{Read, Write};
    use std::net::{SocketAddr, TcpStream};

    let addr = SocketAddr::from(([127, 0, 0, 1], 8085));
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_millis(300)).ok()?;
    stream.set_read_timeout(Some(Duration::from_millis(500))).ok()?;

    let request = b"GET /data.json HTTP/1.1\r\nHost: 127.0.0.1:8085\r\nConnection: close\r\n\r\n";
    stream.write_all(request).ok()?;

    let mut response = String::new();
    stream.read_to_string(&mut response).ok()?;

    let body = response.split("\r\n\r\n").nth(1)?;
    Some(body.to_string())
}

#[cfg(target_os = "windows")]
fn parse_json_node(node: &JsonSensorNode, path: &str, result: &mut WmiHardwareSensors) {
    let lower_text = node.text.to_lowercase();
    let current_path = if path.is_empty() {
        lower_text.clone()
    } else {
        format!("{path}/{lower_text}")
    };

    let is_temp = node.image_url.contains("temperature") || node.value.contains("°C");
    let is_fan = node.image_url.contains("fan") || node.value.contains("RPM");
    let is_voltage = node.image_url.contains("voltage") || (node.value.ends_with('V') && !node.value.ends_with("eV"));
    let is_power = node.image_url.contains("power") || node.value.ends_with('W');

    let val_num = node
        .value
        .split_whitespace()
        .next()
        .and_then(|s| s.replace(',', ".").parse::<f32>().ok());

    if let Some(val) = val_num {
        if is_temp {
            if current_path.contains("cpu") {
                if lower_text.contains("package")
                    || lower_text.contains("core max")
                    || lower_text.contains("total")
                    || result.cpu_temp_c.is_none()
                {
                    result.cpu_temp_c = Some(val);
                }
            } else if current_path.contains("gpu")
                && (lower_text.contains("core") || result.gpu_temp_c.is_none())
            {
                result.gpu_temp_c = Some(val);
            }
        } else if is_fan {
            if current_path.contains("cpu") || lower_text.contains("cpu") || lower_text.contains("fan #1") {
                if result.cpu_fan_rpm.is_none() {
                    result.cpu_fan_rpm = Some(val as u32);
                }
            } else if current_path.contains("gpu") {
                result.gpu_fan_rpm = Some(val as u32);
            } else {
                result.case_fans_rpm.push((node.text.clone(), val as u32));
            }
        } else if is_voltage {
            if lower_text.contains("+12v") || lower_text.contains("12v") {
                result.voltage_12v = Some(val);
            } else if lower_text.contains("+5v") || lower_text.contains("5v") {
                result.voltage_5v = Some(val);
            } else if lower_text.contains("+3.3v") || lower_text.contains("3.3v") || lower_text.contains("3v3") {
                result.voltage_3v3 = Some(val);
            } else if (lower_text.contains("vcore") || lower_text.contains("cpu vdd")) && (0.5..=1.7).contains(&val) {
                result.vcore = Some(val);
            }
        } else if is_power {
            if current_path.contains("cpu")
                && (lower_text.contains("package") || lower_text.contains("total") || result.cpu_power_w.is_none())
            {
                result.cpu_power_w = Some(val);
            } else if current_path.contains("gpu")
                && (lower_text.contains("package") || lower_text.contains("board") || result.gpu_power_w.is_none())
            {
                result.gpu_power_w = Some(val);
            }
        }
    }

    for child in &node.children {
        parse_json_node(child, &current_path, result);
    }
}

#[cfg(target_os = "windows")]
fn query_wmi_hardware_sensors() -> WmiHardwareSensors {
    use wmi::WMIConnection;

    // 1. Try LibreHardwareMonitor WMI namespace (legacy/standard)
    if let Ok(con) = WMIConnection::with_namespace_path(r"ROOT\LibreHardwareMonitor") {
        if let Ok(sensors) = con.raw_query::<LhmSensor>("SELECT Name, SensorType, Value, Identifier FROM Sensor") {
            if !sensors.is_empty() {
                return parse_lhm_sensors(sensors, "LibreHardwareMonitor");
            }
        }
    }

    // 2. Try OpenHardwareMonitor WMI namespace
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

    if let Some(&(best_temp, _)) = cpu_temp_candidates.iter().max_by_key(|&&(_, p)| p) {
        result.cpu_temp_c = Some(best_temp);
    }

    result
}

#[cfg(not(target_os = "windows"))]
fn query_all_sensor_sources() -> WmiHardwareSensors {
    WmiHardwareSensors::default()
}
