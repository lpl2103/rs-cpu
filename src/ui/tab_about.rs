//! About & System Report Generation tab.

use super::theme::AppTheme;
use super::widgets::{section_header, spec_row};
use crate::hardware::SystemHardware;
use eframe::egui::{self, Button, Color32, RichText, ScrollArea, Ui};
use std::fmt::Write as _;
use std::fs;

/// State for the About / Export view.
#[derive(Debug, Clone, Default)]
pub struct AboutTabState {
    /// Feedback message after exporting report.
    pub export_message: Option<String>,
}

/// Renders the About and Report Export tab.
pub fn render(
    ui: &mut Ui,
    theme: AppTheme,
    hardware: &SystemHardware,
    state: &mut AboutTabState,
) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(4.0);

        // Application Banner
        theme.card_frame().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("⚡ MODERN CPU-Z")
                        .size(20.0)
                        .color(theme.accent_primary())
                        .strong(),
                );
                ui.label(
                    RichText::new("High-Performance Hardware Introspection & Telemetry in Rust")
                        .size(12.0)
                        .color(theme.text_secondary()),
                );
                ui.label(
                    RichText::new("v0.1.0 • 100% Native Cross-Platform • Zero Kernel Drivers")
                        .size(11.0)
                        .color(theme.accent_secondary()),
                );
            });
        });

        ui.add_space(8.0);

        // System Environment Specs
        theme.card_frame().show(ui, |ui| {
            section_header(ui, theme, "💻", "Operating System & Environment");

            spec_row(ui, theme, "Operating System", &hardware.os_name);
            spec_row(ui, theme, "OS Version", &hardware.os_version);
            spec_row(ui, theme, "Architecture", std::env::consts::ARCH);
            spec_row(ui, theme, "Target OS", std::env::consts::OS);
            spec_row(ui, theme, "Rust Edition", "Rust 2021 Edition (High-Perf)");
            spec_row(ui, theme, "Compiler / CRT", "MSVC Static CRT / Strip & Fat LTO");
        });

        ui.add_space(8.0);

        // Report Export Tools
        theme.card_frame().show(ui, |ui| {
            section_header(ui, theme, "📄", "Hardware Report & Diagnostics Export");

            ui.label(
                RichText::new("Export complete hardware and system parameters for diagnostic analysis or validation:")
                    .size(12.0)
                    .color(theme.text_secondary()),
            );

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui
                    .add(
                        Button::new(RichText::new("📥 Save Report (.TXT)").strong().size(12.5))
                            .min_size(egui::vec2(150.0, 30.0)),
                    )
                    .clicked()
                {
                    let filename = format!("cpu-z-report-{}.txt", chrono::Local::now().format("%Y%m%d-%H%M%S"));
                    let report_txt = generate_text_report(hardware);
                    match fs::write(&filename, report_txt) {
                        Ok(()) => {
                            state.export_message = Some(format!("Report saved to: {filename}"));
                        }
                        Err(e) => {
                            state.export_message = Some(format!("Failed to save report: {e}"));
                        }
                    }
                }

                if ui
                    .add(
                        Button::new(RichText::new("💾 Save Report (.JSON)").strong().size(12.5))
                            .min_size(egui::vec2(150.0, 30.0)),
                    )
                    .clicked()
                {
                    let filename = format!("cpu-z-report-{}.json", chrono::Local::now().format("%Y%m%d-%H%M%S"));
                    match serde_json::to_string_pretty(hardware) {
                        Ok(json) => match fs::write(&filename, json) {
                            Ok(()) => {
                                state.export_message = Some(format!("JSON dump saved to: {filename}"));
                            }
                            Err(e) => {
                                state.export_message = Some(format!("Failed to save JSON: {e}"));
                            }
                        },
                        Err(e) => {
                            state.export_message = Some(format!("Serialization error: {e}"));
                        }
                    }
                }
            });

            if let Some(msg) = &state.export_message {
                ui.add_space(8.0);
                ui.label(
                    RichText::new(msg)
                        .size(12.0)
                        .color(Color32::from_rgb(16, 185, 129))
                        .strong(),
                );
            }
        });

        ui.add_space(8.0);
    });
}

/// Generates human-readable plain text report mimicking CPU-Z text dumps.
fn generate_text_report(hardware: &SystemHardware) -> String {
    let mut out = String::new();
    out.push_str("--------------------------------------------------\n");
    out.push_str(" MODERN CPU-Z HARDWARE DIAGNOSTIC REPORT\n");
    let _ = writeln!(out, " Generated: {}", chrono::Local::now().to_rfc2822());
    out.push_str("--------------------------------------------------\n\n");

    out.push_str("[PROCESSOR]\n");
    let _ = writeln!(out, "  Name:             {}", hardware.cpu.name);
    let _ = writeln!(out, "  Vendor:           {}", hardware.cpu.vendor);
    let _ = writeln!(out, "  Code Name:        {}", hardware.cpu.code_name);
    let _ = writeln!(out, "  Package/Socket:   {}", hardware.cpu.package_socket);
    let _ = writeln!(out, "  Technology:       {}", hardware.cpu.technology);
    let _ = writeln!(out, "  Family/Model/Stp: {:X} / {:X} / {}", hardware.cpu.family, hardware.cpu.model, hardware.cpu.stepping);
    let _ = writeln!(out, "  Cores / Threads:  {} Cores, {} Threads", hardware.cpu.physical_cores, hardware.cpu.logical_threads);
    let _ = writeln!(out, "  Instructions:     {}\n", hardware.cpu.instructions.join(" "));

    out.push_str("[MOTHERBOARD]\n");
    let _ = writeln!(out, "  Manufacturer:     {}", hardware.motherboard.manufacturer);
    let _ = writeln!(out, "  Model:            {}", hardware.motherboard.model);
    let _ = writeln!(out, "  Version:          {}", hardware.motherboard.version);
    let _ = writeln!(out, "  Chipset:          {}", hardware.motherboard.chipset);
    let _ = writeln!(out, "  BIOS Vendor:      {}", hardware.motherboard.bios_vendor);
    let _ = writeln!(out, "  BIOS Version:     {}", hardware.motherboard.bios_version);
    let _ = writeln!(out, "  BIOS Date:        {}\n", hardware.motherboard.bios_date);

    out.push_str("[MEMORY]\n");
    let _ = writeln!(out, "  Total Size:       {} MB ({} GB)", hardware.memory.total_mb, hardware.memory.total_mb / 1024);
    let _ = writeln!(out, "  Type:             {}", hardware.memory.memory_type);
    let _ = writeln!(out, "  Channel Mode:     {}", hardware.memory.channel_mode);
    let _ = writeln!(out, "  DRAM Frequency:   {:.1} MHz", hardware.memory.dram_frequency_mhz);
    let _ = writeln!(out, "  Timings:          CL{}-{}-{}-{}\n", hardware.memory.cl, hardware.memory.trcd, hardware.memory.trp, hardware.memory.tras);

    out.push_str("[GRAPHICS]\n");
    for (i, gpu) in hardware.gpus.iter().enumerate() {
        let _ = writeln!(out, "  GPU #{i}:          {}", gpu.name);
        let _ = writeln!(out, "  VRAM:             {} MB ({})", gpu.vram_mb, gpu.memory_type);
        let _ = writeln!(out, "  Driver:           {}", gpu.driver_version);
    }

    out
}
