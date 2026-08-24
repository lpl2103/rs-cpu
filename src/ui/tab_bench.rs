//! Benchmark tab with interactive Single & Multi Thread tests and reference comparisons.

use super::theme::AppTheme;
use super::widgets::{section_header, spec_row};
use crate::bench::{BenchManager, BenchStatus, REFERENCE_CPUS};
use crate::hardware::SystemHardware;
use eframe::egui::{self, Button, Color32, CornerRadius, RichText, ScrollArea, Ui};

/// Renders the Benchmark & Stress Test tab.
pub fn render(
    ui: &mut Ui,
    theme: AppTheme,
    hardware: &SystemHardware,
    bench: &mut BenchManager,
) {
    let current_status = {
        bench.status.lock().map_or(BenchStatus::Idle, |s| s.clone())
    };
    let current_single = bench.single_score.lock().map_or(0.0, |s| *s);
    let current_multi = bench.multi_score.lock().map_or(0.0, |m| *m);

    let is_busy = matches!(
        current_status,
        BenchStatus::RunningSingle { .. } | BenchStatus::RunningMulti { .. } | BenchStatus::StressTesting { .. }
    );

    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(4.0);

        // Benchmark Controls Card
        theme.card_frame().show(ui, |ui| {
            section_header(ui, theme, "📊", "CPU Benchmark & Stress Test");

            ui.horizontal(|ui| {
                ui.label(RichText::new("Reference Comparison:").size(12.5).color(theme.text_secondary()));
                
                let selected_ref = &REFERENCE_CPUS[bench.selected_ref_idx];
                egui::ComboBox::from_id_salt("bench_ref_selector")
                    .selected_text(RichText::new(format!("{} ({})", selected_ref.name, selected_ref.config)).color(theme.text_primary()))
                    .show_ui(ui, |ui| {
                        for (idx, r) in REFERENCE_CPUS.iter().enumerate() {
                            ui.selectable_value(&mut bench.selected_ref_idx, idx, format!("{} ({})", r.name, r.config));
                        }
                    });
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Action Buttons
            ui.horizontal(|ui| {
                let bench_btn_text = match current_status {
                    BenchStatus::RunningSingle { .. } => "Running Single-Thread...".to_string(),
                    BenchStatus::RunningMulti { .. } => "Running Multi-Thread...".to_string(),
                    _ => "🚀 Bench CPU".to_string(),
                };

                let bench_btn = Button::new(RichText::new(bench_btn_text).strong().size(13.0))
                    .min_size(egui::vec2(140.0, 32.0));

                if ui.add_enabled(!is_busy, bench_btn).clicked() {
                    bench.start_bench(hardware.cpu.logical_threads);
                }

                let stress_active = matches!(current_status, BenchStatus::StressTesting { .. });
                let stress_text = if stress_active {
                    "⏹ Stop Stress"
                } else {
                    "🔥 Stress CPU"
                };

                let stress_btn = Button::new(
                    RichText::new(stress_text)
                        .strong()
                        .size(13.0)
                        .color(if stress_active { Color32::from_rgb(239, 68, 68) } else { theme.text_primary() }),
                )
                .min_size(egui::vec2(130.0, 32.0));

                if ui.add(stress_btn).clicked() {
                    bench.toggle_stress(hardware.cpu.logical_threads);
                }

                if is_busy
                    && ui.button(RichText::new("Cancel").color(Color32::from_rgb(239, 68, 68))).clicked()
                {
                    bench.stop();
                }
            });

            // Status Progress display
            match &current_status {
                BenchStatus::RunningSingle { progress } => {
                    ui.add_space(8.0);
                    render_bench_progress_bar(ui, theme, "Benchmarking Single Thread...", *progress);
                }
                BenchStatus::RunningMulti { progress } => {
                    ui.add_space(8.0);
                    render_bench_progress_bar(ui, theme, "Benchmarking Multi Thread...", *progress);
                }
                BenchStatus::StressTesting { elapsed_secs } => {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("🔥 Stress Test in Progress:").color(Color32::from_rgb(239, 68, 68)).strong());
                        ui.label(RichText::new(format!("{elapsed_secs}s active")).color(theme.text_primary()));
                    });
                }
                _ => {}
            }
        });

        ui.add_space(8.0);

        // Scores Comparison Card
        let ref_cpu = &REFERENCE_CPUS[bench.selected_ref_idx];
        theme.card_frame().show(ui, |ui| {
            section_header(ui, theme, "🏆", "Benchmark Results & Comparison");

            // Single Thread Result
            ui.label(RichText::new("CPU Single Thread").size(13.5).color(theme.accent_primary()).strong());
            ui.add_space(4.0);
            render_score_comparison(
                ui,
                theme,
                "This Processor",
                current_single,
                ref_cpu.name,
                ref_cpu.single_score,
                1100.0,
            );

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // Multi Thread Result
            ui.label(RichText::new("CPU Multi Thread").size(13.5).color(theme.accent_primary()).strong());
            ui.add_space(4.0);
            let max_multi = (ref_cpu.multi_score.max(current_multi) * 1.25).max(18000.0);
            render_score_comparison(
                ui,
                theme,
                "This Processor",
                current_multi,
                ref_cpu.name,
                ref_cpu.multi_score,
                max_multi,
            );

            if current_single > 0.0 && current_multi > 0.0 {
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(6.0);
                let ratio = current_multi / current_single;
                spec_row(ui, theme, "Multi-Thread Ratio", &format!("{ratio:.2} x"));
            }
        });

        ui.add_space(8.0);
    });
}

fn render_bench_progress_bar(ui: &mut Ui, theme: AppTheme, label: &str, fraction: f32) {
    ui.label(RichText::new(label).size(12.0).color(theme.text_secondary()));
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 6.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, CornerRadius::same(3), Color32::from_rgb(40, 45, 60));
    let mut filled = rect;
    filled.set_width(rect.width() * fraction.clamp(0.0, 1.0));
    ui.painter().rect_filled(filled, CornerRadius::same(3), theme.accent_primary());
}

fn render_score_comparison(
    ui: &mut Ui,
    theme: AppTheme,
    label_a: &str,
    score_a: f64,
    label_b: &str,
    score_b: f64,
    max_scale: f64,
) {
    // Current CPU Bar
    ui.horizontal(|ui| {
        ui.label(RichText::new(label_a).size(12.0).color(theme.text_primary()).strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(format!("{score_a:.0}")).size(13.0).color(theme.accent_primary()).strong());
        });
    });

    let frac_a = (score_a / max_scale).clamp(0.0, 1.0) as f32;
    let (rect_a, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 12.0), egui::Sense::hover());
    ui.painter().rect_filled(rect_a, CornerRadius::same(4), match theme {
        AppTheme::Dark => Color32::from_rgb(30, 36, 48),
        AppTheme::Light => Color32::from_rgb(230, 235, 242),
    });
    if frac_a > 0.001 {
        let mut filled = rect_a;
        filled.set_width(rect_a.width() * frac_a);
        ui.painter().rect_filled(filled, CornerRadius::same(4), theme.accent_primary());
    }

    ui.add_space(4.0);

    // Reference CPU Bar
    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("Reference: {label_b}")).size(11.5).color(theme.text_secondary()));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(format!("{score_b:.0}")).size(12.5).color(theme.text_secondary()).strong());
        });
    });

    let frac_b = (score_b / max_scale).clamp(0.0, 1.0) as f32;
    let (rect_b, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 10.0), egui::Sense::hover());
    ui.painter().rect_filled(rect_b, CornerRadius::same(4), match theme {
        AppTheme::Dark => Color32::from_rgb(30, 36, 48),
        AppTheme::Light => Color32::from_rgb(230, 235, 242),
    });
    if frac_b > 0.001 {
        let mut filled = rect_b;
        filled.set_width(rect_b.width() * frac_b);
        ui.painter().rect_filled(
            filled,
            CornerRadius::same(4),
            match theme {
                AppTheme::Dark => Color32::from_rgb(80, 95, 120),
                AppTheme::Light => Color32::from_rgb(160, 175, 195),
            },
        );
    }
}
