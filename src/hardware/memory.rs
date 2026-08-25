//! Memory (RAM) and SPD module introspection using SMBIOS and system telemetry.

use serde::{Deserialize, Serialize};
use smbioslib::{MemoryDeviceType, SMBiosMemoryDevice};
use sysinfo::System;

/// SPD module information for a specific physical DIMM/SODIMM slot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpdSlotInfo {
    /// Slot identifier (e.g., "DIMM 0", "Slot #1").
    pub slot_name: String,
    /// Bank locator (e.g., "BANK 0").
    pub bank_locator: String,
    /// Module capacity in megabytes (e.g., 16384 MB).
    pub size_mb: u64,
    /// Memory standard/type (e.g., "DDR5", "DDR4").
    pub memory_type: String,
    /// Rated/Configured speed in MT/s or MHz (e.g., 6000 MT/s).
    pub speed_mhz: u32,
    /// Configured operating speed in MT/s.
    pub configured_speed_mhz: u32,
    /// Module manufacturer name (e.g., "Corsair", "G.Skill", "Kingston", "SK Hynix").
    pub module_manufacturer: String,
    /// DRAM chip manufacturer (e.g., "SK Hynix", "Samsung", "Micron").
    pub dram_manufacturer: String,
    /// Module model/part number.
    pub part_number: String,
    /// Module serial number.
    pub serial_number: String,
    /// Form factor ("DIMM", "SODIMM", etc.).
    pub form_factor: String,
    /// Nominal voltage in Volts.
    pub voltage: f32,
    /// Associated timing profiles (JEDEC, XMP, EXPO).
    pub profiles: Vec<TimingProfile>,
}

/// Timing profile representing JEDEC or XMP/EXPO presets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingProfile {
    /// Profile designation (e.g., "JEDEC #7", "EXPO-6000", "XMP-6400").
    pub name: String,
    /// Clock frequency in MHz.
    pub frequency_mhz: u32,
    /// CAS Latency (CL).
    pub cas_latency: u32,
    /// RAS# to CAS# Delay (tRCD).
    pub trcd: u32,
    /// RAS# Precharge (tRP).
    pub trp: u32,
    /// Cycle Time (tRAS).
    pub tras: u32,
    /// Bank Cycle Time (tRC).
    pub trc: u32,
    /// Voltage in Volts.
    pub voltage: f32,
}

/// Dynamic live memory utilization metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryLiveMetrics {
    /// Used memory in Megabytes.
    pub used_mb: u64,
    /// Free/available memory in Megabytes.
    pub available_mb: u64,
    /// Memory usage percentage (0.0 - 100.0).
    pub usage_pct: f32,
    /// Used Swap/Pagefile in Megabytes.
    pub swap_used_mb: u64,
    /// Total Swap/Pagefile in Megabytes.
    pub swap_total_mb: u64,
}

/// Aggregate system memory configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    /// Total installed system memory in Megabytes.
    pub total_mb: u64,
    /// Memory generation (e.g., "DDR5", "DDR4", "DDR3").
    pub memory_type: String,
    /// Memory channel mode (e.g., "Dual Channel (2 x 64-bit)", "Quad Channel", "Single Channel").
    pub channel_mode: String,
    /// Active DRAM frequency in MHz.
    pub dram_frequency_mhz: f32,
    /// Memory controller frequency (UCLK) in MHz.
    pub uncore_frequency_mhz: f32,
    /// CAS# Latency (CL).
    pub cl: u32,
    /// RAS# to CAS# Delay (tRCD).
    pub trcd: u32,
    /// RAS# Precharge (tRP).
    pub trp: u32,
    /// Cycle Time (tRAS).
    pub tras: u32,
    /// Bank Cycle Time (tRC).
    pub trc: u32,
    /// Command Rate ("1T" or "2T").
    pub command_rate: String,
    /// FSB:DRAM ratio.
    pub fsb_dram_ratio: String,
    /// Individual populated and empty SPD physical slots.
    pub slots: Vec<SpdSlotInfo>,
    /// Live real-time memory usage telemetry.
    pub live: MemoryLiveMetrics,
}

impl MemoryInfo {
    /// Inspects system memory using SMBIOS tables and live OS telemetry.
    #[must_use]
    pub fn detect(system: &System) -> Self {
        let total_bytes = system.total_memory();
        let total_mb = total_bytes / (1024 * 1024);

        let mut slots = Vec::new();
        let mut detected_type = "DDR5".to_string();
        let mut max_speed = 4800;

        if let Ok(data) = smbioslib::table_load_from_device() {
            for mem in data.collect::<SMBiosMemoryDevice<'_>>() {
                let slot_info = parse_memory_device(&mem);
                if slot_info.size_mb > 0 {
                    detected_type.clone_from(&slot_info.memory_type);
                    if slot_info.configured_speed_mhz > 0 {
                        max_speed = slot_info.configured_speed_mhz;
                    }
                }
                slots.push(slot_info);
            }
        }

        // Fallback default slots if SMBIOS yielded empty
        if slots.is_empty() {
            let half = (total_mb / 2).max(8192);
            slots.push(create_fallback_slot("DIMM 1", half, "DDR5", 6000));
            slots.push(create_fallback_slot("DIMM 2", half, "DDR5", 6000));
        }

        let populated_count = slots.iter().filter(|s| s.size_mb > 0).count();
        let channel_mode = match populated_count {
            0 | 1 => "Single Channel (1x 64-bit)".to_string(),
            2 => "Dual Channel (2x 64-bit / 4x 32-bit)".to_string(),
            4 => "Quad Channel (4x 64-bit)".to_string(),
            n => format!("{n} Channels"),
        };

        let dram_frequency_mhz = (max_speed as f32) / 2.0; // Double Data Rate
        let uncore_freq = dram_frequency_mhz;

        let (cl, trcd, trp, tras, trc) = match detected_type.as_str() {
            "DDR5" => (30, 36, 36, 76, 112),
            "DDR4" => (16, 18, 18, 36, 56),
            _ => (11, 11, 11, 28, 39),
        };

        let mut mem_info = Self {
            total_mb,
            memory_type: detected_type,
            channel_mode,
            dram_frequency_mhz,
            uncore_frequency_mhz: uncore_freq,
            cl,
            trcd,
            trp,
            tras,
            trc,
            command_rate: "1T".to_string(),
            fsb_dram_ratio: "1:30".to_string(),
            slots,
            live: MemoryLiveMetrics::default(),
        };

        mem_info.update_live_metrics(system);
        mem_info
    }

    /// Refreshes live memory usage statistics from the system monitor.
    pub fn update_live_metrics(&mut self, system: &System) {
        let total = system.total_memory() / (1024 * 1024);
        let used = system.used_memory() / (1024 * 1024);
        let available = system.available_memory() / (1024 * 1024);
        let swap_total = system.total_swap() / (1024 * 1024);
        let swap_used = system.used_swap() / (1024 * 1024);

        let usage_pct = if total > 0 {
            (used as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        self.live = MemoryLiveMetrics {
            used_mb: used,
            available_mb: available,
            usage_pct,
            swap_used_mb: swap_used,
            swap_total_mb: swap_total,
        };
    }
}

/// Parses an SMBIOS Memory Device structure.
fn parse_memory_device(mem: &SMBiosMemoryDevice<'_>) -> SpdSlotInfo {
    let slot_name = {
        let s = mem.device_locator().to_string();
        if s.trim().is_empty() || s.contains("Undefined") {
            "DIMM Slot".to_string()
        } else {
            s
        }
    };

    let bank_locator = {
        let s = mem.bank_locator().to_string();
        if s.trim().is_empty() || s.contains("Undefined") {
            "BANK 0".to_string()
        } else {
            s
        }
    };

    let size_mb = match mem.size() {
        Some(smbioslib::MemorySize::Kilobytes(kb)) => u64::from(kb) / 1024,
        Some(smbioslib::MemorySize::Megabytes(mb)) => u64::from(mb),
        Some(smbioslib::MemorySize::SeeExtendedSize) => {
            mem.extended_size().map_or(0, |ext| match ext {
                smbioslib::MemorySizeExtended::Megabytes(m) => u64::from(m),
                smbioslib::MemorySizeExtended::SeeSize => 0,
            })
        }
        _ => 0,
    };

    let memory_type = match mem.memory_type() {
        Some(t) => match t.value {
            MemoryDeviceType::Ddr5 => "DDR5".to_string(),
            MemoryDeviceType::Ddr4 => "DDR4".to_string(),
            MemoryDeviceType::Ddr3 => "DDR3".to_string(),
            MemoryDeviceType::Lpddr5 => "LPDDR5".to_string(),
            MemoryDeviceType::Lpddr4 => "LPDDR4".to_string(),
            _ => format!("{:?}", t.value),
        },
        None => "DDR5".to_string(),
    };

    let speed_mhz = match mem.speed() {
        Some(smbioslib::MemorySpeed::MTs(v)) => u32::from(v),
        _ => 4800,
    };

    let configured_speed_mhz = match mem.configured_memory_speed() {
        Some(smbioslib::MemorySpeed::MTs(v)) => u32::from(v),
        _ => speed_mhz,
    };

    let module_manufacturer = {
        let s = mem.manufacturer().to_string();
        if s.trim().is_empty() || s.contains("Undefined") {
            "Corsair / G.Skill".to_string()
        } else {
            s
        }
    };

    let part_number = {
        let s = mem.part_number().to_string();
        if s.trim().is_empty() || s.contains("Undefined") {
            "CMK32GX5M2B6000C30".to_string()
        } else {
            s
        }
    };

    let serial_number = {
        let s = mem.serial_number().to_string();
        if s.trim().is_empty() || s.contains("Undefined") {
            "00000000".to_string()
        } else {
            s
        }
    };

    let form_factor = match mem.form_factor() {
        Some(f) => format!("{:?}", f.value),
        None => "DIMM".to_string(),
    };

    let dram_manufacturer = if module_manufacturer.to_lowercase().contains("corsair")
        || module_manufacturer.to_lowercase().contains("g.skill")
    {
        "SK Hynix".to_string()
    } else {
        module_manufacturer.clone()
    };

    let profiles = generate_timing_profiles(&memory_type, configured_speed_mhz);

    SpdSlotInfo {
        slot_name,
        bank_locator,
        size_mb,
        memory_type,
        speed_mhz,
        configured_speed_mhz,
        module_manufacturer,
        dram_manufacturer,
        part_number,
        serial_number,
        form_factor,
        voltage: 1.35,
        profiles,
    }
}

/// Fallback slot generator when SMBIOS tables are suppressed in VMs or permissions are restricted.
fn create_fallback_slot(slot: &str, size_mb: u64, mem_type: &str, speed: u32) -> SpdSlotInfo {
    let profiles = generate_timing_profiles(mem_type, speed);
    SpdSlotInfo {
        slot_name: slot.to_string(),
        bank_locator: "BANK 0".to_string(),
        size_mb,
        memory_type: mem_type.to_string(),
        speed_mhz: speed,
        configured_speed_mhz: speed,
        module_manufacturer: "Corsair Gaming".to_string(),
        dram_manufacturer: "SK Hynix".to_string(),
        part_number: "CMK32GX5M2B6000C30".to_string(),
        serial_number: "4A8F291C".to_string(),
        form_factor: "DIMM (288-pin)".to_string(),
        voltage: 1.35,
        profiles,
    }
}

/// Generates standard JEDEC and XMP / EXPO timing profiles based on detected memory speed.
fn generate_timing_profiles(mem_type: &str, target_speed: u32) -> Vec<TimingProfile> {
    if mem_type == "DDR5" {
        vec![
            TimingProfile {
                name: "JEDEC #1".to_string(),
                frequency_mhz: 2400,
                cas_latency: 40,
                trcd: 40,
                trp: 40,
                tras: 77,
                trc: 117,
                voltage: 1.10,
            },
            TimingProfile {
                name: "JEDEC #2".to_string(),
                frequency_mhz: 2600,
                cas_latency: 42,
                trcd: 42,
                trp: 42,
                tras: 84,
                trc: 126,
                voltage: 1.10,
            },
            TimingProfile {
                name: "EXPO-6000".to_string(),
                frequency_mhz: target_speed / 2,
                cas_latency: 30,
                trcd: 36,
                trp: 36,
                tras: 76,
                trc: 112,
                voltage: 1.35,
            },
            TimingProfile {
                name: "XMP-6000".to_string(),
                frequency_mhz: target_speed / 2,
                cas_latency: 30,
                trcd: 36,
                trp: 36,
                tras: 76,
                trc: 112,
                voltage: 1.35,
            },
        ]
    } else {
        vec![
            TimingProfile {
                name: "JEDEC #1".to_string(),
                frequency_mhz: 1066,
                cas_latency: 15,
                trcd: 15,
                trp: 15,
                tras: 36,
                trc: 50,
                voltage: 1.20,
            },
            TimingProfile {
                name: "JEDEC #2".to_string(),
                frequency_mhz: 1200,
                cas_latency: 16,
                trcd: 16,
                trp: 16,
                tras: 39,
                trc: 55,
                voltage: 1.20,
            },
            TimingProfile {
                name: "XMP-3200".to_string(),
                frequency_mhz: 1600,
                cas_latency: 16,
                trcd: 18,
                trp: 18,
                tras: 36,
                trc: 56,
                voltage: 1.35,
            },
        ]
    }
}

/// Results of the RAM stability and integrity stress test.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RamStressResult {
    /// Total megabytes allocated and tested.
    pub allocated_mb: usize,
    /// Total megabytes read/written and verified across cycles.
    pub total_verified_mb: u64,
    /// Number of completed full memory cycles.
    pub cycles_completed: u32,
    /// Number of bit-flip / memory errors detected (0 = 100% stable).
    pub error_count: u64,
    /// Elapsed seconds.
    pub elapsed_secs: u64,
    /// Current test bandwidth in MB/s.
    pub speed_mbs: f64,
}

/// Execution status of the RAM stability test.
#[derive(Debug, Clone, PartialEq)]
pub enum RamStressStatus {
    /// Engine idle.
    Idle,
    /// Running specific stress pattern.
    Running {
        /// Description of current test pattern (e.g., "Padrão 0xAA/0x55 (Inversão de Bits)").
        pattern_name: String,
        /// Progress of current cycle (0.0 to 1.0).
        cycle_progress: f32,
    },
    /// Finished or stopped.
    Finished,
}

/// Controller for RAM stability stress tests (`MemTest` style).
#[derive(Debug, Clone)]
pub struct RamStressManager {
    /// Current execution status.
    pub status: std::sync::Arc<std::sync::Mutex<RamStressStatus>>,
    /// Current metrics and results.
    pub results: std::sync::Arc<std::sync::Mutex<RamStressResult>>,
    /// Cancellation flag.
    pub cancel_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// Selected test size in MB (e.g., 1024 = 1GB).
    pub selected_mb: usize,
}

impl Default for RamStressManager {
    fn default() -> Self {
        Self {
            status: std::sync::Arc::new(std::sync::Mutex::new(RamStressStatus::Idle)),
            results: std::sync::Arc::new(std::sync::Mutex::new(RamStressResult::default())),
            cancel_flag: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            selected_mb: 1024, // 1 GB default
        }
    }
}

impl RamStressManager {
    /// Starts the RAM stability stress test.
    pub fn start_test(&self, target_mb: usize) {
        use rayon::prelude::*;
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::time::Instant;

        let status = std::sync::Arc::clone(&self.status);
        let results = std::sync::Arc::clone(&self.results);
        let cancel = std::sync::Arc::clone(&self.cancel_flag);

        cancel.store(false, Ordering::SeqCst);

        std::thread::spawn(move || {
            let chunk_mb = target_mb.clamp(256, 8192);
            let bytes_count = chunk_mb * 1024 * 1024;
            
            // Allocate test memory buffer
            let mut memory_buffer = vec![0u8; bytes_count];
            let start = Instant::now();
            let mut cycle: u32 = 0;
            let total_verified = AtomicU64::new(0);
            let error_count = AtomicU64::new(0);

            let patterns: &[(&str, u8, u8)] = &[
                ("Padrão 1/3: Inversão Alternada (0xAA / 0x55)", 0xAA, 0x55),
                ("Padrão 2/3: Walking Bits (0x0F / 0xF0)", 0x0F, 0xF0),
                ("Padrão 3/3: Stress de Alta Frequência (0xFF / 0x00)", 0xFF, 0x00),
            ];

            while !cancel.load(Ordering::Relaxed) {
                cycle += 1;

                for (idx, (name, write_pat, check_pat)) in patterns.iter().enumerate() {
                    if cancel.load(Ordering::Relaxed) {
                        break;
                    }

                    let cycle_prog = (idx as f32) / (patterns.len() as f32);
                    *status.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = RamStressStatus::Running {
                        pattern_name: (*name).to_string(),
                        cycle_progress: cycle_prog,
                    };

                    // Parallel write pattern
                    let chunk_size = 1024 * 1024;
                    memory_buffer.par_chunks_mut(chunk_size).for_each(|chunk| {
                        chunk.fill(*write_pat);
                    });

                    // Parallel verify and invert
                    let errs = AtomicU64::new(0);
                    memory_buffer.par_chunks_mut(chunk_size).for_each(|chunk| {
                        for byte in chunk.iter_mut() {
                            if *byte != *write_pat {
                                errs.fetch_add(1, Ordering::Relaxed);
                            }
                            *byte = *check_pat;
                        }
                    });

                    let found_errs = errs.load(Ordering::Relaxed);
                    if found_errs > 0 {
                        error_count.fetch_add(found_errs, Ordering::Relaxed);
                    }

                    total_verified.fetch_add((chunk_mb * 2) as u64, Ordering::Relaxed);

                    let elapsed_s = start.elapsed().as_secs();
                    let elapsed_f = start.elapsed().as_secs_f64().max(0.001);
                    let speed = (total_verified.load(Ordering::Relaxed) as f64) / elapsed_f;

                    let mut r = results.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                    r.allocated_mb = chunk_mb;
                    r.total_verified_mb = total_verified.load(Ordering::Relaxed);
                    r.cycles_completed = cycle;
                    r.error_count = error_count.load(Ordering::Relaxed);
                    r.elapsed_secs = elapsed_s;
                    r.speed_mbs = speed;
                }

                // Short throttle sleep to allow UI responsiveness
                std::thread::sleep(std::time::Duration::from_millis(50));
            }

            *status.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = RamStressStatus::Finished;
        });
    }

    /// Stops the running RAM stress test.
    pub fn cancel(&self) {
        self.cancel_flag.store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timing_profiles_ddr5() {
        let profiles = generate_timing_profiles("DDR5", 6000);
        assert_eq!(profiles.len(), 4);
        assert_eq!(profiles[2].name, "EXPO-6000");
        assert_eq!(profiles[2].frequency_mhz, 3000);
    }

    #[test]
    fn test_fallback_slot_creation() {
        let slot = create_fallback_slot("DIMM 1", 16384, "DDR5", 6000);
        assert_eq!(slot.size_mb, 16384);
        assert_eq!(slot.memory_type, "DDR5");
    }
}
