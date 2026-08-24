//! Main Application State and egui Frame integration.

use crate::bench::BenchManager;
use crate::hardware::HardwareEngine;
use crate::ui::theme::AppTheme;
use crate::ui::{
    tab_about, tab_bench, tab_cpu, tab_graphics, tab_mainboard, tab_memory, tab_unified,
    AboutTabState, GraphicsTabState, MemoryTabState,
};
use eframe::egui::{self, Button, CentralPanel, Color32, CornerRadius, RichText, TopBottomPanel};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Active navigation tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    /// Unified Dashboard (CPU, Motherboard, Memory all together).
    Unified,
    /// Detailed CPU tab.
    Cpu,
    /// Detailed Motherboard tab.
    Mainboard,
    /// Detailed Memory & SPD tab.
    Memory,
    /// Detailed Graphics tab.
    Graphics,
    /// Benchmark & Stress Test tab.
    Bench,
    /// About & Report Export tab.
    About,
}

/// Main Application state for M-CPU.
pub struct ModernCpuZApp {
    /// Hardware telemetry and introspection engine.
    pub engine: HardwareEngine,
    /// CPU Benchmark manager.
    pub bench: BenchManager,
    /// Currently active navigation tab.
    pub active_tab: ActiveTab,
    /// Active visual theme.
    pub theme: AppTheme,
    /// Memory & SPD tab state.
    pub memory_state: MemoryTabState,
    /// Graphics tab state.
    pub graphics_state: GraphicsTabState,
    /// About tab state.
    pub about_state: AboutTabState,
    /// Timestamp of last telemetry update.
    pub last_update: Instant,
}

impl ModernCpuZApp {
    /// Creates and initializes the application with Segoe UI typography and hardware engine.
    #[must_use]
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        tracing::info!("Initializing M-CPU Hardware Engine & Typography...");

        // Load and apply Segoe UI font
        let mut fonts = egui::FontDefinitions::default();
        let mut font_loaded = false;

        #[cfg(target_os = "windows")]
        {
            if let Ok(font_data) = std::fs::read("C:\\Windows\\Fonts\\segoeui.ttf") {
                fonts.font_data.insert(
                    "segoe_ui".to_owned(),
                    Arc::new(egui::FontData::from_owned(font_data)),
                );
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, "segoe_ui".to_owned());
                font_loaded = true;
            }

            if let Ok(bold_data) = std::fs::read("C:\\Windows\\Fonts\\segoeuib.ttf") {
                fonts.font_data.insert(
                    "segoe_ui_bold".to_owned(),
                    Arc::new(egui::FontData::from_owned(bold_data)),
                );
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Standard Linux font paths
            let linux_paths = [
                "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
                "/usr/share/fonts/TTF/DejaVuSans.ttf",
            ];
            for path in linux_paths {
                if let Ok(font_data) = std::fs::read(path) {
                    fonts.font_data.insert(
                        "system_font".to_owned(),
                        Arc::new(egui::FontData::from_owned(font_data)),
                    );
                    fonts
                        .families
                        .entry(egui::FontFamily::Proportional)
                        .or_default()
                        .insert(0, "system_font".to_owned());
                    font_loaded = true;
                    break;
                }
            }
        }

        if font_loaded {
            cc.egui_ctx.set_fonts(fonts);
        }

        let engine = HardwareEngine::new();
        let bench = BenchManager::default();

        Self {
            engine,
            bench,
            active_tab: ActiveTab::Unified,
            theme: AppTheme::Dark,
            memory_state: MemoryTabState::default(),
            graphics_state: GraphicsTabState::default(),
            about_state: AboutTabState::default(),
            last_update: Instant::now(),
        }
    }
}

impl eframe::App for ModernCpuZApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Periodic non-blocking telemetry refresh (every 500ms)
        if self.last_update.elapsed() >= Duration::from_millis(500) {
            self.engine.refresh_live_metrics();
            self.last_update = Instant::now();
        }

        // Apply theme styling
        self.theme.apply(ctx);

        // Top Navigation Bar
        TopBottomPanel::top("top_panel")
            .frame(
                egui::Frame::new()
                    .fill(self.theme.card_bg())
                    .stroke(egui::Stroke::new(1.0_f32, self.theme.card_border()))
                    .inner_margin(egui::Margin::symmetric(14, 10)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Logo / Title
                    ui.label(
                        RichText::new("⚡ M-CPU")
                            .size(18.0)
                            .color(self.theme.accent_primary())
                            .strong(),
                    );
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Tab Navigation Buttons (PT-BR)
                    let tabs = [
                        (ActiveTab::Unified, "🖥️ Visão Geral"),
                        (ActiveTab::Cpu, "🔲 Processador"),
                        (ActiveTab::Mainboard, "🖧 Placa-Mãe"),
                        (ActiveTab::Memory, "💾 Memória & SPD"),
                        (ActiveTab::Graphics, "🎮 Gráficos"),
                        (ActiveTab::Bench, "📊 Benchmark"),
                        (ActiveTab::About, "ℹ️ Sobre"),
                    ];

                    for (tab, label) in tabs {
                        let is_active = self.active_tab == tab;
                        let text = if is_active {
                            RichText::new(label)
                                .size(13.5)
                                .color(self.theme.accent_primary())
                                .strong()
                        } else {
                            RichText::new(label)
                                .size(13.5)
                                .color(self.theme.text_secondary())
                        };

                        let btn = Button::new(text)
                            .fill(if is_active {
                                match self.theme {
                                    AppTheme::Dark => Color32::from_rgb(30, 42, 60),
                                    AppTheme::Light => Color32::from_rgb(222, 235, 252),
                                }
                            } else {
                                Color32::TRANSPARENT
                            })
                            .corner_radius(CornerRadius::same(6))
                            .min_size(egui::vec2(0.0, 26.0));

                        if ui.add(btn).clicked() {
                            self.active_tab = tab;
                        }
                    }

                    // Right-aligned controls (Theme Toggle and Manual Refresh)
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let theme_icon = match self.theme {
                            AppTheme::Dark => "☀️ Claro",
                            AppTheme::Light => "🌙 Escuro",
                        };

                        if ui
                            .button(RichText::new(theme_icon).size(13.0).color(self.theme.text_primary()))
                            .on_hover_text("Alternar entre Tema Claro e Escuro")
                            .clicked()
                        {
                            self.theme = self.theme.toggle();
                        }

                        if ui
                            .button(RichText::new("🔄 Atualizar").size(13.0).color(self.theme.text_primary()))
                            .on_hover_text("Atualizar toda a telemetria de hardware agora")
                            .clicked()
                        {
                            self.engine.refresh_live_metrics();
                        }
                    });
                });
            });

        // Main Central Content Panel
        CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(self.theme.bg_color())
                    .inner_margin(egui::Margin::same(12)),
            )
            .show(ctx, |ui| match self.active_tab {
                ActiveTab::Unified => {
                    tab_unified::render(ui, self.theme, &self.engine.data);
                }
                ActiveTab::Cpu => {
                    tab_cpu::render(ui, self.theme, &self.engine.data);
                }
                ActiveTab::Mainboard => {
                    tab_mainboard::render(ui, self.theme, &self.engine.data);
                }
                ActiveTab::Memory => {
                    tab_memory::render(ui, self.theme, &self.engine.data, &mut self.memory_state);
                }
                ActiveTab::Graphics => {
                    tab_graphics::render(ui, self.theme, &self.engine.data, &mut self.graphics_state);
                }
                ActiveTab::Bench => {
                    tab_bench::render(ui, self.theme, &self.engine.data, &mut self.bench);
                }
                ActiveTab::About => {
                    tab_about::render(ui, self.theme, &self.engine.data, &mut self.about_state);
                }
            });

        // Request smooth repaint for real-time live clock meters
        ctx.request_repaint_after(Duration::from_millis(500));
    }
}
