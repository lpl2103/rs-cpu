//! Unified "All-in-One" Dashboard displaying CPU, Motherboard, and Memory together.

use super::theme::AppTheme;
use super::widgets::{feature_badge, load_gauge, section_header, spec_row, stat_metric_box};
use crate::hardware::SystemHardware;
use eframe::egui::{self, RichText, ScrollArea, Ui};

/// Renders the Unified All-in-One Dashboard.
pub fn render(ui: &mut Ui, theme: AppTheme, hardware: &SystemHardware) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(4.0);

        // --- TOP LIVE TELEMETRY BAR ---
        theme.card_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("⚡ Live System Telemetry")
                        .size(13.0)
                        .color(theme.accent_primary())
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let hours = hardware.uptime_secs / 3600;
                    let mins = (hardware.uptime_secs % 3600) / 60;
                    ui.label(
                        RichText::new(format!("Uptime: {hours}h {mins}m | {} ({})", hardware.os_name, hardware.os_version))
                            .size(11.0)
                            .color(theme.text_secondary()),
                    );
                });
            });

            ui.add_space(6.0);

            ui.columns(4, |cols| {
                cols[0].vertical(|ui| {
                    let freq = hardware.cpu.live.avg_frequency_mhz;
                    stat_metric_box(ui, theme, "Avg CPU Clock", &format!("{freq:.0} MHz"), &format!("x{:.1} Multiplier", hardware.cpu.live.multiplier));
                });
                cols[1].vertical(|ui| {
                    let load = hardware.cpu.live.global_load_pct;
                    stat_metric_box(ui, theme, "CPU Utilization", &format!("{load:.1}%"), &format!("{} Cores / {} Threads", hardware.cpu.physical_cores, hardware.cpu.logical_threads));
                });
                cols[2].vertical(|ui| {
                    let used_gb = (hardware.memory.live.used_mb as f32) / 1024.0;
                    let total_gb = (hardware.memory.total_mb as f32) / 1024.0;
                    stat_metric_box(ui, theme, "Memory Used", &format!("{used_gb:.1} / {total_gb:.1} GB"), &format!("{:.1}% in use", hardware.memory.live.usage_pct));
                });
                cols[3].vertical(|ui| {
                    let dram_clk = hardware.memory.dram_frequency_mhz;
                    stat_metric_box(ui, theme, "DRAM Speed", &format!("{dram_clk:.0} MHz"), &format!("{} ({})", hardware.memory.memory_type, hardware.memory.channel_mode));
                });
            });

            ui.add_space(6.0);

            ui.columns(2, |cols| {
                cols[0].vertical(|ui| {
                    load_gauge(ui, theme, "CPU Load", hardware.cpu.live.global_load_pct, &format!("{:.1}%", hardware.cpu.live.global_load_pct));
                });
                cols[1].vertical(|ui| {
                    load_gauge(ui, theme, "RAM Usage", hardware.memory.live.usage_pct, &format!("{} MB / {} MB", hardware.memory.live.used_mb, hardware.memory.total_mb));
                });
            });
        });

        ui.add_space(8.0);

        // --- SECTION 1: PROCESSOR (CPU) ---
        theme.card_frame().show(ui, |ui| {
            section_header(ui, theme, "🔲", "Processor (CPU)");

            ui.columns(2, |cols| {
                cols[0].vertical(|ui| {
                    spec_row(ui, theme, "Processor Name", &hardware.cpu.name);
                    spec_row(ui, theme, "Code Name", &hardware.cpu.code_name);
                    spec_row(ui, theme, "Package / Socket", &hardware.cpu.package_socket);
                    spec_row(ui, theme, "Technology / Node", &hardware.cpu.technology);
                    spec_row(ui, theme, "Core VID / Voltage", &hardware.cpu.core_voltage);
                    spec_row(ui, theme, "Specification", &hardware.cpu.name);
                });

                cols[1].vertical(|ui| {
                    spec_row(ui, theme, "Family", &format!("{:X}", hardware.cpu.family));
                    spec_row(ui, theme, "Model", &format!("{:X}", hardware.cpu.model));
                    spec_row(ui, theme, "Stepping", &format!("{}", hardware.cpu.stepping));
                    spec_row(ui, theme, "Ext. Family / Model", &format!("{:X} / {:X}", hardware.cpu.ext_family, hardware.cpu.ext_model));
                    spec_row(ui, theme, "Revision", &hardware.cpu.revision);
                    spec_row(ui, theme, "Cores / Threads", &format!("{} Cores, {} Threads", hardware.cpu.physical_cores, hardware.cpu.logical_threads));
                });
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            // Instructions Badges
            ui.label(RichText::new("Supported Instructions").size(12.0).color(theme.text_secondary()));
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                for inst in &hardware.cpu.instructions {
                    feature_badge(ui, theme, inst, true);
                }
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            // Caches and Clocks
            ui.columns(2, |cols| {
                cols[0].vertical(|ui| {
                    ui.label(RichText::new("Live Clocks").size(12.5).color(theme.accent_primary()).strong());
                    ui.add_space(2.0);
                    spec_row(ui, theme, "Core Speed", &format!("{:.1} MHz", hardware.cpu.live.avg_frequency_mhz));
                    spec_row(ui, theme, "Multiplier", &format!("x {:.1}", hardware.cpu.live.multiplier));
                    spec_row(ui, theme, "Bus Speed (BCLK)", &format!("{:.1} MHz", hardware.cpu.live.bus_speed_mhz));
                });

                cols[1].vertical(|ui| {
                    ui.label(RichText::new("Cache Topology").size(12.5).color(theme.accent_primary()).strong());
                    ui.add_space(2.0);
                    for cache in &hardware.cpu.caches {
                        let label = format!("L{} {} Cache", cache.level, cache.cache_type);
                        let val = if cache.size_kb >= 1024 {
                            format!("{} MB ({})", cache.size_kb / 1024, cache.associativity)
                        } else {
                            format!("{} KB ({})", cache.size_kb, cache.associativity)
                        };
                        spec_row(ui, theme, &label, &val);
                    }
                });
            });
        });

        ui.add_space(8.0);

        // --- SECTION 2 & 3: MOTHERBOARD & MEMORY (SIDE BY SIDE COLUMNS) ---
        ui.columns(2, |cols| {
            // LEFT COLUMN: MOTHERBOARD
            cols[0].vertical(|ui| {
                theme.card_frame().show(ui, |ui| {
                    section_header(ui, theme, "🖧", "Motherboard (Mainboard)");

                    spec_row(ui, theme, "Manufacturer", &hardware.motherboard.manufacturer);
                    spec_row(ui, theme, "Model", &hardware.motherboard.model);
                    spec_row(ui, theme, "Version / Rev", &hardware.motherboard.version);
                    spec_row(ui, theme, "Chipset", &hardware.motherboard.chipset);
                    spec_row(ui, theme, "Bus Specs", &hardware.motherboard.bus_specs);
                    spec_row(ui, theme, "Graphic Interface", &hardware.motherboard.graphic_interface);

                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(6.0);

                    ui.label(RichText::new("BIOS Information").size(12.5).color(theme.accent_primary()).strong());
                    ui.add_space(2.0);
                    spec_row(ui, theme, "Brand / Vendor", &hardware.motherboard.bios_vendor);
                    spec_row(ui, theme, "Version", &hardware.motherboard.bios_version);
                    spec_row(ui, theme, "Release Date", &hardware.motherboard.bios_date);
                    spec_row(ui, theme, "SMBIOS Version", &hardware.motherboard.smbios_version);
                });
            });

            // RIGHT COLUMN: MEMORY (RAM)
            cols[1].vertical(|ui| {
                theme.card_frame().show(ui, |ui| {
                    section_header(ui, theme, "💾", "Memory (RAM) & SPD");

                    let total_gb = (hardware.memory.total_mb as f32) / 1024.0;
                    spec_row(ui, theme, "Type & Channel", &format!("{} ({})", hardware.memory.memory_type, hardware.memory.channel_mode));
                    spec_row(ui, theme, "Total Size", &format!("{total_gb:.1} GBytes ({:.0} MB)", hardware.memory.total_mb));
                    spec_row(ui, theme, "DRAM Frequency", &format!("{:.1} MHz", hardware.memory.dram_frequency_mhz));
                    spec_row(ui, theme, "Uncore / Controller", &format!("{:.1} MHz", hardware.memory.uncore_frequency_mhz));
                    spec_row(ui, theme, "FSB:DRAM", &hardware.memory.fsb_dram_ratio);

                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(6.0);

                    ui.label(RichText::new("Primary Timings").size(12.5).color(theme.accent_primary()).strong());
                    ui.add_space(2.0);
                    spec_row(ui, theme, "CAS# Latency (CL)", &format!("{} clocks", hardware.memory.cl));
                    spec_row(ui, theme, "RAS# to CAS# Delay (tRCD)", &format!("{} clocks", hardware.memory.trcd));
                    spec_row(ui, theme, "RAS# Precharge (tRP)", &format!("{} clocks", hardware.memory.trp));
                    spec_row(ui, theme, "Cycle Time (tRAS)", &format!("{} clocks", hardware.memory.tras));
                    spec_row(ui, theme, "Bank Cycle Time (tRC)", &format!("{} clocks", hardware.memory.trc));
                    spec_row(ui, theme, "Command Rate (CR)", &hardware.memory.command_rate);

                    if let Some(slot) = hardware.memory.slots.iter().find(|s| s.size_mb > 0) {
                        ui.add_space(4.0);
                        spec_row(ui, theme, "Installed Module", &format!("{} ({} MB)", slot.module_manufacturer, slot.size_mb));
                        spec_row(ui, theme, "Part Number", &slot.part_number);
                    }
                });
            });
        });

        // --- SECTION 4: GRAPHICS CARD ---
        if let Some(gpu) = hardware.gpus.first() {
            ui.add_space(8.0);
            theme.card_frame().show(ui, |ui| {
                section_header(ui, theme, "🎮", "Graphics Adapter (GPU)");

                ui.columns(2, |cols| {
                    cols[0].vertical(|ui| {
                        spec_row(ui, theme, "GPU Name", &gpu.name);
                        spec_row(ui, theme, "Vendor", &gpu.vendor);
                        spec_row(ui, theme, "Code Name / Arch", &gpu.code_name);
                        spec_row(ui, theme, "Technology", &gpu.technology);
                    });
                    cols[1].vertical(|ui| {
                        spec_row(ui, theme, "Memory Size (VRAM)", &format!("{} MB ({} GB)", gpu.vram_mb, gpu.vram_mb / 1024));
                        spec_row(ui, theme, "Memory Type & Bus", &format!("{} ({})", gpu.memory_type, gpu.bus_width));
                        spec_row(ui, theme, "Core & Mem Clocks", &format!("{} MHz / {} MHz", gpu.core_clock_mhz, gpu.memory_clock_mhz));
                        spec_row(ui, theme, "Driver Version", &gpu.driver_version);
                    });
                });
            });
        }

        ui.add_space(8.0);
    });
}
