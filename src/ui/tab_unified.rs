//! Unified "All-in-One" Dashboard displaying CPU, Motherboard, and Memory together.

use super::theme::AppTheme;
use super::widgets::{instructions_grid, load_gauge, section_header, spec_row, stat_metric_box};
use crate::hardware::SystemHardware;
use eframe::egui::{self, RichText, ScrollArea, Ui};

/// Renders the Unified All-in-One Dashboard in Portuguese (PT-BR).
pub fn render(ui: &mut Ui, theme: AppTheme, hardware: &SystemHardware) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(4.0);

        // --- BARRA SUPERIOR DE TELEMETRIA AO VIVO ---
        theme.card_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("⚡ Telemetria do Sistema em Tempo Real")
                        .size(14.5)
                        .color(theme.accent_primary())
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let hours = hardware.uptime_secs / 3600;
                    let mins = (hardware.uptime_secs % 3600) / 60;
                    ui.label(
                        RichText::new(format!("Atividade: {hours}h {mins}m | {} ({})", hardware.os_name, hardware.os_version))
                            .size(12.0)
                            .color(theme.text_secondary()),
                    );
                });
            });

            ui.add_space(8.0);

            ui.columns(4, |cols| {
                cols[0].vertical(|ui| {
                    let freq = hardware.cpu.live.avg_frequency_mhz;
                    stat_metric_box(ui, theme, "Clock Médio CPU", &format!("{freq:.0} MHz"), &format!("x{:.1} Multiplicador", hardware.cpu.live.multiplier));
                });
                cols[1].vertical(|ui| {
                    let load = hardware.cpu.live.global_load_pct;
                    stat_metric_box(ui, theme, "Uso da CPU", &format!("{load:.1}%"), &format!("{} Núcleos / {} Threads", hardware.cpu.physical_cores, hardware.cpu.logical_threads));
                });
                cols[2].vertical(|ui| {
                    let used_gb = (hardware.memory.live.used_mb as f32) / 1024.0;
                    let total_gb = (hardware.memory.total_mb as f32) / 1024.0;
                    stat_metric_box(ui, theme, "Memória em Uso", &format!("{used_gb:.1} / {total_gb:.1} GB"), &format!("{:.1}% em uso", hardware.memory.live.usage_pct));
                });
                cols[3].vertical(|ui| {
                    let dram_clk = hardware.memory.dram_frequency_mhz;
                    stat_metric_box(ui, theme, "Velocidade DRAM", &format!("{dram_clk:.0} MHz"), &format!("{} ({})", hardware.memory.memory_type, hardware.memory.channel_mode));
                });
            });

            ui.add_space(8.0);

            ui.columns(2, |cols| {
                cols[0].vertical(|ui| {
                    load_gauge(ui, theme, "Carga da CPU", hardware.cpu.live.global_load_pct, &format!("{:.1}%", hardware.cpu.live.global_load_pct));
                });
                cols[1].vertical(|ui| {
                    load_gauge(ui, theme, "Uso de RAM", hardware.memory.live.usage_pct, &format!("{} MB / {} MB", hardware.memory.live.used_mb, hardware.memory.total_mb));
                });
            });
        });

        ui.add_space(8.0);

        // --- SEÇÃO 1: PROCESSADOR (CPU) ---
        theme.card_frame().show(ui, |ui| {
            section_header(ui, theme, "🔲", "Processador (CPU)");

            ui.columns(2, |cols| {
                cols[0].vertical(|ui| {
                    spec_row(ui, theme, "Nome do Processador", &hardware.cpu.name);
                    spec_row(ui, theme, "Codinome", &hardware.cpu.code_name);
                    spec_row(ui, theme, "Soquete / Pacote", &hardware.cpu.package_socket);
                    spec_row(ui, theme, "Litografia / Processo", &hardware.cpu.technology);
                    spec_row(ui, theme, "Tensão VID do Núcleo", &hardware.cpu.core_voltage);
                    spec_row(ui, theme, "Especificação", &hardware.cpu.name);
                });

                cols[1].vertical(|ui| {
                    spec_row(ui, theme, "Família", &format!("{:X}", hardware.cpu.family));
                    spec_row(ui, theme, "Modelo", &format!("{:X}", hardware.cpu.model));
                    spec_row(ui, theme, "Stepping", &format!("{}", hardware.cpu.stepping));
                    spec_row(ui, theme, "Família/Modelo Ext.", &format!("{:X} / {:X}", hardware.cpu.ext_family, hardware.cpu.ext_model));
                    spec_row(ui, theme, "Revisão", &hardware.cpu.revision);
                    spec_row(ui, theme, "Núcleos / Threads", &format!("{} Núcleos, {} Threads", hardware.cpu.physical_cores, hardware.cpu.logical_threads));
                });
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Instruções em Colunas Organizadas
            ui.label(RichText::new("Instruções Suportadas").size(13.0).color(theme.text_secondary()).strong());
            ui.add_space(6.0);
            instructions_grid(ui, theme, &hardware.cpu.instructions);

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Caches e Clocks
            ui.columns(2, |cols| {
                cols[0].vertical(|ui| {
                    ui.label(RichText::new("Clocks em Tempo Real").size(13.5).color(theme.accent_primary()).strong());
                    ui.add_space(4.0);
                    spec_row(ui, theme, "Frequência do Núcleo", &format!("{:.1} MHz", hardware.cpu.live.avg_frequency_mhz));
                    spec_row(ui, theme, "Multiplicador", &format!("x {:.1}", hardware.cpu.live.multiplier));
                    spec_row(ui, theme, "Barramento (BCLK)", &format!("{:.1} MHz", hardware.cpu.live.bus_speed_mhz));
                });

                cols[1].vertical(|ui| {
                    ui.label(RichText::new("Topologia de Cache").size(13.5).color(theme.accent_primary()).strong());
                    ui.add_space(4.0);
                    for cache in &hardware.cpu.caches {
                        let label = format!("Cache L{} {}", cache.level, cache.cache_type);
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

        // --- SEÇÃO 2 & 3: PLACA-MÃE & MEMÓRIA (COLUNAS LADO A LADO) ---
        ui.columns(2, |cols| {
            // COLUNA DA ESQUERDA: PLACA-MÃE
            cols[0].vertical(|ui| {
                theme.card_frame().show(ui, |ui| {
                    section_header(ui, theme, "🖧", "Placa-Mãe (Motherboard)");

                    spec_row(ui, theme, "Fabricante", &hardware.motherboard.manufacturer);
                    spec_row(ui, theme, "Modelo", &hardware.motherboard.model);
                    spec_row(ui, theme, "Versão / Rev", &hardware.motherboard.version);
                    spec_row(ui, theme, "Chipset", &hardware.motherboard.chipset);
                    spec_row(ui, theme, "Barramentos", &hardware.motherboard.bus_specs);
                    spec_row(ui, theme, "Interface Gráfica", &hardware.motherboard.graphic_interface);

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.label(RichText::new("Informações da BIOS").size(13.5).color(theme.accent_primary()).strong());
                    ui.add_space(4.0);
                    spec_row(ui, theme, "Fabricante BIOS", &hardware.motherboard.bios_vendor);
                    spec_row(ui, theme, "Versão", &hardware.motherboard.bios_version);
                    spec_row(ui, theme, "Data de Lançamento", &hardware.motherboard.bios_date);
                    spec_row(ui, theme, "Versão SMBIOS", &hardware.motherboard.smbios_version);
                });
            });

            // COLUNA DA DIREITA: MEMÓRIA RAM
            cols[1].vertical(|ui| {
                theme.card_frame().show(ui, |ui| {
                    section_header(ui, theme, "💾", "Memória RAM & SPD");

                    let total_gb = (hardware.memory.total_mb as f32) / 1024.0;
                    spec_row(ui, theme, "Tipo & Canal", &format!("{} ({})", hardware.memory.memory_type, hardware.memory.channel_mode));
                    spec_row(ui, theme, "Tamanho Total", &format!("{total_gb:.1} GBytes ({:.0} MB)", hardware.memory.total_mb));
                    spec_row(ui, theme, "Frequência DRAM", &format!("{:.1} MHz", hardware.memory.dram_frequency_mhz));
                    spec_row(ui, theme, "Frequência Uncore", &format!("{:.1} MHz", hardware.memory.uncore_frequency_mhz));
                    spec_row(ui, theme, "Proporção FSB:DRAM", &hardware.memory.fsb_dram_ratio);

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.label(RichText::new("Timings Primários").size(13.5).color(theme.accent_primary()).strong());
                    ui.add_space(4.0);
                    spec_row(ui, theme, "Latência CAS# (CL)", &format!("{} clocks", hardware.memory.cl));
                    spec_row(ui, theme, "Atraso RAS# para CAS# (tRCD)", &format!("{} clocks", hardware.memory.trcd));
                    spec_row(ui, theme, "Pré-carga RAS# (tRP)", &format!("{} clocks", hardware.memory.trp));
                    spec_row(ui, theme, "Tempo de Ciclo (tRAS)", &format!("{} clocks", hardware.memory.tras));
                    spec_row(ui, theme, "Ciclo de Banco (tRC)", &format!("{} clocks", hardware.memory.trc));
                    spec_row(ui, theme, "Taxa de Comando (CR)", &hardware.memory.command_rate);

                    if let Some(slot) = hardware.memory.slots.iter().find(|s| s.size_mb > 0) {
                        ui.add_space(6.0);
                        spec_row(ui, theme, "Módulo Instalado", &format!("{} ({} MB)", slot.module_manufacturer, slot.size_mb));
                        spec_row(ui, theme, "Part Number", &slot.part_number);
                    }
                });
            });
        });

        // --- SEÇÃO 4: PLACA DE VÍDEO (GPU) ---
        if let Some(gpu) = hardware.gpus.first() {
            ui.add_space(8.0);
            theme.card_frame().show(ui, |ui| {
                section_header(ui, theme, "🎮", "Placa de Vídeo (GPU)");

                ui.columns(2, |cols| {
                    cols[0].vertical(|ui| {
                        spec_row(ui, theme, "Nome da GPU", &gpu.name);
                        spec_row(ui, theme, "Fabricante", &gpu.vendor);
                        spec_row(ui, theme, "Codinome / Arquitetura", &gpu.code_name);
                        spec_row(ui, theme, "Litografia / Processo", &gpu.technology);
                    });
                    cols[1].vertical(|ui| {
                        spec_row(ui, theme, "Memória de Vídeo (VRAM)", &format!("{} MB ({} GB)", gpu.vram_mb, gpu.vram_mb / 1024));
                        spec_row(ui, theme, "Tipo & Barramento", &format!("{} ({})", gpu.memory_type, gpu.bus_width));
                        spec_row(ui, theme, "Clocks Núcleo / Memória", &format!("{} MHz / {} MHz", gpu.core_clock_mhz, gpu.memory_clock_mhz));
                        spec_row(ui, theme, "Versão do Driver", &gpu.driver_version);
                    });
                });
            });
        }

        ui.add_space(8.0);
    });
}
