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

/// Renders the About and Report Export tab in Portuguese (PT-BR).
pub fn render(
    ui: &mut Ui,
    theme: AppTheme,
    hardware: &SystemHardware,
    state: &mut AboutTabState,
) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(4.0);

        // Banner Principal do Aplicativo
        theme.card_frame().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("⚡ M-CPU")
                        .size(24.0)
                        .color(theme.accent_primary())
                        .strong(),
                );
                ui.add_space(2.0);
                ui.label(
                    RichText::new("Diagnóstico e Telemetria de Hardware de Alta Performance em Rust")
                        .size(13.5)
                        .color(theme.text_secondary()),
                );
                ui.add_space(2.0);
                ui.label(
                    RichText::new("v0.1.0 • 100% Nativo Multiplataforma • Sem Drivers de Kernel")
                        .size(12.0)
                        .color(theme.accent_secondary()),
                );
            });
        });

        ui.add_space(8.0);

        // Especificações do Ambiente
        theme.card_frame().show(ui, |ui| {
            section_header(ui, theme, "💻", "Sistema Operacional & Ambiente");

            spec_row(ui, theme, "Sistema Operacional", &hardware.os_name);
            spec_row(ui, theme, "Versão do SO", &hardware.os_version);
            spec_row(ui, theme, "Arquitetura", std::env::consts::ARCH);
            spec_row(ui, theme, "SO Alvo", std::env::consts::OS);
            spec_row(ui, theme, "Edição Rust", "Rust 2021 Edition (High-Perf)");
            spec_row(ui, theme, "Compilador / CRT", "MSVC CRT Estático / Strip & Fat LTO");
        });

        ui.add_space(8.0);

        // Ferramentas de Exportação de Relatórios
        theme.card_frame().show(ui, |ui| {
            section_header(ui, theme, "📄", "Exportação de Relatórios de Diagnóstico");

            ui.label(
                RichText::new("Exporte todas as especificações e telemetria de hardware para análise ou validação:")
                    .size(13.0)
                    .color(theme.text_secondary()),
            );

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui
                    .add(
                        Button::new(RichText::new("📥 Salvar Relatório (.TXT)").strong().size(13.5))
                            .min_size(egui::vec2(170.0, 34.0)),
                    )
                    .clicked()
                {
                    let filename = format!("m-cpu-relatorio-{}.txt", chrono::Local::now().format("%Y%m%d-%H%M%S"));
                    let report_txt = generate_text_report(hardware);
                    match fs::write(&filename, report_txt) {
                        Ok(()) => {
                            state.export_message = Some(format!("Relatório salvo com sucesso em: {filename}"));
                        }
                        Err(e) => {
                            state.export_message = Some(format!("Falha ao salvar relatório: {e}"));
                        }
                    }
                }

                if ui
                    .add(
                        Button::new(RichText::new("💾 Salvar Relatório (.JSON)").strong().size(13.5))
                            .min_size(egui::vec2(170.0, 34.0)),
                    )
                    .clicked()
                {
                    let filename = format!("m-cpu-relatorio-{}.json", chrono::Local::now().format("%Y%m%d-%H%M%S"));
                    match serde_json::to_string_pretty(hardware) {
                        Ok(json) => match fs::write(&filename, json) {
                            Ok(()) => {
                                state.export_message = Some(format!("Dados JSON salvos com sucesso em: {filename}"));
                            }
                            Err(e) => {
                                state.export_message = Some(format!("Falha ao salvar JSON: {e}"));
                            }
                        },
                        Err(e) => {
                            state.export_message = Some(format!("Erro de serialização: {e}"));
                        }
                    }
                }
            });

            if let Some(msg) = &state.export_message {
                ui.add_space(10.0);
                ui.label(
                    RichText::new(msg)
                        .size(13.0)
                        .color(Color32::from_rgb(16, 185, 129))
                        .strong(),
                );
            }
        });

        ui.add_space(8.0);
    });
}

/// Generates human-readable plain text report mimicking M-CPU text dumps.
fn generate_text_report(hardware: &SystemHardware) -> String {
    let mut out = String::new();
    out.push_str("--------------------------------------------------\n");
    out.push_str(" M-CPU RELATORIO DE DIAGNOSTICO DE HARDWARE\n");
    let _ = writeln!(out, " Gerado em: {}", chrono::Local::now().to_rfc2822());
    out.push_str("--------------------------------------------------\n\n");

    out.push_str("[PROCESSADOR]\n");
    let _ = writeln!(out, "  Nome:             {}", hardware.cpu.name);
    let _ = writeln!(out, "  Fabricante:       {}", hardware.cpu.vendor);
    let _ = writeln!(out, "  Codinome:         {}", hardware.cpu.code_name);
    let _ = writeln!(out, "  Soquete/Pacote:   {}", hardware.cpu.package_socket);
    let _ = writeln!(out, "  Litografia:       {}", hardware.cpu.technology);
    let _ = writeln!(out, "  Familia/Mod/Stp:  {:X} / {:X} / {}", hardware.cpu.family, hardware.cpu.model, hardware.cpu.stepping);
    let _ = writeln!(out, "  Nucleos / Threads:{} Nucleos, {} Threads", hardware.cpu.physical_cores, hardware.cpu.logical_threads);
    let _ = writeln!(out, "  Instrucoes:       {}\n", hardware.cpu.instructions.join(" "));

    out.push_str("[PLACA-MAE]\n");
    let _ = writeln!(out, "  Fabricante:       {}", hardware.motherboard.manufacturer);
    let _ = writeln!(out, "  Modelo:           {}", hardware.motherboard.model);
    let _ = writeln!(out, "  Versao:           {}", hardware.motherboard.version);
    let _ = writeln!(out, "  Chipset:          {}", hardware.motherboard.chipset);
    let _ = writeln!(out, "  BIOS Fabricante:  {}", hardware.motherboard.bios_vendor);
    let _ = writeln!(out, "  BIOS Versao:      {}", hardware.motherboard.bios_version);
    let _ = writeln!(out, "  BIOS Data:        {}\n", hardware.motherboard.bios_date);

    out.push_str("[MEMORIA RAM]\n");
    let _ = writeln!(out, "  Capacidade Total: {} MB ({} GB)", hardware.memory.total_mb, hardware.memory.total_mb / 1024);
    let _ = writeln!(out, "  Tipo:             {}", hardware.memory.memory_type);
    let _ = writeln!(out, "  Canais:           {}", hardware.memory.channel_mode);
    let _ = writeln!(out, "  Frequencia DRAM:  {:.1} MHz", hardware.memory.dram_frequency_mhz);
    let _ = writeln!(out, "  Timings:          CL{}-{}-{}-{}\n", hardware.memory.cl, hardware.memory.trcd, hardware.memory.trp, hardware.memory.tras);

    out.push_str("[GRAFICOS]\n");
    for (i, gpu) in hardware.gpus.iter().enumerate() {
        let _ = writeln!(out, "  GPU #{i}:          {}", gpu.name);
        let _ = writeln!(out, "  VRAM:             {} MB ({})", gpu.vram_mb, gpu.memory_type);
        let _ = writeln!(out, "  Driver:           {}", gpu.driver_version);
    }

    out
}
