//! Detailed CPU View tab.

use super::theme::AppTheme;
use super::widgets::{instructions_grid, load_gauge, section_header, spec_row};
use crate::hardware::SystemHardware;
use eframe::egui::{RichText, ScrollArea, Ui};

/// Renders the dedicated detailed CPU tab in Portuguese (PT-BR).
pub fn render(ui: &mut Ui, theme: AppTheme, hardware: &SystemHardware) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(4.0);

        // Informações Gerais do Processador
        theme.card_frame().show(ui, |ui| {
            section_header(ui, theme, "🔲", "Informações do Processador");

            spec_row(ui, theme, "Nome", &hardware.cpu.name);
            spec_row(ui, theme, "Codinome", &hardware.cpu.code_name);
            spec_row(ui, theme, "Soquete / Pacote", &hardware.cpu.package_socket);
            spec_row(ui, theme, "Litografia / Processo", &hardware.cpu.technology);
            spec_row(ui, theme, "Tensão VID", &hardware.cpu.core_voltage);
            spec_row(ui, theme, "Especificação", &hardware.cpu.name);

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            ui.columns(2, |cols| {
                cols[0].vertical(|ui| {
                    spec_row(ui, theme, "Família", &format!("{:X}", hardware.cpu.family));
                    spec_row(ui, theme, "Modelo", &format!("{:X}", hardware.cpu.model));
                    spec_row(ui, theme, "Stepping", &format!("{}", hardware.cpu.stepping));
                });
                cols[1].vertical(|ui| {
                    spec_row(ui, theme, "Família Ext.", &format!("{:X}", hardware.cpu.ext_family));
                    spec_row(ui, theme, "Modelo Ext.", &format!("{:X}", hardware.cpu.ext_model));
                    spec_row(ui, theme, "Revisão", &hardware.cpu.revision);
                });
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            ui.label(RichText::new("Instruções Suportadas").size(13.0).color(theme.text_secondary()).strong());
            ui.add_space(6.0);
            instructions_grid(ui, theme, &hardware.cpu.instructions);
        });

        ui.add_space(8.0);

        // Clocks & Caches
        ui.columns(2, |cols| {
            cols[0].vertical(|ui| {
                theme.card_frame().show(ui, |ui| {
                    section_header(ui, theme, "⚡", "Clocks em Tempo Real");

                    spec_row(ui, theme, "Frequência do Núcleo", &format!("{:.1} MHz", hardware.cpu.live.avg_frequency_mhz));
                    spec_row(ui, theme, "Multiplicador", &format!("x {:.1}", hardware.cpu.live.multiplier));
                    spec_row(ui, theme, "Barramento (BCLK)", &format!("{:.1} MHz", hardware.cpu.live.bus_speed_mhz));
                    spec_row(ui, theme, "FSB Nominal", &format!("{:.1} MHz", hardware.cpu.live.bus_speed_mhz * 4.0));
                });
            });

            cols[1].vertical(|ui| {
                theme.card_frame().show(ui, |ui| {
                    section_header(ui, theme, "📦", "Topologia de Cache");

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

        // Núcleos e Threads
        theme.card_frame().show(ui, |ui| {
            section_header(ui, theme, "📊", "Atividade Detalhada por Núcleo");

            spec_row(ui, theme, "Núcleos Físicos", &format!("{}", hardware.cpu.physical_cores));
            spec_row(ui, theme, "Threads Lógicas", &format!("{}", hardware.cpu.logical_threads));

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Per core load gauges
            let total_threads = hardware.cpu.live.per_core_load_pct.len();
            ui.columns(2, |cols| {
                for i in 0..total_threads {
                    let col_idx = i % 2;
                    let load = hardware.cpu.live.per_core_load_pct.get(i).copied().unwrap_or(0.0);
                    let freq = hardware.cpu.live.core_frequencies_mhz.get(i).copied().unwrap_or(0.0);
                    cols[col_idx].vertical(|ui| {
                        load_gauge(
                            ui,
                            theme,
                            &format!("Núcleo #{i}"),
                            load,
                            &format!("{load:.0}% ({freq:.0} MHz)"),
                        );
                        ui.add_space(4.0);
                    });
                }
            });
        });

        ui.add_space(8.0);
    });
}
