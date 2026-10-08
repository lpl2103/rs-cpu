//! Automated downloader, extractor, and Windows PATH configurator for `LibreHardwareMonitor`.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

/// Current progress status of the `LibreHardwareMonitor` installer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallerStatus {
    /// Idle, no action taken.
    Idle,
    /// Downloading official zip from GitHub releases.
    Downloading,
    /// Extracting files to destination folder.
    Extracting,
    /// Configuring headless background server mode.
    Configuring,
    /// Registering install directory in Windows PATH.
    AddingToPath,
    /// Installation successfully completed.
    Success(String),
    /// An error occurred during installation.
    Error(String),
}

/// Thread-safe manager for `LibreHardwareMonitor` installation.
#[derive(Clone)]
pub struct LhmInstaller {
    status: Arc<Mutex<InstallerStatus>>,
    is_running: Arc<AtomicBool>,
    has_notified: Arc<AtomicBool>,
}

impl Default for LhmInstaller {
    fn default() -> Self {
        Self::new()
    }
}

impl LhmInstaller {
    /// Creates a new installer manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            status: Arc::new(Mutex::new(InstallerStatus::Idle)),
            is_running: Arc::new(AtomicBool::new(false)),
            has_notified: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Returns the current installer status.
    #[must_use]
    pub fn get_status(&self) -> InstallerStatus {
        self.status.lock().map_or(InstallerStatus::Idle, |s| s.clone())
    }

    /// Checks if an installation process is currently active.
    #[must_use]
    pub fn is_busy(&self) -> bool {
        self.is_running.load(Ordering::Relaxed)
    }

    /// Checks if a successful installation just finished and needs to be consumed.
    #[must_use]
    pub fn check_and_clear_success(&self) -> bool {
        if let Ok(lock) = self.status.lock() {
            if matches!(*lock, InstallerStatus::Success(_)) && !self.has_notified.swap(true, Ordering::SeqCst) {
                return true;
            }
        }
        false
    }

    /// Triggers asynchronous download, extraction, and PATH configuration.
    pub fn start_installation(&self) {
        if self.is_running.swap(true, Ordering::SeqCst) {
            return;
        }

        self.has_notified.store(false, Ordering::SeqCst);
        let status_clone = Arc::clone(&self.status);
        let is_running_clone = Arc::clone(&self.is_running);

        thread::Builder::new()
            .name("lhm-installer".to_string())
            .spawn(move || {
                let result = run_installer_pipeline(&status_clone);
                if let Err(e) = result {
                    if let Ok(mut lock) = status_clone.write_or_poison() {
                        *lock = InstallerStatus::Error(e);
                    }
                }
                is_running_clone.store(false, Ordering::SeqCst);
            })
            .ok();
    }
}

trait LockHelper<T> {
    fn write_or_poison(&self) -> Result<std::sync::MutexGuard<'_, T>, String>;
}

impl<T> LockHelper<T> for Mutex<T> {
    fn write_or_poison(&self) -> Result<std::sync::MutexGuard<'_, T>, String> {
        self.lock().map_err(|e| e.to_string())
    }
}

#[cfg(target_os = "windows")]
fn run_installer_pipeline(status: &Arc<Mutex<InstallerStatus>>) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let target_dir = determine_install_dir()?;
    let temp_zip = std::env::temp_dir().join("LibreHardwareMonitor_Setup.zip");

    // 1. Download
    if let Ok(mut lock) = status.write_or_poison() {
        *lock = InstallerStatus::Downloading;
    }

    let download_url = "https://github.com/LibreHardwareMonitor/LibreHardwareMonitor/releases/latest/download/LibreHardwareMonitor.zip";

    let curl_status = std::process::Command::new("curl.exe")
        .args(["-sL", "-o", temp_zip.to_str().unwrap_or("setup.zip"), download_url])
        .creation_flags(CREATE_NO_WINDOW)
        .status();

    let download_ok = curl_status.is_ok_and(|s| s.success());

    if !download_ok || !temp_zip.exists() || fs::metadata(&temp_zip).map_or(0, |m| m.len()) < 100_000 {
        // Fallback to PowerShell Invoke-WebRequest
        let ps_cmd = format!(
            "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; Invoke-WebRequest -Uri '{}' -OutFile '{}' -UseBasicParsing",
            download_url,
            temp_zip.display()
        );
        let ps_status = std::process::Command::new("powershell.exe")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &ps_cmd])
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .map_err(|e| format!("Falha ao invocar PowerShell para download: {e}"))?;

        if !ps_status.success() || !temp_zip.exists() {
            return Err("Falha ao baixar LibreHardwareMonitor do GitHub. Verifique a conexão com a internet.".to_string());
        }
    }

    // 2. Extract
    if let Ok(mut lock) = status.write_or_poison() {
        *lock = InstallerStatus::Extracting;
    }

    if !target_dir.exists() {
        fs::create_dir_all(&target_dir).map_err(|e| format!("Não foi possível criar pasta de destino: {e}"))?;
    }

    let tar_status = std::process::Command::new("tar.exe")
        .args(["-xf", temp_zip.to_str().unwrap_or("setup.zip"), "-C", target_dir.to_str().unwrap_or(".")])
        .creation_flags(CREATE_NO_WINDOW)
        .status();

    let tar_ok = tar_status.is_ok_and(|s| s.success());

    if !tar_ok {
        // Fallback to PowerShell Expand-Archive
        let ps_extract = format!(
            "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
            temp_zip.display(),
            target_dir.display()
        );
        let ps_status = std::process::Command::new("powershell.exe")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &ps_extract])
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .map_err(|e| format!("Falha ao extrair arquivo zip: {e}"))?;

        if !ps_status.success() {
            return Err("Falha ao descompactar os arquivos do LibreHardwareMonitor.".to_string());
        }
    }

    // Clean up temporary zip file
    let _ = fs::remove_file(&temp_zip);

    // 3. Configure headless web server settings
    if let Ok(mut lock) = status.write_or_poison() {
        *lock = InstallerStatus::Configuring;
    }
    configure_lhm_config(&target_dir);

    // 4. Add to Windows PATH
    if let Ok(mut lock) = status.write_or_poison() {
        *lock = InstallerStatus::AddingToPath;
    }
    add_directory_to_windows_path(&target_dir)?;

    // 5. Success
    let msg = format!("Instalado com sucesso em: {}", target_dir.display());
    if let Ok(mut lock) = status.write_or_poison() {
        *lock = InstallerStatus::Success(msg);
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn run_installer_pipeline(_status: &Arc<Mutex<InstallerStatus>>) -> Result<(), String> {
    Err("Instalação do LibreHardwareMonitor suportada apenas no Windows.".to_string())
}

#[cfg(target_os = "windows")]
fn determine_install_dir() -> Result<PathBuf, String> {
    // 1. Try Program Files (standard system location)
    if let Ok(prog_files) = std::env::var("ProgramFiles") {
        let p = PathBuf::from(prog_files).join("LibreHardwareMonitor");
        if fs::create_dir_all(&p).is_ok() {
            return Ok(p);
        }
    }

    // 2. Try %LOCALAPPDATA%\Programs\LibreHardwareMonitor (standard user location)
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        let p = PathBuf::from(local_app_data).join("Programs").join("LibreHardwareMonitor");
        if fs::create_dir_all(&p).is_ok() {
            return Ok(p);
        }
    }

    // 3. Fallback to alongside current executable
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(parent) = current_exe.parent() {
            let p = parent.join("LibreHardwareMonitor");
            if fs::create_dir_all(&p).is_ok() {
                return Ok(p);
            }
        }
    }

    Err("Não foi possível determinar diretório gravável para instalação.".to_string())
}

#[cfg(target_os = "windows")]
fn configure_lhm_config(target_dir: &Path) {
    let config_path = target_dir.join("LibreHardwareMonitor.config");

    let default_config = r#"<?xml version="1.0" encoding="utf-8"?>
<configuration>
  <appSettings>
    <add key="listenerIp" value="127.0.0.1" />
    <add key="listenerPort" value="8085" />
    <add key="authenticationEnabled" value="false" />
    <add key="runWebServerMenuItem" value="true" />
    <add key="startMinMenuItem" value="true" />
    <add key="minTrayMenuItem" value="true" />
    <add key="minCloseMenuItem" value="true" />
  </appSettings>
</configuration>
"#;

    if !config_path.exists() {
        let _ = fs::write(&config_path, default_config);
    } else if let Ok(existing) = fs::read_to_string(&config_path) {
        if !existing.contains("runWebServerMenuItem") {
            let modified = existing.replace(
                "</appSettings>",
                "  <add key=\"listenerIp\" value=\"127.0.0.1\" />\n    <add key=\"listenerPort\" value=\"8085\" />\n    <add key=\"runWebServerMenuItem\" value=\"true\" />\n    <add key=\"startMinMenuItem\" value=\"true\" />\n    <add key=\"minTrayMenuItem\" value=\"true\" />\n    <add key=\"minCloseMenuItem\" value=\"true\" />\n  </appSettings>",
            );
            let _ = fs::write(&config_path, modified);
        }
    }
}

#[cfg(target_os = "windows")]
fn add_directory_to_windows_path(target_dir: &Path) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let target_str = target_dir.to_str().ok_or("Caminho inválido")?;

    // Update current process PATH immediately
    if let Ok(current_path) = std::env::var("PATH") {
        if !current_path.to_lowercase().contains(&target_str.to_lowercase()) {
            std::env::set_var("PATH", format!("{current_path};{target_str}"));
        }
    }

    // Persist to Windows User PATH and broadcast WM_SETTINGCHANGE
    let ps_script = format!(
        r#"$p = [Environment]::GetEnvironmentVariable('Path', 'User');
if (-not $p.ToLower().Contains('{0}'.ToLower())) {{
    $new = if ($p.EndsWith(';')) {{ "$p{0}" }} else {{ "$p;{0}" }};
    [Environment]::SetEnvironmentVariable('Path', $new, 'User');
}}
try {{
    $mp = [Environment]::GetEnvironmentVariable('Path', 'Machine');
    if (-not $mp.ToLower().Contains('{0}'.ToLower())) {{
        $mnew = if ($mp.EndsWith(';')) {{ "$mp{0}" }} else {{ "$mp;{0}" }};
        [Environment]::SetEnvironmentVariable('Path', $mnew, 'Machine');
    }}
}} catch {{}}
"#,
        target_str.replace('\'', "''")
    );

    let status = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &ps_script])
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map_err(|e| format!("Falha ao registrar PATH no Windows: {e}"))?;

    if !status.success() {
        return Err("Falha ao registrar caminho no PATH do Windows.".to_string());
    }

    tracing::info!("Diretório {} adicionado ao PATH do Windows com sucesso.", target_str);
    Ok(())
}
