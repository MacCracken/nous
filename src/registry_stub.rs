//! Stub marketplace registry for standalone nous.
//!
//! Provides the minimal LocalRegistry interface that the resolver uses.
//! In the full AGNOS system, this is replaced by mela's real implementation.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::Result;

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
pub struct LocalRegistry {
    base_dir: PathBuf,
}

impl LocalRegistry {
    /// Open or create a registry at the given directory.
    pub fn new(base_dir: &Path) -> Result<Self> {
        Ok(Self {
            base_dir: base_dir.to_path_buf(),
        })
    }

    /// Search installed packages by query string.
    #[must_use]
    pub fn search(&self, query: &str) -> Vec<InstalledMarketplacePackage> {
        self.list_installed()
            .into_iter()
            .filter(|p| {
                p.manifest.agent.name.contains(query)
                    || p.manifest.agent.description.contains(query)
            })
            .collect()
    }

    /// List all installed marketplace packages.
    #[must_use]
    pub fn list_installed(&self) -> Vec<InstalledMarketplacePackage> {
        let manifests_dir = self.base_dir.join("installed");
        let mut packages = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&manifests_dir) {
            for entry in entries.flatten() {
                let manifest_path = entry.path().join("manifest.json");
                if manifest_path.exists()
                    && let Ok(data) = std::fs::read_to_string(&manifest_path)
                    && let Ok(manifest) = serde_json::from_str::<MarketplaceManifest>(&data)
                {
                    let size = dir_size(&entry.path());
                    packages.push(InstalledMarketplacePackage {
                        manifest,
                        installed_size: size,
                        install_path: entry.path(),
                        installed_at: chrono::Utc::now(),
                    });
                }
            }
        }

        packages
    }

    /// Get a specific installed package by name.
    #[must_use]
    pub fn get_package(&self, name: &str) -> Option<InstalledMarketplacePackage> {
        self.list_installed()
            .into_iter()
            .find(|p| p.manifest.agent.name == name)
    }

    /// Install a package from a tarball path.
    ///
    /// Stub — real implementation lives in mela. Kept for interface parity
    /// so consumers can code against the same API shape now.
    #[allow(dead_code)]
    pub fn install_package(&mut self, _tarball: &Path, _dest: Option<&Path>) -> Result<()> {
        Ok(())
    }
}

fn dir_size(path: &Path) -> u64 {
    std::fs::read_dir(path)
        .map(|entries| {
            entries
                .flatten()
                .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
                .sum()
        })
        .unwrap_or(0)
}
