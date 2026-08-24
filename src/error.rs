//! Error definitions for Modern CPU-Z.
//!
//! Provides a strongly-typed error hierarchy utilizing [`thiserror`].

use thiserror::Error;

/// Main error type for all hardware introspection, benchmarking, and reporting routines.
#[derive(Debug, Error)]
pub enum AppError {
    /// Failed to query x86 CPUID instruction.
    #[error("CPUID query failure: {0}")]
    Cpuid(String),

    /// Failed to parse or read SMBIOS / DMI table.
    #[error("SMBIOS error: {0}")]
    Smbios(String),

    /// Operating system I/O error occurred during telemetry inspection.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization or deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Generic telemetry or platform extraction error.
    #[error("Hardware telemetry error: {0}")]
    Hardware(String),
}

/// Convenience alias for `Result<T, AppError>`.
pub type Result<T> = std::result::Result<T, AppError>;
