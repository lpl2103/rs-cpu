//! Modern CPU-Z Application Entrypoint.

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

    tracing::info!("Starting Modern CPU-Z v0.1.0...");

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Modern CPU-Z")
            .with_inner_size([820.0, 750.0])
            .with_min_inner_size([680.0, 560.0])
            .with_active(true),
        ..Default::default()
    };

    eframe::run_native(
        "Modern CPU-Z",
        native_options,
        Box::new(|cc| Ok(Box::new(ModernCpuZApp::new(cc)))),
    )
}
