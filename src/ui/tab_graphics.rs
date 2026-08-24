//! Detailed Graphics View tab (GPU-Z style).

use super::theme::AppTheme;
use super::widgets::{brand_logo_badge, gpu_tech_badge, load_gauge, section_header, spec_row, stat_metric_box};
use crate::hardware::SystemHardware;
use eframe::egui::{self, RichText, ScrollArea, Ui};

/// State for graphics tab.
#[derive(Debug, Clone, Default)]
pub struct GraphicsTabState {
    /// Currently selected GPU adapter index.
    pub selected_gpu: usize,
}

/// Renders the enhanced GPU-Z style Graphics tab in Portuguese (PT-BR).
pub fn render(
    ui: &mut Ui,
    theme: AppTheme,
    hardware: &SystemHardware,
    state: &mut GraphicsTabState,
) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(4.0);

        if let Some(gpu) = hardware.gpus.get(state.selected_gpu) {
            // Cartão Superior: Identificação e Logo da Marca
            theme.card_frame().show(ui, |ui| {
                ui.horizontal(|ui| {
                    section_header(ui, theme, "🎮", "Placa Gráfica (GPU-Z)");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        brand_logo_badge(ui, &gpu.vendor, &gpu.name);
                    });
                });

                if hardware.gpus.len() > 1 {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Selecionar Placa Gráfica:").size(13.5).color(theme.text_secondary()));
                        egui::ComboBox::from_id_salt("gpu_selector")
                            .selected_text(RichText::new(&gpu.name).size(13.5).color(theme.text_primary()))
                            .show_ui(ui, |ui| {
                                for (idx, g) in hardware.gpus.iter().enumerate() {
                                    ui.selectable_value(&mut state.selected_gpu, idx, &g.name);
                                }
                            });
                    });
                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(6.0);
                }

                ui.columns(2, |cols| {
                    cols[0].vertical(|ui| {
                        spec_row(ui, theme, "Nome da GPU", &gpu.name);
                        spec_row(ui, theme, "Fabricante / Subvendor", &gpu.subvendor);
                        spec_row(ui, theme, "Codinome / GPU Core", &gpu.code_name);
                        spec_row(ui, theme, "Litografia / Processo", &gpu.technology);
                        spec_row(ui, theme, "Tamanho do Die", &gpu.die_size);
                        spec_row(ui, theme, "Transistores", &gpu.transistors);
                    });

                    cols[1].vertical(|ui| {
                        spec_row(ui, theme, "Shaders / CUDA Cores", &format!("{} Unificados", gpu.shaders));
                        spec_row(ui, theme, "Texture Fillrate", &format!("{:.1} GTexel/s", gpu.texture_fillrate));
                        spec_row(ui, theme, "Pixel Fillrate", &format!("{:.1} GPixel/s", gpu.pixel_fillrate));
                        spec_row(ui, theme, "Versão do Driver", &gpu.driver_version);
                        spec_row(ui, theme, "Data do Driver", &gpu.driver_date);
                        spec_row(ui, theme, "Suporte DirectX", "DirectX 12 (FL 12_2)");
                    });
                });
            });

            ui.add_space(8.0);

            // Sensores ao Vivo (GPU-Z Live Telemetry)
            theme.card_frame().show(ui, |ui| {
                section_header(ui, theme, "⚡", "Sensores & Telemetria em Tempo Real (GPU-Z)");

                ui.columns(4, |cols| {
                    cols[0].vertical(|ui| {
                        let temp_text = format!("{:.0} °C", gpu.live_temp_c);
                        stat_metric_box(ui, theme, "Temperatura GPU", &temp_text, if gpu.live_temp_c < 55.0 { "Excelente" } else { "Aquecida" });
                    });
                    cols[1].vertical(|ui| {
                        let load_text = format!("{:.1}%", gpu.live_load_pct);
                        stat_metric_box(ui, theme, "Carga do Núcleo", &load_text, "GPU Core Load");
                    });
                    cols[2].vertical(|ui| {
                        let fan_text = if gpu.live_fan_rpm > 0 { format!("{} RPM", gpu.live_fan_rpm) } else { "0 RPM (Silencioso)".to_string() };
                        stat_metric_box(ui, theme, "Ventoinha GPU", &fan_text, "Fan Speed");
                    });
                    cols[3].vertical(|ui| {
                        let pwr_text = format!("{:.1} W", gpu.live_power_w);
                        stat_metric_box(ui, theme, "Consumo Estimado", &pwr_text, "Board Power");
                    });
                });

                ui.add_space(8.0);

                ui.columns(2, |cols| {
                    cols[0].vertical(|ui| {
                        let temp_pct = ((gpu.live_temp_c - 30.0) / 60.0 * 100.0).clamp(0.0, 100.0);
                        load_gauge(ui, theme, "Temperatura da GPU", temp_pct, &format!("{:.1} °C", gpu.live_temp_c));
                    });
                    cols[1].vertical(|ui| {
                        load_gauge(ui, theme, "Uso do Processador Gráfico", gpu.live_load_pct, &format!("{:.1}%", gpu.live_load_pct));
                    });
                });
            });

            ui.add_space(8.0);

            // Memória de Vídeo (VRAM) & Clocks
            ui.columns(2, |cols| {
                cols[0].vertical(|ui| {
                    theme.card_frame().show(ui, |ui| {
                        section_header(ui, theme, "💾", "Memória de Vídeo (VRAM)");

                        spec_row(ui, theme, "Capacidade VRAM", &format!("{} MBytes ({:.0} GB)", gpu.vram_mb, gpu.vram_mb / 1024));
                        spec_row(ui, theme, "Tipo de Memória", &gpu.memory_type);
                        spec_row(ui, theme, "Largura de Barramento", &gpu.bus_width);
                        spec_row(ui, theme, "Largura de Banda", &format!("{:.1} GB/s", gpu.bandwidth_gbs));
                    });
                });

                cols[1].vertical(|ui| {
                    theme.card_frame().show(ui, |ui| {
                        section_header(ui, theme, "⏱", "Frequências de Clock");

                        spec_row(ui, theme, "Clock Base", &format!("{} MHz", gpu.base_clock_mhz));
                        spec_row(ui, theme, "Clock Boost", &format!("{} MHz", gpu.boost_clock_mhz));
                        spec_row(ui, theme, "Clock de Memória", &format!("{} MHz", gpu.memory_clock_mhz));
                    });
                });
            });

            ui.add_space(8.0);

            // Tecnologias Gráficas com Tooltips Interativos
            theme.card_frame().show(ui, |ui| {
                section_header(ui, theme, "✨", "Tecnologias & Recursos Suportados (Passe o cursor)");

                ui.horizontal_wrapped(|ui| {
                    for tech in &gpu.technologies {
                        gpu_tech_badge(ui, theme, tech);
                    }
                });
            });
        }

        ui.add_space(8.0);
    });
}
