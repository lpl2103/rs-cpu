//! Visual Themes and styling definitions for Modern CPU-Z.

use eframe::egui::{self, Color32, CornerRadius, Margin, Stroke, Visuals};
use serde::{Deserialize, Serialize};

/// Active visual theme selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppTheme {
    /// Modern Dark slate theme with cyan and emerald accents.
    Dark,
    /// Titanium Light clean theme with deep sapphire accents.
    Light,
}

impl AppTheme {
    /// Toggles between Dark and Light themes.
    #[must_use]
    pub const fn toggle(self) -> Self {
        match self {
            Self::Dark => Self::Light,
            Self::Light => Self::Dark,
        }
    }

    /// Primary background color.
    #[must_use]
    pub const fn bg_color(self) -> Color32 {
        match self {
            Self::Dark => Color32::from_rgb(15, 17, 23),
            Self::Light => Color32::from_rgb(244, 246, 250),
        }
    }

    /// Container / Card background color.
    #[must_use]
    pub const fn card_bg(self) -> Color32 {
        match self {
            Self::Dark => Color32::from_rgb(24, 27, 36),
            Self::Light => Color32::from_rgb(255, 255, 255),
        }
    }

    /// Subtle card border color.
    #[must_use]
    pub const fn card_border(self) -> Color32 {
        match self {
            Self::Dark => Color32::from_rgb(45, 52, 68),
            Self::Light => Color32::from_rgb(222, 228, 238),
        }
    }

    /// Accent primary color (Cyan in dark, Royal Blue in light).
    #[must_use]
    pub const fn accent_primary(self) -> Color32 {
        match self {
            Self::Dark => Color32::from_rgb(0, 210, 255),
            Self::Light => Color32::from_rgb(14, 116, 222),
        }
    }

    /// Accent secondary / success color (Emerald green).
    #[must_use]
    pub const fn accent_secondary(self) -> Color32 {
        match self {
            Self::Dark => Color32::from_rgb(16, 185, 129),
            Self::Light => Color32::from_rgb(5, 150, 105),
        }
    }

    /// Primary text color.
    #[must_use]
    pub const fn text_primary(self) -> Color32 {
        match self {
            Self::Dark => Color32::from_rgb(243, 244, 246),
            Self::Light => Color32::from_rgb(17, 24, 39),
        }
    }

    /// Secondary / muted label text color.
    #[must_use]
    pub const fn text_secondary(self) -> Color32 {
        match self {
            Self::Dark => Color32::from_rgb(156, 163, 175),
            Self::Light => Color32::from_rgb(107, 114, 128),
        }
    }

    /// Applies theme visuals to egui context.
    pub fn apply(self, ctx: &egui::Context) {
        let mut visuals = match self {
            Self::Dark => Visuals::dark(),
            Self::Light => Visuals::light(),
        };

        visuals.override_text_color = Some(self.text_primary());
        visuals.panel_fill = self.bg_color();
        visuals.window_fill = self.bg_color();
        visuals.faint_bg_color = self.card_bg();
        visuals.extreme_bg_color = match self {
            Self::Dark => Color32::from_rgb(10, 12, 16),
            Self::Light => Color32::from_rgb(238, 242, 246),
        };

        visuals.widgets.noninteractive.bg_fill = self.card_bg();
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, self.card_border());
        visuals.widgets.noninteractive.corner_radius = CornerRadius::same(8);

        visuals.widgets.inactive.bg_fill = self.card_bg();
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, self.card_border());
        visuals.widgets.inactive.corner_radius = CornerRadius::same(6);

        visuals.widgets.hovered.bg_fill = match self {
            Self::Dark => Color32::from_rgb(36, 42, 56),
            Self::Light => Color32::from_rgb(235, 240, 248),
        };
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.5_f32, self.accent_primary());
        visuals.widgets.hovered.corner_radius = CornerRadius::same(6);

        visuals.widgets.active.bg_fill = self.accent_primary();
        visuals.widgets.active.corner_radius = CornerRadius::same(6);

        ctx.set_visuals(visuals);
    }

    /// Returns standard card frame styling.
    pub fn card_frame(self) -> egui::Frame {
        egui::Frame::new()
            .fill(self.card_bg())
            .stroke(Stroke::new(1.0_f32, self.card_border()))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::same(12))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_toggle() {
        let dark = AppTheme::Dark;
        assert_eq!(dark.toggle(), AppTheme::Light);
        assert_eq!(AppTheme::Light.toggle(), AppTheme::Dark);
    }

    #[test]
    fn test_theme_colors() {
        let dark = AppTheme::Dark;
        let light = AppTheme::Light;
        assert_ne!(dark.bg_color(), light.bg_color());
        assert_ne!(dark.text_primary(), light.text_primary());
    }
}
