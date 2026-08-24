//! Modern CPU-Z Core Library.
//!
//! High-performance, cross-platform hardware diagnostic, profiling, and telemetry engine.

#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    missing_docs,
    rust_2018_idioms
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::too_many_lines,
    clippy::suboptimal_flops
)]

pub mod app;
pub mod bench;
pub mod error;
pub mod hardware;
pub mod ui;

pub use app::ModernCpuZApp;
pub use error::{AppError, Result};
