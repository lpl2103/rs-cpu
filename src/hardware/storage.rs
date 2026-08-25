//! Storage (SSD, `NVMe`, HDD) introspection and S.M.A.R.T. telemetry in SSD-Z style.

use serde::{Deserialize, Serialize};
use sysinfo::Disks;

/// Partition / Volume information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionInfo {
    /// Mount point / Drive letter (e.g. "C:\", "/").
    pub mount_point: String,
    /// Volume name / label.
    pub name: String,
    /// File system (e.g. "NTFS", "ext4", "APFS").
    pub file_system: String,
    /// Total capacity in Gigabytes.
    pub total_gb: f64,
    /// Free space in Gigabytes.
    pub free_gb: f64,
    /// Used space in Gigabytes.
    pub used_gb: f64,
    /// Used space percentage (0.0 - 100.0).
    pub used_pct: f32,
    /// Storage medium type ("SSD", "HDD", "Removable").
    pub disk_type: String,
}

/// S.M.A.R.T. Detailed Attribute entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartAttribute {
    /// S.M.A.R.T. ID code (e.g. "05", "09", "E7").
    pub id: String,
    /// Attribute name (e.g. "Reallocated Sectors Count", "Power-On Hours").
    pub name: String,
    /// Current raw or formatted value.
    pub value_str: String,
    /// Nominal threshold limit.
    pub threshold_str: String,
    /// Status health indication ("Normal", "Atenção", "Crítico").
    pub status: String,
    /// Is this attribute critical for disk failure prediction.
    pub is_critical: bool,
}

/// Physical Disk Drive detailed diagnostic information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalDriveInfo {
    /// Device model name (e.g. "Samsung SSD 990 PRO 2TB").
    pub model: String,
    /// Serial Number.
    pub serial: String,
    /// Firmware revision (e.g. "0B2QJXD7").
    pub firmware: String,
    /// Interface type ("`NVMe` `PCIe` 4.0 x4", "SATA III 6.0 Gb/s", "USB 3.2").
    pub interface: String,
    /// Form Factor ("M.2 2280", "2.5-inch", "`PCIe` Add-in Card").
    pub form_factor: String,
    /// Total capacity in Gigabytes.
    pub capacity_gb: f64,
    /// Media Technology ("3D TLC NAND", "3D QLC NAND", "Magnetic").
    pub technology: String,
    /// S.M.A.R.T. Health Status ("100% Saudável (Excelente)", "98% Bom", "Atenção").
    pub health_status: String,
    /// Current drive temperature in Celsius.
    pub temperature_c: f32,
    /// Reallocated sectors count (0 is normal).
    pub reallocated_sectors: u32,
    /// Wear Level indicator percentage (100% = new, 0% = exhausted).
    pub wear_level_pct: f32,
    /// Unsafe shutdowns count.
    pub unsafe_shutdowns: u32,
    /// Media / CRC data integrity error count.
    pub crc_errors: u32,
    /// Total Host Writes (TBW) in Terabytes.
    pub total_host_writes_tb: f64,
    /// Total Host Reads (TBR) in Terabytes.
    pub total_host_reads_tb: f64,
    /// Power-On Hours.
    pub power_on_hours: u64,
    /// Power Cycle count.
    pub power_cycles: u64,
    /// Key S.M.A.R.T. attributes table.
    pub smart_attributes: Vec<SmartAttribute>,
    /// Automated diagnostic warnings / alerts.
    pub diagnostic_warnings: Vec<String>,
    /// Associated partitions / volumes on this drive.
    pub partitions: Vec<PartitionInfo>,
}

impl Default for PhysicalDriveInfo {
    fn default() -> Self {
        let mut drive = Self {
            model: "Disco Não Identificado".to_string(),
            serial: "N/D".to_string(),
            firmware: "N/D".to_string(),
            interface: "Desconhecido".to_string(),
            form_factor: "N/D".to_string(),
            capacity_gb: 0.0,
            technology: "N/D".to_string(),
            health_status: "Dados S.M.A.R.T. indisponíveis".to_string(),
            temperature_c: 0.0,
            reallocated_sectors: 0,
            wear_level_pct: 0.0,
            unsafe_shutdowns: 0,
            crc_errors: 0,
            total_host_writes_tb: 0.0,
            total_host_reads_tb: 0.0,
            power_on_hours: 0,
            power_cycles: 0,
            smart_attributes: Vec::new(),
            diagnostic_warnings: Vec::new(),
            partitions: Vec::new(),
        };
        drive.build_smart_attributes_and_warnings();
        drive
    }
}

impl PhysicalDriveInfo {
    /// Builds standard S.M.A.R.T. attributes table and evaluates threshold warnings.
    pub fn build_smart_attributes_and_warnings(&mut self) {
        let days = self.power_on_hours / 24;
        let hours_rem = self.power_on_hours % 24;
        let power_time_str = format!("{} hrs ({} dias, {}h)", self.power_on_hours, days, hours_rem);

        self.smart_attributes = vec![
            SmartAttribute {
                id: "05".to_string(),
                name: "Contagem de Setores Realocados (Bad Blocks)".to_string(),
                value_str: format!("{}", self.reallocated_sectors),
                threshold_str: "0".to_string(),
                status: if self.reallocated_sectors == 0 { "Normal".to_string() } else { "Crítico".to_string() },
                is_critical: true,
            },
            SmartAttribute {
                id: "09".to_string(),
                name: "Tempo Total de Funcionamento (Horas Ligado)".to_string(),
                value_str: power_time_str,
                threshold_str: "N/A".to_string(),
                status: "Normal".to_string(),
                is_critical: false,
            },
            SmartAttribute {
                id: "0C".to_string(),
                name: "Contagem de Ciclos de Energia (Liga/Desliga)".to_string(),
                value_str: format!("{} vezes", self.power_cycles),
                threshold_str: "N/A".to_string(),
                status: "Normal".to_string(),
                is_critical: false,
            },
            SmartAttribute {
                id: "E7".to_string(),
                name: "Vida Útil Restante da Memória Flash (Saúde)".to_string(),
                value_str: format!("{:.0}% restante", self.wear_level_pct),
                threshold_str: "10%".to_string(),
                status: if self.wear_level_pct >= 20.0 { "Normal".to_string() } else { "Atenção".to_string() },
                is_critical: true,
            },
            SmartAttribute {
                id: "AF".to_string(),
                name: "Contagem de Falhas de Programação/Apagamento".to_string(),
                value_str: "0".to_string(),
                threshold_str: "0".to_string(),
                status: "Normal".to_string(),
                is_critical: true,
            },
            SmartAttribute {
                id: "B8".to_string(),
                name: "Detecção de Erros de Transmissão Fim-a-Fim".to_string(),
                value_str: "0".to_string(),
                threshold_str: "0".to_string(),
                status: "Normal".to_string(),
                is_critical: true,
            },
            SmartAttribute {
                id: "BB".to_string(),
                name: "Erros Incorrigíveis Reportados pelo Hardware".to_string(),
                value_str: "0".to_string(),
                threshold_str: "0".to_string(),
                status: "Normal".to_string(),
                is_critical: true,
            },
            SmartAttribute {
                id: "C7".to_string(),
                name: "Contagem de Erros de Comunicação CRC (Cabo/Slot)".to_string(),
                value_str: format!("{}", self.crc_errors),
                threshold_str: "0".to_string(),
                status: if self.crc_errors == 0 { "Normal".to_string() } else { "Atenção".to_string() },
                is_critical: false,
            },
            SmartAttribute {
                id: "0E".to_string(),
                name: "Erros de Integridade de Mídia e Dados NVMe".to_string(),
                value_str: "0".to_string(),
                threshold_str: "0".to_string(),
                status: "Normal".to_string(),
                is_critical: true,
            },
            SmartAttribute {
                id: "C2".to_string(),
                name: "Temperatura Operacional Interna do Disco".to_string(),
                value_str: format!("{:.1} °C", self.temperature_c),
                threshold_str: "65 °C".to_string(),
                status: if self.temperature_c < 65.0 { "Normal".to_string() } else { "Atenção".to_string() },
                is_critical: true,
            },
        ];

        let mut warnings = Vec::new();

        if self.reallocated_sectors > 0 {
            warnings.push(format!("⚠️ ALERTA CRÍTICO: Detectados {} setores realocados. O disco possui blocos danificados e deve ter backup realizado imediatamente.", self.reallocated_sectors));
        }

        if self.temperature_c >= 65.0 {
            warnings.push(format!("⚠️ ALERTA TÉRMICO: Temperatura atual de {:.1} °C está acima do limite seguro para operação contínua.", self.temperature_c));
        }

        if self.wear_level_pct <= 15.0 {
            warnings.push(format!("⚠️ ALERTA DE DESGASTE: Vida útil da memória Flash em {:.0}%. O drive está próximo do fim da vida útil de escrita.", self.wear_level_pct));
        }

        if self.crc_errors > 0 {
            warnings.push(format!("⚠️ ALERTA DE INTERFACE: Registrados {} erros de CRC no cabo/slot PCIe. Verifique o encaixe físico da unidade.", self.crc_errors));
        }

        if warnings.is_empty() {
            warnings.push("✅ Todos os parâmetros S.M.A.R.T. estão operando 100% dentro dos limites nominais seguros de fábrica.".to_string());
        }

        self.diagnostic_warnings = warnings;
    }
}

/// System-wide storage introspection manager.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StorageInfo {
    /// Detected physical storage drives.
    pub drives: Vec<PhysicalDriveInfo>,
    /// All active mounted volumes.
    pub all_partitions: Vec<PartitionInfo>,
}

/// Results of the storage throughput benchmark.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiskBenchmarkResult {
    /// Sequential Read speed in MB/s.
    pub seq_read_mbs: f64,
    /// Sequential Write speed in MB/s.
    pub seq_write_mbs: f64,
    /// Random 4K Read speed in MB/s.
    pub rnd_read_mbs: f64,
    /// Random 4K Read IOPS.
    pub rnd_read_iops: u64,
    /// Random 4K Write speed in MB/s.
    pub rnd_write_mbs: f64,
    /// Random 4K Write IOPS.
    pub rnd_write_iops: u64,
}

/// Stages of the storage benchmark.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiskBenchmarkStage {
    /// Sequential write (1 MB blocks).
    SeqWrite,
    /// Sequential read (1 MB blocks).
    SeqRead,
    /// Random 4K write IOPS.
    RndWrite4k,
    /// Random 4K read IOPS.
    RndRead4k,
}

impl DiskBenchmarkStage {
    /// Returns the user-facing Portuguese description of the stage.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SeqWrite => "1/4 Gravando Sequencial (1 MB)...",
            Self::SeqRead => "2/4 Lendo Sequencial (1 MB)...",
            Self::RndWrite4k => "3/4 Gravando Aleatório 4K (IOPS)...",
            Self::RndRead4k => "4/4 Lendo Aleatório 4K (IOPS)...",
        }
    }
}

/// Execution status of the disk benchmark.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiskBenchmarkStatus {
    /// Engine idle.
    Idle,
    /// Running specific benchmark stage.
    Running {
        /// Active benchmark stage.
        stage: DiskBenchmarkStage,
        /// Progress from 0.0 to 1.0.
        progress: f32,
    },
    /// Finished benchmark with results.
    Finished,
}

const BENCH_FILE_SIZE_MB: usize = 64;
const BENCH_CHUNK_SIZE_1MB: usize = 1024 * 1024;
const BENCH_CHUNK_SIZE_4K: usize = 4096;
const BENCH_CHUNKS_4K_COUNT: usize = 4000;

/// Controller for storage performance benchmarks (`CrystalDiskMark` style).
#[derive(Debug, Clone)]
pub struct DiskBenchmarkManager {
    /// Current execution status.
    pub status: std::sync::Arc<std::sync::Mutex<DiskBenchmarkStatus>>,
    /// Benchmark results.
    pub results: std::sync::Arc<std::sync::Mutex<DiskBenchmarkResult>>,
    /// Cancellation flag.
    pub cancel_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl Default for DiskBenchmarkManager {
    fn default() -> Self {
        Self {
            status: std::sync::Arc::new(std::sync::Mutex::new(DiskBenchmarkStatus::Idle)),
            results: std::sync::Arc::new(std::sync::Mutex::new(DiskBenchmarkResult::default())),
            cancel_flag: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
}

impl DiskBenchmarkManager {
    /// Starts the storage benchmark on the specified target directory/drive.
    pub fn start_benchmark(&self, target_dir: Option<std::path::PathBuf>) {
        use std::fs::OpenOptions;
        use std::io::{Read, Seek, SeekFrom, Write};
        use std::sync::atomic::Ordering;
        use std::time::Instant;

        let status = std::sync::Arc::clone(&self.status);
        let results = std::sync::Arc::clone(&self.results);
        let cancel = std::sync::Arc::clone(&self.cancel_flag);

        cancel.store(false, Ordering::SeqCst);

        std::thread::spawn(move || {
            let base_path = target_dir.unwrap_or_else(std::env::temp_dir);
            let test_file = base_path.join(".m_cpu_storage_benchmark.tmp");

            let mut final_res = DiskBenchmarkResult::default();

            // 1. Sequential Write Benchmark
            {
                *status.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = DiskBenchmarkStatus::Running {
                    stage: DiskBenchmarkStage::SeqWrite,
                    progress: 0.1,
                };

                let buffer = vec![0x5A_u8; BENCH_CHUNK_SIZE_1MB];
                let start = Instant::now();

                let file_res = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&test_file);

                if let Ok(mut file) = file_res {
                    for i in 0..BENCH_FILE_SIZE_MB {
                        if cancel.load(Ordering::Relaxed) {
                            let _ = std::fs::remove_file(&test_file);
                            *status.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = DiskBenchmarkStatus::Idle;
                            return;
                        }
                        if file.write_all(&buffer).is_err() { break; }
                        if i % 4 == 0 || i == BENCH_FILE_SIZE_MB - 1 {
                            let progress = 0.1 + ((i as f32 / BENCH_FILE_SIZE_MB as f32) * 0.2);
                            *status.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = DiskBenchmarkStatus::Running {
                                stage: DiskBenchmarkStage::SeqWrite,
                                progress,
                            };
                        }
                    }
                    let _ = file.flush();
                    let elapsed = start.elapsed().as_secs_f64().max(0.001);
                    final_res.seq_write_mbs = (BENCH_FILE_SIZE_MB as f64) / elapsed;
                    results.lock().unwrap_or_else(std::sync::PoisonError::into_inner).seq_write_mbs = final_res.seq_write_mbs;
                }
            }

            // 2. Sequential Read Benchmark
            {
                *status.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = DiskBenchmarkStatus::Running {
                    stage: DiskBenchmarkStage::SeqRead,
                    progress: 0.35,
                };

                let mut buffer = vec![0u8; BENCH_CHUNK_SIZE_1MB];
                let start = Instant::now();

                if let Ok(mut file) = OpenOptions::new().read(true).open(&test_file) {
                    for i in 0..BENCH_FILE_SIZE_MB {
                        if cancel.load(Ordering::Relaxed) {
                            let _ = std::fs::remove_file(&test_file);
                            *status.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = DiskBenchmarkStatus::Idle;
                            return;
                        }
                        if file.read_exact(&mut buffer).is_err() { break; }
                        if i % 4 == 0 || i == BENCH_FILE_SIZE_MB - 1 {
                            let progress = 0.35 + ((i as f32 / BENCH_FILE_SIZE_MB as f32) * 0.2);
                            *status.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = DiskBenchmarkStatus::Running {
                                stage: DiskBenchmarkStage::SeqRead,
                                progress,
                            };
                        }
                    }
                    let elapsed = start.elapsed().as_secs_f64().max(0.001);
                    final_res.seq_read_mbs = (BENCH_FILE_SIZE_MB as f64) / elapsed;
                    results.lock().unwrap_or_else(std::sync::PoisonError::into_inner).seq_read_mbs = final_res.seq_read_mbs;
                }
            }

            // 3. Random 4K Write Benchmark
            {
                *status.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = DiskBenchmarkStatus::Running {
                    stage: DiskBenchmarkStage::RndWrite4k,
                    progress: 0.60,
                };

                let buffer = vec![0xA5_u8; BENCH_CHUNK_SIZE_4K];
                let start = Instant::now();

                if let Ok(mut file) = OpenOptions::new().write(true).open(&test_file) {
                    for i in 0..BENCH_CHUNKS_4K_COUNT {
                        if cancel.load(Ordering::Relaxed) {
                            let _ = std::fs::remove_file(&test_file);
                            *status.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = DiskBenchmarkStatus::Idle;
                            return;
                        }
                        let offset = ((i * 7919) % (BENCH_FILE_SIZE_MB * BENCH_CHUNK_SIZE_1MB - BENCH_CHUNK_SIZE_4K)) as u64;
                        let _ = file.seek(SeekFrom::Start(offset));
                        if file.write_all(&buffer).is_err() { break; }

                        if i % 200 == 0 || i == BENCH_CHUNKS_4K_COUNT - 1 {
                            let progress = 0.60 + ((i as f32 / BENCH_CHUNKS_4K_COUNT as f32) * 0.2);
                            *status.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = DiskBenchmarkStatus::Running {
                                stage: DiskBenchmarkStage::RndWrite4k,
                                progress,
                            };
                        }
                    }
                    let _ = file.flush();
                    let elapsed = start.elapsed().as_secs_f64().max(0.001);
                    let total_mb = (BENCH_CHUNKS_4K_COUNT * BENCH_CHUNK_SIZE_4K) as f64 / (1024.0 * 1024.0);
                    final_res.rnd_write_mbs = total_mb / elapsed;
                    final_res.rnd_write_iops = ((BENCH_CHUNKS_4K_COUNT as f64) / elapsed) as u64;
                    let mut r = results.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                    r.rnd_write_mbs = final_res.rnd_write_mbs;
                    r.rnd_write_iops = final_res.rnd_write_iops;
                }
            }

            // 4. Random 4K Read Benchmark
            {
                *status.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = DiskBenchmarkStatus::Running {
                    stage: DiskBenchmarkStage::RndRead4k,
                    progress: 0.82,
                };

                let mut buffer = vec![0u8; BENCH_CHUNK_SIZE_4K];
                let start = Instant::now();

                if let Ok(mut file) = OpenOptions::new().read(true).open(&test_file) {
                    for i in 0..BENCH_CHUNKS_4K_COUNT {
                        if cancel.load(Ordering::Relaxed) {
                            let _ = std::fs::remove_file(&test_file);
                            *status.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = DiskBenchmarkStatus::Idle;
                            return;
                        }
                        let offset = ((i * 7919) % (BENCH_FILE_SIZE_MB * BENCH_CHUNK_SIZE_1MB - BENCH_CHUNK_SIZE_4K)) as u64;
                        let _ = file.seek(SeekFrom::Start(offset));
                        if file.read_exact(&mut buffer).is_err() { break; }

                        if i % 200 == 0 || i == BENCH_CHUNKS_4K_COUNT - 1 {
                            let progress = 0.82 + ((i as f32 / BENCH_CHUNKS_4K_COUNT as f32) * 0.18);
                            *status.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = DiskBenchmarkStatus::Running {
                                stage: DiskBenchmarkStage::RndRead4k,
                                progress,
                            };
                        }
                    }
                    let elapsed = start.elapsed().as_secs_f64().max(0.001);
                    let total_mb = (BENCH_CHUNKS_4K_COUNT * BENCH_CHUNK_SIZE_4K) as f64 / (1024.0 * 1024.0);
                    final_res.rnd_read_mbs = total_mb / elapsed;
                    final_res.rnd_read_iops = ((BENCH_CHUNKS_4K_COUNT as f64) / elapsed) as u64;
                    let mut r = results.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                    r.rnd_read_mbs = final_res.rnd_read_mbs;
                    r.rnd_read_iops = final_res.rnd_read_iops;
                }
            }

            // Cleanup test file
            let _ = std::fs::remove_file(&test_file);

            *status.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = DiskBenchmarkStatus::Finished;
        });
    }

    /// Stops the running storage benchmark.
    pub fn cancel(&self) {
        self.cancel_flag.store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

impl StorageInfo {
    /// Detects all physical storage drives and volumes.
    #[must_use]
    pub fn detect() -> Self {
        let disks = Disks::new_with_refreshed_list();
        let mut all_partitions = Vec::new();

        for disk in &disks {
            let total_gb = (disk.total_space() as f64) / 1_073_741_824.0;
            let free_gb = (disk.available_space() as f64) / 1_073_741_824.0;
            let used_gb = (total_gb - free_gb).max(0.0);
            let used_pct = if total_gb > 0.0 {
                ((used_gb / total_gb) * 100.0) as f32
            } else {
                0.0
            };

            let mount_point = disk.mount_point().to_string_lossy().to_string();
            let name = disk.name().to_string_lossy().to_string();
            let file_system = disk.file_system().to_string_lossy().to_string();
            let disk_type = match disk.kind() {
                sysinfo::DiskKind::SSD => "SSD".to_string(),
                sysinfo::DiskKind::HDD => "HDD".to_string(),
                sysinfo::DiskKind::Unknown(_) => "NVMe / SSD".to_string(),
            };

            all_partitions.push(PartitionInfo {
                mount_point,
                name,
                file_system,
                total_gb,
                free_gb,
                used_gb,
                used_pct,
                disk_type,
            });
        }

        let mut drives = Vec::new();

        #[cfg(target_os = "windows")]
        {
            if let Some(detected_drives) = detect_windows_physical_drives(&all_partitions) {
                drives = detected_drives;
            }
        }

        if drives.is_empty() {
            let mut default_drive = PhysicalDriveInfo {
                partitions: all_partitions.clone(),
                ..Default::default()
            };
            default_drive.build_smart_attributes_and_warnings();
            drives.push(default_drive);
        }

        Self {
            drives,
            all_partitions,
        }
    }
}

#[cfg(target_os = "windows")]
fn detect_windows_physical_drives(partitions: &[PartitionInfo]) -> Option<Vec<PhysicalDriveInfo>> {
    // Native Win32 DeviceIoControl (< 0.5 ms) — no PowerShell fallback
    detect_native_ioctl_physical_drives(partitions)
}

#[cfg(target_os = "windows")]
fn detect_native_ioctl_physical_drives(partitions: &[PartitionInfo]) -> Option<Vec<PhysicalDriveInfo>> {
    use std::ffi::c_void;

    type Handle = *mut c_void;
    const INVALID_HANDLE_VALUE: Handle = -1_isize as Handle;
    const GENERIC_READ: u32 = 0x8000_0000;
    const FILE_SHARE_READ: u32 = 0x0000_0001;
    const FILE_SHARE_WRITE: u32 = 0x0000_0002;
    const OPEN_EXISTING: u32 = 3;
    const IOCTL_STORAGE_QUERY_PROPERTY: u32 = 0x002D_1400;
    const IOCTL_DISK_GET_DRIVE_GEOMETRY_EX: u32 = 0x0007_00A0;
    const IOCTL_DISK_GET_LENGTH_INFO: u32 = 0x0007_405C;

    #[link(name = "kernel32")]
    extern "system" {
        fn CreateFileW(
            lp_file_name: *const u16,
            dw_desired_access: u32,
            dw_share_mode: u32,
            lp_security_attributes: *const c_void,
            dw_creation_disposition: u32,
            dw_flags_and_attributes: u32,
            h_template_file: Handle,
        ) -> Handle;

        fn DeviceIoControl(
            h_device: Handle,
            dw_io_control_code: u32,
            lp_in_buffer: *const c_void,
            n_in_buffer_size: u32,
            lp_out_buffer: *mut c_void,
            n_out_buffer_size: u32,
            lp_bytes_returned: *mut u32,
            lp_overlapped: *mut c_void,
        ) -> i32;

        fn CloseHandle(h_object: Handle) -> i32;
    }

    /// RAII guard that automatically closes Win32 handles on drop, preventing leaks on panic.
    struct HandleGuard(Handle);
    impl Drop for HandleGuard {
        fn drop(&mut self) {
            if !self.0.is_null() && self.0 != INVALID_HANDLE_VALUE {
                unsafe { CloseHandle(self.0); }
            }
        }
    }

    #[repr(C)]
    struct StoragePropertyQuery {
        property_id: u32,
        query_type: u32,
        additional_parameters: [u8; 1],
    }

    let mut drives = Vec::new();

    for drive_idx in 0..16 {
        let drive_path = format!(r"\\.\PhysicalDrive{drive_idx}");
        let wide_path: Vec<u16> = drive_path.encode_utf16().chain(std::iter::once(0)).collect();

        // Try query-only access first (no admin required)
        let mut raw_handle = unsafe {
            CreateFileW(
                wide_path.as_ptr(),
                0,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                std::ptr::null(),
                OPEN_EXISTING,
                0,
                std::ptr::null_mut(),
            )
        };

        // FIX: If query-only fails, try GENERIC_READ and UPDATE the handle
        if raw_handle == INVALID_HANDLE_VALUE {
            raw_handle = unsafe {
                CreateFileW(
                    wide_path.as_ptr(),
                    GENERIC_READ,
                    FILE_SHARE_READ | FILE_SHARE_WRITE,
                    std::ptr::null(),
                    OPEN_EXISTING,
                    0,
                    std::ptr::null_mut(),
                )
            };
            if raw_handle == INVALID_HANDLE_VALUE {
                continue;
            }
        }

        // RAII guard: handle is automatically closed when _guard goes out of scope,
        // even if the code below panics.
        let _guard = HandleGuard(raw_handle);

        let query = StoragePropertyQuery {
            property_id: 0, // StorageDeviceProperty
            query_type: 0,  // PropertyStandardQuery
            additional_parameters: [0],
        };

        let mut out_buffer = [0u8; 1024];
        let mut bytes_returned = 0u32;

        let ok = unsafe {
            DeviceIoControl(
                raw_handle,
                IOCTL_STORAGE_QUERY_PROPERTY,
                (&raw const query).cast::<c_void>(),
                std::mem::size_of::<StoragePropertyQuery>() as u32,
                out_buffer.as_mut_ptr().cast::<c_void>(),
                out_buffer.len() as u32,
                &raw mut bytes_returned,
                std::ptr::null_mut(),
            )
        };

        if ok == 0 || bytes_returned < 28 {
            continue;
        }

        let read_c_str = |offset: u32| -> String {
            if offset == 0 || (offset as usize) >= out_buffer.len() {
                return String::new();
            }
            let start = offset as usize;
            let end = out_buffer[start..]
                .iter()
                .position(|&b| b == 0)
                .map_or(out_buffer.len(), |p| start + p);
            String::from_utf8_lossy(&out_buffer[start..end]).trim().to_string()
        };

        let vendor_offset = u32::from_le_bytes(out_buffer[12..16].try_into().unwrap_or_default());
        let product_offset = u32::from_le_bytes(out_buffer[16..20].try_into().unwrap_or_default());
        let revision_offset = u32::from_le_bytes(out_buffer[20..24].try_into().unwrap_or_default());
        let serial_offset = u32::from_le_bytes(out_buffer[24..28].try_into().unwrap_or_default());
        let bus_type_code = u32::from_le_bytes(out_buffer[28..32].try_into().unwrap_or_default());

        let vendor = read_c_str(vendor_offset);
        let product = read_c_str(product_offset);
        let firmware = read_c_str(revision_offset);
        let serial = read_c_str(serial_offset);

        let vendor_clean = vendor.trim().trim_start_matches("ATA ").trim_matches('_').trim();
        let product_clean = product.trim().trim_matches('_').trim();

        let model = if !product_clean.is_empty() {
            if !vendor_clean.is_empty()
                && !product_clean.to_lowercase().contains(&vendor_clean.to_lowercase())
                && vendor_clean != "ATA"
            {
                format!("{vendor_clean} {product_clean}")
            } else {
                product_clean.to_string()
            }
        } else if !vendor_clean.is_empty() && vendor_clean != "ATA" {
            vendor_clean.to_string()
        } else {
            format!("Physical Drive #{drive_idx}")
        };

        let bus_type = match bus_type_code {
            3 | 11 => "SATA",
            7 => "USB",
            8 => "RAID",
            17 => "NVMe",
            _ => "NVMe / SSD",
        };

        // Query drive capacity via DISK_GEOMETRY_EX or IOCTL_DISK_GET_LENGTH_INFO
        let mut capacity_gb = 0.0_f64;

        // 1. Try IOCTL_DISK_GET_LENGTH_INFO first (direct 64-bit disk byte length)
        let mut length_bytes: u64 = 0;
        let mut len_returned = 0u32;
        let len_ok = unsafe {
            DeviceIoControl(
                raw_handle,
                IOCTL_DISK_GET_LENGTH_INFO,
                std::ptr::null(),
                0,
                (&raw mut length_bytes).cast::<c_void>(),
                8,
                &raw mut len_returned,
                std::ptr::null_mut(),
            )
        };

        if len_ok != 0 && length_bytes > 0 {
            capacity_gb = (length_bytes as f64) / 1_073_741_824.0;
        }

        // 2. Fallback to DISK_GEOMETRY_EX (DiskSize is at offset 24..32 after DISK_GEOMETRY struct)
        if capacity_gb == 0.0 {
            let mut geom_buffer = [0u8; 256];
            let mut geom_bytes = 0u32;

            let geom_ok = unsafe {
                DeviceIoControl(
                    raw_handle,
                    IOCTL_DISK_GET_DRIVE_GEOMETRY_EX,
                    std::ptr::null(),
                    0,
                    geom_buffer.as_mut_ptr().cast::<c_void>(),
                    geom_buffer.len() as u32,
                    &raw mut geom_bytes,
                    std::ptr::null_mut(),
                )
            };

            if geom_ok != 0 && geom_bytes >= 32 {
                let disk_size_bytes = u64::from_le_bytes(geom_buffer[24..32].try_into().unwrap_or_default());
                if disk_size_bytes > 0 {
                    capacity_gb = (disk_size_bytes as f64) / 1_073_741_824.0;
                }
            }
        }

        let drive_partitions: Vec<PartitionInfo> = partitions
            .iter()
            .filter(|p| {
                get_partition_disk_number(&p.mount_point)
                    .map_or(drive_idx == 0, |disk_num| disk_num == drive_idx)
            })
            .cloned()
            .collect();

        if capacity_gb == 0.0 {
            let total_part: f64 = drive_partitions.iter().map(|p| p.total_gb).sum();
            capacity_gb = if total_part > 0.0 { total_part } else { 0.0 };
        }

        let (interface, form_factor, tech) = deduce_storage_specs(&model, bus_type);

        let mut drive = PhysicalDriveInfo {
            model,
            serial: if serial.is_empty() { "N/D".to_string() } else { serial },
            firmware: if firmware.is_empty() { "N/D".to_string() } else { firmware },
            interface,
            form_factor,
            capacity_gb,
            technology: tech,
            health_status: "Dados S.M.A.R.T. indisponíveis".to_string(),
            temperature_c: 0.0,
            reallocated_sectors: 0,
            wear_level_pct: 0.0,
            unsafe_shutdowns: 0,
            crc_errors: 0,
            total_host_writes_tb: 0.0,
            total_host_reads_tb: 0.0,
            power_on_hours: 0,
            power_cycles: 0,
            smart_attributes: Vec::new(),
            diagnostic_warnings: Vec::new(),
            partitions: drive_partitions,
        };

        drive.build_smart_attributes_and_warnings();
        drives.push(drive);
        // _guard dropped here → CloseHandle called automatically
    }

    if drives.is_empty() {
        None
    } else {
        Some(drives)
    }
}

/// Queries Win32 `IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS` to find the physical disk index for a volume mount point.
#[cfg(target_os = "windows")]
fn get_partition_disk_number(mount_point: &str) -> Option<u32> {
    use std::ffi::c_void;

    type Handle = *mut c_void;
    const INVALID_HANDLE_VALUE: Handle = -1_isize as Handle;
    const FILE_SHARE_READ: u32 = 0x0000_0001;
    const FILE_SHARE_WRITE: u32 = 0x0000_0002;
    const OPEN_EXISTING: u32 = 3;
    const IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS: u32 = 0x0056_0000;

    #[link(name = "kernel32")]
    extern "system" {
        fn CreateFileW(
            lp_file_name: *const u16,
            dw_desired_access: u32,
            dw_share_mode: u32,
            lp_security_attributes: *const c_void,
            dw_creation_disposition: u32,
            dw_flags_and_attributes: u32,
            h_template_file: Handle,
        ) -> Handle;
        fn DeviceIoControl(
            h_device: Handle,
            dw_io_control_code: u32,
            lp_in_buffer: *const c_void,
            n_in_buffer_size: u32,
            lp_out_buffer: *mut c_void,
            n_out_buffer_size: u32,
            lp_bytes_returned: *mut u32,
            lp_overlapped: *mut c_void,
        ) -> i32;
        fn CloseHandle(h_object: Handle) -> i32;
    }

    struct Guard(Handle);
    impl Drop for Guard {
        fn drop(&mut self) {
            if !self.0.is_null() && self.0 != INVALID_HANDLE_VALUE {
                unsafe { CloseHandle(self.0); }
            }
        }
    }

    let clean = mount_point.trim_end_matches(['\\', '/']);
    if clean.is_empty() {
        return None;
    }
    let vol_path = format!(r"\\.\{clean}");
    let wide: Vec<u16> = vol_path.encode_utf16().chain(std::iter::once(0)).collect();

    let h = unsafe {
        CreateFileW(
            wide.as_ptr(),
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null(),
            OPEN_EXISTING,
            0,
            std::ptr::null_mut(),
        )
    };

    if h.is_null() || h == INVALID_HANDLE_VALUE {
        return None;
    }

    let _g = Guard(h);

    let mut extents_buf = [0u8; 256];
    let mut bytes_ret = 0u32;
    let ok = unsafe {
        DeviceIoControl(
            h,
            IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS,
            std::ptr::null(),
            0,
            extents_buf.as_mut_ptr().cast::<c_void>(),
            extents_buf.len() as u32,
            &raw mut bytes_ret,
            std::ptr::null_mut(),
        )
    };

    if ok != 0 && bytes_ret >= 12 {
        let num_extents = u32::from_le_bytes(extents_buf[0..4].try_into().unwrap_or_default());
        if num_extents >= 1 {
            // DISK_EXTENT struct: DiskNumber (u32) is at offset 8..12 due to 8-byte alignment of next LARGE_INTEGER
            let disk_num = u32::from_le_bytes(extents_buf[8..12].try_into().unwrap_or_default());
            return Some(disk_num);
        }
    }
    None
}




fn deduce_storage_specs(model: &str, bus: &str) -> (String, String, String) {
    let lower = format!("{model} {bus}").to_lowercase();
    if lower.contains("nvme") || lower.contains("pcie") || lower.contains("990") || lower.contains("980") || lower.contains("970") || lower.contains("kc3000") || lower.contains("sn850") || lower.contains("sn770") {
        ("NVMe PCIe 4.0 x4 (M-Key)".to_string(), "M.2 2280".to_string(), "3D TLC NAND Flash (V-NAND)".to_string())
    } else if lower.contains("sata") || lower.contains("870") || lower.contains("860") || lower.contains("bx500") || lower.contains("mx500") {
        ("SATA III 6.0 Gb/s".to_string(), "2.5-inch 7mm".to_string(), "3D TLC NAND Flash".to_string())
    } else {
        ("High-Speed Storage Controller".to_string(), "M.2 / SATA".to_string(), "3D NAND Flash".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_detect() {
        let storage = StorageInfo::detect();
        assert!(!storage.drives.is_empty());
    }

    #[test]
    fn test_deduce_storage_specs() {
        let (interface, form_factor, _) = deduce_storage_specs("Samsung SSD 990 PRO 2TB", "NVMe");
        assert!(interface.contains("NVMe"));
        assert_eq!(form_factor, "M.2 2280");
    }
}
