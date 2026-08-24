//! Hardware telemetry engine and aggregation module.

pub mod cpu;
pub mod gpu;
pub mod memory;
pub mod motherboard;

pub use cpu::{CacheInfo, CpuInfo, CpuLiveMetrics};
pub use gpu::GpuInfo;
pub use memory::{MemoryInfo, MemoryLiveMetrics, SpdSlotInfo, TimingProfile};
pub use motherboard::MotherboardInfo;

use serde::{Deserialize, Serialize};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

/// Central hardware state combining all detected components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHardware {
    /// Processor specifications and live telemetry.
    pub cpu: CpuInfo,
    /// Motherboard and BIOS specifications.
    pub motherboard: MotherboardInfo,
    /// Memory configuration, SPD slots, and live usage.
    pub memory: MemoryInfo,
    /// Graphics cards installed.
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
    /// Current aggregated snapshot of hardware information.
    pub data: SystemHardware,
}

impl Default for HardwareEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl HardwareEngine {
    /// Initializes hardware inspection, reading CPUID, SMBIOS, and platform telemetry.
    #[must_use]
    pub fn new() -> Self {
        let mut system = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );

        // First sleep/refresh cycle to establish CPU usage baseline
        system.refresh_cpu_all();
        system.refresh_memory();

        let cpu = CpuInfo::detect(&system);
        let motherboard = MotherboardInfo::detect();
        let memory = MemoryInfo::detect(&system);
        let gpus = GpuInfo::detect();

        let os_name = System::name().unwrap_or_else(|| "Windows / Linux".to_string());
        let os_version = System::os_version().unwrap_or_else(|| "10 / 11".to_string());
        let uptime_secs = System::uptime();

        let data = SystemHardware {
            cpu,
            motherboard,
            memory,
            gpus,
            os_name,
            os_version,
            uptime_secs,
        };

        Self { system, data }
    }

    /// Performs a non-blocking live refresh of dynamic metrics (CPU clocks, load, memory usage).
    pub fn refresh_live_metrics(&mut self) {
        self.system.refresh_cpu_all();
        self.system.refresh_memory();

        self.data.cpu.update_live_metrics(&self.system);
        self.data.memory.update_live_metrics(&self.system);
        self.data.uptime_secs = System::uptime();
    }
}
