//! Stub marketplace registry for standalone nous.
//!
//! Provides the minimal LocalRegistry interface that the resolver uses.
//! In the full AGNOS system, this is replaced by mela's real implementation.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{NousError, Result};

/// A locally installed marketplace package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledMarketplacePackage {
    pub manifest: MarketplaceManifest,
    pub installed_size: u64,
    pub install_path: PathBuf,
    pub installed_at: chrono::DateTime<chrono::Utc>,
}

/// Marketplace manifest (simplified for resolver use).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketplaceManifest {
    pub agent: AgentInfo,
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    #[serde(default)]
    pub runtime: String,
}

/// Agent identity within a marketplace manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentInfo {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
}

/// Local package registry — reads installed marketplace packages from disk.
///
/// Packages are loaded from the `installed/` subdirectory at construction time
/// and cached in memory. Call [`LocalRegistry::reload`] to refresh.
#[derive(Debug)]
pub struct LocalRegistry {
    base_dir: PathBuf,
    /// Cached packages, sorted by name.
    packages: Vec<InstalledMarketplacePackage>,
}

impl LocalRegistry {
    /// The base directory this registry reads from.
    #[must_use]
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    /// Open a registry at the given directory, reading all installed packages.
    ///
    /// Returns an error if the base directory does not exist or is not readable.
    pub fn new(base_dir: &Path) -> Result<Self> {
        if !base_dir.exists() {
            return Err(NousError::RegistryIo {
                path: base_dir.to_path_buf(),
                source: std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "marketplace directory does not exist",
                ),
            });
        }

        let mut reg = Self {
            base_dir: base_dir.to_path_buf(),
            packages: Vec::new(),
        };
        reg.reload();
        Ok(reg)
    }

    /// Re-scan the filesystem and refresh the cached package list.
    pub fn reload(&mut self) {
        self.packages = Self::scan_installed(&self.base_dir);
    }

    /// Search cached packages by query string.
    #[must_use]
    pub fn search(&self, query: &str) -> Vec<&InstalledMarketplacePackage> {
        self.packages
            .iter()
            .filter(|p| {
                p.manifest.agent.name.contains(query)
                    || p.manifest.agent.description.contains(query)
            })
            .collect()
    }

    /// List all installed marketplace packages (from cache).
    #[must_use]
    pub fn list_installed(&self) -> &[InstalledMarketplacePackage] {
        &self.packages
    }

    /// Get a specific installed package by name (from cache).
    #[must_use]
    pub fn get_package(&self, name: &str) -> Option<&InstalledMarketplacePackage> {
        self.packages.iter().find(|p| p.manifest.agent.name == name)
    }

    /// Install a package from a tarball path.
    ///
    /// Stub — real implementation lives in mela. Kept for interface parity
    /// so consumers can code against the same API shape now.
    #[allow(dead_code)]
    pub fn install_package(&mut self, _tarball: &Path, _dest: Option<&Path>) -> Result<()> {
        Ok(())
    }

    /// Read the `installed/` directory and return all valid packages, sorted by name.
    fn scan_installed(base_dir: &Path) -> Vec<InstalledMarketplacePackage> {
        let manifests_dir = base_dir.join("installed");
        let mut packages = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&manifests_dir) {
            for entry in entries.flatten() {
                let manifest_path = entry.path().join("manifest.json");
                if manifest_path.exists()
                    && let Ok(data) = std::fs::read_to_string(&manifest_path)
                    && let Ok(manifest) = serde_json::from_str::<MarketplaceManifest>(&data)
                {
                    let size = dir_size_recursive(&entry.path());
                    let installed_at = std::fs::metadata(&manifest_path)
                        .and_then(|m| m.modified())
                        .ok()
                        .and_then(|t| {
                            let dur = t.duration_since(std::time::UNIX_EPOCH).ok()?;
                            chrono::DateTime::from_timestamp(
                                dur.as_secs() as i64,
                                dur.subsec_nanos(),
                            )
                        })
                        .unwrap_or_else(chrono::Utc::now);
                    packages.push(InstalledMarketplacePackage {
                        manifest,
                        installed_size: size,
                        install_path: entry.path(),
                        installed_at,
                    });
                }
            }
        }

        packages.sort_by(|a, b| a.manifest.agent.name.cmp(&b.manifest.agent.name));
        packages
    }
}

fn dir_size_recursive(path: &Path) -> u64 {
    std::fs::read_dir(path)
        .map(|entries| {
            entries
                .flatten()
                .map(|e| {
                    let meta = match e.metadata() {
                        Ok(m) => m,
                        Err(_) => return 0,
                    };
                    if meta.is_dir() {
                        dir_size_recursive(&e.path())
                    } else {
                        meta.len()
                    }
                })
                .sum()
        })
        .unwrap_or(0)
}
