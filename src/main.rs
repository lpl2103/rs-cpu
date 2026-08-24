//! M-CPU Application Entrypoint.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use modern_cpu_z::ModernCpuZApp;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> eframe::Result {
    // Initialize structured logging via tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "modern_cpu_z=info,warn".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    tracing::info!("Iniciando M-CPU v0.1.0...");

    let app_icon = modern_cpu_z::ui::create_app_icon();

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("M-CPU")
            .with_icon(app_icon)
            .with_inner_size([1120.0, 860.0])
            .with_min_inner_size([880.0, 640.0])
            .with_active(true),
        ..Default::default()
    };

    eframe::run_native(
        "M-CPU",
        native_options,
        Box::new(|cc| Ok(Box::new(ModernCpuZApp::new(cc)))),
    )
}
