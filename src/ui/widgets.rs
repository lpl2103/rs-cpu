//! Reusable modern UI widgets and presentation helpers.

use super::theme::AppTheme;
use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke, Ui};

/// Renders a section header with icon and title.
pub fn section_header(ui: &mut Ui, theme: AppTheme, icon: &str, title: &str) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(icon)
                .size(18.0)
                .color(theme.accent_primary())
                .strong(),
        );
        ui.label(
            RichText::new(title)
                .size(15.0)
                .color(theme.text_primary())
                .strong(),
        );
    });
    ui.add_space(6.0);
}

/// Renders a key-value specification row.
pub fn spec_row(ui: &mut Ui, theme: AppTheme, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.set_min_height(20.0);
        ui.label(
            RichText::new(label)
                .size(12.5)
                .color(theme.text_secondary()),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(value)
                    .size(12.5)
                    .color(theme.text_primary())
                    .strong(),
            );
        });
    });
}

/// Renders a compact instruction set or feature badge chip.
pub fn feature_badge(ui: &mut Ui, theme: AppTheme, text: &str, active: bool) {
    let (bg, border, text_col) = if active {
        (
            match theme {
                AppTheme::Dark => Color32::from_rgb(16, 40, 50),
                AppTheme::Light => Color32::from_rgb(225, 242, 254),
            },
            theme.accent_primary(),
            theme.accent_primary(),
        )
    } else {
        (
            match theme {
                AppTheme::Dark => Color32::from_rgb(28, 32, 40),
                AppTheme::Light => Color32::from_rgb(240, 242, 245),
            },
            match theme {
                AppTheme::Dark => Color32::from_rgb(45, 50, 60),
                AppTheme::Light => Color32::from_rgb(210, 215, 222),
            },
            theme.text_secondary(),
        )
    };

    egui::Frame::new()
        .fill(bg)
        .stroke(Stroke::new(1.0_f32, border))
        .corner_radius(CornerRadius::same(5))
        .inner_margin(Margin::symmetric(6, 3))
        .show(ui, |ui| {
            ui.label(
                RichText::new(text)
                    .size(11.0)
                    .color(text_col)
                    .strong(),
            );
        });
}

/// Renders a modern progress/load gauge.
pub fn load_gauge(ui: &mut Ui, theme: AppTheme, label: &str, pct: f32, subtext: &str) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(label)
                .size(12.0)
                .color(theme.text_secondary()),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(subtext)
                    .size(12.0)
                    .color(theme.text_primary())
                    .strong(),
            );
        });
    });

    let fraction = (pct / 100.0).clamp(0.0, 1.0);
    let bar_color = if fraction > 0.85 {
        Color32::from_rgb(239, 68, 68) // Red
    } else if fraction > 0.60 {
        Color32::from_rgb(245, 158, 11) // Amber
    } else {
        theme.accent_secondary() // Green / Cyan
    };

    let desired_width = ui.available_width();
    let (rect, _response) = ui.allocate_exact_size(egui::vec2(desired_width, 8.0), egui::Sense::hover());
    
    // Background track
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(4),
        match theme {
            AppTheme::Dark => Color32::from_rgb(35, 40, 52),
            AppTheme::Light => Color32::from_rgb(225, 230, 238),
        },
    );

    // Filled progress
    if fraction > 0.001 {
        let mut filled_rect = rect;
        filled_rect.set_width(rect.width() * fraction);
        ui.painter().rect_filled(filled_rect, CornerRadius::same(4), bar_color);
    }
}

/// Renders a high-impact metric stat box.
pub fn stat_metric_box(ui: &mut Ui, theme: AppTheme, title: &str, value: &str, sub: &str) {
    egui::Frame::new()
        .fill(match theme {
            AppTheme::Dark => Color32::from_rgb(30, 35, 48),
            AppTheme::Light => Color32::from_rgb(240, 244, 250),
        })
        .stroke(Stroke::new(1.0_f32, theme.card_border()))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new(title).size(10.5).color(theme.text_secondary()));
                ui.label(RichText::new(value).size(15.0).color(theme.text_primary()).strong());
                ui.label(RichText::new(sub).size(10.0).color(theme.accent_primary()));
            });
        });
}
