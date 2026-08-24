//! Detailed Storage (SSD / `NVMe` / HDD) View tab (SSD-Z style).

use super::theme::AppTheme;
use super::widgets::{load_gauge, section_header, spec_row, stat_metric_box};
use crate::hardware::SystemHardware;
use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, ScrollArea, Stroke, Ui};

/// State for the storage tab view.
#[derive(Debug, Clone, Default)]
pub struct StorageTabState {
    /// Currently selected storage drive index.
    pub selected_drive: usize,
}

/// Renders the SSD-Z style Storage diagnostics tab in Portuguese (PT-BR).
pub fn render(
    ui: &mut Ui,
    theme: AppTheme,
    hardware: &SystemHardware,
    state: &mut StorageTabState,
) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(4.0);

        // Seletor de Unidade de Armazenamento
        theme.card_frame().show(ui, |ui| {
            section_header(ui, theme, "💽", "Unidades de Armazenamento (SSD / NVMe / HDD)");

            ui.horizontal(|ui| {
                ui.label(RichText::new("Selecionar Disco:").size(14.0).color(theme.text_secondary()));

                let count = hardware.storage.drives.len();
                if state.selected_drive >= count {
                    state.selected_drive = 0;
                }

                let current_label = hardware.storage.drives.get(state.selected_drive).map_or_else(
                    || "Disco 0".to_string(),
                    |d| format!("{} ({} GB) - {}", d.model, d.capacity_gb as u64, d.interface),
                );

                egui::ComboBox::from_id_salt("storage_drive_selector")
                    .selected_text(RichText::new(current_label).size(13.5).color(theme.text_primary()))
                    .show_ui(ui, |ui| {
                        for (idx, drive) in hardware.storage.drives.iter().enumerate() {
                            let label = format!("Disco #{} - {} ({:.0} GB)", idx, drive.model, drive.capacity_gb);
                            ui.selectable_value(&mut state.selected_drive, idx, label);
                        }
                    });
            });
        });

        ui.add_space(8.0);

        if let Some(drive) = hardware.storage.drives.get(state.selected_drive) {
            // Avisos e Diagnósticos Automatizados S.M.A.R.T.
            theme.card_frame().show(ui, |ui| {
                section_header(ui, theme, "🛡", "Diagnósticos de Saúde & Alertas S.M.A.R.T.");

                for warning in &drive.diagnostic_warnings {
                    let is_crit = warning.contains("ALERTA");
                    let (bg, border, text_col) = if is_crit {
                        (
                            Color32::from_rgb(45, 20, 15),
                            Color32::from_rgb(239, 68, 68),
                            Color32::from_rgb(255, 220, 220),
                        )
                    } else {
                        (
                            match theme {
                                AppTheme::Dark => Color32::from_rgb(18, 38, 26),
                                AppTheme::Light => Color32::from_rgb(230, 250, 238),
                            },
                            theme.accent_secondary(),
                            theme.accent_secondary(),
                        )
                    };

                    egui::Frame::new()
                        .fill(bg)
                        .stroke(Stroke::new(1.0_f32, border))
                        .corner_radius(CornerRadius::same(6))
                        .inner_margin(Margin::symmetric(10, 6))
                        .show(ui, |ui| {
                            ui.label(RichText::new(warning).size(13.0).color(text_col).strong());
                        });
                    ui.add_space(4.0);
                }
            });

            ui.add_space(8.0);

            // Métricas em Destaque (SSD-Z Live Cards)
            theme.card_frame().show(ui, |ui| {
                ui.columns(4, |cols| {
                    cols[0].vertical(|ui| {
                        stat_metric_box(ui, theme, "Saúde S.M.A.R.T.", &drive.health_status, &format!("{:.0}% Vida Útil Flash", drive.wear_level_pct));
                    });
                    cols[1].vertical(|ui| {
                        let temp_text = format!("{:.0} °C", drive.temperature_c);
                        stat_metric_box(ui, theme, "Temperatura", &temp_text, if drive.temperature_c < 55.0 { "Temperatura Ideal" } else { "Atenção Térmica" });
                    });
                    cols[2].vertical(|ui| {
                        let realloc_text = format!("{} setores", drive.reallocated_sectors);
                        stat_metric_box(ui, theme, "Setores Realocados", &realloc_text, if drive.reallocated_sectors == 0 { "Nenhum Bloco Ruim" } else { "Crítico: Falha Física" });
                    });
                    cols[3].vertical(|ui| {
                        let days = drive.power_on_hours / 24;
                        let hours_rem = drive.power_on_hours % 24;
                        let hours_text = format!("{days}d {hours_rem}h");
                        stat_metric_box(ui, theme, "Tempo Total Ligado", &hours_text, &format!("{} ciclos de energia", drive.power_cycles));
                    });
                });

                ui.add_space(8.0);

                // Gauges
                ui.columns(2, |cols| {
                    cols[0].vertical(|ui| {
                        let temp_pct = ((drive.temperature_c - 20.0) / 60.0 * 100.0).clamp(0.0, 100.0);
                        load_gauge(ui, theme, "Temperatura Operacional", temp_pct, &format!("{:.1} °C", drive.temperature_c));
                    });
                    cols[1].vertical(|ui| {
                        load_gauge(ui, theme, "Saúde da Memória Flash", drive.wear_level_pct, &format!("{:.0}% restante", drive.wear_level_pct));
                    });
                });
            });

            ui.add_space(8.0);

            // Especificações do Controlador e Disco
            theme.card_frame().show(ui, |ui| {
                section_header(ui, theme, "🔍", "Especificações do Drive & Controlador");

                ui.columns(2, |cols| {
                    cols[0].vertical(|ui| {
                        spec_row(ui, theme, "Modelo do Drive", &drive.model);
                        spec_row(ui, theme, "Interface / Barramento", &drive.interface);
                        spec_row(ui, theme, "Formato Físico", &drive.form_factor);
                        spec_row(ui, theme, "Capacidade Total", &format!("{:.1} GBytes ({:.0} GB)", drive.capacity_gb, drive.capacity_gb));
                    });

                    cols[1].vertical(|ui| {
                        spec_row(ui, theme, "Número de Série", &drive.serial);
                        spec_row(ui, theme, "Versão de Firmware", &drive.firmware);
                        spec_row(ui, theme, "Tecnologia de Memória", &drive.technology);
                        spec_row(ui, theme, "Total Lido (TBR)", &format!("{:.1} TB", drive.total_host_reads_tb));
                        spec_row(ui, theme, "Total Escrito (TBW)", &format!("{:.1} TB", drive.total_host_writes_tb));
                    });
                });
            });

            ui.add_space(8.0);

            // Tabela Detalhada de Atributos S.M.A.R.T.
            theme.card_frame().show(ui, |ui| {
                section_header(ui, theme, "📊", "Tabela de Atributos S.M.A.R.T. Principais");

                egui::Grid::new("smart_attributes_table_grid")
                    .striped(true)
                    .min_col_width(85.0)
                    .show(ui, |ui| {
                        ui.label(RichText::new("ID").strong().color(theme.text_secondary()));
                        ui.label(RichText::new("Nome do Atributo").strong().color(theme.accent_primary()));
                        ui.label(RichText::new("Valor Atual").strong().color(theme.text_primary()));
                        ui.label(RichText::new("Limite Limiar").strong().color(theme.text_secondary()));
                        ui.label(RichText::new("Status").strong().color(theme.text_primary()));
                        ui.end_row();

                        for attr in &drive.smart_attributes {
                            ui.label(RichText::new(&attr.id).strong().color(theme.text_secondary()));
                            ui.label(RichText::new(&attr.name).color(theme.text_primary()));
                            ui.label(RichText::new(&attr.value_str).strong().color(theme.text_primary()));
                            ui.label(RichText::new(&attr.threshold_str).color(theme.text_secondary()));

                            let status_color = if attr.status == "Normal" {
                                theme.accent_secondary()
                            } else {
                                Color32::from_rgb(239, 68, 68)
                            };
                            ui.label(RichText::new(&attr.status).strong().color(status_color));
                            ui.end_row();
                        }
                    });
            });

            ui.add_space(8.0);

            // Mapa de Partições e Volumes
            theme.card_frame().show(ui, |ui| {
                section_header(ui, theme, "📁", "Partições & Volumes Montados");

                if drive.partitions.is_empty() {
                    ui.label(RichText::new("Nenhuma partição montada associada a esta unidade física.").color(theme.text_secondary()));
                } else {
                    for part in &drive.partitions {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("Volume [{}]", part.mount_point)).strong().size(14.0).color(theme.accent_primary()));
                            ui.label(RichText::new(format!("{} ({})", part.name, part.file_system)).size(13.0).color(theme.text_secondary()));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(RichText::new(format!("{:.1} GB livres de {:.1} GB", part.free_gb, part.total_gb)).strong().size(13.0).color(theme.text_primary()));
                            });
                        });

                        ui.add_space(4.0);
                        let fraction = (part.used_pct / 100.0).clamp(0.0, 1.0);
                        let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 10.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, CornerRadius::same(5), match theme {
                            AppTheme::Dark => Color32::from_rgb(32, 38, 50),
                            AppTheme::Light => Color32::from_rgb(220, 226, 236),
                        });
                        if fraction > 0.001 {
                            let mut filled = rect;
                            filled.set_width(rect.width() * fraction);
                            ui.painter().rect_filled(filled, CornerRadius::same(5), theme.accent_primary());
                        }
                        ui.add_space(6.0);
                    }
                }
            });
        }

        ui.add_space(8.0);
    });
}
