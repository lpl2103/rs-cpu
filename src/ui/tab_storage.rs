//! Detailed Storage (SSD / `NVMe` / HDD) View tab (SSD-Z style).

use super::theme::AppTheme;
use super::widgets::{load_gauge, section_header, spec_row, stat_metric_box};
use crate::hardware::SystemHardware;
use eframe::egui::{self, Color32, CornerRadius, RichText, ScrollArea, Ui};

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
            // Métricas em Destaque (SSD-Z Live Cards)
            theme.card_frame().show(ui, |ui| {
                ui.columns(4, |cols| {
                    cols[0].vertical(|ui| {
                        stat_metric_box(ui, theme, "Saúde S.M.A.R.T.", &drive.health_status, "Status: Saudável");
                    });
                    cols[1].vertical(|ui| {
                        let temp_text = format!("{:.0} °C", drive.temperature_c);
                        stat_metric_box(ui, theme, "Temperatura", &temp_text, if drive.temperature_c < 55.0 { "Temperatura Ideal" } else { "Atenção Térmica" });
                    });
                    cols[2].vertical(|ui| {
                        let tbw_text = format!("{:.1} TB", drive.total_host_writes_tb);
                        stat_metric_box(ui, theme, "Total Escrito (TBW)", &tbw_text, "Host Writes");
                    });
                    cols[3].vertical(|ui| {
                        let hours_text = format!("{} hrs", drive.power_on_hours);
                        stat_metric_box(ui, theme, "Tempo de Uso", &hours_text, &format!("{} ciclos lig/desl", drive.power_cycles));
                    });
                });

                ui.add_space(8.0);

                // Gauge de Temperatura do SSD
                let temp_pct = ((drive.temperature_c - 20.0) / 60.0 * 100.0).clamp(0.0, 100.0);
                load_gauge(
                    ui,
                    theme,
                    "Temperatura Operacional do Drive",
                    temp_pct,
                    &format!("{:.1} °C", drive.temperature_c),
                );
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
                    });
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
