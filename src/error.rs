//! Nous error types.
//!
//! Provides typed errors so consumers can match on failure modes
//! rather than downcasting opaque `anyhow` boxes.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// The result type used throughout the nous public API.
pub type Result<T> = std::result::Result<T, NousError>;

/// Errors produced by the nous resolver.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum NousError {
    /// A system command (apt-cache, dpkg-query, apt) failed to execute.
    #[error("failed to run `{command}`: {source}")]
    CommandExec {
        /// The command that failed.
        command: String,
        /// The underlying I/O error.
        source: std::io::Error,
    },

    /// Reading or writing to the marketplace registry directory failed.
    #[error("registry I/O error at `{path}`: {source}")]
    RegistryIo {
        /// The path involved.
        path: PathBuf,
        /// The underlying I/O error.
        source: std::io::Error,
    },

    /// A marketplace manifest file could not be parsed.
    #[error("invalid manifest at `{path}`: {source}")]
    InvalidManifest {
        /// Path to the bad manifest.
        path: PathBuf,
        /// The JSON parse error.
        source: serde_json::Error,
    },

    /// A version constraint string could not be parsed.
    ///
    /// Reserved for P1 dependency resolution.
    #[error("invalid version constraint `{constraint}`: {reason}")]
    InvalidVersionConstraint {
        /// The constraint string that failed to parse.
        constraint: String,
        /// Why it failed.
        reason: String,
    },

    /// Circular dependency detected during graph resolution.
    ///
    /// Reserved for P1 dependency resolution.
    #[error("dependency cycle detected: {}", format_cycle(cycle))]
    DependencyCycle {
        /// The package names forming the cycle.
        cycle: Vec<String>,
    },

    /// Conflicting version requirements for a package.
    ///
    /// Reserved for P1 dependency resolution.
    #[error("version conflict for `{package}`: {description}")]
    VersionConflict {
        /// The package with conflicting requirements.
        package: String,
        /// Human-readable description of the conflict.
        description: String,
    },
}

fn format_cycle(cycle: &[String]) -> String {
    cycle.join(" -> ")
}

/// Serializable snapshot of a [`NousError`] for logging and wire transport.
///
/// `NousError` itself cannot be `Serialize`/`Deserialize` because it wraps
/// `std::io::Error` and `serde_json::Error`. This companion type captures the
/// essential information in a serializable form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum NousErrorKind {
    /// Command execution failure.
    CommandExec {
        /// The command that failed.
        command: String,
        /// The error message.
        message: String,
    },
    /// Registry I/O failure.
    RegistryIo {
        /// The path involved.
        path: PathBuf,
        /// The error message.
        message: String,
    },
    /// Invalid manifest.
    InvalidManifest {
        /// Path to the bad manifest.
        path: PathBuf,
        /// The error message.
        message: String,
    },
    /// Invalid version constraint.
    InvalidVersionConstraint {
        /// The constraint string.
        constraint: String,
        /// Why it failed.
        reason: String,
    },
    /// Dependency cycle.
    DependencyCycle {
        /// The package names forming the cycle.
        cycle: Vec<String>,
    },
    /// Version conflict.
    VersionConflict {
        /// The package with conflicting requirements.
        package: String,
        /// Description of the conflict.
        description: String,
    },
}

impl From<&NousError> for NousErrorKind {
    fn from(err: &NousError) -> Self {
        match err {
            NousError::CommandExec { command, source } => NousErrorKind::CommandExec {
                command: command.clone(),
                message: source.to_string(),
            },
            NousError::RegistryIo { path, source } => NousErrorKind::RegistryIo {
                path: path.clone(),
                message: source.to_string(),
            },
            NousError::InvalidManifest { path, source } => NousErrorKind::InvalidManifest {
                path: path.clone(),
                message: source.to_string(),
            },
            NousError::InvalidVersionConstraint { constraint, reason } => {
                NousErrorKind::InvalidVersionConstraint {
                    constraint: constraint.clone(),
                    reason: reason.clone(),
                }
            }
            NousError::DependencyCycle { cycle } => NousErrorKind::DependencyCycle {
                cycle: cycle.clone(),
            },
            NousError::VersionConflict {
                package,
                description,
            } => NousErrorKind::VersionConflict {
                package: package.clone(),
                description: description.clone(),
            },
        }
    }
}
