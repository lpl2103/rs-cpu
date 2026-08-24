//! Detailed Graphics View tab.

use super::theme::AppTheme;
use super::widgets::{section_header, spec_row};
use crate::hardware::SystemHardware;
use eframe::egui::{self, RichText, ScrollArea, Ui};

/// State for graphics tab.
#[derive(Debug, Clone, Default)]
pub struct GraphicsTabState {
    /// Currently selected GPU adapter index.
    pub selected_gpu: usize,
}

/// Renders the dedicated Graphics tab in Portuguese (PT-BR).
pub fn render(
    ui: &mut Ui,
    theme: AppTheme,
    hardware: &SystemHardware,
    state: &mut GraphicsTabState,
) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(4.0);

        theme.card_frame().show(ui, |ui| {
            section_header(ui, theme, "🎮", "Dispositivo de Exibição");

            if hardware.gpus.len() > 1 {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Selecionar Placa Gráfica:").size(13.5).color(theme.text_secondary()));
                    egui::ComboBox::from_id_salt("gpu_selector")
                        .selected_text(
                            hardware.gpus.get(state.selected_gpu).map_or("GPU 0", |g| g.name.as_str()),
                        )
                        .show_ui(ui, |ui| {
                            for (idx, gpu) in hardware.gpus.iter().enumerate() {
                                ui.selectable_value(&mut state.selected_gpu, idx, &gpu.name);
                            }
                        });
                });
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);
            }

            if let Some(gpu) = hardware.gpus.get(state.selected_gpu) {
                ui.columns(2, |cols| {
                    cols[0].vertical(|ui| {
                        spec_row(ui, theme, "Nome da GPU", &gpu.name);
                        spec_row(ui, theme, "Fabricante da Placa", &gpu.vendor);
                        spec_row(ui, theme, "Codinome / Arquitetura", &gpu.code_name);
                        spec_row(ui, theme, "Litografia / Processo", &gpu.technology);
                        spec_row(ui, theme, "Versão do Driver", &gpu.driver_version);
                    });

                    cols[1].vertical(|ui| {
                        spec_row(ui, theme, "Memória de Vídeo (VRAM)", &format!("{} MBytes ({} GB)", gpu.vram_mb, gpu.vram_mb / 1024));
                        spec_row(ui, theme, "Tipo de Memória", &gpu.memory_type);
                        spec_row(ui, theme, "Largura do Barramento", &gpu.bus_width);
                        spec_row(ui, theme, "Clock do Núcleo", &format!("{} MHz", gpu.core_clock_mhz));
                        spec_row(ui, theme, "Clock da Memória", &format!("{} MHz", gpu.memory_clock_mhz));
                    });
                });
            }
        });

        ui.add_space(8.0);
    });
}
