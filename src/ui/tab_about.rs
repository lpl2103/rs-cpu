//! About & System Report Generation tab.

use super::theme::AppTheme;
use super::widgets::{section_header, spec_row};
use crate::hardware::SystemHardware;
use eframe::egui::{self, Button, RichText, ScrollArea, Ui};
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
                    RichText::new("⚡ RS-CPU")
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

            ui.horizontal_wrapped(|ui| {
                let report_txt = generate_text_report(hardware);

                if ui
                    .add(
                        Button::new(RichText::new("📋 Copiar para Área de Transferência").strong().size(13.5))
                            .min_size(egui::vec2(220.0, 34.0)),
                    )
                    .on_hover_text("Copia o relatório de diagnóstico completo em texto para a área de transferência")
                    .clicked()
                {
                    ui.ctx().copy_text(report_txt.clone());
                    state.export_message = Some("✅ Relatório completo copiado para a Área de Transferência!".to_string());
                }

                if ui
                    .add(
                        Button::new(RichText::new("📥 Salvar (.TXT)").strong().size(13.5))
                            .min_size(egui::vec2(130.0, 34.0)),
                    )
                    .clicked()
                {
                    let filename = format!("m-cpu-relatorio-{}.txt", chrono::Local::now().format("%Y%m%d-%H%M%S"));
                    let path = get_safe_export_path(&filename);
                    match fs::write(&path, report_txt) {
                        Ok(()) => {
                            state.export_message = Some(format!("✅ Relatório TXT salvo com sucesso em:\n{}", path.display()));
                        }
                        Err(e) => {
                            state.export_message = Some(format!("❌ Falha ao salvar relatório: {e}"));
                        }
                    }
                }

                if ui
                    .add(
                        Button::new(RichText::new("💾 Salvar (.JSON)").strong().size(13.5))
                            .min_size(egui::vec2(130.0, 34.0)),
                    )
                    .clicked()
                {
                    let filename = format!("m-cpu-relatorio-{}.json", chrono::Local::now().format("%Y%m%d-%H%M%S"));
                    let path = get_safe_export_path(&filename);
                    match serde_json::to_string_pretty(hardware) {
                        Ok(json) => match fs::write(&path, json) {
                            Ok(()) => {
                                state.export_message = Some(format!("✅ Dados JSON salvos com sucesso em:\n{}", path.display()));
                            }
                            Err(e) => {
                                state.export_message = Some(format!("❌ Falha ao salvar JSON: {e}"));
                            }
                        },
                        Err(e) => {
                            state.export_message = Some(format!("❌ Erro de serialização: {e}"));
                        }
                    }
                }

                if ui
                    .add(
                        Button::new(RichText::new("📊 Salvar (.CSV)").strong().size(13.5))
                            .min_size(egui::vec2(130.0, 34.0)),
                    )
                    .clicked()
                {
                    let filename = format!("m-cpu-relatorio-{}.csv", chrono::Local::now().format("%Y%m%d-%H%M%S"));
                    let path = get_safe_export_path(&filename);
                    let csv = generate_csv_report(hardware);
                    match fs::write(&path, csv) {
                        Ok(()) => {
                            state.export_message = Some(format!("✅ Tabela CSV salva com sucesso em:\n{}", path.display()));
                        }
                        Err(e) => {
                            state.export_message = Some(format!("❌ Falha ao salvar CSV: {e}"));
                        }
                    }
                }

                if ui
                    .add(
                        Button::new(RichText::new("🌐 Salvar (.HTML)").strong().size(13.5))
                            .min_size(egui::vec2(130.0, 34.0)),
                    )
                    .clicked()
                {
                    let filename = format!("m-cpu-relatorio-{}.html", chrono::Local::now().format("%Y%m%d-%H%M%S"));
                    let path = get_safe_export_path(&filename);
                    let html = generate_html_report(hardware);
                    match fs::write(&path, html) {
                        Ok(()) => {
                            state.export_message = Some(format!("✅ Relatório HTML salvo com sucesso em:\n{}", path.display()));
                        }
                        Err(e) => {
                            state.export_message = Some(format!("❌ Falha ao salvar HTML: {e}"));
                        }
                    }
                }
            });

            if let Some(msg) = &state.export_message {
                ui.add_space(10.0);
                let is_error = msg.starts_with('❌');
                ui.label(
                    RichText::new(msg)
                        .size(13.0)
                        .color(if is_error { theme.color_error() } else { theme.color_success() })
                        .strong(),
                );
            }
        });

        ui.add_space(8.0);
    });
}

/// Safely resolves a writable path for exports (e.g. Documents, Downloads, or `temp_dir` fallback).
fn get_safe_export_path(filename: &str) -> std::path::PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Ok(profile) = std::env::var("USERPROFILE") {
            let docs = std::path::PathBuf::from(profile).join("Documents");
            if docs.is_dir() {
                return docs.join(filename);
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(home) = std::env::var("HOME") {
            let docs = std::path::PathBuf::from(home).join("Documents");
            if docs.is_dir() {
                return docs.join(filename);
            }
        }
    }

    std::env::temp_dir().join(filename)
}

/// Generates CSV report of all system specifications.
fn generate_csv_report(hardware: &SystemHardware) -> String {
    let mut out = String::from("Categoria,Propriedade,Valor\n");
    let _ = writeln!(out, "Sistema,Sistema Operacional,\"{}\"", hardware.os_name);
    let _ = writeln!(out, "Sistema,Versao do SO,\"{}\"", hardware.os_version);
    let _ = writeln!(out, "Processador,Nome,\"{}\"", hardware.cpu.name);
    let _ = writeln!(out, "Processador,Fabricante,\"{}\"", hardware.cpu.vendor);
    let _ = writeln!(out, "Processador,Codinome,\"{}\"", hardware.cpu.code_name);
    let _ = writeln!(out, "Processador,Soquete,\"{}\"", hardware.cpu.package_socket);
    let _ = writeln!(out, "Processador,Litografia,\"{}\"", hardware.cpu.technology);
    let _ = writeln!(out, "Processador,Nucleos Fisicos,\"{}\"", hardware.cpu.physical_cores);
    let _ = writeln!(out, "Processador,Threads Logicas,\"{}\"", hardware.cpu.logical_threads);
    let _ = writeln!(out, "Placa-Mae,Fabricante,\"{}\"", hardware.motherboard.manufacturer);
    let _ = writeln!(out, "Placa-Mae,Modelo,\"{}\"", hardware.motherboard.model);
    let _ = writeln!(out, "Placa-Mae,Chipset,\"{}\"", hardware.motherboard.chipset);
    let _ = writeln!(out, "Placa-Mae,BIOS Versao,\"{}\"", hardware.motherboard.bios_version);
    let _ = writeln!(out, "Memoria,Total MB,\"{}\"", hardware.memory.total_mb);
    let _ = writeln!(out, "Memoria,Tipo,\"{}\"", hardware.memory.memory_type);
    let _ = writeln!(out, "Memoria,Canais,\"{}\"", hardware.memory.channel_mode);
    let _ = writeln!(out, "Memoria,Frequencia DRAM,\"{:.1} MHz\"", hardware.memory.dram_frequency_mhz);

    for (idx, gpu) in hardware.gpus.iter().enumerate() {
        let _ = writeln!(out, "GPU,GPU #{idx} Nome,\"{}\"", gpu.name);
        let _ = writeln!(out, "GPU,GPU #{idx} VRAM,\"{}\"", gpu.vram_mb);
        let _ = writeln!(out, "GPU,GPU #{idx} Driver,\"{}\"", gpu.driver_version);
    }

    for (idx, drive) in hardware.storage.drives.iter().enumerate() {
        let _ = writeln!(out, "Armazenamento,Drive #{idx} Modelo,\"{}\"", drive.model);
        let _ = writeln!(out, "Armazenamento,Drive #{idx} Capacidade,\"{:.1} GB\"", drive.capacity_gb);
        let _ = writeln!(out, "Armazenamento,Drive #{idx} Interface,\"{}\"", drive.interface);
    }

    out
}

/// Generates a professional HTML report.
fn generate_html_report(hardware: &SystemHardware) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="pt-BR">
<head>
<meta charset="UTF-8">
<title>RS-CPU Relatório de Diagnóstico - {}</title>
<style>
body {{ font-family: 'Segoe UI', system-ui, -apple-system, sans-serif; background: #0f1117; color: #f3f4f6; margin: 0; padding: 24px; }}
.container {{ max-width: 900px; margin: 0 auto; }}
h1 {{ color: #00d2ff; margin-bottom: 4px; }}
.subtitle {{ color: #9ca3af; font-size: 14px; margin-bottom: 24px; }}
.card {{ background: #181b24; border: 1px solid #2d3444; border-radius: 8px; padding: 18px; margin-bottom: 16px; }}
h2 {{ color: #10b981; font-size: 16px; margin-top: 0; border-bottom: 1px solid #2d3444; padding-bottom: 8px; }}
table {{ width: 100%; border-collapse: collapse; }}
td {{ padding: 8px 4px; font-size: 14px; }}
td.label {{ color: #9ca3af; width: 35%; }}
td.value {{ color: #f3f4f6; font-weight: 600; text-align: right; }}
tr:nth-child(even) td {{ background: rgba(255,255,255,0.02); }}
</style>
</head>
<body>
<div class="container">
<h1>⚡ RS-CPU Diagnóstico de Hardware</h1>
<div class="subtitle">Gerado em: {} • Sistema: {} ({})</div>

<div class="card">
<h2>🔲 Processador (CPU)</h2>
<table>
<tr><td class="label">Modelo:</td><td class="value">{}</td></tr>
<tr><td class="label">Fabricante:</td><td class="value">{}</td></tr>
<tr><td class="label">Codinome:</td><td class="value">{}</td></tr>
<tr><td class="label">Soquete:</td><td class="value">{}</td></tr>
<tr><td class="label">Litografia:</td><td class="value">{}</td></tr>
<tr><td class="label">Núcleos / Threads:</td><td class="value">{} Núcleos, {} Threads</td></tr>
</table>
</div>

<div class="card">
<h2>🖧 Placa-Mãe & BIOS</h2>
<table>
<tr><td class="label">Fabricante:</td><td class="value">{}</td></tr>
<tr><td class="label">Modelo:</td><td class="value">{}</td></tr>
<tr><td class="label">Chipset:</td><td class="value">{}</td></tr>
<tr><td class="label">BIOS:</td><td class="value">{} (Versão {})</td></tr>
</table>
</div>

<div class="card">
<h2>💾 Memória RAM</h2>
<table>
<tr><td class="label">Capacidade Total:</td><td class="value">{} MB ({:.1} GB)</td></tr>
<tr><td class="label">Tipo & Canais:</td><td class="value">{} ({})</td></tr>
<tr><td class="label">Frequência DRAM:</td><td class="value">{:.1} MHz</td></tr>
<tr><td class="label">Timings:</td><td class="value">CL{}-{}-{}-{}</td></tr>
</table>
</div>

<div class="card">
<h2>🎮 Placa Gráfica</h2>
<table>
<tr><td class="label">GPU:</td><td class="value">{}</td></tr>
<tr><td class="label">VRAM:</td><td class="value">{} MB ({})</td></tr>
<tr><td class="label">Driver:</td><td class="value">{}</td></tr>
</table>
</div>
</div>
</body>
</html>"#,
        hardware.cpu.name,
        chrono::Local::now().to_rfc2822(),
        hardware.os_name,
        hardware.os_version,
        hardware.cpu.name,
        hardware.cpu.vendor,
        hardware.cpu.code_name,
        hardware.cpu.package_socket,
        hardware.cpu.technology,
        hardware.cpu.physical_cores,
        hardware.cpu.logical_threads,
        hardware.motherboard.manufacturer,
        hardware.motherboard.model,
        hardware.motherboard.chipset,
        hardware.motherboard.bios_vendor,
        hardware.motherboard.bios_version,
        hardware.memory.total_mb,
        hardware.memory.total_mb as f64 / 1024.0,
        hardware.memory.memory_type,
        hardware.memory.channel_mode,
        hardware.memory.dram_frequency_mhz,
        hardware.memory.cl,
        hardware.memory.trcd,
        hardware.memory.trp,
        hardware.memory.tras,
        hardware.gpus.first().map_or("N/D", |g| g.name.as_str()),
        hardware.gpus.first().map_or(0, |g| g.vram_mb),
        hardware.gpus.first().map_or("N/D", |g| g.memory_type.as_str()),
        hardware.gpus.first().map_or("N/D", |g| g.driver_version.as_str()),
    )
}

/// Generates human-readable plain text report mimicking RS-CPU text dumps.
fn generate_text_report(hardware: &SystemHardware) -> String {
    let mut out = String::new();
    out.push_str("--------------------------------------------------\n");
    out.push_str(" RS-CPU RELATORIO DE DIAGNOSTICO DE HARDWARE\n");
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
