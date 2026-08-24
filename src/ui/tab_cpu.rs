//! Detailed CPU View tab.

use super::theme::AppTheme;
use super::widgets::{feature_badge, load_gauge, section_header, spec_row};
use crate::hardware::SystemHardware;
use eframe::egui::{RichText, ScrollArea, Ui};

/// Renders the dedicated detailed CPU tab.
pub fn render(ui: &mut Ui, theme: AppTheme, hardware: &SystemHardware) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(4.0);

        // Processor General
        theme.card_frame().show(ui, |ui| {
            section_header(ui, theme, "🔲", "Processor Information");

            spec_row(ui, theme, "Name", &hardware.cpu.name);
            spec_row(ui, theme, "Code Name", &hardware.cpu.code_name);
            spec_row(ui, theme, "Package / Socket", &hardware.cpu.package_socket);
            spec_row(ui, theme, "Technology", &hardware.cpu.technology);
            spec_row(ui, theme, "Core VID", &hardware.cpu.core_voltage);
            spec_row(ui, theme, "Specification", &hardware.cpu.name);

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            ui.columns(2, |cols| {
                cols[0].vertical(|ui| {
                    spec_row(ui, theme, "Family", &format!("{:X}", hardware.cpu.family));
                    spec_row(ui, theme, "Model", &format!("{:X}", hardware.cpu.model));
                    spec_row(ui, theme, "Stepping", &format!("{}", hardware.cpu.stepping));
                });
                cols[1].vertical(|ui| {
                    spec_row(ui, theme, "Ext. Family", &format!("{:X}", hardware.cpu.ext_family));
                    spec_row(ui, theme, "Ext. Model", &format!("{:X}", hardware.cpu.ext_model));
                    spec_row(ui, theme, "Revision", &hardware.cpu.revision);
                });
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            ui.label(RichText::new("Instructions").size(12.0).color(theme.text_secondary()));
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                for inst in &hardware.cpu.instructions {
                    feature_badge(ui, theme, inst, true);
                }
            });
        });

        ui.add_space(8.0);

        // Clocks & Caches
        ui.columns(2, |cols| {
            cols[0].vertical(|ui| {
                theme.card_frame().show(ui, |ui| {
                    section_header(ui, theme, "⚡", "Live Clocks");

                    spec_row(ui, theme, "Core Speed", &format!("{:.1} MHz", hardware.cpu.live.avg_frequency_mhz));
                    spec_row(ui, theme, "Multiplier", &format!("x {:.1}", hardware.cpu.live.multiplier));
                    spec_row(ui, theme, "Bus Speed", &format!("{:.1} MHz", hardware.cpu.live.bus_speed_mhz));
                    spec_row(ui, theme, "Rated FSB", &format!("{:.1} MHz", hardware.cpu.live.bus_speed_mhz * 4.0));
                });
            });

            cols[1].vertical(|ui| {
                theme.card_frame().show(ui, |ui| {
                    section_header(ui, theme, "📦", "Cache Hierarchy");

                    for cache in &hardware.cpu.caches {
                        let label = format!("L{} {} Cache", cache.level, cache.cache_type);
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

        // Cores and Threads telemetry
        theme.card_frame().show(ui, |ui| {
            section_header(ui, theme, "📊", "Per-Core Activity");

            spec_row(ui, theme, "Cores", &format!("{}", hardware.cpu.physical_cores));
            spec_row(ui, theme, "Threads", &format!("{}", hardware.cpu.logical_threads));

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            // Display per core load gauges in a 2-column grid
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
                            &format!("Core #{i}"),
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
