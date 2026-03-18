use crate::prelude::*;
use std::path::{Path, PathBuf};

use super::ProjectConfig;

/// Config file names in priority order
const CONFIG_FILENAMES: &[&str] = &[
    "codspeed.yaml",
    "codspeed.yml",
    ".codspeed.yaml",
    ".codspeed.yml",
];

/// A project configuration paired with the path it was loaded from.
#[derive(Debug)]
pub struct DiscoveredProjectConfig {
    pub config: ProjectConfig,
    pub path: PathBuf,
}

impl DiscoveredProjectConfig {
    /// Discover and load project configuration file
    ///
    /// # Search Strategy
    /// 1. If `config_path_override` is provided, load from that path only (error if not found)
    /// 2. Otherwise, search for config files in current directory and upward to git root
    /// 3. Try filenames in priority order: codspeed.yaml, codspeed.yml, .codspeed.yaml, .codspeed.yml
    pub fn discover_and_load(
        config_path_override: Option<&Path>,
        current_dir: &Path,
    ) -> Result<Option<DiscoveredProjectConfig>> {
        // Case 1: Explicit --config path provided
        if let Some(config_path) = config_path_override {
            let config = ProjectConfig::load_from_path(config_path)
                .with_context(|| format!("Failed to load config from {}", config_path.display()))?;

            return Ok(Some(DiscoveredProjectConfig {
                config,
                path: config_path.to_path_buf(),
            }));
        }

        // Case 2: Search for config files
        let search_dirs = Self::get_search_directories(current_dir)?;

        for dir in search_dirs {
            for filename in CONFIG_FILENAMES {
                let candidate_path = dir.join(filename);
                if candidate_path.exists() {
                    debug!("Found config file at {}", candidate_path.display());
                    let config = ProjectConfig::load_from_path(&candidate_path)?;

                    return Ok(Some(DiscoveredProjectConfig {
                        config,
                        path: candidate_path,
                    }));
                }
            }
        }

        // No config found - this is OK
        Ok(None)
    }

    /// Returns the directory containing the config file.
    pub fn config_dir(&self) -> Option<PathBuf> {
        let canonical_path = self
            .path
            .canonicalize()
            .unwrap_or_else(|_| self.path.clone());
        canonical_path.parent().map(|p| p.to_path_buf())
    }

    /// Get list of directories to search for config files
    ///
    /// Returns directories from current_dir upward to git root (if in a git repo)
    fn get_search_directories(current_dir: &Path) -> Result<Vec<PathBuf>> {
        let mut dirs = vec![current_dir.to_path_buf()];

        // Try to find git repository root
        if let Some(git_root) = crate::cli::run::helpers::find_repository_root(current_dir) {
            // Add parent directories up to git root
            let mut dir = current_dir.to_path_buf();
            while let Some(parent) = dir.parent() {
                if parent == git_root {
                    if !dirs.contains(&git_root) {
                        dirs.push(git_root.clone());
                    }
                    break;
                }
                if !dirs.contains(&parent.to_path_buf()) {
                    dirs.push(parent.to_path_buf());
                }
                dir = parent.to_path_buf();
            }
        }

        Ok(dirs)
    }
}
