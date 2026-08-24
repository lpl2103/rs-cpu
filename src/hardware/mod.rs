//! Hardware telemetry engine and aggregation module.

pub mod cpu;
pub mod gpu;
pub mod memory;
pub mod motherboard;
pub mod storage;

pub use cpu::{CacheInfo, CpuInfo, CpuLiveMetrics};
pub use gpu::GpuInfo;
pub use memory::{MemoryInfo, MemoryLiveMetrics, SpdSlotInfo, TimingProfile};
pub use motherboard::MotherboardInfo;
pub use storage::{PartitionInfo, PhysicalDriveInfo, StorageInfo};

use serde::{Deserialize, Serialize};
use sysinfo::{Components, CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

/// Central hardware state combining all detected components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHardware {
    /// Processor specifications and live telemetry.
    pub cpu: CpuInfo,
    /// Motherboard and BIOS specifications.
    pub motherboard: MotherboardInfo,
    /// Memory configuration, SPD slots, and live usage.
    pub memory: MemoryInfo,
    /// Storage drives and volume maps (SSD-Z).
    pub storage: StorageInfo,
    /// Graphics cards installed (GPU-Z).
    pub gpus: Vec<GpuInfo>,
    /// Operating System details.
    pub os_name: String,
    /// OS Version and kernel.
    pub os_version: String,
    /// System uptime in seconds.
    pub uptime_secs: u64,
}

/// Active hardware engine managing background polling and data refreshes.
pub struct HardwareEngine {
    /// Low-level sysinfo monitor instance.
    pub system: System,
    /// Sensor components (temperatures, fans).
    pub components: Components,
    /// Current aggregated snapshot of hardware information.
    pub data: SystemHardware,
}

impl Default for HardwareEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl HardwareEngine {
    /// Initializes hardware inspection, reading CPUID, SMBIOS, Storage, and platform telemetry.
    #[must_use]
    pub fn new() -> Self {
        let mut system = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );

        system.refresh_cpu_all();
        system.refresh_memory();

        let components = Components::new_with_refreshed_list();

        let cpu = CpuInfo::detect(&system, &components);
        let motherboard = MotherboardInfo::detect();
        let memory = MemoryInfo::detect(&system);
        let storage = StorageInfo::detect();
        let gpus = GpuInfo::detect();

        let os_name = System::name().unwrap_or_else(|| "Windows / Linux".to_string());
        let os_version = System::os_version().unwrap_or_else(|| "10 / 11".to_string());
        let uptime_secs = System::uptime();

        let data = SystemHardware {
            cpu,
            motherboard,
            memory,
            storage,
            gpus,
            os_name,
            os_version,
            uptime_secs,
        };

        Self { system, components, data }
    }

    /// Performs a non-blocking live refresh of dynamic metrics (CPU clocks, load, temp, fan, memory usage, GPU).
    pub fn refresh_live_metrics(&mut self) {
        self.system.refresh_cpu_all();
        self.system.refresh_memory();
        self.components.refresh(true);

        self.data.cpu.update_live_metrics(&self.system, &self.components);
        self.data.memory.update_live_metrics(&self.system);
        for gpu in &mut self.data.gpus {
            gpu.update_live_metrics();
        }
        self.data.uptime_secs = System::uptime();
    }
}
