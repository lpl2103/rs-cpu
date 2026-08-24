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

/// Renders the dedicated Graphics tab.
pub fn render(
    ui: &mut Ui,
    theme: AppTheme,
    hardware: &SystemHardware,
    state: &mut GraphicsTabState,
) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(4.0);

        theme.card_frame().show(ui, |ui| {
            section_header(ui, theme, "🎮", "Display Device");

            if hardware.gpus.len() > 1 {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Select Display Device:").color(theme.text_secondary()));
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
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);
            }

            if let Some(gpu) = hardware.gpus.get(state.selected_gpu) {
                ui.columns(2, |cols| {
                    cols[0].vertical(|ui| {
                        spec_row(ui, theme, "Name", &gpu.name);
                        spec_row(ui, theme, "Board Manuf.", &gpu.vendor);
                        spec_row(ui, theme, "Code Name", &gpu.code_name);
                        spec_row(ui, theme, "Technology", &gpu.technology);
                        spec_row(ui, theme, "Driver Version", &gpu.driver_version);
                    });

                    cols[1].vertical(|ui| {
                        spec_row(ui, theme, "Memory Size", &format!("{} MBytes ({} GB)", gpu.vram_mb, gpu.vram_mb / 1024));
                        spec_row(ui, theme, "Memory Type", &gpu.memory_type);
                        spec_row(ui, theme, "Bus Width", &gpu.bus_width);
                        spec_row(ui, theme, "Core Clock", &format!("{} MHz", gpu.core_clock_mhz));
                        spec_row(ui, theme, "Memory Clock", &format!("{} MHz", gpu.memory_clock_mhz));
                    });
                });
            }
        });

        ui.add_space(8.0);
    });
}
