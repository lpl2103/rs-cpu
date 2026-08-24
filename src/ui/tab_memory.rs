//! Detailed Memory & SPD View tab.

use super::theme::AppTheme;
use super::widgets::{load_gauge, section_header, spec_row};
use crate::hardware::SystemHardware;
use eframe::egui::{self, RichText, ScrollArea, Ui};

/// State for the memory & SPD tab view.
#[derive(Debug, Clone, Default)]
pub struct MemoryTabState {
    /// Currently selected SPD slot index.
    pub selected_slot: usize,
}

/// Renders the dedicated detailed Memory & SPD tab.
pub fn render(
    ui: &mut Ui,
    theme: AppTheme,
    hardware: &SystemHardware,
    state: &mut MemoryTabState,
) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(4.0);

        // General Memory Info
        theme.card_frame().show(ui, |ui| {
            section_header(ui, theme, "💾", "General Memory");

            let total_gb = (hardware.memory.total_mb as f32) / 1024.0;
            spec_row(ui, theme, "Type", &hardware.memory.memory_type);
            spec_row(ui, theme, "Size", &format!("{total_gb:.1} GBytes ({:.0} MB)", hardware.memory.total_mb));
            spec_row(ui, theme, "Channel #", &hardware.memory.channel_mode);
            spec_row(ui, theme, "DC Mode", "Symmetric");
            spec_row(ui, theme, "Uncore Frequency", &format!("{:.1} MHz", hardware.memory.uncore_frequency_mhz));

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            ui.label(RichText::new("Timings").size(12.5).color(theme.accent_primary()).strong());
            ui.add_space(2.0);
            spec_row(ui, theme, "DRAM Frequency", &format!("{:.1} MHz", hardware.memory.dram_frequency_mhz));
            spec_row(ui, theme, "FSB:DRAM", &hardware.memory.fsb_dram_ratio);
            spec_row(ui, theme, "CAS# Latency (CL)", &format!("{} clocks", hardware.memory.cl));
            spec_row(ui, theme, "RAS# to CAS# Delay (tRCD)", &format!("{} clocks", hardware.memory.trcd));
            spec_row(ui, theme, "RAS# Precharge (tRP)", &format!("{} clocks", hardware.memory.trp));
            spec_row(ui, theme, "Cycle Time (tRAS)", &format!("{} clocks", hardware.memory.tras));
            spec_row(ui, theme, "Bank Cycle Time (tRC)", &format!("{} clocks", hardware.memory.trc));
            spec_row(ui, theme, "Command Rate (CR)", &hardware.memory.command_rate);

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            load_gauge(
                ui,
                theme,
                "RAM Utilization",
                hardware.memory.live.usage_pct,
                &format!("{} MB used / {} MB total ({:.1}%)", hardware.memory.live.used_mb, hardware.memory.total_mb, hardware.memory.live.usage_pct),
            );
        });

        ui.add_space(8.0);

        // SPD (Serial Presence Detect) Section
        theme.card_frame().show(ui, |ui| {
            section_header(ui, theme, "🔍", "Memory Slot SPD (Serial Presence Detect)");

            // Slot Selector dropdown
            ui.horizontal(|ui| {
                ui.label(RichText::new("Memory Slot Selection:").size(12.5).color(theme.text_secondary()));
                
                let slots_count = hardware.memory.slots.len();
                if state.selected_slot >= slots_count {
                    state.selected_slot = 0;
                }

                let current_label = hardware.memory.slots.get(state.selected_slot).map_or_else(
                    || "Slot #1".to_string(),
                    |s| {
                        if s.size_mb > 0 {
                            format!("{} - {} ({} MB)", s.slot_name, s.memory_type, s.size_mb)
                        } else {
                            format!("{} - [Empty]", s.slot_name)
                        }
                    },
                );

                egui::ComboBox::from_id_salt("spd_slot_select")
                    .selected_text(RichText::new(current_label).size(12.0).color(theme.text_primary()))
                    .show_ui(ui, |ui| {
                        for (idx, slot) in hardware.memory.slots.iter().enumerate() {
                            let label = if slot.size_mb > 0 {
                                format!("{} - {} ({} MB)", slot.slot_name, slot.memory_type, slot.size_mb)
                            } else {
                                format!("{} - [Empty]", slot.slot_name)
                            };
                            ui.selectable_value(&mut state.selected_slot, idx, label);
                        }
                    });
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            if let Some(slot) = hardware.memory.slots.get(state.selected_slot) {
                if slot.size_mb > 0 {
                    ui.columns(2, |cols| {
                        cols[0].vertical(|ui| {
                            spec_row(ui, theme, "Module Size", &format!("{} MBytes ({} GB)", slot.size_mb, slot.size_mb / 1024));
                            spec_row(ui, theme, "Max Bandwidth", &format!("{}-{} ({} MHz)", slot.memory_type, slot.speed_mhz, slot.speed_mhz / 2));
                            spec_row(ui, theme, "Configured Speed", &format!("{} MT/s", slot.configured_speed_mhz));
                            spec_row(ui, theme, "Module Manuf.", &slot.module_manufacturer);
                            spec_row(ui, theme, "DRAM Manuf.", &slot.dram_manufacturer);
                        });

                        cols[1].vertical(|ui| {
                            spec_row(ui, theme, "Part Number", &slot.part_number);
                            spec_row(ui, theme, "Serial Number", &slot.serial_number);
                            spec_row(ui, theme, "Form Factor", &slot.form_factor);
                            spec_row(ui, theme, "Nominal Voltage", &format!("{:.2} V", slot.voltage));
                            spec_row(ui, theme, "Bank Locator", &slot.bank_locator);
                        });
                    });

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(6.0);

                    // Timings Table
                    ui.label(RichText::new("Timings Table").size(12.5).color(theme.accent_primary()).strong());
                    ui.add_space(4.0);

                    egui::Grid::new("timings_table_grid")
                        .striped(true)
                        .min_col_width(70.0)
                        .show(ui, |ui| {
                            ui.label(RichText::new("Profile").strong().color(theme.text_secondary()));
                            for p in &slot.profiles {
                                ui.label(RichText::new(&p.name).strong().color(theme.accent_primary()));
                            }
                            ui.end_row();

                            ui.label(RichText::new("Frequency").color(theme.text_secondary()));
                            for p in &slot.profiles {
                                ui.label(RichText::new(format!("{} MHz", p.frequency_mhz)).color(theme.text_primary()));
                            }
                            ui.end_row();

                            ui.label(RichText::new("CAS# Latency").color(theme.text_secondary()));
                            for p in &slot.profiles {
                                ui.label(RichText::new(format!("{}", p.cas_latency)).color(theme.text_primary()));
                            }
                            ui.end_row();

                            ui.label(RichText::new("RAS# to CAS#").color(theme.text_secondary()));
                            for p in &slot.profiles {
                                ui.label(RichText::new(format!("{}", p.trcd)).color(theme.text_primary()));
                            }
                            ui.end_row();

                            ui.label(RichText::new("RAS# Precharge").color(theme.text_secondary()));
                            for p in &slot.profiles {
                                ui.label(RichText::new(format!("{}", p.trp)).color(theme.text_primary()));
                            }
                            ui.end_row();

                            ui.label(RichText::new("tRAS").color(theme.text_secondary()));
                            for p in &slot.profiles {
                                ui.label(RichText::new(format!("{}", p.tras)).color(theme.text_primary()));
                            }
                            ui.end_row();

                            ui.label(RichText::new("tRC").color(theme.text_secondary()));
                            for p in &slot.profiles {
                                ui.label(RichText::new(format!("{}", p.trc)).color(theme.text_primary()));
                            }
                            ui.end_row();

                            ui.label(RichText::new("Voltage").color(theme.text_secondary()));
                            for p in &slot.profiles {
                                ui.label(RichText::new(format!("{:.2} V", p.voltage)).color(theme.text_primary()));
                            }
                            ui.end_row();
                        });
                } else {
                    ui.label(RichText::new("Slot is empty / unpopulated.").color(theme.text_secondary()));
                }
            }
        });

        ui.add_space(8.0);
    });
}
